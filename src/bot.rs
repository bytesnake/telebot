//! This is the actual Bot module. For ergonomic reasons there is a RcBot which uses the real bot
//! as an underlying field. You should always use RcBot.

use objects;
use functions::FunctionGetMe;
use failure::{Error, Fail, ResultExt};
use error::{ErrorKind, TelegramError};
use file::File;

use std::{str, time::Duration, collections::HashMap, rc::Rc, cell::{Cell, RefCell}};

use tokio_core::reactor::{Core, Handle, Interval};
use hyper::{Body, Client, Request, Uri, header::CONTENT_TYPE, client::{HttpConnector, ResponseFuture}};
use hyper_tls::HttpsConnector;
use hyper_multipart::client::multipart;
use serde_json::{self, value::Value};
use futures::{stream, Future, IntoFuture, Stream, sync::mpsc::{self, UnboundedSender}};

/// A clonable, single threaded bot
///
/// The outer API gets implemented on RcBot
#[derive(Clone)]
pub struct RcBot {
    pub inner: Rc<Bot>,
}

impl RcBot {
    pub fn new(handle: Handle, key: &str) -> RcBot {
        RcBot {
            inner: Rc::new(Bot::new(handle, key)),
        }
    }
}

/// The main bot structure
pub struct Bot {
    pub key: String,
    pub name: RefCell<Option<String>>,
    pub handle: Handle,
    pub last_id: Cell<u32>,
    pub update_interval: Cell<u64>,
    pub timeout: Cell<u64>,
    pub handlers: RefCell<HashMap<String, UnboundedSender<(RcBot, objects::Message)>>>,
    pub unknown_handler: RefCell<Option<UnboundedSender<(RcBot, objects::Message)>>>,
}

impl Bot {
    pub fn new(handle: Handle, key: &str) -> Bot {
        debug!("Create a new bot with the key {}", key);

        Bot {
            handle: handle.clone(),
            key: key.into(),
            name: RefCell::new(None),
            last_id: Cell::new(0),
            update_interval: Cell::new(1000),
            timeout: Cell::new(30),
            handlers: RefCell::new(HashMap::new()),
            unknown_handler: RefCell::new(None),
        }
    }

    /// Creates a new request and adds a JSON message to it. The returned Future contains a the
    /// reply as a string.  This method should be used if no file is added becontext a JSON msg is
    /// always compacter than a formdata one.
    pub fn fetch_json(
        &self,
        func: &'static str,
        msg: &str,
    ) -> impl Future<Item = String, Error = Error> {
        debug!("Send JSON: {}", msg);

        let request = self.build_json(func, String::from(msg));

        request
            .into_future()
            .and_then(|(client, request)| _fetch(client.request(request)))
    }

    /// Builds the HTTP header for a JSON request. The JSON is already converted to a str and is
    /// appended to the POST header.
    fn build_json(
        &self,
        func: &'static str,
        msg: String,
    ) -> Result<(Client<HttpsConnector<HttpConnector>, Body>, Request<Body>), Error> {
        let url: Result<Uri, _> =
            format!("https://api.telegram.org/bot{}/{}", self.key, func).parse();

        let client = Client::builder()
            .build(HttpsConnector::new(2).context(ErrorKind::HttpsInitializeError)?);

        let req = Request::post(url.context(ErrorKind::Uri)?)
            .header(CONTENT_TYPE, "application/json")
            .body(msg.into())
            .context(ErrorKind::Hyper)?;

        Ok((client, req))
    }

    /// Creates a new request with some byte content (e.g. a file). The method properties have to be
    /// in the formdata setup and cannot be sent as JSON.
    pub fn fetch_formdata(
        &self,
        func: &'static str,
        msg: &Value,
        file: File,
        kind: &str,
    ) -> impl Future<Item = String, Error = Error> {
        debug!("Send formdata: {}", msg.to_string());

        let request = self.build_formdata(func, msg, file, kind);

        request
            .into_future()
            .and_then(|(client, request)| _fetch(client.request(request)))
    }

