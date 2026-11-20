-- Documents : dossiers (arborescence par catégorie) et fichiers (PDF, images).
-- Les fichiers eux-mêmes sont stockés sur le disque ; la base garde leurs métadonnées.
CREATE TABLE folders (
    id          TEXT PRIMARY KEY NOT NULL,
    project_id  TEXT NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    parent_id   TEXT REFERENCES folders (id) ON DELETE CASCADE,
    category    TEXT NOT NULL CHECK (category IN (
                  'scenario', 'storyboard', 'preparation', 'planning', 'decors',
                  'costumes', 'materiel', 'administratif', 'autres')),
    name        TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    version     INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX idx_folders_project ON folders (project_id);

CREATE TABLE documents (
    id             TEXT PRIMARY KEY NOT NULL,
    project_id     TEXT NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    folder_id      TEXT REFERENCES folders (id),
    category       TEXT NOT NULL CHECK (category IN (
                     'scenario', 'storyboard', 'preparation', 'planning', 'decors',
                     'costumes', 'materiel', 'administratif', 'autres')),
    name           TEXT NOT NULL,
    original_name  TEXT NOT NULL,
    stored_name    TEXT NOT NULL,
    kind           TEXT NOT NULL CHECK (kind IN ('pdf', 'image')),
    mime_type      TEXT NOT NULL,
    size_bytes     INTEGER NOT NULL,
    scene_id       TEXT REFERENCES scenes (id) ON DELETE SET NULL,
    member_id      TEXT REFERENCES team_members (id) ON DELETE SET NULL,
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL,
    version        INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX idx_documents_project ON documents (project_id, category);
