//! The bot module

use crate::objects;
//use functions::FunctionGetMe;
use crate::error::{ErrorKind, TelegramError};
use crate::file::File;

use std::{str, time::{Duration, Instant}, collections::HashMap, sync::Arc};
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::timer::Interval;
use hyper::{Body as Body2, Client, Request, Uri, header::CONTENT_TYPE, client::{HttpConnector, ResponseFuture}, rt::Stream};
use hyper_tls::HttpsConnector;
use hyper_multipart::client::multipart;
use serde_json::{self, value::Value};
use futures::{stream, Future, future::IntoFuture, sync::mpsc::{self, UnboundedSender}};
use failure::{Error, Fail, ResultExt};
use hyper_multipart_rfc7578::client::multipart::Body;

/// A clonable request handle struct
/// Allows the construction of requests to the Telegram server
#[derive(Clone)]
pub struct RequestHandle {
    key: String,
    pub inner: Arc<hyper::Client<HttpsConnector<HttpConnector>, Body2>>
}

impl RequestHandle {
    /// Creates a new request and adds a JSON message to it. The returned Future contains a the
    /// reply as a string.  This method should be used if no file is added becontext a JSON msg is
    /// always compacter than a formdata one.
    pub fn fetch_json(
        &self,
        func: &'static str,
        msg: &str,
    ) -> impl Future<Item = String, Error = Error> {

        debug!("Send JSON {}: {}", func, msg);
        let request = self.build_json(func, String::from(msg)).unwrap();

        _fetch(self.inner.request(request))
    }

    /// Builds the HTTP header for a JSON request. The JSON is already converted to a str and is
    /// appended to the POST header.
    fn build_json(
        &self,
        func: &'static str,
        msg: String,
    ) -> Result<Request<Body2>, Error> {
        let url: Result<Uri, _> =
            format!("https://api.telegram.org/bot{}/{}", self.key, func).parse();

        debug!("Send message {}", msg);

        let req = Request::post(url.context(ErrorKind::Uri)?)
            .header(CONTENT_TYPE, "application/json")
            .body(msg.into())
            .context(ErrorKind::Hyper)?;

        Ok(req)
    }
    ///
    /// Creates a new request with some byte content (e.g. a file). The method properties have to be
    /// in the formdata setup and cannot be sent as JSON.
    pub fn fetch_formdata(
        &self,
        func: &'static str,
        msg: &Value,
        files: Vec<File>,
        kind: &str,
    ) -> impl Future<Item = String, Error = Error> {
        debug!("Send formdata {}: {}", func, msg.to_string());

        let request = self.build_formdata(func, msg, files, kind).unwrap();
        _fetch(self.inner.request(request))
    }

    /// Builds the HTTP header for a formdata request. The file content is read and then append to
    /// the formdata. Each key-value pair has a own line.
    fn build_formdata(
        &self,
        func: &'static str,
        msg: &Value,
        files: Vec<File>,
        _kind: &str,
    ) -> Result<Request<Body2>,Error> {
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

            form.add_text(key, val);
        }

        for file in files {
            match file {
                File::Memory { name, source } => {
                    form.add_reader_file(name.clone(), source, name);
                }
                File::Disk { path } => {
                    form.add_file(path.clone().file_name().unwrap().to_str().unwrap(), path).context(ErrorKind::NoFile)?;
                },
                _ => {}
            }
        }

        let req = form.set_body_convert::<Body2, Body>(&mut req_builder).context(ErrorKind::Hyper)?;