    /// Builds the HTTP header for a formdata request. The file content is read and then append to
    /// the formdata. Each key-value pair has a own line.
    fn build_formdata(
        &self,
        func: &'static str,
        msg: &Value,
        file: File,
        kind: &str,
    ) -> Result<
        (
            Client<HttpsConnector<HttpConnector>, Body>,
            Request<Body>,
        ),
        Error,
    > {
        let client: Client<HttpsConnector<_>, Body> = Client::builder()
            .keep_alive(true)
            .build(HttpsConnector::new(4).context(ErrorKind::HttpsInitializeError)?);

        let url: Result<Uri, _> =
            format!("https://api.telegram.org/bot{}/{}", self.key, func).parse();

        let mut req_builder = Request::post(url.context(ErrorKind::Uri)?);
        let mut form = multipart::Form::default();

        let msg = msg.as_object().ok_or(ErrorKind::JsonNotMap)?;

        // add properties
        for (key, val) in msg.iter() {
            let val = match val {
                &Value::String(ref val) => format!("{}", val),
                etc => format!("{}", etc),
            };

            form.add_text(key, val.as_ref());
        }

        match file {
            File::Memory { name, source } => {
                form.add_reader_file(kind, source, name);
            }
            File::Disk { path } => {
                form.add_file(kind, path).context(ErrorKind::NoFile)?;
            }
        }

        let req = form.set_body(&mut req_builder).context(ErrorKind::Hyper)?;

        Ok((client, req))
    }
}

/// Calls the Telegram API for the function and awaits the result. The result is then converted
/// to a String and returned in a Future.
pub fn _fetch(fut_res: ResponseFuture) -> impl Future<Item = String, Error = Error> {
    fut_res
        .and_then(move |res| res.into_body().concat2())
        .map_err(|e| Error::from(e.context(ErrorKind::Hyper)))
        .and_then(move |response_chunks| {
            let s = str::from_utf8(&response_chunks)?;

            debug!("Got a result from telegram: {}", s);
            // try to parse the result as a JSON and find the OK field.
            // If the ok field is true, then the string in "result" will be returned
            let req = serde_json::from_str::<Value>(&s).context(ErrorKind::JsonParse)?;

            let ok = req.get("ok")
                .and_then(Value::as_bool)
                .ok_or(ErrorKind::Json)?;

            if ok {
                if let Some(result) = req.get("result") {
                    return Ok(serde_json::to_string(result).context(ErrorKind::JsonSerialize)?);
                }
            }

            let e = match req.get("description").and_then(Value::as_str) {
                Some(err) => {
                    Error::from(TelegramError::new(err.into()).context(ErrorKind::Telegram))
                }
                None => Error::from(ErrorKind::Telegram),
            };

            Err(Error::from(e.context(ErrorKind::Telegram)))
        })
}

impl RcBot {
    /// Sets the update interval to an integer in milliseconds
    pub fn update_interval(self, interval: u64) -> RcBot {
        self.inner.update_interval.set(interval);

        self
    }

    /// Sets the timeout interval for long polling
    pub fn timeout(self, timeout: u64) -> RcBot {
        self.inner.timeout.set(timeout);

        self
    }

    /// Creates a new command and returns a stream which will yield a message when the command is send
    pub fn new_cmd(
        &self,
        cmd: &str,
    ) -> impl Stream<Item = (RcBot, objects::Message), Error = Error> {
        let (sender, receiver) = mpsc::unbounded();

        let cmd = if cmd.starts_with("/") {
            cmd.into()
        } else {
            format!("/{}", cmd)
        };

        self.inner.handlers.borrow_mut().insert(cmd.into(), sender);

        receiver.map_err(|_| Error::from(ErrorKind::Channel))
    }

