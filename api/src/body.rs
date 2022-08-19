use serde::{Deserialize, Serialize};
use uuid::Uuid;

// template

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateOut {
    pub id: Uuid,
    pub title: String,
    pub field_amount: i64,
    pub player_amount: i64,
    pub owned: bool,
    pub approved: bool,
    pub access_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateIn {
    pub title: String,
    pub fields: Vec<String>,
}

// game

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct GameOut {
    pub id: Uuid,
    pub open: bool,
    pub continued: bool,
    pub access_code: String,
    pub fields: Vec<Vec<FieldOut>>,
    pub players: Vec<PlayerOut>,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldOut {
    pub id: Uuid,
    pub text: String,
    pub position: u32,
    pub checked: bool,
    pub bingo: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerOut {
    pub user_id: Uuid,
    pub username: String,
    pub bingos: i32,
    pub hits: Vec<bool>,
    pub is_me: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub enum MessageOut {
    #[serde(rename_all(serialize = "camelCase"))]
    Game {
        id: Uuid,
        open: bool,
        access_code: String,
    },
    #[serde(rename_all(serialize = "camelCase"))]
    Fields(Vec<Vec<FieldOut>>),
    #[serde(rename_all(serialize = "camelCase"))]
    Players(Vec<PlayerOut>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct UsernameIn {
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct PgGameUpdateNotification {
    pub game_id: Uuid,
}
