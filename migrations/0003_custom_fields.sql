CREATE TABLE IF NOT EXISTS custom_fields (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    field_key TEXT NOT NULL,
    field_label TEXT NOT NULL,
    value TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE,
    UNIQUE (project_id, field_key)
);

CREATE TABLE IF NOT EXISTS project_custom_fields (
    project_id TEXT NOT NULL,
    field_id TEXT NOT NULL,
    PRIMARY KEY (project_id, field_id),
    FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY(field_id) REFERENCES custom_fields(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_custom_fields_project
    ON custom_fields(project_id, sort_order);