        Ok(req)
    }
}
/// Calls the Telegram API for the function and awaits the result. The result is then converted
/// to a String and returned in a Future.
//pub fn _fetch<T: futures_retry::ErrorHandler<std::io::Error>>(fut_res: FutureRetry<FnMut() -> ResponseFuture, T>) -> impl Future<Item = String, Error = Error> {
//pub fn _fetch<T: Future<Item=hyper::Response<hyper::Body>, Error=hyper::error::Error>>(fut_res: T) -> impl Future<Item = String, Error = Error> {
pub fn _fetch(fut_res: ResponseFuture) -> impl Future<Item = String, Error = Error> {
    fut_res
        .and_then(move |res| res.into_body().concat2())
        .map_err(|e| {
            eprintln!("{:?}", e);

            Error::from(e.context(ErrorKind::Hyper))
        })
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

/// The main bot structure
///
/// Contains all configuration like `key`, `name`, etc. important handles to message the user and
/// `request` to issue requests to the Telegram server
#[derive(Clone)]
pub struct Bot {
    pub request: RequestHandle,
    key: String,
    name: Option<String>,
    update_interval: u64,
    timeout: u64,
    pub handlers: HashMap<String, UnboundedSender<(RequestHandle, objects::Message)>>,
    pub unknown_cmd_handler: Option<UnboundedSender<(RequestHandle, objects::Message)>>,
    pub unknown_text_handler: Option<UnboundedSender<(RequestHandle, objects::Message)>>,
    pub callback_handler: Option<UnboundedSender<(RequestHandle, objects::CallbackQuery)>>,
    pub inline_handler: Option<UnboundedSender<(RequestHandle, objects::InlineQuery)>>
}

impl Bot {
    pub fn new(key: &str) -> Bot {
        let client = Client::builder()
                .keep_alive(true)
                .keep_alive_timeout(Some(Duration::from_secs(3600)))
                .build(HttpsConnector::new(4).unwrap());

        Bot {
            request: RequestHandle { inner: Arc::new(client), key: key.into() },
            key: key.into(),
            name: None,
            update_interval: 2000,
            timeout: 3600,
            handlers: HashMap::new(),
            unknown_cmd_handler: None,
            unknown_text_handler: None,
            callback_handler: None,
            inline_handler: None
        }
    }

    /// Sets the update interval to an integer in milliseconds
    pub fn update_interval(mut self, interval: u64) -> Bot {
        self.update_interval = interval;

        self
    }

    /// Sets the timeout interval for long polling
    pub fn timeout(mut self, timeout: u64) -> Bot {
        self.timeout = timeout;

        let client = Client::builder()
                .keep_alive(true)
                .keep_alive_timeout(Some(Duration::from_secs(timeout)))
                .build(HttpsConnector::new(4).unwrap());

        self.request = RequestHandle { inner: Arc::new(client), key: self.key.clone() };

        self
    }

    /// Creates a new command and returns a stream which will yield a message when the command is send
    pub fn new_cmd(
        &mut self,
        cmd: &str,
    ) -> impl Stream<Item = (RequestHandle, objects::Message), Error = Error> {
        let (sender, receiver) = mpsc::unbounded();

        let cmd = if cmd.starts_with("/") {
            cmd.into()
        } else {
            format!("/{}", cmd)
        };

        self.handlers.insert(cmd.into(), sender);

        receiver.map_err(|_| Error::from(ErrorKind::Channel))
    }

    /// Creates a new text based command and returns a stream which will yield a message when the text is sent
    pub fn new_text(
        &mut self,
        cmd: &str,
    ) -> impl Stream<Item = (RequestHandle, objects::Message), Error = Error> {
        let (sender, receiver) = mpsc::unbounded();

        self.handlers.insert(cmd.into(), sender);

        receiver.map_err(|_| Error::from(ErrorKind::Channel))
    }

    /// Returns a stream which will yield a message when none of previously registered text based command matches
    pub fn unknown_text(&mut self) -> impl Stream<Item = (RequestHandle, objects::Message), Error = Error> {
        let (sender, receiver) = mpsc::unbounded();

        self.unknown_text_handler = Some(sender);

        receiver.then(|x| x.map_err(|_| Error::from(ErrorKind::Channel)))
    }

    /// Returns a stream which will yield a message when none of previously registered commands matches
    pub fn unknown_cmd(&mut self) -> impl Stream<Item = (RequestHandle, objects::Message), Error = Error> {
        let (sender, receiver) = mpsc::unbounded();

        self.unknown_cmd_handler = Some(sender);

        receiver.then(|x| x.map_err(|_| Error::from(ErrorKind::Channel)))
    }

    /// Returns a stream which will yield a received CallbackQuery
    pub fn callback(&mut self) -> impl Stream<Item = (RequestHandle, objects::CallbackQuery), Error = Error> {
        let (sender, receiver) = mpsc::unbounded();

        self.callback_handler = Some(sender);

        receiver.then(|x| x.map_err(|_| Error::from(ErrorKind::Channel)))
    }

    /// Returns a stream which will yield a received CallbackQuery
    pub fn inline(&mut self) -> impl Stream<Item = (RequestHandle, objects::InlineQuery), Error = Error> {
        let (sender, receiver) = mpsc::unbounded();

        self.inline_handler = Some(sender);

        receiver.then(|x| x.map_err(|_| Error::from(ErrorKind::Channel)))
    }

    pub fn resolve_name(&self) -> impl Future<Item = Option<String>, Error = Error> {
        use crate::functions::FunctionGetMe;

        // create a new task which resolves the bot name and then set it in the struct
        let resolve_name = self.request.get_me().send()
            .map(move |user| {
                if let Some(name) = user.1.username {
                    return Some(format!("@{}", name));
                } else {
                    return None;
                }
            });

        resolve_name
    }

    pub fn process_updates(self, last_id: Arc<AtomicUsize>) -> impl Stream<Item = (RequestHandle, objects::Update), Error = Error> {
        use crate::functions::FunctionGetUpdates;

        self.request.get_updates()
            .offset(last_id.load(Ordering::Relaxed) as i64)
            .timeout(self.timeout as i64)
            .send()
            .into_stream()
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
                if last_id.load(Ordering::Relaxed) < x.update_id as usize + 1 {
                    last_id.store(x.update_id as usize + 1, Ordering::Relaxed);
                }

                Ok(x)
            })
        .filter_map(move |mut val| {
            debug!("Got an update from Telegram: {:?}", val);
    
            if let Some(_) = val.callback_query {
                if let Some(sender) = self.callback_handler.clone() {
                    sender
                        .unbounded_send((self.request.clone(), val.callback_query.unwrap()))
                        .unwrap_or_else(|e| error!("Error: {}", e));
                    return None;
                }
            }
    
            if let Some(_) = val.inline_query {
                if let Some(sender) = self.inline_handler.clone() {
                    sender
                        .unbounded_send((self.request.clone(), val.inline_query.unwrap()))
                        .unwrap_or_else(|e| error!("Error: {}", e));
                    return None;
                }
            }

            let mut sndr: Option<UnboundedSender<(RequestHandle, objects::Message)>> = None;
    
            if let Some(ref mut message) = val.message {
                if let Some(true) = message.entities.as_ref().and_then(|x| x.get(0)).map(|x| x.kind == "bot_command") {
                    if let Some(text) = message.text.clone() {
                        let mut content = text.split_whitespace();
                        if let Some(mut cmd) = content.next() {
                            if let Some(name) = self.name.as_ref() {
                                if cmd.ends_with(name.as_str()) {
                                    cmd = cmd.rsplitn(2, '@').skip(1).next().unwrap();
                                }
                            }
                            if let Some(sender) = self.handlers.get(cmd)
                            {
                                sndr = Some(sender.clone());
                                message.text = Some(content.collect::<Vec<&str>>().join(" "));
                            } else if let Some(ref sender) =
                                self.unknown_cmd_handler
                            {
                                sndr = Some(sender.clone());
                            }
                        }
                    }
                } else if let Some(text) = message.text.clone() {
                    let mut content = text.split_whitespace();
                    if let Some(cmd) = content.next() {
                        if let Some(sender) = self.handlers.get(cmd)
                        {
                            sndr = Some(sender.clone());
                            message.text = Some(content.collect::<Vec<&str>>().join(" "));
                        }
                        else if let Some(ref sender) = self.unknown_text_handler
                        {
                            sndr = Some(sender.clone());
                            message.text = Some(content.collect::<Vec<&str>>().join(" "));
                        }
                    }
                    else if let Some(ref sender) = self.unknown_text_handler
                    {
                        sndr = Some(sender.clone());
                        message.text = Some(content.collect::<Vec<&str>>().join(" "));
                    }
                }
            }
    
            if let Some(sender) = sndr {
                sender
                    .unbounded_send((self.request.clone(), val.message.unwrap()))
                    .unwrap_or_else(|e| error!("Error: {}", e));
                return None;
            } else {
                return Some((self.request.clone(), val));
            }
        })
    }

    ///
    /// The main update loop, the update function is called every update_interval milliseconds
    /// When an update is available the last_id will be updated and the message is filtered
    /// for commands
    /// The message is forwarded to the returned stream if no command was found
    pub fn get_stream(
        mut self,
        name: Option<String>
    ) -> impl Stream<Item = (RequestHandle, objects::Update), Error = Error> { 
        self.name = name;
        let last_id = Arc::new(AtomicUsize::new(0));

        let duration = Duration::from_millis(self.update_interval);
        Interval::new(Instant::now(), duration)
            .map_err(|x| Error::from(x.context(ErrorKind::IntervalTimer)))
            .map(move |_| self.clone().process_updates(last_id.clone()))
            .flatten()
            /*.inspect_err(|err| println!("Error on {:?}", err))
            .then(|r| futures::future::ok(stream::iter_ok::<_, Error>(r)))
            .flatten()*/
    }

    pub fn into_future(&self) -> impl Future<Item = (), Error = Error> {
        let bot = self.clone();

        self.resolve_name()
            .and_then(|name| bot.get_stream(name).for_each(|_| Ok(())))
            .map(|_| ())
    }

    pub fn run_with<I>(self, other: I) 
    where
        I: IntoFuture<Error = Error>,
        <I as IntoFuture>::Future: Send + 'static,
        <I as IntoFuture>::Item: Send
    {
        tokio::run(
            self.into_future().join(other)
            .map(|_| ())
            .map_err(|e| {
                eprintln!("Error: could not resolve the bot name!");

                for (i, cause) in e.iter_causes().enumerate() {
                    println!(" => {}: {}", i, cause);
                }
            })
        );
    }

    pub fn run(self) {
        self.run_with(Ok(()));
    }
}
