use telebot::Bot;
use futures::stream::Stream;
use std::env;

// import all available functions
use telebot::functions::*;

fn main() {
    // Create the bot
    let bot = Bot::new(&env::var("TELEGRAM_BOT_KEY").unwrap()).update_interval(200);

    // Enter the main loop
    loop {
        bot.clone().run();
    }
}
