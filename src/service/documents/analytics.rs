use axum::Json;
use axum::extract::{State, Path};
use axum::http::StatusCode;

use crate::service;
use crate::database;

pub(crate) type AnalyticsResponse = Vec<AnalyticsResult>;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnalyticsResult {
    id: uuid::fmt::Simple,
    visit_count: u64,
    #[serde(with = "time::serde::rfc3339::option")]
    first_visit: Option<time::OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    latest_visit: Option<time::OffsetDateTime>,
}

impl From<&database::document::Analytics> for AnalyticsResult {
    fn from(analytics: &database::document::Analytics) -> Self {
        Self {
            id: analytics.url_id.simple(),
            visit_count: analytics.visit_count,
            first_visit: analytics.first_visit.map(time::PrimitiveDateTime::assume_utc),
            latest_visit: analytics.latest_visit.map(time::PrimitiveDateTime::assume_utc),
        }
    }
}

/// Return a list of information for each URL contained in document in order or appearance.
/// Information includes ID, number of visits, first visit time and last visit time.
pub(crate) async fn analytics(
    State(state): State<service::State>,
    Path(doc_id_simple): Path<uuid::fmt::Simple>,
) -> Result<Json<AnalyticsResponse>, StatusCode> {
    let analytics = database::document::get_analytics(&doc_id_simple.as_uuid(), &state.pool)
        .await
        .map_err(|err| service::server_error(err, "Could not aggregate document analytics"))?;

    Ok(Json(analytics.iter().map(AnalyticsResult::from).collect()))
}
