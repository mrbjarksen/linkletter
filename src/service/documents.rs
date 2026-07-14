use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use linkify::{LinkFinder, LinkKind};
use url::Url;

use crate::service;
use crate::database;

mod urls;
mod visits;
mod analytics;
pub(crate) use self::urls::*;
pub(crate) use self::visits::*;
pub(crate) use self::analytics::*;

#[derive(serde::Serialize)]
pub(crate) struct ProcessResponse {
    id: uuid::fmt::Simple,
    replacement: String,
}

/// Process a document and return it back with URLs replaced.
pub(crate) async fn process(
    State(state): State<service::State>,
    content: String,
) -> Result<Json<ProcessResponse>, StatusCode> {
    // Start a database transaction, as multiple records will be inserted
    let mut transaction = state.pool.begin()
        .await
        .map_err(|err| service::server_error(err, "Could not open transaction"))?;

    // First persist the document (since `doc_id` is a foreign key in `url`)
    let doc_id = database::document::persist(&content, &mut *transaction)
        .await
        .map_err(|err| service::server_error(err, "Could not persist document"))?;

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
            .map_err(|err| service::server_error(err, "Libraries do not agree on URL validity"))?;

        let url_id = database::url::persist(&url, &doc_id, cur_index, &mut *transaction)
            .await
            .map_err(|err| service::server_error(err, "Could not persist URL"))?;

        let new_url = state.settings.host_url
            .join(&format!("/visit/{}", url_id.simple()))
            .expect("host_url and `/visit/{url_id}` known to be valid");

        new_content.push_str(new_url.as_str());
        cur_index += 1
    }

    transaction.commit()
        .await
        .map_err(|err| service::server_error(err, "Could not commit transaction"))?;

    Ok(Json(ProcessResponse {
        id: doc_id.simple(),
        replacement: new_content
    }))
}
