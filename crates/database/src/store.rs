use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use cineplan_core::{
    CoreError, CoreResult, Project, ProjectRepository, Scene, SceneRepository,
};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteRow};
use sqlx::{Row, SqlitePool};
use std::path::Path;
use std::str::FromStr;
use uuid::Uuid;

/// Base SQLite locale. Se clone à bas coût (le pool est partagé).
#[derive(Clone)]
pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    /// Ouvre (ou crée) la base et applique les migrations en attente.
    pub async fn connect(path: &Path) -> CoreResult<Self> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .map_err(storage)?;
        Self::migrated(pool).await
    }

    /// Base en mémoire, pour les tests.
    pub async fn in_memory() -> CoreResult<Self> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .map_err(storage)?
            .foreign_keys(true);
        // Une seule connexion : chaque connexion mémoire est une base distincte.
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .map_err(storage)?;
        Self::migrated(pool).await
    }

    async fn migrated(pool: SqlitePool) -> CoreResult<Self> {
        sqlx::migrate!("../../migrations")
            .run(&pool)
            .await
            .map_err(storage)?;
        Ok(Self { pool })
    }
}

// ---------- conversions et erreurs ----------

fn storage(e: impl std::fmt::Display) -> CoreError {
    CoreError::Storage(e.to_string())
}

fn map_sqlx(e: sqlx::Error) -> CoreError {
    if let sqlx::Error::Database(db) = &e {
        if db.is_unique_violation() {
            return CoreError::Validation(
                "Ce numéro de scène est déjà utilisé dans ce projet.".into(),
            );
        }
        if db.is_foreign_key_violation() {
            return CoreError::NotFound;
        }
    }
    storage(e)
}

fn ts(d: &DateTime<Utc>) -> String {
    d.to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn get<'r, T>(row: &'r SqliteRow, col: &str) -> CoreResult<T>
where
    T: sqlx::Decode<'r, sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite>,
{
    row.try_get(col).map_err(storage)
}

fn get_uuid(row: &SqliteRow, col: &str) -> CoreResult<Uuid> {
    Uuid::parse_str(&get::<String>(row, col)?).map_err(storage)
}

fn get_time(row: &SqliteRow, col: &str) -> CoreResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&get::<String>(row, col)?)
        .map(|d| d.with_timezone(&Utc))
        .map_err(storage)
}

fn get_date(row: &SqliteRow, col: &str) -> CoreResult<Option<NaiveDate>> {
    get::<Option<String>>(row, col)?
        .map(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(storage))
        .transpose()
}

fn get_minutes(row: &SqliteRow, col: &str) -> CoreResult<Option<u32>> {
    Ok(get::<Option<i64>>(row, col)?.map(|v| v as u32))
}

fn project_from_row(row: &SqliteRow) -> CoreResult<Project> {
    Ok(Project {
        id: get_uuid(row, "id")?,
        title: get(row, "title")?,
        description: get(row, "description")?,
        genre: get(row, "genre")?,
        estimated_duration_min: get_minutes(row, "estimated_duration_min")?,
        status: get::<String>(row, "status")?.parse()?,
        start_date: get_date(row, "start_date")?,
        end_date: get_date(row, "end_date")?,
        created_at: get_time(row, "created_at")?,
        updated_at: get_time(row, "updated_at")?,
        version: get(row, "version")?,
    })
}

fn scene_from_row(row: &SqliteRow) -> CoreResult<Scene> {
    Ok(Scene {
        id: get_uuid(row, "id")?,
        project_id: get_uuid(row, "project_id")?,
        number: get::<i64>(row, "number")? as u32,
        title: get(row, "title")?,
        int_ext: get::<String>(row, "int_ext")?.parse()?,
        time_of_day: get::<String>(row, "time_of_day")?.parse()?,
        location_name: get(row, "location_name")?,
        summary: get(row, "summary")?,
        estimated_duration_min: get_minutes(row, "estimated_duration_min")?,
        created_at: get_time(row, "created_at")?,
        updated_at: get_time(row, "updated_at")?,
        version: get(row, "version")?,
    })
}

// ---------- projets ----------

