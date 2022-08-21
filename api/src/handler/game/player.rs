use crate::{
    body::{PlayerOut, UsernameIn},
    error::Result,
    server::{AppState, Identity},
};
use axum::{
    extract::{Extension, Path},
    Json,
};
use sqlx::PgConnection;
use uuid::Uuid;

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

pub async fn ger_players(
    game_id: Uuid,
    user_id: Uuid,
    conn: &mut PgConnection,
) -> Result<Vec<PlayerOut>> {
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
    .fetch_all(&mut *conn)
    .await?
    .into_iter()
    .map(|v| PlayerOut {
        user_id: v.user_id,
        username: v.username,
        bingos: super::bingos(v.hits.clone().unwrap_or_default()),
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
