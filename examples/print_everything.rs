extern crate futures;
extern crate telebot;
extern crate tokio_core;

use telebot::RcBot;
use tokio_core::reactor::Core;
use futures::stream::Stream;
use std::env;
use futures::IntoFuture;

fn main() {
    // Create a new tokio core
    let mut lp = Core::new().unwrap();

    // Create the bot
    let bot = RcBot::new(lp.handle(), &env::var("TELEGRAM_BOT_KEY").unwrap()).update_interval(200);

    let stream = bot.get_stream().and_then(|(_, msg)| {
        println!("Received: {:#?}", msg);

        Ok(())
    })
    .map_err(|e| {
        eprintln!("Got an error: ");
        for (i, cause) in e.iter_causes().enumerate() {
            eprintln!(" => {}: {}", i, cause);
        }
    });

    // enter the main loop
    let res = lp.run(stream.for_each(|_| Ok(())).into_future());
    if let Err(_) = res {
        eprintln!("Event loop shutdown with an error!");
    }
}
