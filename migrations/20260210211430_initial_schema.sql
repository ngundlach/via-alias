CREATE TABLE IF NOT EXISTS redirects (
    id INTEGER NOT NULL PRIMARY KEY,
    alias TEXT NOT NULL UNIQUE,
    url TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_redirects_alias ON redirects(alias);
