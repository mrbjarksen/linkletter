use uuid::Uuid;

/// Persist document content.
/// Returns assigned ID.
pub(crate) async fn persist<'e, E>(content: &str, executor: E) -> sqlx::Result<Uuid>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    let doc_id = Uuid::now_v7();

    sqlx::query!(
        "INSERT INTO document(doc_id, content) VALUES($1, $2)",
        doc_id.as_bytes().as_slice(),
        content
    )
    .execute(executor)
    .await?;

    Ok(doc_id)
}

#[derive(sqlx::FromRow)]
pub(crate) struct Analytics {
    #[sqlx(try_from = "Vec<u8>")]
    pub(crate) url_id: Uuid,
    #[sqlx(try_from = "i64")]
    pub(crate) visit_count: u64,
    pub(crate) first_visit: Option<time::PrimitiveDateTime>,
    pub(crate) latest_visit: Option<time::PrimitiveDateTime>,
}

pub(crate) async fn get_analytics<'e, E>(doc_id: &Uuid, executor: E) -> sqlx::Result<Vec<Analytics>>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    sqlx::query_as(
        r#"
        SELECT
            url.url_id,
            COUNT(visit_id) AS visit_count,
            MIN(visit_timestamp) AS first_visit,
            MAX(visit_timestamp) AS latest_visit
        FROM url
        LEFT JOIN visit ON visit.url_id = url.url_id
        WHERE url.doc_id = $1
        GROUP BY url.url_id
        ORDER BY url.index_in_doc ASC
        "#
    )
    .bind(doc_id.as_bytes().as_slice())
    .fetch_all(executor)
    .await
}
