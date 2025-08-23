use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub username: &'a str,
    pub messages: Vec<MessageView>, // (username, message)
}

pub struct MessageView {
    pub username: String,
    pub text: String,
}