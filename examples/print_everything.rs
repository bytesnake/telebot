use telebot::Bot;
use futures::stream::Stream;
use std::env;
use futures::{IntoFuture, Future};

fn main() {
    // Create the bot
    let bot = Bot::new(&env::var("TELEGRAM_BOT_KEY").unwrap()).update_interval(200);

    let stream = bot.get_stream(None).for_each(|(_, msg)| {
        println!("Received: {:#?}", msg);

        Ok(())
    });

    // enter the main loop
    tokio::run(stream.into_future().map_err(|_| ()));
    /*let res = lp.run(stream.for_each(|_| Ok(())).into_future());
    if let Err(err) = res {
        eprintln!("Event loop shutdown:");
        for (i, cause) in err.iter_causes().enumerate() {
            eprintln!(" => {}: {}", i, cause);
        }
    }*/
}
