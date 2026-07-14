use axum::extract::{State, Path};
use axum::http::StatusCode;

use crate::service;
use crate::database;

mod visits;
pub(crate) use self::visits::*;

/// Return URL associated with ID.
pub(crate) async fn get(
    State(state): State<service::State>,
    Path(url_id_simple): Path<uuid::fmt::Simple>,
) -> Result<String, StatusCode> {
    let url_id = url_id_simple.as_uuid().clone();

    let url = database::url::fetch(&url_id, &state.pool)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
            _ => service::server_error(err, "Could not fetch URL"),
        })?;

    Ok(url.as_str().into())
}
