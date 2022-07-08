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
    word_amount: i64,
}

pub async fn list(
    Extension(state): Extension<AppState>,
    auth_code: Identity,
) -> Result<Json<Vec<TemplateOut>>> {
    let pool = &state.pool;

    Ok(Json(vec![TemplateOut {
        id: Uuid::new_v4(),
        title: "String".to_string(),
        word_amount: 0,
    }]))
}
