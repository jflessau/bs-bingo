use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum Error {
    InvalidCredentials,
    BadRequest(String),
    NotFound,
    InternalServer,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Error::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string())
            }
            Error::BadRequest(error) => (StatusCode::BAD_REQUEST, error),
            Error::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            Error::InternalServer => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

impl From<dotenv::Error> for Error {
    fn from(err: dotenv::Error) -> Self {
        tracing::error!("dotenv error: {}", err.to_string());
        Error::InternalServer
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        if let sqlx::Error::RowNotFound = err {
            tracing::warn!("row not found");
            Error::NotFound
        } else {
            tracing::error!("sqlx error: {}", err.to_string());
            Error::InternalServer
        }
    }
}

impl From<uuid::Error> for Error {
    fn from(err: uuid::Error) -> Self {
        Error::BadRequest(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::BadRequest(err.to_string())
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
