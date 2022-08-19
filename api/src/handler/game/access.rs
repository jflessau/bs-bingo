use crate::{
    body::GameOut,
    error::{Error, Result},
    handler::game::{field::prepare_fields, player::get_players},
    server::{AppState, Identity},
};
use axum::{
    extract::{Extension, Path},
    Json,
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sqlx::postgres::PgPool;
use uuid::Uuid;

pub async fn handle_start_game(
    identity: Identity,
    Path((game_template_id, grid_size)): Path<(Uuid, i64)>,
    Extension(state): Extension<AppState>,
) -> Result<Json<GameOut>> {
    let pool = &state.pool;
    let user_id = identity.user_id;

    if !(2..=8).contains(&grid_size) {
        return Err(Error::BadRequest(
            "Grid size must be grater than one and less than 9.".to_string(),
        ));
    }

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
    .fetch_optional(pool)
    .await?;

    if let Some(game) = game {
        join_game(user_id, game.access_code, pool).await
    } else {
        let game_template = sqlx::query!(
            r#"
                select id from bingo.game_templates
                where id = $1 and (created_by = $2 or approved = true)
            "#,
            game_template_id,
            user_id
        )
        .fetch_one(pool)
        .await?;

        let field_amount = sqlx::query!(
            "select count(id) amount from bingo.field_templates where game_template_id = $1",
            game_template.id
        )
        .fetch_one(pool)
        .await?
        .amount
        .unwrap_or(0);

        if field_amount < grid_size {
            return Err(Error::BadRequest(format!(
                "Game template has not enough fields ({}) for the selected grid size of {}.",
                field_amount, grid_size
            )));
        }

        let game_access_code: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();

        let game = sqlx::query!(
            r#"
                insert into bingo.games (game_template_id, access_code, grid_size, created_by)
                values ($1, $2, $3, $4)
                returning *
            "#,
            game_template_id,
            game_access_code,
            grid_size as i32,
            user_id
        )
        .fetch_one(pool)
        .await?;

        let fields =
            prepare_fields(game.game_template_id, game.id, user_id, grid_size, pool).await?;

        let players = get_players(game.id, user_id, pool).await?;

        let username = players
            .iter()
            .find(|v| v.is_me)
            .map(|v| v.username.clone())
            .unwrap_or_else(|| {
                tracing::warn!("could not find user in list of players");
                "unknown".to_string()
            });

        Ok(Json(GameOut {
            id: game.id,
            open: true,
            continued: false,
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

pub async fn join_game(user_id: Uuid, access_code: String, pool: &PgPool) -> Result<Json<GameOut>> {
    let game = sqlx::query!(
        r#"
            select 
                gt.id as game_template_id, 
                g.id,
                g.closed,
                g.access_code,
                g.grid_size
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

    let fields = prepare_fields(
        game.game_template_id,
        game.id,
        user_id,
        game.grid_size.into(),
        pool,
    )
    .await?;

    let players = get_players(game.id, user_id, pool).await?;

    let username = players
        .iter()
        .find(|v| v.is_me)
        .map(|v| v.username.clone())
        .unwrap_or_else(|| {
            tracing::warn!("could not find user in list of players");
            "unknown".to_string()
        });

    Ok(Json(GameOut {
        id: game.id,
        open: !game.closed,
        continued: true,
        access_code: game.access_code,
        fields,
        players,
        username,
    }))
}
