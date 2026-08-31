CREATE VIRTUAL TABLE IF NOT EXISTS project_search USING fts5(
    project_id UNINDEXED,
    name,
    description,
    notes,
    path,
    language,
    tags,
    links
);

CREATE TRIGGER IF NOT EXISTS project_search_insert AFTER INSERT ON projects
BEGIN
    INSERT INTO project_search (
        project_id, name, description, notes, path, language, tags, links
    ) VALUES (
        NEW.id, NEW.name, coalesce(NEW.description, ''), coalesce(NEW.notes, ''),
        NEW.path, coalesce(NEW.language, ''), '', ''
    );
END;

CREATE TRIGGER IF NOT EXISTS project_search_update AFTER UPDATE ON projects
BEGIN
    DELETE FROM project_search WHERE project_id = OLD.id;
    INSERT INTO project_search (
        project_id, name, description, notes, path, language, tags, links
    ) VALUES (
        NEW.id, NEW.name, coalesce(NEW.description, ''), coalesce(NEW.notes, ''),
        NEW.path, coalesce(NEW.language, ''), '', ''
    );
END;

CREATE TRIGGER IF NOT EXISTS project_search_delete AFTER DELETE ON projects
BEGIN
    DELETE FROM project_search WHERE project_id = OLD.id;
END;

-- ????????????
INSERT INTO project_search (project_id, name, description, notes, path, language, tags, links)
SELECT id, name, coalesce(description, ''), coalesce(notes, ''),
       path, coalesce(language, ''), '', ''
FROM projects;
