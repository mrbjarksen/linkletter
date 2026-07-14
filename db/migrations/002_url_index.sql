CREATE UNIQUE INDEX IF NOT EXISTS ix_url_doc_position
ON url(doc_id, index_in_doc);

CREATE INDEX IF NOT EXISTS ix_visit_url
ON visit(url_id);
