extern crate futures;
extern crate telebot;
extern crate tokio_core;

use telebot::{RcBot, file::File};
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

    let handle = bot.new_cmd("/send_mediagroup")
        .and_then(|(bot, msg)| {
            bot.mediagroup(msg.chat.id)
                .file(File::Url("https://upload.wikimedia.org/wikipedia/commons/f/f4/Honeycrisp.jpg".into()))
                .file(File::Url("https://upload.wikimedia.org/wikipedia/en/3/3e/Pooh_Shepard1928.jpg".into()))
                .file("examples/bee.jpg")
                .send()
        })
        .map_err(|err| println!("{:?}", err.cause()));

    bot.register(handle);

    // enter the main loop
    bot.run(&mut lp).unwrap();
}
