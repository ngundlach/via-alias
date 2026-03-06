CREATE TEMPORARY TABLE IF NOT EXISTS redirects_temp AS
SELECT
    id,
    alias,
    url
FROM redirects;

DROP TABLE redirects;

CREATE TABLE IF NOT EXISTS redirects (
    id TEXT NOT NULL PRIMARY KEY,
    alias TEXT NOT NULL UNIQUE,
    url TEXT NOT NULL,
    owner TEXT, -- keeping it nullable for migration of redirects before ownership was introduced
    FOREIGN KEY(owner) REFERENCES users(id)
);

INSERT INTO redirects(id, alias, url, owner)
SELECT id, alias, url, NULL FROM redirects_temp;

DROP TABLE redirects_temp;
