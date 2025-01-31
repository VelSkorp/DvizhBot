#[derive(Debug)]
pub enum MsgType {
    GetMe,
    GetUpdates,
    SendMessage,
    SendPhoto,
    EditMessageText,
    EditMessageReplyMarkup,
    GetChatAdministrators,
    BanChatMember,
}

pub fn msg_type_to_str(t: &MsgType) -> &'static str {
    match t {
        MsgType::GetMe => "getMe",
        MsgType::GetUpdates => "getUpdates",
        MsgType::SendMessage => "sendMessage",
        MsgType::SendPhoto => "sendPhoto",
        MsgType::EditMessageText => "editMessageText",
        MsgType::EditMessageReplyMarkup => "editMessageReplyMarkup",
        MsgType::GetChatAdministrators => "getChatAdministrators",
        MsgType::BanChatMember => "banChatMember",
    }
}
