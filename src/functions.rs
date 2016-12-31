//! Available telegram functions, copied from https://core.telegram.org/bots/api#available-methods
//!
//! telebot-derive implements setter, getter and send methods to each struct

use bot::{Bot, RcBot};
use serde_json;
use objects;
use objects::{Integer, NotImplemented};
use error::Error;
use futures::Future;
use std::io;
use std::rc::Rc;

/// A simple method for testing your bot's auth token. Requires no parameters. Returns basic
/// information about the bot in form of a User object.
#[derive(TelegramFunction, Serialize)]
#[function = "getMe"]
#[answer = "User"]
#[bot_function = "get_me"]
pub struct GetMe;

/// Use this method to send text messages. On success, the sent Message is returned.
#[derive(TelegramFunction,Serialize)]
#[function = "sendMessage"]
#[answer = "Message"]
#[bot_function = "send_message"]
pub struct Message {
    chat_id: Integer,
    text: String,
#[serde(skip_serializing_if="Option::is_none")]
    parse_mode: Option<String>,
#[serde(skip_serializing_if="Option::is_none")]
    disable_web_page_preview: Option<bool>,
#[serde(skip_serializing_if="Option::is_none")]
    disable_notificaton: Option<bool>,
#[serde(skip_serializing_if="Option::is_none")]
    reply_to_message_id: Option<Integer>,
#[serde(skip_serializing_if="Option::is_none")]
    reply_markup: Option<NotImplemented>
}


#[derive(TelegramFunction, Serialize)]
#[function = "getUpdates"]
#[answer = "Updates"]
#[bot_function = "get_updates"]
pub struct GetUpdates {
#[serde(skip_serializing_if="Option::is_none")]
    offset: Option<Integer>,
#[serde(skip_serializing_if="Option::is_none")]
    limit: Option<Integer>,
#[serde(skip_serializing_if="Option::is_none")]
    timeout: Option<Integer>,
#[serde(skip_serializing_if="Option::is_none")]
    allowed_updates: Option<Vec<String>>
}

/// Use this method to send photos. On success, the sent Message is returned.
#[derive(TelegramFunction, Serialize)]
#[function = "sendPhoto"]
#[answer = "Message"]
#[bot_function = "photo"]
pub struct SendPhoto {
    chat_id: Integer,
    photo: Option<String>,
#[serde(skip_serializing_if="Option::is_none")]
    caption: Option<String>,
#[serde(skip_serializing_if="Option::is_none")]
    disable_notification: Option<bool>,
#[serde(skip_serializing_if="Option::is_none")]
    reply_to_message_id: Option<bool>,
#[serde(skip_serializing_if="Option::is_none")]
    reply_markup: Option<NotImplemented>
}

/// Use this method to send audio files, if you want Telegram clients to display them in the music
/// player. Your audio must be in the .mp3 format. On success, the sent Message is returned. Bots
/// can currently send audio files of up to 50 MB in size, this limit may be changed in the future.
///
/// For sending voice messages, use the sendVoice method instead.
#[derive(TelegramFunction, Serialize)]
#[function = "sendAudio"]
#[answer = "Message"]
#[bot_function = "audio"]
pub struct SendAudio {
    chat_id: Integer,
    audio: Option<String>,
#[serde(skip_serializing_if="Option::is_none")]
    caption: Option<String>,
#[serde(skip_serializing_if="Option::is_none")]
    duration: Option<Integer>,
#[serde(skip_serializing_if="Option::is_none")]
    performer: Option<String>,
#[serde(skip_serializing_if="Option::is_none")]
    title: Option<String>,
#[serde(skip_serializing_if="Option::is_none")]
    disable_notification: Option<bool>,
#[serde(skip_serializing_if="Option::is_none")]
    reply_to_message_id: Option<Integer>,
#[serde(skip_serializing_if="Option::is_none")]
    reply_markup: Option<NotImplemented>
}

/// Use this method to send general files. On success, the sent Message is returned. Bots can
/// currently send files of any type of up to 50 MB in size, this limit may be changed in the
/// future.
#[derive(TelegramFunction, Serialize)]
#[function = "sendDocument"]
#[answer = "Message"]
#[bot_function = "document"]
pub struct SendDocument {
    chat_id: Integer,
    document: Option<String>,
#[serde(skip_serializing_if="Option::is_none")]
    caption: Option<String>,
#[serde(skip_serializing_if="Option::is_none")]
    disable_notification: Option<bool>,
#[serde(skip_serializing_if="Option::is_none")]
    reply_to_message_id: Option<Integer>,
#[serde(skip_serializing_if="Option::is_none")]
    reply_markup: Option<NotImplemented>
}

