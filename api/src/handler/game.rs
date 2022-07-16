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
use itertools::Itertools;
use rand::{distributions::Alphanumeric, seq::SliceRandom, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::sync::mpsc::{channel, Receiver, Sender};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldOut {
    id: Uuid,
    text: String,
    position: u32,
    checked: bool,
    bingo: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerOut {
    user_id: Uuid,
    username: String,
    bingos: i32,
    hits: Vec<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageIn {
    #[serde(rename_all = "camelCase")]
    StartGame { game_template_id: Uuid },
    #[serde(rename_all(deserialize = "camelCase"))]
    JoinGame { access_code: String },
    #[serde(rename_all = "camelCase")]
    FieldUpdate { id: Uuid, checked: bool },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub enum MessageOut {
    #[serde(rename_all(serialize = "camelCase"))]
    GameUpdate {
        id: Uuid,
        open: bool,
        access_code: String,
    },
    #[serde(rename_all(serialize = "camelCase"))]
    FieldsUpdate(Vec<Vec<FieldOut>>),
    #[serde(rename_all(serialize = "camelCase"))]
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
    tracing::info!("handle socket");

    let (tx, rx) = channel();

    tokio::join!(digest(&pool, user_id, tx), send_messages(socket, rx));
}

async fn send_messages(mut socket: WebSocket, rx: Receiver<Vec<String>>) {
    loop {
        if let Ok(responses) = rx.recv() {
            for response in responses {
                if let Err(err) = socket.send(Message::Text(response)).await {
                    tracing::error!("Failed to send response: {:?}", err);
                }
            }
        }
    }
}

async fn digest(pool: &PgPool, user_id: Uuid, tx: Sender<Vec<String>>) {
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(text) => match respond(text, &pool, user_id).await {
                    Ok(responses) => {
                        tracing::info!("responses: {}", responses.len());
                        tx.send(responses)
                            .expect("Sending responses via mpsc failes.");
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
    tracing::info!("text: {}", text);
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
        id: game.id,
        open: true,
        access_code: game.access_code,
    });

    let fields = get_fields(game.game_template_id, game.id, user_id, pool).await?;
    responses.push(MessageOut::FieldsUpdate(fields));

    let players = get_players(game.id, pool).await?;
    responses.push(MessageOut::PlayersUpdate(players));

    Ok(responses
        .iter()
        .map(|v| serde_json::to_string(&v).expect("Fails to serialize MessageOut."))
        .collect::<Vec<String>>())
}

async fn join_game(access_code: String, user_id: Uuid, pool: &PgPool) -> Result<Vec<String>> {
    let mut responses: Vec<MessageOut> = Vec::new();

    let game = sqlx::query!(
        r#"
            select 
                gt.id as game_template_id, 
                g.id as id,
                g.closed as closed,
                g.access_code as access_code
            from 
                bingo.games as g
            inner join
                bingo.game_templates as gt on g.game_template_id = gt.id
            where 
                g.access_code = $1
        "#,
        access_code
    )
    .fetch_one(pool)
    .await?;

    responses.push(MessageOut::GameUpdate {
        id: game.id,
        open: !game.closed,
        access_code: game.access_code,
    });

    let fields = get_fields(game.game_template_id, game.id, user_id, pool).await?;
    responses.push(MessageOut::FieldsUpdate(fields));

    let players = get_players(game.id, pool).await?;
    responses.push(MessageOut::PlayersUpdate(players));

    Ok(responses
        .iter()
        .map(|v| serde_json::to_string(&v).expect("Fails to serialize MessageOut."))
        .collect::<Vec<String>>())
}

async fn update_field(
    id: Uuid,
    checked: bool,
    user_id: Uuid,
    pool: &PgPool,
) -> Result<Vec<String>> {
    let mut responses: Vec<MessageOut> = Vec::new();

    let game = sqlx::query!(
        r#"
            select
                g.id as id,
                g.game_template_id as game_template_id
            from 
                bingo.fields as f
            inner join 
                bingo.games as g on f.game_id = g.id
            where 
                f.id = $1 and f.user_id = $2 and g.closed = false
        "#,
        id,
        user_id,
    )
    .fetch_one(pool)
    .await?;

    sqlx::query!(
        "update bingo.fields set checked = $1 where id = $2",
        checked,
        id
    )
    .execute(pool)
    .await?;

    let fields = get_fields(game.game_template_id, game.id, user_id, pool).await?;
    responses.push(MessageOut::FieldsUpdate(fields));

    let players = get_players(game.id, pool).await?;
    responses.push(MessageOut::PlayersUpdate(players));

    Ok(responses
        .iter()
        .map(|v| serde_json::to_string(&v).expect("Fails to serialize MessageOut."))
        .collect::<Vec<String>>())
}

async fn create_fields(
    game_template_id: Uuid,
    game_id: Uuid,
    user_id: Uuid,
    pool: &PgPool,
) -> Result<()> {
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
        sqlx::query!(
            r#"
                insert into bingo.fields (game_id, field_template_id, position, user_id)
                values ($1, $2, $3, $4)
            "#,
            game_id,
            field_template.id,
            i as i16,
            user_id,
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn get_fields(
    game_template_id: Uuid,
    game_id: Uuid,
    user_id: Uuid,
    pool: &PgPool,
) -> Result<Vec<Vec<FieldOut>>> {
    let existing_fields = sqlx::query!(
        r#"
            select 
                f.id
            from bingo.fields as f
            inner join bingo.field_templates as ft 
                on f.field_template_id = ft.id
            where 
                f.game_id = $1 and f.user_id = $2
            order by 
                position
        "#,
        game_id,
        user_id,
    )
    .fetch_all(pool)
    .await?;

    tracing::info!("existing_fields.len(): {}", existing_fields.len());

    if existing_fields.is_empty() {
        tracing::info!("create fields and players");

        create_fields(game_template_id, game_id, user_id, pool).await?;
        sqlx::query!(
            r#"
                insert into bingo.players ("user_id", game_id, "username")
                values ($1, $2, $3)
            "#,
            user_id,
            game_id,
            "unknown player",
        )
        .execute(pool)
        .await?;
    } else {
        tracing::info!("list fields and players");
    }

    let mut fields = sqlx::query!(
        r#"
            select 
                f.id as id,
                f.checked as checked,
                ft.caption as caption
            from bingo.fields as f
            inner join bingo.field_templates as ft 
                on f.field_template_id = ft.id
            where 
                f.game_id = $1 and f.user_id = $2
            order by 
                position
        "#,
        game_id,
        user_id,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|v| FieldOut {
        id: v.id,
        text: v.caption,
        position: 0,
        checked: v.checked,
        bingo: false,
    })
    .collect::<Vec<FieldOut>>();

    // TODO: figure our how to solve this with iter().chunks(5)

    let mut result: Vec<Vec<FieldOut>> = Vec::new();
    let mut v: Vec<FieldOut> = Vec::new();
    for (i, field) in fields.into_iter().enumerate() {
        if i % 5 == 0 {
            if i == 5 {
                result = vec![v]
            } else {
                result.push(v);
            }
            v = vec![field];
        } else {
            v.push(field);
        }
    }

    result.push(v);

    Ok(result)
}

async fn get_players(game_id: Uuid, pool: &PgPool) -> Result<Vec<PlayerOut>> {
    let players = sqlx::query!(
        r#"
            select
                p.user_id as user_id,
                p.username as "username",
                array_agg(f.checked order by f.position asc) as hits
            from 
                bingo.players as p
            join bingo.fields as f on f.user_id = p.user_id
            join bingo.field_templates as ft on f.field_template_id = ft.id
            where 
                p.game_id = $1 and f.game_id = $1
            
            group by p.user_id, p.username
        "#,
        game_id,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|v| PlayerOut {
        user_id: v.user_id,
        username: v.username,
        bingos: 0, // TODO
        hits: v.hits.unwrap_or(vec![]),
    })
    .collect::<Vec<PlayerOut>>();

    Ok(players)
}
