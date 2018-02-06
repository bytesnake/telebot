//! This is the actual Bot module. For ergonomic reasons there is a RcBot which uses the real bot
//! as an underlying field. You should always use RcBot.

use objects;
use failure::{Error, Fail, ResultExt};
use error::ErrorKind;

use std::str;
use std::io;
use std::time::Duration;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};

use curl::easy::{Easy, Form, InfoType, List};
use tokio_curl::Session;
use tokio_core::reactor::{Core, Handle, Interval};
use serde_json;
use serde_json::value::Value;
use futures::{stream, Future, IntoFuture, Stream};
use futures::sync::mpsc;
use futures::sync::mpsc::UnboundedSender;

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
    pub handle: Handle,
    pub last_id: Cell<u32>,
    pub update_interval: Cell<u64>,
    pub timeout: Cell<u64>,
    pub handlers: RefCell<HashMap<String, UnboundedSender<(RcBot, objects::Message)>>>,
    pub default_handler: RefCell<Option<UnboundedSender<(RcBot, objects::Message)>>>,
    pub session: Session,
}

impl Bot {
    pub fn new(handle: Handle, key: &str) -> Bot {
        debug!("Create a new bot with the key {}", key);

        Bot {
            handle: handle.clone(),
            key: key.into(),
            last_id: Cell::new(0),
            update_interval: Cell::new(1000),
            timeout: Cell::new(30),
            handlers: RefCell::new(HashMap::new()),
            default_handler: RefCell::new(None),
            session: Session::new(handle.clone()),
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

        let json = self.build_json(msg);

        self._fetch(func, json)
    }

    /// Builds the CURL header for a JSON request. The JSON is already converted to a str and is
    /// appended to the POST header.
    fn build_json(&self, msg: &str) -> Result<Easy, Error> {
        let mut header = List::new();

        header
            .append("Content-Type: application/json")
            .context(ErrorKind::cURL)?;

        let mut a = Easy::new();

        a.http_headers(header).context(ErrorKind::cURL)?;
        a.post_fields_copy(msg.as_bytes()).context(ErrorKind::cURL)?;
        a.post(true).context(ErrorKind::cURL)?;

        Ok(a)
    }

    /// Creates a new request with some byte content (e.g. a file). The method properties have to be
    /// in the formdata setup and cannot be sent as JSON.
    pub fn fetch_formdata<T>(
        &self,
        func: &'static str,
        msg: &Value,
        file: T,
        kind: &str,
        file_name: &str,
    ) -> impl Future<Item = String, Error = Error>
    where
        T: io::Read,
    {
        debug!("Send formdata: {}", msg.to_string());

        let formdata = self.build_formdata(msg, file, kind, file_name);

        self._fetch(func, formdata)
    }

    /// Builds the CURL header for a formdata request. The file content is read and then append to
    /// the formdata. Each key-value pair has a own line.
    fn build_formdata<T>(
        &self,
        msg: &Value,
        mut file: T,
        kind: &str,
        file_name: &str,
    ) -> Result<Easy, Error>
    where
        T: io::Read,
    {
        let mut content = Vec::new();

        file.read_to_end(&mut content).context(ErrorKind::IO)?;

        let msg = msg.as_object().ok_or(ErrorKind::Unknown)?;

        let mut form = Form::new();

        // add properties
        for (key, val) in msg.iter() {
            let val = match val {
                &Value::String(ref val) => format!("{}", val),
                etc => format!("{}", etc),
            };

            form.part(key)
                .contents(val.as_bytes())
                .add()
                .context(ErrorKind::Form)?;
        }

        form.part(kind)
            .buffer(file_name, content)
            .content_type("application/octet-stream")
            .add()
            .context(ErrorKind::Form)?;

        let mut a = Easy::new();

        a.post(true).context(ErrorKind::cURL)?;
        a.httppost(form).context(ErrorKind::cURL)?;

        Ok(a)
    }

    /// Calls the Telegram API for the function and awaits the result. The result is then converted
    /// to a String and returned in a Future.
    pub fn _fetch(
        &self,
        func: &str,
        a: Result<Easy, Error>,
    ) -> impl Future<Item = String, Error = Error> {
        let response_vec = Arc::new(Mutex::new(Vec::new()));
        let r1 = Arc::clone(&response_vec);

        let session = self.session.clone();

        a.and_then(|a| self.prepare_fetch(a, r1, func))
            .into_future()
            .and_then(move |a| {
                session
                    .perform(a)
                    //.context(ErrorKind::TokioCurl)
                    .map_err(|_| Error::from(ErrorKind::TokioCurl))
                    //.map_err(|x| x.context(ErrorKind::TokioCurl))
                    .map(|_| response_vec)
            })
            .and_then(move |response_vec| {
                let vec = &(response_vec.lock().map_err(|_| ErrorKind::Unknown)?);
                let s = str::from_utf8(vec).context(ErrorKind::UTF8Decode)?;

                debug!("Got a result from telegram: {}", s);
                // try to parse the result as a JSON and find the OK field.
                // If the ok field is true, then the string in "result" will be returned
                let req = serde_json::from_str::<Value>(&s).context(ErrorKind::JSON)?;

                let ok = req.get("ok")
                    .and_then(Value::as_bool)
                    .ok_or(ErrorKind::JSON)?;

                if ok {
                    if let Some(result) = req.get("result") {
                        return Ok(serde_json::to_string(result).context(ErrorKind::JSON)?);
                    }
                }

                match req.get("description").and_then(Value::as_str) {
                    Some(err) => Err(Error::from(
                        format_err!("{}", err).context(ErrorKind::Telegram),
                    )),
                    None => Err(Error::from(
                        format_err!("No description!").context(ErrorKind::Telegram),
                    )),
                }
            })
    }

    /// Configures cURL to call to the right address and write the response to a vector.
    fn prepare_fetch(
        &self,
        mut a: Easy,
        response_vec: Arc<Mutex<Vec<u8>>>,
        func: &str,
    ) -> Result<Easy, Error> {
        let url = &format!("https://api.telegram.org/bot{}/{}", self.key, func);

        a.url(url).context(ErrorKind::cURL)?;

        a.write_function(move |data| match response_vec.lock() {
            Ok(ref mut vec) => {
                vec.extend_from_slice(data);
                Ok(data.len())
            }
            Err(_) => Ok(0),
        }).context(ErrorKind::cURL)?;

        a.debug_function(|info, data| {
            match info {
                InfoType::DataOut => {
                    println!("DataOut");
                }
                InfoType::Text => {
                    println!("Text");
                }
                InfoType::HeaderOut => {
                    println!("HeaderOut");
                }
                InfoType::SslDataOut => {
                    println!("SslDataOut");
                }
                _ => println!("something else"),
            }

            println!("{:?}", String::from_utf8_lossy(data));
        }).context(ErrorKind::cURL)?;

        Ok(a)
    }
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

        self.inner.handlers.borrow_mut().insert(cmd.into(), sender);

        receiver.then(|x| x.map_err(|_| Error::from(ErrorKind::Channel)))
    }

