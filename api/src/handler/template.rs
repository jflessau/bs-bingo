use crate::{
    body::{TemplateIn, TemplateOut},
    error::{Error, Result},
    AppState, Identity,
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
            *
        from
            (
                select
                    distinct on (gt.id) 
                    gt.id,
                    gt.title,
                    count(ft.id) field_amount,
                    gt.created_by = $1 owned,
                    p.user_id is not null resumable
                from
                    bingo.game_templates gt
                    join bingo.field_templates ft on ft.game_template_id = gt.id
                    left outer join bingo.games g on g.game_template_id = gt.id
                    left outer join bingo.players p on g.id = p.game_id
                    and p.user_id = $1
                where
                    (
                        gt.created_by = $1
                        or (
                            gt.public = true
                            and gt.approved = true
                        )
                    )
                group by
                    gt.id,
                    p.user_id,
                    g.id
            ) as sq
        order by
            sq.owned desc,
            title desc
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
        owned: v.owned.unwrap_or(false),
        resumable: v.resumable.unwrap_or(false),
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
