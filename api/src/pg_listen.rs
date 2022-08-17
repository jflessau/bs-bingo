use crate::{
    body::PgGameUpdateNotification,
    error::{Error, Result},
};
use chrono::{DateTime, Utc};
use sqlx::postgres::{PgListener, PgPool};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::time::{sleep, Duration};
use uuid::Uuid;

pub async fn listen(pool: &PgPool, sender: Arc<Mutex<HashMap<Uuid, DateTime<Utc>>>>) {
    tracing::info!("fn listen");

    if let Err(err) = process_notifications(pool, sender).await {
        tracing::error!("fn process_notifications failes, err: {:?}", err);
    }
}

async fn process_notifications(
    pool: &PgPool,
    sender: Arc<Mutex<HashMap<Uuid, DateTime<Utc>>>>,
) -> Result<(), Error> {
    let mut listener = PgListener::connect_with(pool).await?;

    listener
        .listen_all(vec!["fields_update", "players_update"])
        .await?;

    loop {
        tracing::info!("listen loop");

        let notification = listener.recv().await?;
        tracing::info!("got pg notifications");
        let game_update: PgGameUpdateNotification = serde_json::from_str(notification.payload())?;
        let game_id = game_update.game_id;

        if let Ok(mut recently_updates_games) = sender.try_lock() {
            recently_updates_games.insert(game_id, Utc::now());
        } else {
            tracing::warn!("try_lock fails for write");
        };
    }
}
