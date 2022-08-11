use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts},
    routing::{get, patch, post},
    Router,
};
use axum_extra::extract::cookie::CookieJar;
use dotenv::dotenv;
use http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, COOKIE, USER_AGENT},
    Method,
};
use sqlx::postgres::PgPool;
use std::{env, net::SocketAddr};
use tower::ServiceBuilder;
use tower_http::{
    cors::{CorsLayer, Origin},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

mod error;
mod handler;

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
        .expect("database connection failes");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("running migrations failes");

    let port = dotenv::var("PORT")
        .unwrap_or_else(|_| "1313".into())
        .parse::<u16>()
        .expect("invalid PORT");

    let cors = CorsLayer::new()
        .allow_headers(vec![
            ACCEPT,
            AUTHORIZATION,
            CONTENT_TYPE,
            COOKIE,
            USER_AGENT,
        ])
        .allow_credentials(true)
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::DELETE,
            Method::PATCH,
        ])
        .allow_origin(Origin::list(vec![env::var("CORS_ALLOWED_ORIGIN")
            .expect("CORS_ALLOWED_ORIGIN not set")
            .parse()
            .expect("parsing CORS_ALLOWED_ORIGIN failes")]));

    let middleware_stack = ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_response(
                    DefaultOnResponse::new()
                        .level(tracing::Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        )
        .layer(cors)
        .layer(Extension(AppState { pool: pool.clone() }));

    let app = Router::new()
        .route("/health", get(health))
        .route("/auth", get(handler::auth::setup))
        .route("/templates", get(handler::template::list))
        .route("/templates", post(handler::template::create))
        .route("/game/:id", get(handler::game::ws))
        .route("/game/start/:id", get(handler::game::handle_start_game))
        .route("/game/leave/:id", get(handler::game::handle_leave_game))
        .route(
            "/game/join/:access_code",
            get(handler::game::handle_join_game),
        )
        .route(
            "/game/:id/username",
            patch(handler::game::handle_update_username),
        )
        .route("/field/:id", patch(handler::game::handle_update_field))
        .layer(middleware_stack)
        .layer(Extension(pool));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("starting axum server failes");
}

async fn health() -> error::Result<String> {
    Ok("Hi :)".into())
}

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::Pool<sqlx::Postgres>,
}

#[derive(Clone)]
pub struct Identity {
    pub user_id: Uuid,
}

#[async_trait]
impl<B> FromRequest<B> for Identity
where
    B: Send,
{
    type Rejection = error::Error;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let jar: CookieJar = CookieJar::from_request(req)
            .await
            .map_err(|_| error::Error::InvalidCredentials)?;

        if let Ok(Extension(pool)) = Extension::<PgPool>::from_request(req).await {
            if let Some(user_id) = jar.get("user_id").map(|cookie| cookie.value().to_owned()) {
                match Uuid::parse_str(&user_id) {
                    Ok(user_id) => {
                        let user = sqlx::query!(
                            r#"
                                select id
                                from identity.users
                                where id = $1
                            "#,
                            user_id
                        )
                        .fetch_one(&pool)
                        .await;

                        if user.is_ok() {
                            Ok(Identity { user_id })
                        } else {
                            tracing::warn!("received unknown user id");
                            Err(error::Error::InvalidCredentials)
                        }
                    }
                    Err(_) => {
                        tracing::warn!("failed to parse user_id to uuid");
                        Err(error::Error::InvalidCredentials)
                    }
                }
            } else {
                Err(error::Error::InvalidCredentials)
            }
        } else {
            tracing::error!("Database connection failed");
            Err(error::Error::InternalServer)
        }
    }
}
