extern crate erased_serde;
extern crate futures;
extern crate telebot;
extern crate tokio_core;

use telebot::RcBot;
use tokio_core::reactor::Core;
use futures::stream::Stream;
use std::env;
use futures::IntoFuture;

use erased_serde::Serialize;

use telebot::functions::*;
use telebot::objects::*;

fn main() {
    // Create a new tokio core
    let mut lp = Core::new().unwrap();

    // Create the bot
    let bot = RcBot::new(lp.handle(), &env::var("TELEGRAM_BOT_KEY").unwrap()).update_interval(200);

    let stream = bot.get_stream()
        .filter_map(|(bot, msg)| msg.inline_query.map(|query| (bot, query)))
        .and_then(|(bot, query)| {
            let result: Vec<Box<Serialize>> = vec![
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
        });

    // enter the main loop
    lp.run(stream.for_each(|_| Ok(())).into_future()).unwrap();
}
