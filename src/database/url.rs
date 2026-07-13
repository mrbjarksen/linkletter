use url::Url;
use uuid::Uuid;

/// Persist URL with its given document appearance.
/// Returns assigned ID.
pub(crate) async fn persist<'e, E>(
    url: &Url,
    doc_id: &Uuid,
    index_in_doc: u16,
    executor: E,
) -> sqlx::Result<Uuid>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    let url_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO url(url_id, doc_id, index_in_doc, url) VALUES($1, $2, $3, $4)",
        url_id.as_bytes().as_slice(),
        doc_id.as_bytes().as_slice(),
        index_in_doc,
        url.as_str()
    )
    .execute(executor)
    .await?;

    Ok(url_id)
}

pub(crate) async fn fetch<'e, E>(url_id: &Uuid, executor: E) -> sqlx::Result<Url>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    let url = sqlx::query_scalar!(
        "SELECT url FROM url WHERE url_id = $1",
        url_id.as_bytes().as_slice()
    )
    .fetch_one(executor)
    .await?;

    Url::parse(&url).map_err(|err| sqlx::Error::ColumnDecode {
        index: "url".into(),
        source: Box::new(err),
    })
}
