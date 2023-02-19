use crate::{error, handler};
use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts},
    routing::{delete, get, patch, post},
    Router,
};
use axum_extra::extract::cookie::CookieJar;
use chrono::{DateTime, Utc};
use http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, COOKIE, USER_AGENT},
    Method,
};
use sqlx::postgres::PgPool;
use std::{collections::HashMap, env, net::SocketAddr};
use tokio::sync::watch::Receiver;
use tokio::time::{interval, sleep, Duration};
use tower::ServiceBuilder;
use tower_http::{
    cors::{CorsLayer, Origin},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use uuid::Uuid;

pub async fn serve(pool: PgPool, receiver: Receiver<HashMap<Uuid, DateTime<Utc>>>) {
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
            .expect("parsing CORS_ALLOWED_ORIGIN fails")]));

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
        .layer(Extension(AppState {
            pool: pool.clone(),
            receiver: receiver.clone(),
        }));

    let app = Router::new()
        .route("/health", get(health))
        .route("/auth", get(handler::auth::setup))
        .route("/templates", get(handler::template::handle_list_templates))
        .route(
            "/templates",
            post(handler::template::handle_create_template),
        )
        .route(
            "/templates/:id",
            delete(handler::template::handle_delete_template),
        )
        .route("/game/:id", get(handler::game::websocket::ws))
        .route(
            "/game/start/:id/:grid_size",
            get(handler::game::access::handle_start_game),
        )
        .route(
            "/game/leave/:id",
            get(handler::game::access::handle_leave_game),
        )
        .route(
            "/game/join/:access_code",
            get(handler::game::access::handle_join_game),
        )
        .route(
            "/game/:id/username",
            patch(handler::game::player::handle_update_username),
        )
        .route(
            "/field/:id",
            patch(handler::game::field::handle_update_field),
        )
        .layer(middleware_stack)
        .layer(Extension(pool));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            let mut interval = interval(Duration::from_millis(100));
            loop {
                interval.tick().await;
                if let Err(err) = receiver.has_changed() {
                    tracing::error!("channel connecting postgres notifications listener to websocket is broken, error: {}", err);
                    break;
                }
            }
            tracing::info!("shutting down server due to broken channel");
        })
        .await
        .expect("starting axum server fails");
}

async fn health() -> error::Result<String> {
    Ok("Hi :)".into())
}

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::Pool<sqlx::Postgres>,
    pub receiver: Receiver<HashMap<Uuid, DateTime<Utc>>>,
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

        // delay responses for a few milliseconds to be able to debug loading states in the frontend

        sleep(Duration::from_millis(
            dotenv::var("REQUEST_DELAY_MS")
                .unwrap_or_else(|_| "0".into())
                .parse::<u16>()
                .expect("parsing REQUEST_DELAY_MS failes")
                .into(),
        ))
        .await;

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
