use crate::{
    body::{FieldOut, GameOut, MessageOut, PlayerOut, UsernameIn},
    error::Result,
    server::{AppState, Identity},
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension, Path, TypedHeader,
    },
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use rand::{distributions::Alphanumeric, seq::SliceRandom, thread_rng, Rng};
use sqlx::postgres::PgPool;
use uuid::Uuid;

pub async fn ws(
    ws: WebSocketUpgrade,
    _user_agent: Option<TypedHeader<headers::UserAgent>>,
    identity: Identity,
    Path(game_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        handle_socket(socket, &state, identity.user_id, game_id).await
    })
}

async fn handle_socket(socket: WebSocket, state: &AppState, user_id: Uuid, game_id: Uuid) {
    if let Err(err) = send_game_update_messages(socket, state, user_id, game_id).await {
        // if let Err(err) = socket.close().await {
        //     tracing::error!("closing socket failes: {:?}", err);
        // }
        tracing::error!("sending game update messgages failes: {:?}", err);
    }
}

pub async fn send_game_update_messages(
    mut socket: WebSocket,
    state: &AppState,
    user_id: Uuid,
    game_id: Uuid,
) -> Result<Vec<String>> {
    let pool = &state.pool;
    let mut receiver = state.receiver.clone();
    let mut socket_healthy = true;

    // check if game exists

    let game = sqlx::query!(
        r#"
            select 
                g.id,
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
    .await?;

    // let mut listener = PgListener::connect_with(pool).await?;

    // listener
    //     .listen_all(vec!["fields_update", "players_update"])
    //     .await?;

    // while socket is healthy: listen for postgres notifications and send respective updates to client

    let mut latest_game_update_at = Utc::now() - Duration::days(1);

    while socket_healthy {
        // sleep(TokioDuration::from_millis(100)).await;

        if receiver.changed().await.is_ok() {
            let game_updated_recently = receiver
                .borrow()
                .get(&game_id)
                .map(|v| v > &latest_game_update_at)
                .unwrap_or(false);

            if game_updated_recently {
                latest_game_update_at = Utc::now();

                let mut messages = Vec::new();

                if game_id == game.id {
                    let fields = get_fields(game.game_template_id, game_id, user_id, pool).await?;
                    messages.push(serde_json::to_string(&MessageOut::Fields(fields))?);

                    let players = get_players(game_id, user_id, pool).await?;
                    messages.push(serde_json::to_string(&MessageOut::Players(players))?);

                    for message in messages {
                        if let Err(err) = socket.send(Message::Text(message)).await {
                            tracing::warn!("Failed to send message: {:?}", err);
                            socket_healthy = false;
                        }
                    }
                }
            }
        } else {
            tracing::warn!("sender has been dropped");
        }
    }

    Ok(vec![])
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

pub async fn handle_leave_game(
    identity: Identity,
    Path(game_template_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
) -> Result<()> {
    let pool = state.pool;
    let user_id = identity.user_id;

    let mut transaction = pool.begin().await?;

    let game_ids = sqlx::query!(
        "select id from bingo.games where game_template_id = $1",
        game_template_id
    )
    .fetch_all(&mut transaction)
    .await?
    .into_iter()
    .map(|v| v.id)
    .collect::<Vec<Uuid>>();

    sqlx::query!(
        r#"
            delete from 
                bingo.players 
            where 
                game_id = any($1)
                and user_id = $2
        "#,
        &game_ids,
        &user_id
    )
    .execute(&mut transaction)
    .await?;

    sqlx::query!(
        r#"
            delete from 
                bingo.fields 
            where 
                game_id = any($1)
                and user_id = $2
        "#,
        &game_ids,
        &user_id
    )
    .execute(&mut transaction)
    .await?;

    transaction.commit().await?;

    Ok(())
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

async fn create_fields(
    game_template_id: Uuid,
    game_id: Uuid,
    user_id: Uuid,
    pool: &PgPool,
) -> Result<()> {
    let mut field_template_ids = sqlx::query!(
        r#"
            select id from bingo.field_templates
            where game_template_id = $1
        "#,
        game_template_id
    )
    .fetch_all(pool)
    .await?
    .iter()
    .map(|v| v.id)
    .collect::<Vec<Uuid>>();

    field_template_ids.shuffle(&mut thread_rng());
    let field_template_ids = &field_template_ids[0..25].to_vec();

    for (i, field_template_id) in field_template_ids.iter().enumerate() {
        sqlx::query!(
            r#"
                insert into bingo.fields (game_id, field_template_id, position, user_id)
                values ($1, $2, $3, $4)
            "#,
            game_id,
            field_template_id,
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

    // TODO: figure our how to solve this with .chunks(5)

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
                p.game_id = $1 
                and f.game_id = $1
            group by 
                p.user_id, 
                p.username
            order by 
                array_agg(f.checked) desc, 
                "username" desc
        "#,
        game_id,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|v| PlayerOut {
        user_id: v.user_id,
        username: v.username,
        bingos: calc_bingos(v.hits.clone().unwrap_or_default()),
        hits: v.hits.unwrap_or_default(),
        is_me: v.user_id == user_id,
    })
    .collect::<Vec<PlayerOut>>();

    players.sort_by(|a, b| {
        let a_hits = a.hits.iter().filter(|v| **v).count();

        let b_hits = b.hits.iter().filter(|v| **v).count();

        if a.bingos == b.bingos {
            b_hits.partial_cmp(&a_hits).unwrap()
        } else {
            b.bingos.partial_cmp(&a.bingos).unwrap()
        }
    });

    Ok(players)
}

fn calc_bingos(hits: Vec<bool>) -> i32 {
    if hits.len() != 25 {
        tracing::warn!("player has more or less than 25 fields: {}", hits.len());
        return 0;
    }

    let mut bingos = 0;

    // check rows

    for x in 0..5 {
        let x = x * 5;
        if (0..5).all(|y| hits[x + y]) {
            bingos += 1;
        }
    }

    // check columns

    for x in 0..5 {
        if (0..5).all(|y| {
            let y = y * 5;
            hits[x + y]
        }) {
            bingos += 1;
        }
    }

    // check top left to bottom right

    let mut n: i32 = -6;
    if (0..5).all(|_| {
        n += 6;
        hits[n as usize]
    }) {
        bingos += 1;
    }

    // check top right to bottom left

    let mut n: i32 = 0;
    if (0..5).all(|_| {
        n += 4;
        hits[n as usize]
    }) {
        bingos += 1;
    }

    bingos
}
