Telebot - Telegram Bot Library in Rust
======================================

[![Crates.io](https://img.shields.io/crates/v/telebot.svg)](https://crates.io/crates/telebot)

This library allows you to write a Telegram Bot in Rust. It's an almost complete wrapper for the Telegram Bot API and uses tokio-curl to send a request to the Telegram server. Each Telegram function call returns a future.

## Usage
Add this to your `Cargo.toml`
``toml
[dependencies]
telebot = "0.0.3"
``

## Find a Telegram function in the source code
All available functions are listed in src/functions.rs. For example consider sendLocation:
``` rust
/// Use this method to send point on the map. On success, the sent Message is returned.
#[derive(TelegramFunction, Serialize)]
#[function = "sendLocation"]
#[answer = "Message"]
#[bot_function = "location"]
pub struct SendLocation {
    chat_id: u32,
    latitude: f32,
    longitude: f32,
#[serde(skip_serializing_if="Option::is_none")]
    disable_notification: Option<bool>,
#[serde(skip_serializing_if="Option::is_none")]                                                                                                             
    reply_to_message_id: Option<u32>,
#[serde(skip_serializing_if="Option::is_none")]
    reply_markup: Option<NotImplemented>
}
```

The field "answer" defines the name of the function in the local API. Each optional field in the struct can be changed by calling the function with the name of the field.
So for example to send a location silently: ` bot.location(432432, 48.8566, 2.3522).disable_notification(true).send() `

## Example
``` rust
extern crate telebot;
extern crate tokio_core;
extern crate futures;

use telebot::bot;
use tokio_core::reactor::Core;                                                                                                                                          
use futures::stream::Stream;
use futures::Future;

// import all available functions
use telebot::functions::*;

fn main() {
    let mut lp = Core::new().unwrap();
    let bot = bot::RcBot::new(lp.handle(), "<TELEGRAM-BOT-TOKEN>")
        .update_interval(200);

    let handle = bot.new_cmd("/reply")
        .and_then(|(bot, msg)| {
            let mut text = msg.text.unwrap().clone();
            if text.is_empty() {
                text = "<empty>".into();
            }

            bot.send_message(msg.chat.id, text).send()
        });

    bot.register(handle);

    enum LocationErr {
        Telegram(telebot::error::Error),
        WrongLocationFormat
    }

    let handle2 = bot.new_cmd("/location")
        .then(|result| {
            let (bot, msg) = result.expect("Strange telegram error!");

            let (longitude, altitude) = {
                let pos: Vec<Result<f32,_>> = msg.text.clone().unwrap().split_whitespace().take(2).map(|x| x.parse::<f32>()).collect();
                (pos[0].clone(), pos[1].clone())
            };

            if let Ok(longitude) = longitude {
                if let Ok(altitude) = altitude {
                    return Ok((bot, msg, longitude, altitude));
                }
            }

            return Err((bot, msg, LocationErr::WrongLocationFormat));
        })
        .and_then(|(bot, msg, long, alt)| {
            bot.location(msg.chat.id, long, alt).send().map_err(|err| (bot, msg, LocationErr::Telegram(err)))
        })
        .or_else(|(bot, msg, err)| {
            let text = {
                match err {
                    LocationErr::Telegram(err) => format!("Telegram error: {:?}", err),
                    LocationErr::WrongLocationFormat => "Couldn't parse the location!".into()
                }
            };

            bot.send_message(msg.chat.id, text).send()
        });

    bot.register(handle2);

    bot.run(&mut lp).unwrap();
}
```