    pub fn default_cmd(&self) -> impl Stream<Item = (RcBot, objects::Message), Error = Error> {
        let (sender, receiver) = mpsc::unbounded();

        *self.inner.default_handler.borrow_mut() = Some(sender);

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
                let mut forward: Option<String> = None;

                if let Some(ref mut message) = val.message {
                    if let Some(text) = message.text.clone() {
                        let mut content = text.split_whitespace();
                        if let Some(cmd) = content.next() {
                            if self.inner.handlers.borrow_mut().contains_key(cmd) {
                                message.text = Some(content.collect::<Vec<&str>>().join(" "));
                                forward = Some(cmd.into());
                            }
                        }
                    }
                }

                if let Some(cmd) = forward {
                    if let Some(sender) = self.inner.handlers.borrow_mut().get_mut(&cmd) {
                        if let Some(msg) = val.message {
                            sender
                                .unbounded_send((self.clone(), msg))
                                .unwrap_or_else(|e| error!("Error: {}", e));
                        }
                    }
                    return None;
                } else if let Some(ref mut sender) = *self.inner.default_handler.borrow_mut() {
                    if let Some(msg) = val.message {
                        sender
                            .unbounded_send((self.clone(), msg))
                            .unwrap_or_else(|e| error!("Error: {}", e));
                    }
                    return None;
                } else {
                    return Some((self.clone(), val));
                }
            })
    }

    /// helper function to start the event loop
    pub fn run<'a>(&'a self, core: &mut Core) -> Result<(), Error> {
        core.run(self.get_stream().for_each(|_| Ok(())).into_future())
            .context(ErrorKind::Tokio)
            .map_err(Error::from)
    }
}
