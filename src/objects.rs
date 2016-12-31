//! The complete list of telegram types, copied from:
//! https://core.telegram.org/bots/api#available-types
//!
//! on each struct getter, setter and send function will be implemented

/// These objects are redefinitions of basic types. telebot-derive will scope every object in
/// answer, so we need to redefine them here.
pub type Boolean = bool;
pub type Integer = u32;
pub type Vector<T> = Vec<T>;
pub type NotImplemented = ();

/// This object represents a Telegram user or bot.
#[derive(Deserialize, Debug, Clone)]
pub struct User {
    pub id: u32,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>
}

/// This object represents a chat.
#[derive(Deserialize, Debug)]
pub struct Chat {
    pub id: u32,
    #[serde(rename="type")]
    kind: String,
    title: Option<String>,
    username: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    all_members_are_administrators: Option<bool>
}

/// This object represents one special entity in a text message. For example, hashtags, usernames,
/// URLs, etc. 
#[derive(Deserialize, Debug)]
pub struct MessageEntity {
    #[serde(rename="type")]
    kind: String,
    offset: u32,
    length: u32,
    url: Option<String>,
    user: Option<User>
}

/// This object represents a message.
#[derive(Deserialize, Debug)]
pub struct Message {
    message_id: u32,
    pub from: Option<User>,
    date: u32,
    pub chat: Chat,
    forward_from: Option<User>,
    forward_from_chat: Option<User>,
    forward_from_message_id: Option<u32>,
    forward_date: Option<u32>,
    reply_to_message: Option<Box<Message>>,
    edit_date: Option<u32>,
    pub text: Option<String>,
    entities: Option<Vec<MessageEntity>>,
    audio: Option<Audio>,
    document: Option<Document>,
    game: Option<NotImplemented>,
    pub photo: Option<Vec<PhotoSize>>,
    sticker: Option<Sticker>,
    video: Option<Video>,
    voice: Option<Voice>,
    caption: Option<String>,
    contact: Option<Contact>,
    location: Option<Location>,
    venue: Option<Venue>,
    new_chat_member: Option<User>,
    left_chat_member: Option<User>,
    new_chat_title: Option<String>,
    new_chat_photo: Option<Vec<PhotoSize>>,
    delete_chat_photo: Option<bool>,
    group_chat_created: Option<bool>,
    supergroup_chat_created: Option<bool>,
    channel_chat_created: Option<bool>,
    migrate_to_chat_id: Option<u32>,
    migrate_from_chat_id: Option<u32>,
    pinned_message: Option<Box<Message>>
}

impl Message {
    pub fn get_chat_id(&self) -> u32 {
        self.chat.id
    }
}

#[derive(Deserialize, Debug)]
pub struct Updates(pub Vec<Update>);

#[derive(Deserialize, Debug)]
pub struct Update {
    pub update_id: u32,
    pub message: Option<Message>,
    edited_message: Option<Message>,
    channel_post: Option<Message>,
    edited_channel_post: Option<Message>,
    inline_query: Option<()>,
    chosen_inline_result: Option<()>,
    callback_query: Option<()>
}

/// This object represents one size of a photo or a file / sticker thumbnail.
#[derive(Deserialize, Debug, Clone)]
pub struct PhotoSize {
    pub file_id: String,
    width: u32,
    height: u32,
    file_size: Option<u32>
}

/// This object represents an audio file to be treated as music by the Telegram clients.
#[derive(Deserialize, Debug)]
pub struct Audio {
    file_id: String,
    duration: u32,
    performer: Option<String>,
    title: Option<String>,
    mime_type: Option<String>,
    file_size: Option<u32>
}

/// This object represents a general file (as opposed to photos, voice messages and audio files).
#[derive(Deserialize, Debug)]
pub struct Document {
    file_id: String,
    thumb: Option<PhotoSize>,
    file_name: Option<String>,
    mime_type: Option<String>,
    file_size: Option<u32>
}

/// This object represents a sticker.
#[derive(Deserialize, Debug)]
pub struct Sticker {
    file_id: String,
    width: u32,
    height: u32,
    thumb: Option<PhotoSize>,
    emoji: Option<String>,
    file_size: Option<u32>
}

/// This object represents a video file.
#[derive(Deserialize, Debug)]
pub struct Video {
    file_id: String,
    width: u32,
    height: u32,
    duration: u32,
    thumb: Option<PhotoSize>,
    mime_type: Option<String>,
    file_size: Option<String>
}

