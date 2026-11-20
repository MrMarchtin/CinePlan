use crate::store::{get, get_time, get_uuid, map_sqlx, storage, ts, SqliteStore};
use async_trait::async_trait;
use chrono::NaiveDate;
use cineplan_core::{CoreError, CoreResult, DayItem, PlanningRepository, ShootingDay};
use sqlx::sqlite::SqliteRow;
use uuid::Uuid;

fn day_from_row(row: &SqliteRow, member_ids: Vec<Uuid>) -> CoreResult<ShootingDay> {
    Ok(ShootingDay {
        id: get_uuid(row, "id")?,
        project_id: get_uuid(row, "project_id")?,
        date: NaiveDate::parse_from_str(&get::<String>(row, "date")?, "%Y-%m-%d")
            .map_err(storage)?,
        start_min: get::<i64>(row, "start_min")? as u16,
        end_min: get::<i64>(row, "end_min")? as u16,
        location: get(row, "location")?,
        notes: get(row, "notes")?,
        member_ids,
        created_at: get_time(row, "created_at")?,
        updated_at: get_time(row, "updated_at")?,
        version: get(row, "version")?,
    })
}

fn item_from_row(row: &SqliteRow) -> CoreResult<DayItem> {
    let scene_id = match get::<Option<String>>(row, "scene_id")? {
        Some(s) => Some(Uuid::parse_str(&s).map_err(storage)?),
        None => None,
    };
    Ok(DayItem {
        id: get_uuid(row, "id")?,
        day_id: get_uuid(row, "day_id")?,
        position: get::<i64>(row, "position")? as u32,
        kind: get::<String>(row, "kind")?.parse()?,
        scene_id,
        label: get(row, "label")?,
        duration_min: get::<i64>(row, "duration_min")? as u32,
        delay_min: get::<i64>(row, "delay_min")? as i32,
        fixed_start_min: get::<Option<i64>>(row, "fixed_start_min")?.map(|v| v as u16),
    })
}

impl SqliteStore {
    async fn load_day_members(&self, day_id: &str) -> CoreResult<Vec<Uuid>> {
        sqlx::query("SELECT member_id FROM day_members WHERE day_id = ? ORDER BY member_id")
            .bind(day_id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx)?
            .iter()
            .map(|r| get_uuid(r, "member_id"))
            .collect()
    }
}

#[async_trait]
impl PlanningRepository for SqliteStore {
    async fn list_days(&self, project_id: Uuid) -> CoreResult<Vec<ShootingDay>> {
        let rows = sqlx::query("SELECT * FROM shooting_days WHERE project_id = ? ORDER BY date")
            .bind(project_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx)?;
        let mut days = Vec::with_capacity(rows.len());
        for row in &rows {
            let id: String = get(row, "id")?;
            let members = self.load_day_members(&id).await?;
            days.push(day_from_row(row, members)?);
        }
        Ok(days)
    }

