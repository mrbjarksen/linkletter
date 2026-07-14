use hyper::{HeaderMap, header::USER_AGENT};
use sqlx::{Row, sqlite::SqliteRow};
use std::net::SocketAddr;
use std::str::FromStr;
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

pub(crate) struct Visit {
    pub(crate) url_id: Uuid,
    pub(crate) visit_timestamp: time::PrimitiveDateTime,
    pub(crate) visitor_ip_addr: Option<SocketAddr>,
    pub(crate) visitor_user_agent: Option<String>,
}

impl sqlx::FromRow<'_, SqliteRow> for Visit {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            url_id: Uuid::try_from(row.try_get::<Vec<u8>, &str>("url_id")?).map_err(|err| {
                sqlx::Error::ColumnDecode {
                    index: "url_id".into(),
                    source: Box::new(err),
                }
            })?,
            visit_timestamp: row.try_get("visit_timestamp")?,
            visitor_ip_addr: row.try_get::<Option<&str>, &str>("visitor_ip_addr")?
                .map(SocketAddr::from_str)
                .transpose()
                .map_err(|err| sqlx::Error::ColumnDecode {
                    index: "visitor_ip_addr".into(),
                    source: Box::new(err),
                })?,
            visitor_user_agent: row.try_get("visitor_user_agent")?,
        })
    }
}

/// Get all visits on given URL in order or timestamp.
pub(crate) async fn get_url_visits<'e, E>(url_id: &Uuid, executor: E) -> sqlx::Result<Vec<Visit>>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    sqlx::query_as(
        r#"
        SELECT
            url_id,
            visit_timestamp,
            visitor_ip_addr,
            visitor_user_agent
        FROM visit
        WHERE url_id = $1
        ORDER BY visit_timestamp, rowid ASC
        "#,
    )
    .bind(url_id.as_bytes().as_slice())
    .fetch_all(executor)
    .await
}

/// Get all visits on URLs in given document in order or timestamp.
pub(crate) async fn get_doc_visits<'e, E>(doc_id: &Uuid, executor: E) -> sqlx::Result<Vec<Visit>>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    sqlx::query_as(
        r#"
        SELECT
            visit.url_id,
            visit_timestamp,
            visitor_ip_addr,
            visitor_user_agent
        FROM visit
        JOIN url ON url.url_id = visit.url_id
        WHERE url.doc_id = $1
        ORDER BY visit_timestamp, visit.rowid ASC
        "#,
    )
    .bind(doc_id.as_bytes().as_slice())
    .fetch_all(executor)
    .await
}
