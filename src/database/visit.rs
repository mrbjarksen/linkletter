use hyper::{HeaderMap, header::USER_AGENT};
use std::net::SocketAddr;

use uuid::Uuid;

/// Logs visit of URL with relevant information.
pub(crate) async fn log<'e, E>(
    url_id: &Uuid,
    visitor_ip_addr: &SocketAddr,
    header: &HeaderMap,
    executor: E,
) -> sqlx::Result<()>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    sqlx::query!(
        "INSERT INTO visit(url_id, visitor_ip_addr, visitor_user_agent) VALUES($1, $2, $3)",
        url_id.as_bytes().as_slice(),
        visitor_ip_addr.to_string(),
        get_user_agent(&header)
    )
    .execute(executor)
    .await?;

    Ok(())
}

/// Safely get the `User-Agent` header value as a string slice.
/// Returns `None` if `User-Agent` is not present or cannot be parsed.
fn get_user_agent(header: &HeaderMap) -> Option<&str> {
    header
        .get(USER_AGENT)
        .map(|v| match v.to_str() {
            Ok(user_agent) => Some(user_agent),
            Err(_) => {
                eprintln!("[WARN] Could not parse `User-Agent` header");
                None
            }
        })
        .flatten()
}