#[async_trait]
impl ProjectRepository for SqliteStore {
    async fn list(&self) -> CoreResult<Vec<Project>> {
        sqlx::query("SELECT * FROM projects WHERE deleted_at IS NULL ORDER BY updated_at DESC")
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx)?
            .iter()
            .map(project_from_row)
            .collect()
    }

    async fn get(&self, id: Uuid) -> CoreResult<Project> {
        let row = sqlx::query("SELECT * FROM projects WHERE id = ? AND deleted_at IS NULL")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx)?
            .ok_or(CoreError::NotFound)?;
        project_from_row(&row)
    }

    async fn insert(&self, p: &Project) -> CoreResult<()> {
        sqlx::query(
            "INSERT INTO projects (id, title, description, genre, estimated_duration_min, status, \
             start_date, end_date, created_at, updated_at, version) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(p.id.to_string())
        .bind(&p.title)
        .bind(&p.description)
        .bind(p.genre.as_deref())
        .bind(p.estimated_duration_min.map(i64::from))
        .bind(p.status.as_str())
        .bind(p.start_date.map(|d| d.to_string()))
        .bind(p.end_date.map(|d| d.to_string()))
        .bind(ts(&p.created_at))
        .bind(ts(&p.updated_at))
        .bind(p.version)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(())
    }

    async fn update(&self, p: &Project) -> CoreResult<()> {
        let result = sqlx::query(
            "UPDATE projects SET title = ?, description = ?, genre = ?, estimated_duration_min = ?, \
             status = ?, start_date = ?, end_date = ?, updated_at = ?, version = ? \
             WHERE id = ? AND version = ? AND deleted_at IS NULL",
        )
        .bind(&p.title)
        .bind(&p.description)
        .bind(p.genre.as_deref())
        .bind(p.estimated_duration_min.map(i64::from))
        .bind(p.status.as_str())
        .bind(p.start_date.map(|d| d.to_string()))
        .bind(p.end_date.map(|d| d.to_string()))
        .bind(ts(&p.updated_at))
        .bind(p.version)
        .bind(p.id.to_string())
        .bind(p.version - 1)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;

        if result.rows_affected() == 0 {
            // Soit le projet n'existe plus, soit sa version a changé.
            ProjectRepository::get(self, p.id).await?;
            return Err(CoreError::Conflict);
        }
        Ok(())
    }

    async fn soft_delete(&self, id: Uuid, at: DateTime<Utc>) -> CoreResult<()> {
        let result = sqlx::query(
            "UPDATE projects SET deleted_at = ?, updated_at = ?, version = version + 1 \
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(ts(&at))
        .bind(ts(&at))
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }
}

// ---------- scènes ----------

#[async_trait]
impl SceneRepository for SqliteStore {
    async fn list_by_project(&self, project_id: Uuid) -> CoreResult<Vec<Scene>> {
        sqlx::query("SELECT * FROM scenes WHERE project_id = ? ORDER BY number")
            .bind(project_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx)?
            .iter()
            .map(scene_from_row)
            .collect()
    }

    async fn get(&self, id: Uuid) -> CoreResult<Scene> {
        let row = sqlx::query("SELECT * FROM scenes WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx)?
            .ok_or(CoreError::NotFound)?;
        scene_from_row(&row)
    }

    async fn insert(&self, s: &Scene) -> CoreResult<()> {
        sqlx::query(
            "INSERT INTO scenes (id, project_id, number, title, int_ext, time_of_day, location_name, \
             summary, estimated_duration_min, created_at, updated_at, version) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(s.id.to_string())
        .bind(s.project_id.to_string())
        .bind(i64::from(s.number))
        .bind(&s.title)
        .bind(s.int_ext.as_str())
        .bind(s.time_of_day.as_str())
        .bind(s.location_name.as_deref())
        .bind(&s.summary)
        .bind(s.estimated_duration_min.map(i64::from))
        .bind(ts(&s.created_at))
        .bind(ts(&s.updated_at))
        .bind(s.version)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(())
    }

    async fn update(&self, s: &Scene) -> CoreResult<()> {
        let result = sqlx::query(
            "UPDATE scenes SET number = ?, title = ?, int_ext = ?, time_of_day = ?, location_name = ?, \
             summary = ?, estimated_duration_min = ?, updated_at = ?, version = ? \
             WHERE id = ? AND version = ?",
        )
        .bind(i64::from(s.number))
        .bind(&s.title)
        .bind(s.int_ext.as_str())
        .bind(s.time_of_day.as_str())
        .bind(s.location_name.as_deref())
        .bind(&s.summary)
        .bind(s.estimated_duration_min.map(i64::from))
        .bind(ts(&s.updated_at))
        .bind(s.version)
        .bind(s.id.to_string())
        .bind(s.version - 1)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;

        if result.rows_affected() == 0 {
            SceneRepository::get(self, s.id).await?;
            return Err(CoreError::Conflict);
        }
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> CoreResult<()> {
        let result = sqlx::query("DELETE FROM scenes WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx)?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }
}
