use axum::Json;
use axum::extract::{State, Path};
use axum::http::StatusCode;

use crate::service;
use crate::database;

pub(crate) type VisitsResponse = Vec<VisitsResult>;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VisitsResult {
    url_id: uuid::fmt::Simple,
    #[serde(with = "time::serde::rfc3339")]
    timestamp: time::OffsetDateTime,
    ip_addr: Option<std::net::SocketAddr>,
    user_agent: Option<String>,
}

impl From<&database::visit::Visit> for VisitsResult {
    fn from(visit: &database::visit::Visit) -> Self {
        Self {
            url_id: visit.url_id.simple(),
            timestamp: visit.visit_timestamp.assume_utc(),
            ip_addr: visit.visitor_ip_addr,
            user_agent: visit.visitor_user_agent.clone(),
        }
    }
}

/// Return a list of information for each intercepted visit of URL contained.
/// Information includes time of visit, connecting IP and value of `User-Agent` header.
pub(crate) async fn visits(
    State(state): State<service::State>,
    Path(doc_id_simple): Path<uuid::fmt::Simple>,
) -> Result<Json<VisitsResponse>, StatusCode> {
    let visits = database::visit::get_doc_visits(&doc_id_simple.as_uuid(), &state.pool)
        .await
        .map_err(|err| service::server_error(err, "Could not get document visits"))?;

    Ok(Json(visits.iter().map(VisitsResult::from).collect()))
}
