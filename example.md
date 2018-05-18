# Additional example
In addition to the readme I will show a more sophisticated example here. We want to create a Telegram command which takes two coordinates and then send the position as a map to the user. The command looks like this:

`/location 2.321 12.32`

The full example can be found [here](https://github.com/bytesnake/telebot/blob/master/examples/error_handling.rs).

## What could go wrong
Before we are creating the future chain, lets first think about the error enum. Either the user input is invalid (e.g. there are not two decimals after the command) or the Telegram server could have problems to process the coordinates. Therefore the enum looks like this:
``` rust
enum LocationErr {
    Telegram(Error),
    WrongLocationFormat
}
```

## Create a new command
We want to create a new bot and register a new command `/location`. This is very simple:
``` rust
let bot = RcBot::new(lp.handle(), &env::var("TELEGRAM_BOT_KEY").unwrap());
let handle = bot.new_cmd("/location")
```
Everything else in this doc will modify the stream returned by new_cmd.

## Parse the user input
After receiving a command from the user we want to parse the arguments. This can be achieved by using `split_whitespace` command to create an iterator over words and then parsing the first two elements to f32 elements.

``` rust
if let Some(pos) = msg.text.next() {
    let mut elms = pos.split_whitespace.take(2).filter_map(|x| x.parse::<f32>().ok());

    if let (Some(a), Some(l)) = (elms.next(), elms.next()) {
        return Ok((bot, msg, a, l));
    }
}

return Err((bot, msg, LocationErr::WrongLocationFormat));
```
If anything goes wrong then we will return an error. The trick is now two chain another `and_then` function in case the parsing was successful.

## Send the location
Send the location is straightforward and can be accomplished with the location function. In case of an error we will wrap the Telegram error in the LocationErr::Telegram variant.
``` rust
bot.location(msg.chat.id, long, alt)
    .send()
    .map_err(|err| (bot, msg, LocationErr::Telegram(err)))
```

## Consume the error and send a message, if one occurs
We can now catch any error from the previous chain with an `or_else` function. The error message is first converted to text and then send to the user.
``` rust
let text = match err {
    LocationErr::Telegram(err) => format!("Telegram error: {:?}", err),
    LocationErr::WrongLocationFormat => "Couldn't parse the location!".into()
};

bot.message(msg.chat.id, text).send()
```
## Summarise
If we are looking back at our approach, then it is obvious that we can create any sequence of Telegram calls. If at any point an error occurs, then we will jump to the `or_else` function. We just need enough enum variants and a long chain of `and_then`.
