-- Table representing some document which has had its URLs replaced with service-managed URLS
CREATE TABLE IF NOT EXISTS document(
    doc_id BLOB PRIMARY KEY NOT NULL, -- ID of document (UUIDv7)
    content TEXT                      -- Full content of document before URL replacement
) STRICT, WITHOUT ROWID;

-- Table represeting a single service-managed URL 
CREATE TABLE IF NOT EXISTS url(
    url_id BLOB PRIMARY KEY NOT NULL,               -- ID of URL (UUIDv4)
    doc_id BLOB NOT NULL,                           -- ID of corresponding document
    index_in_doc INTEGER CHECK ( index_in_doc>=0 ), -- Index of URL in its corresponding document
    url TEXT NOT NULL,                              -- Full URL to redirect towards
    FOREIGN KEY(doc_id) REFERENCES document(doc_id)
) STRICT, WITHOUT ROWID;

-- Table representing a single HTTP request to a service-managed URL
CREATE TABLE IF NOT EXISTS visit(
    visit_id INTEGER PRIMARY KEY,                            -- ID of visit (automatically assigned)
    url_id BLOB NOT NULL,                                    -- ID of URL
    visit_timestamp TEXT DEFAULT CURRENT_TIMESTAMP NOT NULL, -- Date and time of visit (ISO-8601)
    visitor_ip_addr TEXT,                                    -- IP address of visitor
    visitor_user_agent TEXT,                                 -- Value of HTTP header `User-Agent`
    FOREIGN KEY(url_id) REFERENCES url(url_id)
) STRICT;
