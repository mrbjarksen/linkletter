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
