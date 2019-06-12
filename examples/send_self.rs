use telebot::Bot;
use futures::stream::Stream;
use std::env;

// import all available functions
use telebot::functions::*;

fn main() {
    // Create the bot
    let mut bot = Bot::new(&env::var("TELEGRAM_BOT_KEY").unwrap()).update_interval(200);

    let handle = bot.new_cmd("/send_self")
        .and_then(|(bot, msg)| {
            bot.document(msg.chat.id)
                .file("examples/send_self.rs")
                .send()
        })
        .for_each(|_| Ok(()));

    // enter the main loop
    bot.run_with(handle);
}
