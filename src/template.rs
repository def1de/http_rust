use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub username: &'a str,
    pub chats: Vec<ChatView>, // (id, name)
}

pub struct ChatView {
    pub id: i64,
    pub name: String,
}

#[derive(Template)]
#[template(path = "chat.html")]
pub struct ChatTemplate<'a> {
    pub username: &'a str,
    pub messages: Vec<MessageView>, // (username, message)
    pub chats: Vec<ChatView>,   // (id, name)
}

pub struct MessageView {
    pub username: String,
    pub text: String,
}