/// Use this method to send .webp stickers. On success, the sent Message is returned.
#[derive(TelegramFunction, Serialize)]
#[function = "sendSticker"]
#[answer = "Message"]
#[bot_function = "sticker"]
pub struct SendSticker {
    chat_id: Integer,
    sticker: Option<String>,
#[serde(skip_serializing_if="Option::is_none")]
    disable_notification: Option<bool>,
#[serde(skip_serializing_if="Option::is_none")]
    reply_to_message_id: Option<Integer>,
#[serde(skip_serializing_if="Option::is_none")]
    reply_markup: Option<NotImplemented>
}

/// Use this method to send video files, Telegram clients support mp4 videos (other formats may be
/// sent as Document). On success, the sent Message is returned. Bots can currently send video
/// files of up to 50 MB in size, this limit may be changed in the future.
#[derive(TelegramFunction, Serialize)]
#[function = "sendVideo"]
#[answer = "Message"]
#[bot_function = "video"]
pub struct SendVideo {
    chat_id: Integer,
    video: Option<String>,
#[serde(skip_serializing_if="Option::is_none")]
    duration: Option<Integer>,
#[serde(skip_serializing_if="Option::is_none")]
    width: Option<Integer>,
#[serde(skip_serializing_if="Option::is_none")]
    height: Option<Integer>,
#[serde(skip_serializing_if="Option::is_none")]
    caption: Option<String>,
#[serde(skip_serializing_if="Option::is_none")]
    disable_notification: Option<bool>,
#[serde(skip_serializing_if="Option::is_none")]
    reply_to_message_id: Option<Integer>,
#[serde(skip_serializing_if="Option::is_none")]
    reply_markup: Option<NotImplemented>
}

/// Use this method to send audio files, if you want Telegram clients to display the file as a
/// playable voice message. For this to work, your audio must be in an .ogg file encoded with OPUS
/// (other formats may be sent as Audio or Document). On success, the sent Message is returned.
/// Bots can currently send voice messages of up to 50 MB in size, this limit may be changed in the
/// future.
#[derive(TelegramFunction, Serialize)]
#[function = "sendVoice"]
#[answer = "Message"]
#[bot_function = "voice"]
pub struct SendVoice {
    chat_id: Integer,
    voice: Option<String>,
#[serde(skip_serializing_if="Option::is_none")]
    caption: Option<String>,
#[serde(skip_serializing_if="Option::is_none")]
    duration: Option<Integer>,
#[serde(skip_serializing_if="Option::is_none")]
    disable_notification: Option<bool>,
#[serde(skip_serializing_if="Option::is_none")]
    reply_to_message_id: Option<Integer>,
#[serde(skip_serializing_if="Option::is_none")]
    reply_markup: Option<NotImplemented>
}

/// Use this method to send point on the map. On success, the sent Message is returned.
#[derive(TelegramFunction, Serialize)]
#[function = "sendLocation"]
#[answer = "Message"]
#[bot_function = "location"]
pub struct SendLocation {
    chat_id: Integer,
    latitude: f32,
    longitude: f32,
#[serde(skip_serializing_if="Option::is_none")]
    disable_notification: Option<bool>,
#[serde(skip_serializing_if="Option::is_none")]
    reply_to_message_id: Option<Integer>,
#[serde(skip_serializing_if="Option::is_none")]
    reply_markup: Option<NotImplemented>
}

/// Use this method to send information about a venue. On success, the sent Message is returned.
#[derive(TelegramFunction, Serialize)]
#[function = "sendVenue"]
#[answer = "Message"]
#[bot_function = "venue"]
pub struct SendVenue {
    chat_id: Integer,
    latitude: f32,
    longitude: f32,
    title: String,
    address: String,
    foursquare_id: Option<String>,
#[serde(skip_serializing_if="Option::is_none")]
    disable_notification: Option<bool>,
#[serde(skip_serializing_if="Option::is_none")]
    reply_to_message_id: Option<Integer>,
#[serde(skip_serializing_if="Option::is_none")]
    reply_markup: Option<NotImplemented>
}

/// Use this method to send phone contacts. On success, the sent Message is returned.
#[derive(TelegramFunction, Serialize)]
#[function = "sendContact"]
#[answer = "Message"]
#[bot_function = "contact"]
pub struct SendContact {
    chat_id: Integer,
    phone_number: String,
    first_name: String,
    last_name: Option<String>,
#[serde(skip_serializing_if="Option::is_none")]
    disable_notification: Option<bool>,
#[serde(skip_serializing_if="Option::is_none")]
    reply_to_message_id: Option<Integer>,
#[serde(skip_serializing_if="Option::is_none")]
    reply_markup: Option<NotImplemented>
}

