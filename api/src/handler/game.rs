use crate::{
    error::{Error, Result},
    AppState, Identity,
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension, TypedHeader,
    },
    response::IntoResponse,
};
use rand::{distributions::Alphanumeric, seq::SliceRandom, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldOut {
    id: Uuid,
    position: u32,
    checked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerOut {
    id: Uuid,
    name: String,
    bingos: i32,
    hits: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageIn {
    StartGame { game_template_id: Uuid },
    JoinGame { access_code: String },
    FieldUpdate { id: Uuid, checked: bool },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageOut {
    GameUpdate { id: Uuid, open: bool, access_code: String },
    FieldsUpdate(Vec<FieldOut>),
    PlayersUpdate(Vec<PlayerOut>),
}

pub async fn ws(
    ws: WebSocketUpgrade,
    _user_agent: Option<TypedHeader<headers::UserAgent>>,
    identity: Identity,
    Extension(state): Extension<AppState>,
) -> impl IntoResponse {
    let pool = state.pool;

    ws.on_upgrade(move |socket| async move { handle_socket(socket, pool, identity.user_id).await })
}

async fn handle_socket(mut socket: WebSocket, pool: PgPool, user_id: Uuid) {
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(text) => match respond(text, &pool, user_id).await {
                    Ok(responses) => {
                        for response in responses {
                            if let Err(err) = socket.send(Message::Text(response)).await {
                                tracing::error!("Failed to send response: {:?}", err);
                            }
                        }
                    }
                    Err(err) => {
                        tracing::error!("Failed to get responses: {:?}", err)
                    }
                },
                Message::Binary(_) => {}
                Message::Ping(_) => {}
                Message::Pong(_) => {}
                Message::Close(_) => {}
            }
        } else {
            tracing::debug!("client disconnected");
            return;
        }
    }
}

async fn respond(text: String, pool: &PgPool, user_id: Uuid) -> Result<Vec<String>> {
    let message: MessageIn = serde_json::from_str(&text)?;

    match message {
        MessageIn::StartGame { game_template_id } => {
            start_game(game_template_id, user_id, pool).await
        }
        MessageIn::JoinGame { access_code } => join_game(access_code, user_id, pool).await,
        MessageIn::FieldUpdate { id, checked } => update_field(id, checked, user_id, pool).await,
    }
}

async fn start_game(game_template_id: Uuid, user_id: Uuid, pool: &PgPool) -> Result<Vec<String>> {
    let mut responses: Vec<MessageOut> = Vec::new();

    // Create game

    let game_template = sqlx::query!(
        r#"
            select * from bingo.game_templates
            where id = $1 and (created_by = $2 or approved = true)
        "#,
        game_template_id,
        user_id
    )
    .fetch_one(pool)
    .await?;

    let game_access_code: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    let game = sqlx::query!(
        r#"
            insert into bingo.games (game_template_id, access_code, created_by)
            values ($1, $2, $3)
            returning *
        "#,
        game_template_id,
        game_access_code,
        user_id
    )
    .fetch_one(pool)
    .await?;

    responses.push(MessageOut::GameUpdate {
        id: game.id
        open: true,
        access_code: game.access_code,
    });

    // Create fields

    let mut field_templates = sqlx::query!(
        r#"
            select * from bingo.field_templates
            where game_template_id = $1
        "#,
        game_template_id
    )
    .fetch_all(pool)
    .await?;

    field_templates.shuffle(&mut thread_rng());

    let mut fields: Vec<FieldOut> = Vec::new();
    for (i, field_template) in field_templates.iter().enumerate() {
        let field = sqlx::query!(
            r#"
                insert into bingo.fields (game_id, field_template_id, user_id)
                values ($1, $2, $3)
                returning * 
            "#,
            game.id,
            field_template.id,
            user_id,
        )
        .fetch_one(pool)
        .await?;

        fields.push(FieldOut {
            id: field.id,
            position: i as u32,
            checked: false,
        })
    }

    responses.push(MessageOut::FieldsUpdate(fields));

    // TODO: get players and add them to response

    Ok(responses
        .iter()
        .map(|v| serde_json::to_string(&v).expect("Failes to serialize MessageOut."))
        .collect::<Vec<String>>())
}

async fn join_game(code: String, user_id: Uuid, pool: &PgPool) -> Result<Vec<String>> {
    // TODO: get game
    // TODO: check code
    // TODO: get field_templates
    // TODO: create fields
    // TODO: get players
    // TODO: respond with game, fields and players
    Ok(vec!["Hi".to_string()])
}

async fn update_field(
    id: Uuid,
    checked: bool,
    user_id: Uuid,
    pool: &PgPool,
) -> Result<Vec<String>> {
    // let field = sqlx::query!(
    //     r#"
    //         select id from bingo.games where id = $1 and closed = false
    //     "#,
        
    // )
    // TODO: ensure game is was not closed
    // TODO: update field
    // TODO: respond with fields

    Ok(vec!["Hi".to_string()])
}
