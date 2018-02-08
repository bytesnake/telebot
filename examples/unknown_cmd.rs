extern crate futures;
extern crate telebot;
extern crate tokio_core;

use telebot::RcBot;
use tokio_core::reactor::Core;
use futures::stream::Stream;
use std::env;

// import all available functions
use telebot::functions::*;

fn main() {
    // Create a new tokio core
    let mut lp = Core::new().unwrap();

    // Create the bot
    let bot = RcBot::new(lp.handle(), &env::var("TELEGRAM_BOT_KEY").unwrap()).update_interval(200);

    // Every possible command is unknown
    let handle = bot.unknown_cmd().and_then(|(bot, msg)| bot.message(msg.chat.id, "Unknown command".into()).send());

    bot.register(handle);

    // Enter the main loop
    bot.run(&mut lp).unwrap();
}
