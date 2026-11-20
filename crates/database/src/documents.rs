use crate::store::{get, get_time, get_uuid, map_sqlx, storage, ts, SqliteStore};
use async_trait::async_trait;
use cineplan_core::{CoreError, CoreResult, DocumentFile, DocumentRepository, Folder};
use sqlx::sqlite::SqliteRow;
use uuid::Uuid;

fn opt_uuid(row: &SqliteRow, col: &str) -> CoreResult<Option<Uuid>> {
    match get::<Option<String>>(row, col)? {
        Some(s) => Ok(Some(Uuid::parse_str(&s).map_err(storage)?)),
        None => Ok(None),
    }
}

fn folder_from_row(row: &SqliteRow) -> CoreResult<Folder> {
    Ok(Folder {
        id: get_uuid(row, "id")?,
        project_id: get_uuid(row, "project_id")?,
        parent_id: opt_uuid(row, "parent_id")?,
        category: get::<String>(row, "category")?.parse()?,
        name: get(row, "name")?,
        created_at: get_time(row, "created_at")?,
        updated_at: get_time(row, "updated_at")?,
        version: get(row, "version")?,
    })
}

fn document_from_row(row: &SqliteRow) -> CoreResult<DocumentFile> {
    Ok(DocumentFile {
        id: get_uuid(row, "id")?,
        project_id: get_uuid(row, "project_id")?,
        folder_id: opt_uuid(row, "folder_id")?,
        category: get::<String>(row, "category")?.parse()?,
        name: get(row, "name")?,
        original_name: get(row, "original_name")?,
        stored_name: get(row, "stored_name")?,
        kind: get::<String>(row, "kind")?.parse()?,
        mime_type: get(row, "mime_type")?,
        size_bytes: get::<i64>(row, "size_bytes")? as u64,
        scene_id: opt_uuid(row, "scene_id")?,
        member_id: opt_uuid(row, "member_id")?,
        created_at: get_time(row, "created_at")?,
        updated_at: get_time(row, "updated_at")?,
        version: get(row, "version")?,
    })
}

#[async_trait]
impl DocumentRepository for SqliteStore {
    async fn list_folders(&self, project_id: Uuid) -> CoreResult<Vec<Folder>> {
        sqlx::query("SELECT * FROM folders WHERE project_id = ? ORDER BY name COLLATE NOCASE")
            .bind(project_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx)?
            .iter()
            .map(folder_from_row)
            .collect()
    }

    async fn get_folder(&self, id: Uuid) -> CoreResult<Folder> {
        let row = sqlx::query("SELECT * FROM folders WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx)?
            .ok_or(CoreError::NotFound)?;
        folder_from_row(&row)
    }

    async fn insert_folder(&self, f: &Folder) -> CoreResult<()> {
        sqlx::query(
            "INSERT INTO folders (id, project_id, parent_id, category, name, created_at, updated_at, version) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(f.id.to_string())
        .bind(f.project_id.to_string())
        .bind(f.parent_id.map(|p| p.to_string()))
        .bind(f.category.as_str())
        .bind(&f.name)
        .bind(ts(&f.created_at))
        .bind(ts(&f.updated_at))
        .bind(f.version)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(())
    }

    async fn update_folder(&self, f: &Folder) -> CoreResult<()> {
        let result = sqlx::query(
            "UPDATE folders SET name = ?, updated_at = ?, version = ? WHERE id = ? AND version = ?",
        )
        .bind(&f.name)
        .bind(ts(&f.updated_at))
        .bind(f.version)
        .bind(f.id.to_string())
        .bind(f.version - 1)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        if result.rows_affected() == 0 {
            DocumentRepository::get_folder(self, f.id).await?;
            return Err(CoreError::Conflict);
        }
        Ok(())
    }

    async fn delete_folder(&self, id: Uuid) -> CoreResult<()> {
        let result = sqlx::query("DELETE FROM folders WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx)?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }

    async fn list_documents(&self, project_id: Uuid) -> CoreResult<Vec<DocumentFile>> {
        sqlx::query("SELECT * FROM documents WHERE project_id = ? ORDER BY name COLLATE NOCASE")
            .bind(project_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx)?
            .iter()
            .map(document_from_row)
            .collect()
    }

    async fn get_document(&self, id: Uuid) -> CoreResult<DocumentFile> {
        let row = sqlx::query("SELECT * FROM documents WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx)?
            .ok_or(CoreError::NotFound)?;
        document_from_row(&row)
    }

    async fn insert_document(&self, d: &DocumentFile) -> CoreResult<()> {
        sqlx::query(
            "INSERT INTO documents (id, project_id, folder_id, category, name, original_name, stored_name, \
             kind, mime_type, size_bytes, scene_id, member_id, created_at, updated_at, version) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(d.id.to_string())
        .bind(d.project_id.to_string())
        .bind(d.folder_id.map(|f| f.to_string()))
        .bind(d.category.as_str())
        .bind(&d.name)
        .bind(&d.original_name)
        .bind(&d.stored_name)
        .bind(d.kind.as_str())
        .bind(&d.mime_type)
        .bind(d.size_bytes as i64)
        .bind(d.scene_id.map(|s| s.to_string()))
        .bind(d.member_id.map(|m| m.to_string()))
        .bind(ts(&d.created_at))
        .bind(ts(&d.updated_at))
        .bind(d.version)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(())
    }

    async fn update_document(&self, d: &DocumentFile) -> CoreResult<()> {
        let result = sqlx::query(
            "UPDATE documents SET folder_id = ?, category = ?, name = ?, scene_id = ?, member_id = ?, \
             updated_at = ?, version = ? WHERE id = ? AND version = ?",
        )
        .bind(d.folder_id.map(|f| f.to_string()))
        .bind(d.category.as_str())
        .bind(&d.name)
        .bind(d.scene_id.map(|s| s.to_string()))
        .bind(d.member_id.map(|m| m.to_string()))
        .bind(ts(&d.updated_at))
        .bind(d.version)
        .bind(d.id.to_string())
        .bind(d.version - 1)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        if result.rows_affected() == 0 {
            DocumentRepository::get_document(self, d.id).await?;
            return Err(CoreError::Conflict);
        }
        Ok(())
    }

    async fn delete_document(&self, id: Uuid) -> CoreResult<()> {
        let result = sqlx::query("DELETE FROM documents WHERE id = ?")
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