    async fn get_day(&self, id: Uuid) -> CoreResult<ShootingDay> {
        let row = sqlx::query("SELECT * FROM shooting_days WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx)?
            .ok_or(CoreError::NotFound)?;
        let members = self.load_day_members(&id.to_string()).await?;
        day_from_row(&row, members)
    }

    async fn insert_day(&self, d: &ShootingDay) -> CoreResult<()> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx)?;
        sqlx::query(
            "INSERT INTO shooting_days (id, project_id, date, start_min, end_min, location, notes, \
             created_at, updated_at, version) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(d.id.to_string())
        .bind(d.project_id.to_string())
        .bind(d.date.to_string())
        .bind(i64::from(d.start_min))
        .bind(i64::from(d.end_min))
        .bind(d.location.as_deref())
        .bind(&d.notes)
        .bind(ts(&d.created_at))
        .bind(ts(&d.updated_at))
        .bind(d.version)
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx)?;
        for member_id in &d.member_ids {
            sqlx::query("INSERT INTO day_members (day_id, member_id) VALUES (?, ?)")
                .bind(d.id.to_string())
                .bind(member_id.to_string())
                .execute(&mut *tx)
                .await
                .map_err(map_sqlx)?;
        }
        tx.commit().await.map_err(map_sqlx)?;
        Ok(())
    }

    async fn update_day(&self, d: &ShootingDay) -> CoreResult<()> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx)?;
        let result = sqlx::query(
            "UPDATE shooting_days SET date = ?, start_min = ?, end_min = ?, location = ?, notes = ?, \
             updated_at = ?, version = ? WHERE id = ? AND version = ?",
        )
        .bind(d.date.to_string())
        .bind(i64::from(d.start_min))
        .bind(i64::from(d.end_min))
        .bind(d.location.as_deref())
        .bind(&d.notes)
        .bind(ts(&d.updated_at))
        .bind(d.version)
        .bind(d.id.to_string())
        .bind(d.version - 1)
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx)?;

        if result.rows_affected() == 0 {
            // Libère la connexion avant de relire (important avec une seule connexion).
            drop(tx);
            PlanningRepository::get_day(self, d.id).await?;
            return Err(CoreError::Conflict);
        }

        sqlx::query("DELETE FROM day_members WHERE day_id = ?")
            .bind(d.id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx)?;
        for member_id in &d.member_ids {
            sqlx::query("INSERT INTO day_members (day_id, member_id) VALUES (?, ?)")
                .bind(d.id.to_string())
                .bind(member_id.to_string())
                .execute(&mut *tx)
                .await
                .map_err(map_sqlx)?;
        }
        tx.commit().await.map_err(map_sqlx)?;
        Ok(())
    }

    async fn delete_day(&self, id: Uuid) -> CoreResult<()> {
        let result = sqlx::query("DELETE FROM shooting_days WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx)?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }

    async fn list_items(&self, day_id: Uuid) -> CoreResult<Vec<DayItem>> {
        sqlx::query("SELECT * FROM day_items WHERE day_id = ? ORDER BY position")
            .bind(day_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx)?
            .iter()
            .map(item_from_row)
            .collect()
    }

    async fn get_item(&self, id: Uuid) -> CoreResult<DayItem> {
        let row = sqlx::query("SELECT * FROM day_items WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx)?
            .ok_or(CoreError::NotFound)?;
        item_from_row(&row)
    }

    async fn insert_item(&self, i: &DayItem) -> CoreResult<()> {
        sqlx::query(
            "INSERT INTO day_items (id, day_id, position, kind, scene_id, label, duration_min, \
             delay_min, fixed_start_min) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(i.id.to_string())
        .bind(i.day_id.to_string())
        .bind(i64::from(i.position))
        .bind(i.kind.as_str())
        .bind(i.scene_id.map(|s| s.to_string()))
        .bind(&i.label)
        .bind(i64::from(i.duration_min))
        .bind(i64::from(i.delay_min))
        .bind(i.fixed_start_min.map(i64::from))
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(())
    }

    async fn update_item(&self, i: &DayItem) -> CoreResult<()> {
        let result = sqlx::query(
            "UPDATE day_items SET label = ?, duration_min = ?, delay_min = ?, fixed_start_min = ? \
             WHERE id = ?",
        )
        .bind(&i.label)
        .bind(i64::from(i.duration_min))
        .bind(i64::from(i.delay_min))
        .bind(i.fixed_start_min.map(i64::from))
        .bind(i.id.to_string())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }

    async fn delete_item(&self, id: Uuid) -> CoreResult<()> {
        let result = sqlx::query("DELETE FROM day_items WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx)?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }

    async fn set_item_order(&self, day_id: Uuid, ordered: &[Uuid]) -> CoreResult<()> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx)?;
        for (position, id) in ordered.iter().enumerate() {
            sqlx::query("UPDATE day_items SET position = ? WHERE id = ? AND day_id = ?")
                .bind(position as i64)
                .bind(id.to_string())
                .bind(day_id.to_string())
                .execute(&mut *tx)
                .await
                .map_err(map_sqlx)?;
        }
        tx.commit().await.map_err(map_sqlx)?;
        Ok(())
    }
}
