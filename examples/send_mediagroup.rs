use telebot::{Bot, file::File};
use futures::stream::Stream;
use std::env;

// import all available functions
use telebot::functions::*;

fn main() {
    // Create the bot
    let mut bot = Bot::new(&env::var("TELEGRAM_BOT_KEY").unwrap()).update_interval(200);

    let handle = bot.new_cmd("/send_mediagroup")
        .and_then(|(bot, msg)| {
            bot.mediagroup(msg.chat.id)
                .file(File::Url("https://upload.wikimedia.org/wikipedia/commons/f/f4/Honeycrisp.jpg".into()))
                .file(File::Url("https://upload.wikimedia.org/wikipedia/en/3/3e/Pooh_Shepard1928.jpg".into()))
                .file("examples/bee.jpg")
                .send()
        })
        .for_each(|_| Ok(()));

    // enter the main loop
    bot.run_with(handle);
}
