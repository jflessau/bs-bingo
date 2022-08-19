use crate::{
    body::FieldOut,
    error::{Error, Result},
    server::{AppState, Identity},
};
use axum::extract::{Extension, Path};
use rand::{seq::SliceRandom, thread_rng};
use sqlx::postgres::PgPool;
use uuid::Uuid;

pub async fn prepare_fields(
    game_template_id: Uuid,
    game_id: Uuid,
    user_id: Uuid,
    grid_size: i64,
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

    if existing_fields.is_empty() || (existing_fields.len() as i64) < grid_size {
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

        if (field_template_ids.len() as i64) < grid_size {
            return Err(Error::BadRequest(format!(
                "Template has not enough fields for grid size of {}",
                grid_size
            )));
        }
        if !(2..=8).contains(&grid_size) {
            return Err(Error::BadRequest(
                "Grid size must be grater than one and less than 9.".to_string(),
            ));
        }

        sqlx::query!(
            "delete from bingo.fields where id = any($1)",
            &existing_fields.iter().map(|v| v.id).collect::<Vec<Uuid>>()
        )
        .execute(pool)
        .await?;
        sqlx::query!(
            "delete from bingo.players where user_id = $1 and game_id = $2",
            user_id,
            game_id
        )
        .execute(pool)
        .await?;

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

        field_template_ids.shuffle(&mut thread_rng());
        let field_template_ids = &field_template_ids[0..(grid_size * grid_size) as usize].to_vec();

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
    }

    list_fields(game_id, user_id, pool).await
}

pub async fn list_fields(
    game_id: Uuid,
    user_id: Uuid,
    pool: &PgPool,
) -> Result<Vec<Vec<FieldOut>>> {
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

    tracing::info!("fields.len(): {}", fields.len());
    tracing::info!(
        "fields.len() as f32).sqrt(): {}",
        (fields.len() as f32).sqrt()
    );
    tracing::info!(
        "(fields.len() as f32).sqrt() as usize: {}",
        (fields.len() as f32).sqrt() as usize
    );

    if fields.len() > 0 {
        let grid_size = (fields.len() as f32).sqrt() as usize;
        for (i, field) in fields.enumerate() {
            if i % grid_size == 0 {
                if i == grid_size {
                    result = vec![v]
                } else {
                    result.push(v);
                }
                v = vec![field];
            } else {
                v.push(field);
            }
        }
    }

    result.push(v);

    Ok(result)
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
