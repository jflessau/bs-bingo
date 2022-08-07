use crate::{error::Result, AppState, Identity};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension, Path, TypedHeader,
    },
    response::IntoResponse,
    Json,
};
use tokio::time::{sleep, Duration};

use rand::{distributions::Alphanumeric, seq::SliceRandom, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
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
    is_me: bool,
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

pub async fn ws(
    ws: WebSocketUpgrade,
    _user_agent: Option<TypedHeader<headers::UserAgent>>,
    identity: Identity,
    Path(game_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
) -> impl IntoResponse {
    let pool = state.pool;

    ws.on_upgrade(move |socket| async move {
        handle_socket(socket, &pool, identity.user_id, game_id).await
    })
}

async fn handle_socket(mut socket: WebSocket, pool: &PgPool, user_id: Uuid, game_id: Uuid) {
    let mut socket_healthy = true;

    while socket_healthy {
        sleep(Duration::from_millis(100)).await;

        // check if game exists

        let game = sqlx::query!(
            r#"
                select 
                    g.game_template_id
                from 
                    bingo.games g
                inner join
                    bingo.players p on p.game_id = g.id
                where 
                    p.user_id = $1 and g.id = $2 and closed = false
            "#,
            user_id,
            game_id,
        )
        .fetch_one(pool)
        .await
        .expect("checking game existence failes");

        let mut messages = Vec::new();

        // get players

        let players = get_players(game_id, user_id, pool)
            .await
            .expect("get_players failes");
        messages.push(
            serde_json::to_string(&MessageOut::Players(players))
                .expect("Fails to serialize MessageOut."),
        );

        // get fields

        let fields = get_fields(game.game_template_id, game_id, user_id, pool)
            .await
            .expect("get_players failes");
        messages.push(
            serde_json::to_string(&MessageOut::Fields(fields))
                .expect("Fails to serialize MessageOut."),
        );

        for message in messages {
            if let Err(err) = socket.send(Message::Text(message)).await {
                tracing::warn!("Failed to send message: {:?}", err);
                socket_healthy = false;
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct GameOut {
    pub id: Uuid,
    pub open: bool,
    pub access_code: String,
    pub fields: Vec<Vec<FieldOut>>,
    pub players: Vec<PlayerOut>,
    pub username: String,
}

pub async fn handle_start_game(
    identity: Identity,
    Path(game_template_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
) -> Result<Json<GameOut>> {
    let pool = state.pool;
    let user_id = identity.user_id;

    let game = sqlx::query!(
        r#"
            select
                g.id,
                g.access_code
            from bingo.games g
            join bingo.players p on p.game_id = g.id
            where 
                g.game_template_id = $1
                and p.user_id = $2
        "#,
        game_template_id,
        user_id,
    )
    .fetch_optional(&pool)
    .await?;

    if let Some(game) = game {
        join_game(user_id, game.access_code, &pool).await
    } else {
        let _game_template = sqlx::query!(
            r#"
                select * from bingo.game_templates
                where id = $1 and (created_by = $2 or approved = true)
            "#,
            game_template_id,
            user_id
        )
        .fetch_one(&pool)
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
        .fetch_one(&pool)
        .await?;

        let fields = get_fields(game.game_template_id, game.id, user_id, &pool).await?;

        let players = get_players(game.id, user_id, &pool).await?;

        let username = players
            .iter()
            .find(|v| v.is_me)
            .map(|v| v.username.clone())
            .unwrap_or_else(|| "unknown".to_string());

        Ok(Json(GameOut {
            id: game.id,
            open: true,
            access_code: game.access_code,
            fields,
            players,
            username,
        }))
    }
}

pub async fn handle_join_game(
    identity: Identity,
    Path(access_code): Path<String>,
    Extension(state): Extension<AppState>,
) -> Result<Json<GameOut>> {
    let pool = state.pool;
    let user_id = identity.user_id;

    join_game(user_id, access_code, &pool).await
}

pub async fn join_game(user_id: Uuid, access_code: String, pool: &PgPool) -> Result<Json<GameOut>> {
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

    let fields = get_fields(game.game_template_id, game.id, user_id, pool).await?;

    let players = get_players(game.id, user_id, pool).await?;

    let username = players
        .iter()
        .find(|v| v.is_me)
        .map(|v| v.username.clone())
        .unwrap_or_else(|| "unknown".to_string());

    Ok(Json(GameOut {
        id: game.id,
        open: !game.closed,
        access_code: game.access_code,
        fields,
        players,
        username,
    }))
}

pub async fn handle_update_field(
    identity: Identity,
    Path(id): Path<Uuid>,
    Extension(state): Extension<AppState>,
) -> Result<()> {
    let pool = state.pool;
    let user_id = identity.user_id;

    let _game = sqlx::query!(
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
    .fetch_one(&pool)
    .await?;

    sqlx::query!(
        "update bingo.fields set checked = not checked where id = $1",
        id
    )
    .execute(&pool)
    .await?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct UsernameIn {
    username: String,
}

pub async fn handle_update_username(
    identity: Identity,
    Json(username): Json<UsernameIn>,
    Path(game_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
) -> Result<()> {
    let pool = state.pool;
    let user_id = identity.user_id;

    sqlx::query!(
        "update bingo.players set username = $1 where user_id = $2 and game_id = $3",
        username.username,
        user_id,
        game_id
    )
    .execute(&pool)
    .await?;

    Ok(())
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

    if existing_fields.is_empty() {
        create_fields(game_template_id, game_id, user_id, pool).await?;
        sqlx::query!(
            r#"
                insert into bingo.players ("user_id", game_id, "username")
                values ($1, $2, $3)
            "#,
            user_id,
            game_id,
            "Anonymous player",
        )
        .execute(pool)
        .await?;
    }

    let fields = sqlx::query!(
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
    });

    // TODO: figure our how to solve this with iter().chunks(5)

    let mut result: Vec<Vec<FieldOut>> = Vec::new();
    let mut v: Vec<FieldOut> = Vec::new();
    for (i, field) in fields.enumerate() {
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

async fn get_players(game_id: Uuid, user_id: Uuid, pool: &PgPool) -> Result<Vec<PlayerOut>> {
    let mut players = sqlx::query!(
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
            order by array_agg(f.checked) desc
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
        hits: v.hits.unwrap_or_default(),
        is_me: v.user_id == user_id,
    })
    .collect::<Vec<PlayerOut>>();

    players.sort_by(|a, b| {
        b.hits
            .iter()
            .filter(|v| **v)
            .count()
            .partial_cmp(&a.hits.iter().filter(|v| **v).count())
            .unwrap()
    });

    Ok(players)
}
