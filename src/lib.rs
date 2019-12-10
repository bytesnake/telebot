//! # Write a telegram bot in Rust
//!
//! This library allows you to write a Telegram Bot in Rust. It's an almost complete wrapper for the Telegram Bot API and uses hyper to send a request to the Telegram server. Each Telegram function call returns a future and carries the actual bot and the answer.
//! You can find all available functions in src/functions.rs. The crate telebot-derive implements all
//! required getter, setter and send functions automatically.
//!
//! # Example usage
//!
//! ```
//! use telebot::Bot;
//! use futures::stream::Stream;
//! use std::env;
//! 
//! // import all available functions
//! use telebot::functions::*;
//! 
//! fn main() {
//!     // Create the bot
//!     let mut bot = Bot::new(&env::var("TELEGRAM_BOT_KEY").unwrap()).update_interval(200);
//! 
//!     // Register a reply command which answers a message
//!     let handle = bot.new_cmd("/reply")
//!         .and_then(|(bot, msg)| {
//!             let mut text = msg.text.unwrap().clone();
//!             if text.is_empty() {
//!                 text = "<empty>".into();
//!             }
//! 
//!             bot.message(msg.chat.id, text).send()
//!         })
//!         .for_each(|_| Ok(()));
//! 
//!     bot.run_with(handle);
//! }
//! ```

#![allow(bare_trait_objects)]
#![allow(unused_attributes)]

#[macro_use]
extern crate telebot_derive;

#[macro_use]
extern crate log;

extern crate hyper_multipart_rfc7578 as hyper_multipart;
#[macro_use]
extern crate serde;

extern crate tokio;

pub use bot::Bot;
pub use error::Error;
pub use file::File;

pub mod bot;
pub mod error;
pub mod objects;
pub mod functions;
pub mod file;
