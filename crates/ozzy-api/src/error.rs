use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal error")]
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            AppError::Conflict(m) => (StatusCode::CONFLICT, m.clone()),
            AppError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "internal error".into()),
        };
        (status, Json(ErrorBody { error: msg })).into_response()
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl From<worker::Error> for AppError {
    fn from(err: worker::Error) -> Self {
        #[cfg(not(test))]
        worker::console_error!("{err}");
        #[cfg(test)]
        eprintln!("{err}");
        AppError::Internal
    }
}

pub type AppResult<T> = Result<T, AppError>;

pub fn bad_request(msg: impl Into<String>) -> AppError {
    AppError::BadRequest(msg.into())
}

pub fn internal<E: std::fmt::Display>(err: E) -> AppError {
    #[cfg(not(test))]
    worker::console_error!("{err}");
    #[cfg(test)]
    eprintln!("{err}");
    AppError::Internal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_response() {
        let resp = AppError::Unauthorized.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let resp = AppError::NotFound.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let resp = AppError::BadRequest("bad".into()).into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let resp = AppError::Conflict("conflict".into()).into_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);

        let resp = AppError::Internal.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_bad_request() {
        let err = bad_request("test bad request");
        assert_eq!(err.to_string(), "bad request: test bad request");
    }

    #[test]
    fn test_internal() {
        let err = internal("some internal error");
        assert_eq!(err.to_string(), "internal error");
    }

    #[test]
    fn test_worker_error() {
        let err: AppError = worker::Error::from("worker error").into();
        assert_eq!(err.to_string(), "internal error");
    }
}
