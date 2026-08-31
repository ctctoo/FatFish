ALTER TABLE links ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;
ALTER TABLE todos ADD COLUMN priority TEXT;

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_links_project_order
    ON links(project_id, sort_order);
