use axum::{
    body::Body,
    extract::FromRef,
    http::{header::ToStrError, StatusCode},
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::{error, warn};
use wr_database::{Database, DbErr};

#[derive(Clone, FromRef)]
pub struct GlobalState {
    pub db: Database,
    pub version: String,
}

#[derive(Debug, Error)]
pub enum ResponseError {
    #[error("internal server error: {0}, {1}")]
    InternalServerError(String, String),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("forbidden: {0}, {1}")]
    Forbidden(String, String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("resource is outdated: {0}")]
    Gone(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("precondition failed: {0}")]
    PreconditionFailed(String),
    #[error("too many requests: {0}, {1}")]
    TooManyRequests(String, String),
    #[error("database error: {0}")]
    DatabaseError(#[from] wr_database::DbErr),
    #[error("serialize error: {0}")]
    SerializeError(#[from] serde_json::Error),
    #[error("file io error: {0}")]
    FileIoError(#[from] std::io::Error),
    #[error("parse int error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("to str error: {0}")]
    ToStrError(#[from] ToStrError),
    #[error("from utf8 error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}

macro_rules! log_with_resp {
    ($code:expr, $summary:expr, $detail:expr) => {{
        if ($code).is_server_error() {
            error!("{}: {}", $summary, $detail);
        } else {
            warn!("{}: {}", $summary, $detail);
        }
        ($code, $summary)
    }};
}

impl IntoResponse for ResponseError {
    fn into_response(self) -> Response<Body> {
        let (status, message) = match self {
            ResponseError::InternalServerError(summary, detail) => {
                log_with_resp!(StatusCode::INTERNAL_SERVER_ERROR, summary, detail)
            }
            ResponseError::Unauthorized(summary) => (StatusCode::UNAUTHORIZED, summary),
            ResponseError::BadRequest(summary) => (StatusCode::BAD_REQUEST, summary),
            ResponseError::Forbidden(summary, detail) => {
                log_with_resp!(StatusCode::FORBIDDEN, summary, detail)
            }
            ResponseError::NotFound(summary) => (StatusCode::NOT_FOUND, summary),
            ResponseError::Conflict(summary) => (StatusCode::CONFLICT, summary),
            ResponseError::TooManyRequests(summary, detail) => {
                log_with_resp!(StatusCode::TOO_MANY_REQUESTS, summary, detail)
            }
            ResponseError::PreconditionFailed(summary) => {
                (StatusCode::PRECONDITION_FAILED, summary)
            }
            ResponseError::DatabaseError(e) => match e {
                DbErr::RecordNotFound(s) => {
                    (StatusCode::NOT_FOUND, format!("record not found: {s}"))
                }
                _ => log_with_resp!(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database internal error".to_owned(),
                    e.to_string()
                ),
            },
            ResponseError::FileIoError(e) => {
                log_with_resp!(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "file io error".to_owned(),
                    e.to_string()
                )
            }
            ResponseError::Gone(summary) => (StatusCode::GONE, summary),
            ResponseError::SerializeError(e) => {
                log_with_resp!(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to serialize data".to_owned(),
                    e.to_string()
                )
            }
            ResponseError::ParseIntError(e) => {
                log_with_resp!(
                    StatusCode::BAD_REQUEST,
                    "failed to parse integer".to_owned(),
                    e.to_string()
                )
            }
            ResponseError::ToStrError(e) => {
                log_with_resp!(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to convert to string".to_owned(),
                    e.to_string()
                )
            }
            ResponseError::FromUtf8Error(e) => {
                log_with_resp!(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to convert from utf8".to_owned(),
                    e.to_string()
                )
            }
        };
        Response::builder()
            .status(status)
            .body(message.into())
            .unwrap()
    }
}
