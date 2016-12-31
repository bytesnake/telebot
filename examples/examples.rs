extern crate telebot;
extern crate tokio_core;
extern crate futures;

use telebot::bot;
use tokio_core::reactor::Core;
use futures::stream::Stream;
use futures::Future;

// import all available functions
use telebot::functions::*;

fn main() {
    // Create a new tokio core
    let mut lp = Core::new().unwrap();

    // Create the bot
    let bot = bot::RcBot::new(lp.handle(), "<api key>")
        .update_interval(200);

    // Register a reply command which answer a message
    let handle = bot.new_cmd("/reply")
        .and_then(|(bot, msg)| {
            let mut text = msg.text.unwrap().clone();
            if text.is_empty() {
                text = "<empty>".into();
            }

            bot.send_message(msg.chat.id, text).send()
        });

    bot.register(handle);
 
    // Register a location command which will send a location to request like /location 2.321 12.32
    enum LocationErr {
        Telegram(telebot::error::Error),
        WrongLocationFormat
    }

    let handle2 = bot.new_cmd("/location")
        .then(|result| {
            let (bot, msg) = result.expect("Strange telegram error!");

            let (longitude, altitude) = {
                let pos: Vec<Result<f32,_>> = msg.text.clone().unwrap().split_whitespace().take(2).map(|x| x.parse::<f32>()).collect();
                (pos[0].clone(), pos[1].clone())
            };

            if let Ok(longitude) = longitude {
                if let Ok(altitude) = altitude {
                    return Ok((bot, msg, longitude, altitude));
                }
            }

            return Err((bot, msg, LocationErr::WrongLocationFormat));
        })
        .and_then(|(bot, msg, long, alt)| {
            bot.location(msg.chat.id, long, alt).send().map_err(|err| (bot, msg, LocationErr::Telegram(err)))
        })
        .or_else(|(bot, msg, err)| {
            let text = {
                match err {
                    LocationErr::Telegram(err) => format!("Telegram error: {:?}", err),
                    LocationErr::WrongLocationFormat => "Couldn't parse the location!".into()
                }
            };

            bot.send_message(msg.chat.id, text).send()
        });

    bot.register(handle2);

    let handle3 = bot.new_cmd("/typing")
        .and_then(|(bot, msg)| bot.chat_action(msg.chat.id, "typing".into()).send());

    bot.register(handle3);

    // Register a get_my_photo command which will send the own profile photo to the chat
    enum PhotoErr {
        Telegram(telebot::error::Error),
        NoPhoto
    }

    let handle4 = bot.new_cmd("/get_my_photo")
        .then(|result| {
            let (bot, msg) = result.expect("Strange telegram error!");
    
            let user_id = msg.from.clone().unwrap().id;

            bot.get_user_profile_photos(user_id).limit(1u32).send()
                .then(|result| {
                    match result {
                        Ok((bot, photos)) => {
                            if photos.total_count == 0 {
                                return Err((bot, msg, PhotoErr::NoPhoto));
                            }
                            
                            return Ok((bot, msg, photos.photos[0][0].clone().file_id))
                        },
                        Err(err) => Err((bot, msg, PhotoErr::Telegram(err)))
                    }
                })
        })
        .and_then(|(bot, msg, file_id)| {
            bot.photo(msg.chat.id).photo(file_id).send().map_err(|err| (bot, msg, PhotoErr::Telegram(err)))
        })
        .or_else(|(bot, msg, err)| {
            let text = match err {
                PhotoErr::Telegram(err) => format!("Telegram Error: {:?}", err),
                PhotoErr::NoPhoto => "No photo exists!".into()
            };

            bot.send_message(msg.chat.id, text).send()
        });
        
    bot.register(handle4);

    // enter the main loop
    bot.run(&mut lp).unwrap();
}
