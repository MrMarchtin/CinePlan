-- Planning : journées de tournage, éléments ordonnés (scènes et pauses), convocations.
-- Les horaires sont en minutes depuis minuit (480 = 08h00).
CREATE TABLE shooting_days (
    id          TEXT PRIMARY KEY NOT NULL,
    project_id  TEXT NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    date        TEXT NOT NULL,
    start_min   INTEGER NOT NULL CHECK (start_min >= 0 AND start_min < 1440),
    end_min     INTEGER NOT NULL CHECK (end_min > start_min AND end_min < 1440),
    location    TEXT,
    notes       TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    version     INTEGER NOT NULL DEFAULT 1,
    UNIQUE (project_id, date)
);

CREATE TABLE day_items (
    id               TEXT PRIMARY KEY NOT NULL,
    day_id           TEXT NOT NULL REFERENCES shooting_days (id) ON DELETE CASCADE,
    position         INTEGER NOT NULL,
    kind             TEXT NOT NULL CHECK (kind IN ('scene', 'break')),
    scene_id         TEXT REFERENCES scenes (id) ON DELETE CASCADE,
    label            TEXT NOT NULL DEFAULT '',
    duration_min     INTEGER NOT NULL CHECK (duration_min > 0),
    delay_min        INTEGER NOT NULL DEFAULT 0,
    fixed_start_min  INTEGER CHECK (fixed_start_min IS NULL OR (fixed_start_min >= 0 AND fixed_start_min < 1440))
);

CREATE INDEX idx_day_items_day ON day_items (day_id, position);

CREATE TABLE day_members (
    day_id     TEXT NOT NULL REFERENCES shooting_days (id) ON DELETE CASCADE,
    member_id  TEXT NOT NULL REFERENCES team_members (id) ON DELETE CASCADE,
    PRIMARY KEY (day_id, member_id)
);
