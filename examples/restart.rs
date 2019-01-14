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

    // Enter the main loop
    while let Err(err) = bot.run(&mut lp) {
        eprintln!("Event loop shutdown:");
        for (i, cause) in err.iter_causes().enumerate() {
            eprintln!(" => {}: {}", i, cause);
        }
    }
}
