use axum::extract::State;
use axum::http::StatusCode;
use url::Url;

use crate::database;

/// Process a document and return it back with URLs replaced.
pub(crate) async fn process(
    State(state): State<super::State>,
    content: String,
) -> Result<String, StatusCode> {
    // Start a database transaction, as multiple records will be inserted
    let mut transaction = state.pool.begin()
        .await
        .map_err(|err| super::server_error(err, "Could not open transaction"))?;

    // First persist the document (since `doc_id` is a foreign key in `url`)
    let doc_id = database::document::persist(&content, &mut *transaction)
        .await
        .map_err(|err| super::server_error(err, "Could not persist document"))?;

    // Then persist each URL in the document sequentially
    match Url::parse(&content) {
        Ok(url) => {
            let url_id = database::url::persist(&url, &doc_id, 0, &mut *transaction)
                .await
                .map_err(|err| super::server_error(err, "Could not persist URL"))?;

            transaction.commit()
                .await
                .map_err(|err| super::server_error(err, "Could not commit transaction"))?;

            let new_url = state.settings.host_url.join(&format!("/visit/{}", url_id.simple()))
                .map_err(|err| super::server_error(err, "Could not parse URL from config"))?;

            Ok(new_url.into())
        }
        Err(_) => {
            transaction.commit()
                .await
                .map_err(|err| super::server_error(err, "Could not commit transaction"))?;

            Ok(content)
        }
    }
}
