use crate::{
    body::{TemplateIn, TemplateOut},
    error::{Error, Result},
    server::{AppState, Identity},
};
use axum::{
    extract::{Extension, Path},
    Json,
};
use std::str;
use uuid::Uuid;

pub async fn handle_list_templates(
    Extension(state): Extension<AppState>,
    identity: Identity,
) -> Result<Json<Vec<TemplateOut>>> {
    let pool = &state.pool;

    let templates = sqlx::query!(
        r#"
            select
                sq.id,
                sq.title,
                sq.field_amount,
                sq.player_amount,
                sq.owned,
                sq.public,
                sq.startable,
                sq.access_code "access_code?"
            from
                (
                    select
                        distinct on (gt.id) 
                        gt.id,
                        gt.title,
                        ft.field_amount,
                        coalesce(g.player_amount, 0) player_amount,
                        gt.created_by = $1 owned,
                        (
                            gt.public
                            and gt.approved
                        ) public,
                        gt.created_by = $1
                        or(
                            gt.public
                            and gt.approved
                        ) startable,
                        joinable_game.access_code,
                        joinable_game_player.game_id joinable_game_player_game_id
                    from
                        bingo.game_templates gt
                        
                        left outer join bingo.games active_game on active_game.game_template_id = gt.id
                        
                        left outer join bingo.players joinable_game_player on joinable_game_player.user_id = $1
                            and joinable_game_player.game_id = active_game.id
                        
                        left outer join bingo.games joinable_game on joinable_game.id = joinable_game_player.game_id
                        
                        left outer join lateral (
                            select
                                g.id,
                                count(p.user_id) player_amount
                            from
                                bingo.games as g
                                join bingo.players p on p.game_id = g.id
                            group by
                                g.id
                        ) g on g.id = joinable_game.id
                        
                        left outer join lateral (
                            select
                                ft.game_template_id,
                                count(ft.game_template_id) field_amount
                            from
                                bingo.field_templates as ft
                            group by
                                ft.game_template_id
                        ) ft on ft.game_template_id = gt.id
                    order by gt.id, joinable_game_player_game_id asc
                ) sq
            where
                startable
                or access_code is not null
            order by
                access_code asc,
                owned desc,
                startable desc
        "#,
        identity.user_id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|v| TemplateOut {
        id: v.id,
        title: v.title,
        field_amount: v.field_amount.unwrap_or(0),
        player_amount: v.player_amount.unwrap_or(0),
        owned: v.owned.unwrap_or(false),
        approved: v.public.unwrap_or(false),
        access_code: v.access_code,
    })
    .collect::<Vec<TemplateOut>>();

    Ok(Json(templates))
}

pub async fn handle_create_template(
    Extension(state): Extension<AppState>,
    Json(payload): Json<TemplateIn>,
    identity: Identity,
) -> Result<String> {
    let pool = &state.pool;

    if payload.title.is_empty() || payload.title.len() > 128 {
        return Err(Error::BadRequest(
            "Title must have at least one and at most 128 characters.".to_string(),
        ));
    }

    let game_template = sqlx::query!(
        r#"
            insert into bingo.game_templates (title, "language", created_by)
            values ($1, $2, $3) returning id
        "#,
        payload.title,
        "ger",
        identity.user_id
    )
    .fetch_one(pool)
    .await?;

    // check if template has at least nine fields

    if payload.fields.len() < 9 {
        return Err(Error::BadRequest(
            "Templates must have at lest 9 fields.".to_string(),
        ));
    }

    // check if all fields are valid

    for field in &payload.fields {
        if field.is_empty() || field.len() > 128 {
            return Err(Error::BadRequest(
                "Field captions must have at least one and at most 128 characters.".to_string(),
            ));
        }
    }

    for field in &payload.fields {
        sqlx::query!(
            r#"
                insert into bingo.field_templates (game_template_id, caption)
                values ($1, $2)
            "#,
            game_template.id,
            field
        )
        .execute(pool)
        .await?;
    }

    Ok("Template successfully creates.".to_string())
}

pub async fn handle_delete_template(
    identity: Identity,
    Path(game_template_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
) -> Result<()> {
    let pool = state.pool;
    let user_id = identity.user_id;

    let mut transaction = pool.begin().await?;

    let game_template = sqlx::query!(
        r#"
            select 
                id
            from
                bingo.game_templates
            where
                id = $1
                and created_by = $2
        "#,
        game_template_id,
        user_id,
    )
    .fetch_one(&mut transaction)
    .await?;

    let game_ids = sqlx::query!(
        r#"
            select 
                id
            from
                bingo.games
            where
                game_template_id = $1
        "#,
        &game_template.id,
    )
    .fetch_all(&mut transaction)
    .await?
    .iter()
    .map(|v| v.id)
    .collect::<Vec<Uuid>>();

    sqlx::query!(
        "delete from bingo.fields where game_id = any($1)",
        &game_ids
    )
    .execute(&mut transaction)
    .await?;

    sqlx::query!(
        "delete from bingo.players where game_id = any($1)",
        &game_ids
    )
    .execute(&mut transaction)
    .await?;

    sqlx::query!("delete from bingo.games where id = any($1)", &game_ids)
        .execute(&mut transaction)
        .await?;

    sqlx::query!(
        "delete from bingo.field_templates where game_template_id = $1",
        &game_template.id
    )
    .execute(&mut transaction)
    .await?;

    sqlx::query!(
        "delete from bingo.game_templates where id = $1",
        &game_template.id
    )
    .execute(&mut transaction)
    .await?;

    transaction.commit().await?;

    Ok(())
}
