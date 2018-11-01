extern crate futures;
extern crate telebot;
extern crate tokio_core;
extern crate env_logger;

use telebot::RcBot;
use tokio_core::reactor::Core;
use futures::stream::Stream;
use std::env;
use futures::IntoFuture;

fn main() {
    env_logger::init();
    // Create a new tokio core
    let mut lp = Core::new().unwrap();

    // Create the bot
    let bot = RcBot::new(lp.handle(), &env::var("TELEGRAM_BOT_KEY").unwrap());

    let stream = bot.get_push_stream().and_then(|(_, msg)| {
        println!("Received: {:#?}", msg);

        Ok(())
    });

    // enter the main loop
    lp.run(stream.for_each(|_| Ok(())).into_future()).unwrap();
}
