use axum::extract::{ConnectInfo, Path, State};
use axum::http::StatusCode;
use axum::response::Redirect;

use crate::database;

/// Process visit and reroute to expected URL
pub(crate) async fn process(
    State(state): State<super::State>,
    Path(url_id_simple): Path<uuid::fmt::Simple>,
    ConnectInfo(visitor_ip_addr): ConnectInfo<std::net::SocketAddr>,
    header: hyper::HeaderMap,
) -> Result<Redirect, StatusCode> {
    let url_id = url_id_simple.as_uuid().clone();

    // Fetch URL from database by ID embedded in path,
    // returning 404 Not Found if it doesn't exist.
    let url = database::url::fetch(&url_id, &state.pool)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
            _ => super::server_error(err, "Could not fetch URL"),
        })?;

    // Log visit information into database asynchronously,
    // such that the user experiences no delay.
    // If this fails, the user is still forwarded as expected.
    tokio::spawn(async move {
        database::visit::log(&url_id, &visitor_ip_addr, &header, &state.pool)
            .await
            .inspect_err(|err| {
                eprintln!("[ERROR] Visit dropped on {url_id}: {err}");
            })
    });

    // Redirect request to original URL using 307 Temporary Redirect
    // such that that the client does not cache the original URL.
    Ok(Redirect::temporary(url.as_str()))
}
