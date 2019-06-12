use telebot::Bot;
use futures::stream::Stream;
use std::env;

use erased_serde::Serialize;

use telebot::functions::*;
use telebot::objects::*;

fn main() {
    // Create the bot
    let mut bot = Bot::new(&env::var("TELEGRAM_BOT_KEY").unwrap()).update_interval(200);

    let stream = bot.inline()
        .and_then(|(bot, query)| {
            let result: Vec<Box<dyn Serialize + Send>> = vec![
                Box::new(
                    InlineQueryResultArticle::new(
                        "Test".into(),
                        Box::new(input_message_content::Text::new("This is a test".into())),
                    ).reply_markup(InlineKeyboardMarkup::new(vec![
                        vec![
                            InlineKeyboardButton::new("Wikipedia".into())
                                .url("http://wikipedia.org"),
                        ],
                    ])),
                ),
            ];

            bot.answer_inline_query(query.id, result)
                .is_personal(true)
                .send()
        })
        .for_each(|_| Ok(()));

    // enter the main loop
    bot.run_with(stream);
    //tokio::spawn(stream.into_future().map_err(|_| ()));

    //lp.run(stream.for_each(|_| Ok(())).into_future()).unwrap();
}
