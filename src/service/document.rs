use axum::{Json, extract::Path};
use axum::extract::State;
use axum::http::StatusCode;
use linkify::{LinkFinder, LinkKind};
use url::Url;

use crate::database;

#[derive(serde::Serialize)]
pub(crate) struct ProcessResponse {
    id: uuid::fmt::Simple,
    replacement: String,
}

/// Process a document and return it back with URLs replaced.
pub(crate) async fn process(
    State(state): State<super::State>,
    content: String,
) -> Result<Json<ProcessResponse>, StatusCode> {
    // Start a database transaction, as multiple records will be inserted
    let mut transaction = state.pool.begin()
        .await
        .map_err(|err| super::server_error(err, "Could not open transaction"))?;

    // First persist the document (since `doc_id` is a foreign key in `url`)
    let doc_id = database::document::persist(&content, &mut *transaction)
        .await
        .map_err(|err| super::server_error(err, "Could not persist document"))?;

    // Then find, persist and replace all URLs in document content.
    // NOTE ON PERFORMANCE:
    // This should ideally be implemented using e.g. `tokio_stream`
    // instead of collecting spans into memory before processing,
    // and URLs should further be persisted concurrently instead of sequentially.
    let mut new_content = String::with_capacity(content.capacity());
    let mut cur_index: u16 = 0;

    let spans = LinkFinder::new()
        .kinds(&[LinkKind::Url])
        .spans(&content)
        .collect::<Vec<_>>();

    for span in spans {
        // Append span text wholesale if it is not an URL
        if span.kind() != Some(&LinkKind::Url) {
            new_content.push_str(span.as_str());
            continue;
        }

        let url = Url::parse(span.as_str())
            .map_err(|err| super::server_error(err, "Libraries do not agree on URL validity"))?;

        let url_id = database::url::persist(&url, &doc_id, cur_index, &mut *transaction)
            .await
            .map_err(|err| super::server_error(err, "Could not persist URL"))?;

        let new_url = state.settings.host_url
            .join(&format!("/visit/{}", url_id.simple()))
            .expect("host_url and `/visit/{url_id}` known to be valid");

        new_content.push_str(new_url.as_str());
        cur_index += 1
    }

    transaction.commit()
        .await
        .map_err(|err| super::server_error(err, "Could not commit transaction"))?;

    Ok(Json(ProcessResponse {
        id: doc_id.simple(),
        replacement: new_content
    }))
}

pub(crate) type AnalyticsResponse = Vec<AnalyticsResult>;

#[derive(serde::Serialize)]
pub(crate) struct AnalyticsResult {
    url_id: uuid::fmt::Simple,
    visit_count: u64,
    #[serde(with = "time::serde::rfc3339::option")]
    first_visit: Option<time::OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    last_visit: Option<time::OffsetDateTime>,
}

impl From<&crate::database::document::Analytics> for AnalyticsResult {
    fn from(analytics: &crate::database::document::Analytics) -> Self {
        Self {
            url_id: analytics.url_id.simple(),
            visit_count: analytics.visit_count,
            first_visit: analytics.first_visit.map(time::PrimitiveDateTime::assume_utc),
            last_visit: analytics.last_visit.map(time::PrimitiveDateTime::assume_utc),
        }
    }
}

/// Return a list of information for each URL contained in document in order or appearance.
/// Information includes ID, number of visits, first visit time and last visit time.
pub(crate) async fn analytics(
    State(state): State<super::State>,
    Path(doc_id_simple): Path<uuid::fmt::Simple>,
) -> Result<Json<AnalyticsResponse>, StatusCode> {
    let analytics = database::document::get_analytics(&doc_id_simple.as_uuid(), &state.pool)
        .await
        .map_err(|err| super::server_error(err, "Could not aggregate document analytics"))?;

    Ok(Json(analytics.iter().map(AnalyticsResult::from).collect()))
}
