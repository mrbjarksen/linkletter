ALTER TABLE visit
RENAME to visit_old;

CREATE TABLE visit(
    visit_id INTEGER PRIMARY KEY,                              -- ID of visit (automatically assigned)
    url_id BLOB NOT NULL,                                      -- ID of URL
    visit_timestamp TEXT DEFAULT(datetime('subsec')) NOT NULL, -- Date and time of visit (ISO-8601)
    visitor_ip_addr TEXT,                                      -- IP address of visitor
    visitor_user_agent TEXT,                                   -- Value of HTTP header `User-Agent`
    FOREIGN KEY(url_id) REFERENCES url(url_id)
) STRICT;

INSERT INTO visit
SELECT * FROM visit_old;

DROP TABLE visit_old;

CREATE INDEX IF NOT EXISTS ix_visit_url
ON visit(url_id);
