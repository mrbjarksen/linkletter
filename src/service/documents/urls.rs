use axum::Json;
use axum::extract::{State, Path};
use axum::http::StatusCode;
use url::Url;

use crate::service;
use crate::database;

pub(crate) type UrlsResponse = Vec<UrlsResult>;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UrlsResult {
    id: uuid::fmt::Simple,
    url: Url,
}

impl From<&database::url::UrlInfo> for UrlsResult {
    fn from(url: &database::url::UrlInfo) -> Self {
        Self {
            id: url.url_id.simple(),
            url: url.url.clone(),
        }
    }
}

/// Return a list of information for each intercepted visit of URL contained.
/// Information includes time of visit, connecting IP and value of `User-Agent` header.
pub(crate) async fn urls(
    State(state): State<service::State>,
    Path(doc_id_simple): Path<uuid::fmt::Simple>,
) -> Result<Json<UrlsResponse>, StatusCode> {
    let visits = database::url::fetch_all_in_doc(&doc_id_simple.as_uuid(), &state.pool)
        .await
        .map_err(|err| service::server_error(err, "Could not get document visits"))?;

    Ok(Json(visits.iter().map(UrlsResult::from).collect()))
}
