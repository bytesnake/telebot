//! # Write a telegram bot in Rust
//!
//! This library allows you to write a Telegram Bot in Rust. It's an almost complete wrapper for the Telegram Bot API and uses tokio-curl to send a request to the Telegram server. Each Telegram function call returns a future and carries the actual bot and the answer.
//! You can find all available functions in src/functions.rs. The getter/setter etc. will be
//! automatically implemented by telebot-derive. 
//!
//! # Example usage
//!
//! ```
//! extern crate telebot;
//! extern crate tokio_core;
//! extern crate futures;

//! use telebot::bot;
//! use tokio_core::reactor::Core;                       
//! use futures::stream::Stream;
//! use futures::Future;
//! use std::fs::File;
//! 
//! // import all available functions
//! use telebot::functions::*;
//! fn main() {
//!     let mut lp = Core::new().unwrap();
//!     let bot = bot::RcBot::new(lp.handle(), "<TELEGRAM-BOT-TOKEN>")
//!         .update_interval(200);
//!     let handle = bot.new_cmd("/reply")
//!     .and_then(|(bot, msg)| {
//!         let mut text = msg.text.unwrap().clone();
//!         if text.is_empty() {
//!             text = "<empty>".into();
//!         }
//!
//!         bot.message(msg.chat.id, text).send()
//!     });
//!     bot.register(handle);
//!
//!     bot.run(&mut lp).unwrap();
//! }
//! ```

#![feature(conservative_impl_trait)]
#![feature(proc_macro)]
#![feature(custom_attribute)]
#![allow(unused_attributes)]

#[macro_use]
extern crate telebot_derive;

#[macro_use]
extern crate serde_derive;

extern crate serde_json;
extern crate curl;
extern crate futures;
extern crate tokio_core;
extern crate tokio_curl;

pub mod bot;
pub mod error;
pub mod objects;
pub mod functions;
pub mod file;
