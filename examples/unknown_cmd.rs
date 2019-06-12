use telebot::Bot;
use futures::{Future, stream::Stream};
use std::env;

// import all available functions
use telebot::functions::*;

fn main() {
    // Create the bot
    let mut bot = Bot::new(&env::var("TELEGRAM_BOT_KEY").unwrap()).update_interval(200);

    let known = bot.new_cmd("/known")
        .and_then(|(bot, msg)| bot.message(msg.chat.id, "This one is known".into()).send())
        .for_each(|_| Ok(()));

    // Every possible command is unknown
    let unknown = bot.unknown_cmd()
        .and_then(|(bot, msg)| bot.message(msg.chat.id, "Unknown command".into()).send())
        .for_each(|_| Ok(()));

    // Enter the main loop
    bot.run_with(known.join(unknown));
}
