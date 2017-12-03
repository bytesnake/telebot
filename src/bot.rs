//! This is the actual Bot module. For ergonomic reasons there is a RcBot which uses the real bot
//! as an underlying field. You should always use RcBot.

use objects;
use error::Error;

use std::str;
use std::io;
use std::time::Duration;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::{RefCell, Cell};
use std::sync::{Arc, Mutex};

use curl::easy::{Easy, List, Form, InfoType};
use tokio_curl::Session;
use tokio_core::reactor::{Handle, Core, Interval};
use serde_json;
use serde_json::value::Value;
use futures::{Future, IntoFuture, Stream, stream};
use futures::future::result;
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
        RcBot { inner: Rc::new(Bot::new(handle, key)) }
    }
}

/// The main bot structure
pub struct Bot {
    pub key: String,
    pub handle: Handle,
    pub last_id: Cell<u32>,
    pub update_interval: Cell<u64>,
    pub handlers: RefCell<HashMap<String, UnboundedSender<(RcBot, objects::Message)>>>,
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
            handlers: RefCell::new(HashMap::new()),
            session: Session::new(handle.clone()),
        }
    }

    /// Creates a new request and adds a JSON message to it. The returned Future contains a the
    /// reply as a string.  This method should be used if no file is added because a JSON msg is
    /// always compacter than a formdata one.
    pub fn fetch_json(
        &self,
        func: &'static str,
        msg: String,
    ) -> impl Future<Item = String, Error = Error> {
        debug!("Send JSON: {}", msg);

        let json = self.build_json(msg);
        let session = self.session.clone();
        let key = self.key.clone();

        result(json).and_then(move |a| _fetch(session, key, func, a))
    }

    fn build_json(&self, msg: String) -> Result<Easy, Error> {
        let mut header = List::new();

        header.append("Content-Type: application/json")?;

        let mut a = Easy::new();

        a.http_headers(header)?;
        a.post_fields_copy(msg.as_bytes())?;
        a.post(true)?;

        Ok(a)
    }


    /// Creates a new request with some byte content (e.g. a file). The method properties have to be
    /// in the formdata setup and cannot be sent as JSON.
    pub fn fetch_formdata<T>(
        &self,
        func: &'static str,
        msg: Value,
        file: T,
        kind: &str,
        file_name: &str,
    ) -> impl Future<Item = String, Error = Error>
    where
        T: io::Read,
    {
        debug!("Send formdata: {}", msg.to_string());

        let formdata = self.build_formdata(msg, file, kind, file_name);
        let session = self.session.clone();
        let key = self.key.clone();

        result(formdata).and_then(move |a| _fetch(session, key, func, a))
    }

    fn build_formdata<T>(
        &self,
        msg: Value,
        mut file: T,
        kind: &str,
        file_name: &str,
    ) -> Result<Easy, Error>
    where
        T: io::Read,
    {
        let mut content = Vec::new();

        file.read_to_end(&mut content)?;

        let msg = msg.as_object().ok_or(Error::Unknown)?;

        let mut form = Form::new();

        // add properties
        for (key, val) in msg.iter() {
            form.part(key).contents(format!("{}", val).as_bytes()).add()?;
        }

        form.part(kind)
            .buffer(file_name, content)
            .content_type("application/octet-stream")
            .add()?;

        let mut a = Easy::new();

        a.post(true)?;
        a.httppost(form)?;

        Ok(a)
    }
}

/// calls cURL and parses the result for an error
pub fn _fetch(
    session: Session,
    key: String,
    func: &str,
    a: Easy,
) -> impl Future<Item = String, Error = Error> {
    let response_vec = Arc::new(Mutex::new(Vec::new()));

    let req = prepare_fetch(a, Arc::clone(&response_vec), &key, func);

    result(req).and_then(move |a| {
        session
            .perform(a)
            .map_err(|x| Error::TokioCurl(x))
            .map(|_| response_vec)
            .and_then(move |response_vec| {
                if let Ok(ref vec) = response_vec.lock() {
                    if let Ok(s) = str::from_utf8(vec) {
                        return Ok(String::from(s));
                    }
                }

                return Err(Error::Unknown);
            })
            .and_then(move |x| {
                debug!("Got a result from telegram: {}", x);
                // try to parse the result as a JSON and find the OK field.
                // If the ok field is true, then the string in "result" will be returned
                if let Ok(req) = serde_json::from_str::<Value>(&x) {
                    if let (Some(ok), res) = (
                        req.get("ok").and_then(Value::as_bool),
                        req.get("result"),
                    )
                    {
                        if ok {
                            if let Some(result) = res {
                                return serde_json::to_string(result).map_err(|e| {
                                    error!("Error: {}", e);

                                    Error::Unknown
                                });
                            }
                        }

                        match req.get("description").and_then(Value::as_str) {
                            Some(err) => return Err(Error::Telegram(err.into())),
                            None => return Err(Error::Telegram("Unknown".into())),
                        }
                    }
                }

                return Err(Error::JSON);
            })
    })
}

fn prepare_fetch(
    mut a: Easy,
    response_vec: Arc<Mutex<Vec<u8>>>,
    key: &str,
    func: &str,
) -> Result<Easy, Error> {
    let url = &format!("https://api.telegram.org/bot{}/{}", key, func);

    a.url(url)?;

    let r2 = response_vec.clone();

    a.write_function(move |data| match r2.lock() {
        Ok(ref mut vec) => {
            vec.extend_from_slice(data);
            Ok(data.len())
        }
        Err(_) => Ok(0),
    })?;

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
    })?;

    Ok(a)
}

impl RcBot {
    /// Sets the update interval to an integer in milliseconds
    pub fn update_interval(self, interval: u64) -> RcBot {
        self.inner.update_interval.set(interval);

        self
    }

    /// Creates a new command and returns a stream which will yield a message when the command is send
    pub fn new_cmd(
        &self,
        cmd: &str,
    ) -> impl Stream<Item = (RcBot, objects::Message), Error = Error> {
        let (sender, receiver) = mpsc::unbounded();

        self.inner.handlers.borrow_mut().insert(cmd.into(), sender);

        receiver.map_err(|_| Error::Unknown)
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
            .map_err(|_| Error::Unknown)
            .and_then(move |_| {
                self.get_updates().offset(self.inner.last_id.get()).send()
            })
            .map(|(_, x)| {
                stream::iter_result(x.0.into_iter().map(|x| Ok(x)).collect::<Vec<
                    Result<
                        objects::Update,
                        Error,
                    >,
                >>())
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
                            match sender.unbounded_send((self.clone(), msg)) {
                                Ok(_) => (),
                                Err(e) => error!("Error: {}", e),
                            }
                        }
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
    }
}