/// This object represents a voice note.
#[derive(Deserialize, Debug)]
pub struct Voice {
    file_id: String,
    duration: u32,
    mime_type: Option<String>,
    file_size: Option<String>
}

/// This object represents a phone contact.
#[derive(Deserialize, Debug)]
pub struct Contact {
    phone_number: String,
    first_name: String,
    last_name: String,
    user_id: u32
}

/// This object represents a point on the map.
#[derive(Deserialize, Debug)]
pub struct Location {
    longitude: f32,
    latitude: f32
}

/// This object represents a venue.
#[derive(Deserialize, Debug)]
pub struct Venue {
    location: Location,
    title: String,
    address: String,
    foursquare_id: Option<String>
}

/// This object represent a user's profile pictures.
#[derive(Deserialize, Debug)]
pub struct UserProfilePhotos {
    pub total_count: u32,
    pub photos: Vec<Vec<PhotoSize>>
}

/// This object represents a file ready to be downloaded. The file can be downloaded via the link
/// https://api.telegram.org/file/bot<token>/<file_path>. It is guaranteed that the link will be
/// valid for at least 1 hour. When the link expires, a new one can be requested by calling
/// getFile.
#[derive(Deserialize, Debug)]
pub struct File {
    file_id: String,
    file_size: Option<u32>,
    file_path: Option<String>
}

/// This object represents a custom keyboard with reply options (see Introduction to bots for
/// details and examples).
#[derive(Deserialize, Debug)]
pub struct ReplyKeyboardMarkup {
    keyboard: Vec<KeyboardButton>,
    resize_keyboard: Option<bool>,
    one_time_keyboard: Option<bool>,
    selective: Option<bool>
}

/// This object represents one button of the reply keyboard. For simple text buttons String can be
/// used instead of this object to specify text of the button. Optional fields are mutually
/// exclusive.
#[derive(Deserialize, Debug)]
pub struct KeyboardButton {
    text: String,
    request_contact: Option<bool>,
    request_location: Option<bool>
}

/// Upon receiving a message with this object, Telegram clients will remove the current custom
/// keyboard and display the default letter-keyboard. By default, custom keyboards are displayed
/// until a new keyboard is sent by a bot. An exception is made for one-time keyboards that are
/// hidden immediately after the user presses a button (see ReplyKeyboardMarkup).
#[derive(Deserialize, Debug)]
pub struct ReplyKeyboardRemove {
    remove_keyboard: bool,
    selective: Option<bool>
}

/// This object represents an inline keyboard that appears right next to the message it belongs to.
#[derive(Deserialize, Debug)]
pub struct InlineKeyboardMarkup {
    inline_keyboard: Vec<InlineKeyboardButton>
}

/// This object represents one button of an inline keyboard. You must use exactly one of the
/// optional fields.
#[derive(Deserialize, Debug)]
pub struct InlineKeyboardButton {
    text: String,
    url: Option<String>,
    callback_data: Option<String>,
    switch_inline_query: Option<String>,
    switch_inline_query_current_chat: Option<String>,
    callback_game: Option<CallbackGame>
}

/// This object represents an incoming callback query from a callback button in an inline keyboard.
/// If the button that originated the query was attached to a message sent by the bot, the field
/// message will be present. If the button was attached to a message sent via the bot (in inline
/// mode), the field inline_message_id will be present. Exactly one of the fields data or
/// game_short_name will be present.
#[derive(Deserialize, Debug)]
pub struct CallbackQuery {
    id: String,
    from: User,
    message: Option<Message>,
    inline_message_id: Option<String>,
    chat_instance: Option<String>,
    data: Option<String>,
    game_short_name: Option<String>
}

/// Upon receiving a message with this object, Telegram clients will display a reply interface to
/// the user (act as if the user has selected the bot‘s message and tapped ’Reply'). This can be
/// extremely useful if you want to create user-friendly step-by-step interfaces without having to
/// sacrifice privacy mode.
#[derive(Deserialize, Debug)]
pub struct ForceReply {
    force_reply: bool,
    selective: Option<bool>
}
    
/// This object contains information about one member of the chat.
#[derive(Deserialize, Debug)]
pub struct ChatMember {
    user: User,
    status: String
}

/// Contains information about why a request was unsuccessfull.
#[derive(Deserialize, Debug)]
pub struct ResponseParameter {
    migrate_to_chat_id: Option<u32>,
    retry_after: Option<u32>
}

/// A placeholder, currently holds no information. Use BotFather to set up your game.
#[derive(Deserialize, Debug)]
pub struct CallbackGame;
