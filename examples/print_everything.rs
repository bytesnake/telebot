extern crate telebot;
extern crate tokio_core;
extern crate futures;

use telebot::bot;
use tokio_core::reactor::Core;
use futures::stream::Stream;
use std::env;
use std::fs::File;
use futures::IntoFuture;

// import all available functions
use telebot::functions::*;

fn main() {
    // Create a new tokio core
    let mut lp = Core::new().unwrap();

    // Create the bot
    let bot = bot::RcBot::new(lp.handle(), &env::var("TELEGRAM_BOT_KEY").unwrap())
        .update_interval(200);

    let stream = bot.get_stream()
        .and_then(|(bot, msg)| {
            println!("Received: {:#?}",msg);

            Ok(())
        });

    // enter the main loop
    lp.run(stream.for_each(|_| Ok(())).into_future()).unwrap();
}
