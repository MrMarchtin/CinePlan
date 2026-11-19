-- CinePlan — schéma initial (SQLite).
-- Identifiants : UUID en texte, générés côté client (compatibles hors connexion
-- et avec la future synchronisation PostgreSQL).
-- Dates : texte ISO 8601 UTC (horodatages) ou AAAA-MM-JJ (dates de production).
-- `version` : incrémentée à chaque modification, pour détecter les conflits.

CREATE TABLE projects (
    id                      TEXT PRIMARY KEY NOT NULL,
    title                   TEXT NOT NULL,
    description             TEXT NOT NULL DEFAULT '',
    genre                   TEXT,
    estimated_duration_min  INTEGER CHECK (estimated_duration_min IS NULL OR estimated_duration_min > 0),
    status                  TEXT NOT NULL DEFAULT 'preparation'
                            CHECK (status IN ('preparation', 'tournage', 'termine', 'archive')),
    start_date              TEXT,
    end_date                TEXT,
    created_at              TEXT NOT NULL,
    updated_at              TEXT NOT NULL,
    version                 INTEGER NOT NULL DEFAULT 1,
    deleted_at              TEXT
);

CREATE INDEX idx_projects_active ON projects (deleted_at, updated_at DESC);

CREATE TABLE scenes (
    id                      TEXT PRIMARY KEY NOT NULL,
    project_id              TEXT NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    number                  INTEGER NOT NULL CHECK (number > 0),
    title                   TEXT NOT NULL,
    int_ext                 TEXT NOT NULL CHECK (int_ext IN ('int', 'ext')),
    time_of_day             TEXT NOT NULL CHECK (time_of_day IN ('jour', 'nuit', 'aube', 'crepuscule')),
    location_name           TEXT,
    summary                 TEXT NOT NULL DEFAULT '',
    estimated_duration_min  INTEGER CHECK (estimated_duration_min IS NULL OR estimated_duration_min > 0),
    created_at              TEXT NOT NULL,
    updated_at              TEXT NOT NULL,
    version                 INTEGER NOT NULL DEFAULT 1,
    UNIQUE (project_id, number)
);
