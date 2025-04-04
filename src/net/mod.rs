use std::sync::LazyLock;

use reqwest::Client;

pub mod manifest;
pub mod meta;

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent(crate::USER_AGENT)
        .build()
        .unwrap()
});

pub enum GameSide {
    Client,
    Server,
}

impl GameSide {
    fn id(&self) -> &str {
        match self {
            GameSide::Client => "client",
            GameSide::Server => "server",
        }
    }
}
