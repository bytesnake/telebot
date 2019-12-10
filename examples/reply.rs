use telebot::Bot;
use futures::stream::Stream;
use std::env;

// import all available functions
use telebot::functions::*;

fn main() {
    // Create the bot
    let mut bot = Bot::new(&env::var("TELEGRAM_BOT_KEY").unwrap()).update_interval(200);

    // Register a reply command which answers a message
    let handle = bot.new_cmd("/reply")
        .and_then(|(bot, msg)| {
            let mut text = msg.text.unwrap().clone();
            if text.is_empty() {
                text = "<empty>".into();
            }

            bot.message(msg.chat.id, text).send()
        })
        .for_each(|_| Ok(()));

    bot.run_with(handle);
}