    /// Returns a stream which will yield a message when none of previously registered commands matches
    pub fn unknown_cmd(&self) -> impl Stream<Item = (RcBot, objects::Message), Error = Error> {
        let (sender, receiver) = mpsc::unbounded();

        *self.inner.unknown_handler.borrow_mut() = Some(sender);

        receiver.then(|x| x.map_err(|_| Error::from(ErrorKind::Channel)))
    }

    /// Register a new commnd
    pub fn register<T>(&self, hnd: T)
    where
        T: Stream + 'static,
    {
        self.inner.handle.spawn(
            hnd.for_each(|_| Ok(()))
                .into_future()
                .map(|_| ())
                .map_err(|_| ()),
        );
    }

    /// The main update loop, the update function is called every update_interval milliseconds
    /// When an update is available the last_id will be updated and the message is filtered
    /// for commands
    /// The message is forwarded to the returned stream if no command was found
    pub fn get_stream<'a>(
        &'a self,
    ) -> impl Stream<Item = (RcBot, objects::Update), Error = Error> + 'a {
        use functions::*;

        let duration = Duration::from_millis(self.inner.update_interval.get());
        Interval::new(duration, &self.inner.handle)
            .into_future()
            .into_stream()
            .flatten()
            .map_err(|x| Error::from(x.context(ErrorKind::IntervalTimer)))
            .and_then(move |_| {
                self.get_updates()
                    .offset(self.inner.last_id.get())
                    .timeout(self.inner.timeout.get() as i64)
                    .send()
            })
            .map(|(_, x)| {
                stream::iter_result(
                    x.0
                        .into_iter()
                        .map(|x| Ok(x))
                        .collect::<Vec<Result<objects::Update, Error>>>(),
                )
            })
            .flatten()
            .and_then(move |x| {
                if self.inner.last_id.get() < x.update_id as u32 + 1 {
                    self.inner.last_id.set(x.update_id as u32 + 1);
                }

                Ok(x)
            })
            .filter_map(move |mut val| {
                debug!("Got an update from Telegram: {:?}", val);

                let mut sndr: Option<UnboundedSender<(RcBot, objects::Message)>> = None;

                if let Some(ref mut message) = val.message {
                    if let Some(text) = message.text.clone() {
                        let mut content = text.split_whitespace();
                        if let Some(mut cmd) = content.next() {
                            if cmd.starts_with("/") {
                                if let Some(name) = self.inner.name.borrow().as_ref() {
                                    if cmd.ends_with(name.as_str()) {
                                        cmd = cmd.rsplitn(2, '@').skip(1).next().unwrap();
                                    }
                                }
                                if let Some(sender) = self.inner.handlers.borrow_mut().get_mut(cmd)
                                {
                                    sndr = Some(sender.clone());
                                    message.text = Some(content.collect::<Vec<&str>>().join(" "));
                                } else if let Some(ref mut sender) =
                                    *self.inner.unknown_handler.borrow_mut()
                                {
                                    sndr = Some(sender.clone());
                                }
                            }
                        }
                    }
                }

                if let Some(sender) = sndr {
                    sender
                        .unbounded_send((self.clone(), val.message.unwrap()))
                        .unwrap_or_else(|e| error!("Error: {}", e));
                    return None;
                } else {
                    return Some((self.clone(), val));
                }
            })
    }

    /// helper function to start the event loop
    pub fn run<'a>(&'a self, core: &mut Core) -> Result<(), Error> {
        // create a local copy of the bot to circumvent lifetime issues
        let bot = self.inner.clone();
        // create a new task which resolves the bot name and then set it in the struct
        let resolve_name = self.get_me().send()
            .map(move |user| {
                if let Some(name) = user.1.username {
                    bot.name.replace(Some(format!("@{}", name)));
                }
            });
        // spawn the task
        self.inner.handle.spawn(resolve_name.map_err(|_| ()));
        core.run(self.get_stream().for_each(|_| Ok(())).into_future())
            .context(ErrorKind::Tokio)
            .map_err(Error::from)
    }
}
