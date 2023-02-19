use dotenv::dotenv;
use sqlx::postgres::PgPool;
use std::{collections::HashMap, env};
use tokio::sync::watch;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod body;
mod error;
mod handler;
mod pg_listen;
mod server;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG").unwrap_or_else(|_| "info,sqlx=warn".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("database connection fails");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("running migrations fails");

    let (sender, receiver) = watch::channel(HashMap::new());

    tokio::select!(
        _ = server::serve(pool.clone(), receiver.clone()) => {
            tracing::error!("serfer::serve shut down"); return;
        },
        _ = pg_listen::listen(&pool, sender) => {
            tracing::error!("pg_listener::listen shut down"); return;
        }
    );
}