/// Use this method when you need to tell the user that something is happening on the bot's side.
/// The status is set for 5 seconds or less (when a message arrives from your bot, Telegram clients
/// clear its typing status). Returns True on success.
#[derive(TelegramFunction, Serialize)]
#[function = "sendChatAction"]
#[answer = "Boolean"]
#[bot_function = "chat_action"]
pub struct SendAction {
    chat_id: Integer,
    action: String
}

/// Use this method to get a list of profile pictures for a user. Returns a UserProfilePhotos
/// object.
#[derive(TelegramFunction, Serialize)]
#[function = "getUserProfilePhotos"]
#[answer = "UserProfilePhotos"]
#[bot_function = "get_user_profile_photos"]
pub struct GetUserProfilePhotos {
    user_id: Integer,
#[serde(skip_serializing_if="Option::is_none")]
    offset: Option<Integer>,
#[serde(skip_serializing_if="Option::is_none")]
    limit: Option<Integer>
}

/// Use this method to get basic info about a file and prepare it for downloading. For the moment,
/// bots can download files of up to 20MB in size. On success, a File object is returned. The file
/// can then be downloaded via the link https://api.telegram.org/file/bot<token>/<file_path>, where
/// <file_path> is taken from the response. It is guaranteed that the link will be valid for at
/// least 1 hour. When the link expires, a new one can be requested by calling getFile again.
#[derive(TelegramFunction, Serialize)]
#[function = "getFile"]
#[answer = "File"]
#[bot_function = "get_file"]
pub struct GetFile {
    file_id: String
}

/// Use this method to kick a user from a group or a supergroup. In the case of supergroups, the
/// user will not be able to return to the group on their own using invite links, etc., unless
/// unbanned first. The bot must be an administrator in the group for this to work. Returns True on
/// success.
#[derive(TelegramFunction, Serialize)]
#[function = "kickChatMember"]
#[answer = "Boolean"]
#[bot_function = "kick_chat_member"]
pub struct KickChatMember {
    chat_id: Integer,
    user_id: Integer
}

/// Use this method for your bot to leave a group, supergroup or channel. Returns True on
/// success.
#[derive(TelegramFunction, Serialize)]
#[function = "leaveChat"]
#[answer = "Boolean"]
#[bot_function = "leave_chat"]
pub struct LeaveChat {
    chat_id: Integer,
}

/// Use this method to unban a previously kicked user in a supergroup. The user will not return to
/// the group automatically, but will be able to join via link, etc. The bot must be an
/// administrator in the group for this to work. Returns True on success.
#[derive(TelegramFunction, Serialize)]
#[function = "unbanChatMember"]
#[answer = "Boolean"]
#[bot_function = "unban_chat_member"]
pub struct UnbanChatMember {
    chat_id: Integer,
    user_id: Integer
}

/// Use this method to get up to date information about the chat (current name of the user for
/// one-on-one conversations, current username of a user, group or channel, etc.). Returns a Chat
/// object on success.
#[derive(TelegramFunction, Serialize)]
#[function = "getChat"]
#[answer = "Chat"]
#[bot_function = "get_chat"]
pub struct GetChat {
    chat_id: Integer
}

/// Use this method to get a list of administrators in a chat. On success, returns an Array of
/// ChatMember objects that contains information about all chat administrators except other bots.
/// If the chat is a group or a supergroup and no administrators were appointed, only the creator
/// will be returned.
#[derive(TelegramFunction, Serialize)]
#[function = "getChatAdministrators"]
#[answer = "Vector<objects::ChatMember>"]
#[bot_function = "unban_chat_administrators"]
pub struct GetChatAdministrators {
    chat_id: Integer
}

/// Use this method to get the number of members in a chat. Returns Int on success.
#[derive(TelegramFunction, Serialize)]
#[function = "getChatMembersCount"]
#[answer = "Integer"]
#[bot_function = "get_chat_members_count"]
pub struct GetChatMemberCounts {
    chat_id: Integer
}

/// Use this method to get information about a member of a chat. Returns a ChatMember object on
/// success.
#[derive(TelegramFunction, Serialize)]
#[function = "getChatMember"]
#[answer = "ChatMember"]
#[bot_function = "get_chat_members_count"]
pub struct GetChatMember {
    chat_id: Integer,
    user_id: Integer
}

/// Use this method to send answers to callback queries sent from inline keyboards. The answer will
/// be displayed to the user as a notification at the top of the chat screen or as an alert. On
/// success, True is returned.
#[derive(TelegramFunction, Serialize)]
#[function = "answerCallbackQuery"]
#[answer = "Boolean"]
#[bot_function = "answer_callback_query"]
pub struct AnwerCallbackQuery {
    callback_query_id: String,
    text: Option<String>,
    show_alert: Option<bool>,
    url: Option<String>,
    cache_time: Option<Integer>
}
