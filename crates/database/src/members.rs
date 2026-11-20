use crate::store::{get, get_time, get_uuid, map_sqlx, storage, ts, SqliteStore};
use async_trait::async_trait;
use chrono::NaiveDate;
use cineplan_core::{CoreError, CoreResult, MemberRepository, TeamMember};
use sqlx::sqlite::SqliteRow;
use uuid::Uuid;

fn join_dates(dates: &[NaiveDate]) -> String {
    dates
        .iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn parse_dates(text: &str) -> CoreResult<Vec<NaiveDate>> {
    text.split(',')
        .filter(|part| !part.is_empty())
        .map(|part| NaiveDate::parse_from_str(part, "%Y-%m-%d").map_err(storage))
        .collect()
}

fn member_from_row(row: &SqliteRow) -> CoreResult<TeamMember> {
    Ok(TeamMember {
        id: get_uuid(row, "id")?,
        project_id: get_uuid(row, "project_id")?,
        first_name: get(row, "first_name")?,
        last_name: get(row, "last_name")?,
        role: get::<String>(row, "role")?.parse()?,
        role_detail: get(row, "role_detail")?,
        phone: get(row, "phone")?,
        email: get(row, "email")?,
        notes: get(row, "notes")?,
        unavailable_dates: parse_dates(&get::<String>(row, "unavailable_dates")?)?,
        created_at: get_time(row, "created_at")?,
        updated_at: get_time(row, "updated_at")?,
        version: get(row, "version")?,
    })
}

#[async_trait]
impl MemberRepository for SqliteStore {
    async fn list_members(&self, project_id: Uuid) -> CoreResult<Vec<TeamMember>> {
        sqlx::query(
            "SELECT * FROM team_members WHERE project_id = ? \
             ORDER BY last_name COLLATE NOCASE, first_name COLLATE NOCASE",
        )
        .bind(project_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx)?
        .iter()
        .map(member_from_row)
        .collect()
    }

    async fn get_member(&self, id: Uuid) -> CoreResult<TeamMember> {
        let row = sqlx::query("SELECT * FROM team_members WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx)?
            .ok_or(CoreError::NotFound)?;
        member_from_row(&row)
    }

    async fn insert_member(&self, m: &TeamMember) -> CoreResult<()> {
        sqlx::query(
            "INSERT INTO team_members (id, project_id, first_name, last_name, role, role_detail, \
             phone, email, notes, unavailable_dates, created_at, updated_at, version) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(m.id.to_string())
        .bind(m.project_id.to_string())
        .bind(&m.first_name)
        .bind(&m.last_name)
        .bind(m.role.as_str())
        .bind(m.role_detail.as_deref())
        .bind(m.phone.as_deref())
        .bind(m.email.as_deref())
        .bind(&m.notes)
        .bind(join_dates(&m.unavailable_dates))
        .bind(ts(&m.created_at))
        .bind(ts(&m.updated_at))
        .bind(m.version)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(())
    }

    async fn update_member(&self, m: &TeamMember) -> CoreResult<()> {
        let result = sqlx::query(
            "UPDATE team_members SET first_name = ?, last_name = ?, role = ?, role_detail = ?, \
             phone = ?, email = ?, notes = ?, unavailable_dates = ?, updated_at = ?, version = ? \
             WHERE id = ? AND version = ?",
        )
        .bind(&m.first_name)
        .bind(&m.last_name)
        .bind(m.role.as_str())
        .bind(m.role_detail.as_deref())
        .bind(m.phone.as_deref())
        .bind(m.email.as_deref())
        .bind(&m.notes)
        .bind(join_dates(&m.unavailable_dates))
        .bind(ts(&m.updated_at))
        .bind(m.version)
        .bind(m.id.to_string())
        .bind(m.version - 1)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;

        if result.rows_affected() == 0 {
            MemberRepository::get_member(self, m.id).await?;
            return Err(CoreError::Conflict);
        }
        Ok(())
    }

    async fn delete_member(&self, id: Uuid) -> CoreResult<()> {
        let result = sqlx::query("DELETE FROM team_members WHERE id = ?")
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
