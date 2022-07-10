use crate::{
    error::{Error, Result},
    AppState, Identity,
};
use axum::{extract::Extension, Json};
use serde::{Deserialize, Serialize};
use std::str;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateOut {
    id: Uuid,
    title: String,
    field_amount: i64,
    owned: bool,
}

pub async fn list(
    Extension(state): Extension<AppState>,
    identity: Identity,
) -> Result<Json<Vec<TemplateOut>>> {
    let pool = &state.pool;

    let templates = sqlx::query!(
        r#"
            select * from (
                select
                    distinct on(gt.id) gt.id,
                    gt.title,
                    count(ft.id) as field_amount,
                    gt.created_by = $1 as owned
                from
                    bingo.game_templates as gt
                    join bingo.field_templates as ft on ft.game_template_id = gt.id
                group by
                    gt.id
            ) as sq
            order by sq.owned desc
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
    })
    .collect::<Vec<TemplateOut>>();

    Ok(Json(templates))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateIn {
    title: String,
    fields: Vec<String>,
}

pub async fn create(
    Extension(state): Extension<AppState>,
    Json(payload): Json<TemplateIn>,
    identity: Identity,
) -> Result<String> {
    let pool = &state.pool;

    if payload.title.len() < 1 || payload.title.len() > 128 {
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
        if field.len() < 1 || field.len() > 128 {
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
