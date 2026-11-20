-- Équipe de tournage : un membre appartient à un projet.
-- unavailable_dates : dates AAAA-MM-JJ séparées par des virgules.
CREATE TABLE team_members (
    id                 TEXT PRIMARY KEY NOT NULL,
    project_id         TEXT NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    first_name         TEXT NOT NULL,
    last_name          TEXT NOT NULL,
    role               TEXT NOT NULL CHECK (role IN (
                         'realisateur', 'assistant_realisateur', 'cadreur', 'chef_operateur',
                         'ingenieur_son', 'decorateur', 'accessoiriste', 'costumier',
                         'acteur', 'figurant', 'producteur', 'autre')),
    role_detail        TEXT,
    phone              TEXT,
    email              TEXT,
    notes              TEXT NOT NULL DEFAULT '',
    unavailable_dates  TEXT NOT NULL DEFAULT '',
    created_at         TEXT NOT NULL,
    updated_at         TEXT NOT NULL,
    version            INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX idx_team_members_project ON team_members (project_id);
