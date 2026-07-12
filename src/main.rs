use std::net::SocketAddr;
use std::str::FromStr;

use axum::{
    Router,
    extract::{ConnectInfo, Path, State},
    http::StatusCode,
    response::Redirect,
    routing::{get, post},
};
use hyper::{HeaderMap, header::USER_AGENT};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool};
use url::Url;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Open or create SQLite database in WAL-mode
    let opts = SqliteConnectOptions::from_str("sqlite::memory:")?
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true)
        .optimize_on_close(true, None);
    let pool = SqlitePool::connect_with(opts).await?;

    // Make sure database schema is up to date
    sqlx::migrate!("db/migrations").run(&pool).await?;

    // Set up service endpoints
    let app = Router::new()
        .route("/documents", post(process_document))
        .route("/visit/{url_id}", get(process_visit))
        .with_state(pool)
        .into_make_service_with_connect_info::<SocketAddr>();

    // Start service
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[axum::debug_handler]
async fn process_document(
    State(pool): State<SqlitePool>,
    content: String,
) -> Result<String, StatusCode> {
    let doc_id = Uuid::now_v7();

    let mut transaction = pool.begin().await.map_err(|err| {
        eprintln!("[ERROR] Could not open transaction: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    sqlx::query!(
        "INSERT INTO document(doc_id, content) VALUES($1, $2)",
        doc_id.as_bytes().as_slice(),
        content
    )
    .execute(&mut *transaction)
    .await
    .map_err(|err| {
        eprintln!("[ERROR] Could not persist document: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match Url::parse(&content) {
        Ok(_) => {
            let url_id = Uuid::new_v4();
            let url = format!("http://localhost:3000/visit/{}", url_id.simple());

            sqlx::query!(
                "INSERT INTO url(url_id, doc_id, index_in_doc, url) VALUES($1, $2, $3, $4)",
                url_id.as_bytes().as_slice(),
                doc_id.as_bytes().as_slice(),
                0,
                content
            )
            .execute(&mut *transaction)
            .await
            .map_err(|err| {
                eprintln!("[ERROR] Could not persist URL: {err}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            transaction
                .commit()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            Ok(url.into())
        }
        Err(_) => {
            transaction
                .commit()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(content)
        }
    }
}

#[axum::debug_handler]
async fn process_visit(
    State(pool): State<SqlitePool>,
    Path(url_id): Path<uuid::fmt::Simple>,
    ConnectInfo(visitor_ip_addr): ConnectInfo<SocketAddr>,
    header: HeaderMap,
) -> Result<Redirect, StatusCode> {
    // Fetch URL from database by ID embedded in path,
    // returning 404 Not Found if it doesn't exist.
    let url = sqlx::query_scalar!(
        "SELECT url FROM url WHERE url_id = $1",
        url_id.as_uuid().as_bytes().as_slice()
    )
    .fetch_one(&pool)
    .await
    .map_err(|err| match err {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    // Log visit information into database asynchronously,
    // such that the user experiences no delay.
    // If this fails, the user is still forwarded as expected.
    tokio::spawn(async move {
        sqlx::query!(
            "INSERT INTO visit(url_id, visitor_ip_addr, visitor_user_agent) VALUES($1, $2, $3)",
            url_id.as_uuid().as_bytes().as_slice(),
            visitor_ip_addr.to_string(),
            get_user_agent(&header)
        )
        .execute(&pool)
        .await
        .inspect_err(|err| {
            eprintln!("[ERROR] Visit dropped on {url_id}: {err}");
        })
    });

    // Redirect request to original URL using
    Ok(Redirect::temporary(url.as_str()))
}

// Safely get the `User-Agent` header value as a string slice.
// Returns `None` if `User-Agent` is not present or cannot be parsed.
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
