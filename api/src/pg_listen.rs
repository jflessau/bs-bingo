use crate::{
    body::PgGameUpdateNotification,
    error::{Error, Result},
};
use chrono::{DateTime, Utc};
use sqlx::postgres::{PgListener, PgPool};
use std::collections::HashMap;
use tokio::sync::watch::Sender;
use uuid::Uuid;

pub async fn listen(pool: &PgPool, sender: Sender<HashMap<Uuid, DateTime<Utc>>>) {
    if let Err(err) = process_notifications(pool, sender).await {
        tracing::error!("fn process_notifications failes, err: {:?}", err);
    }
}

async fn process_notifications(
    pool: &PgPool,
    sender: Sender<HashMap<Uuid, DateTime<Utc>>>,
) -> Result<(), Error> {
    let mut listener = PgListener::connect_with(pool).await?;

    listener
        .listen_all(vec!["fields_update", "players_update"])
        .await?;

    loop {
        let notification = listener.recv().await?;
        let game_update: PgGameUpdateNotification = serde_json::from_str(notification.payload())?;
        let game_id = game_update.game_id;

        sender.send_modify(|v| {
            if v.len() > 100000 {
                *v = HashMap::new();
            } else {
                v.insert(game_id, Utc::now());
            }
        });
    }
}
