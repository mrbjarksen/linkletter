pub(super) mod documents;
pub(super) mod urls;
pub(super) mod visit;

use hyper::StatusCode;
use sqlx::sqlite::SqlitePool;

use crate::settings::ApiSettings;

/// State of service as it is running.
#[derive(Clone)]
pub(crate) struct State {
    /// Settings to be used by API.
    pub(crate) settings: ApiSettings,

    /// Database connection pool shared between API handlers.
    pub(crate) pool: SqlitePool,
}

impl axum::extract::FromRef<State> for SqlitePool {
    fn from_ref(state: &State) -> Self {
        state.pool.clone()
    }
}

pub(crate) fn server_error(err: impl std::error::Error, msg: &str) -> StatusCode {
    eprintln!("[ERROR] {msg}: {err}");
    StatusCode::INTERNAL_SERVER_ERROR
}
