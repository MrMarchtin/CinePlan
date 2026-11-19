use crate::error::{CoreError, CoreResult};
use crate::validation::{date_range, optional_text, positive_duration, required_text, TITLE_MAX};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

str_enum! {
    /// Avancement d'une production.
    ProjectStatus {
        Preparation => "preparation",
        Tournage => "tournage",
        Termine => "termine",
        Archive => "archive",
    }
}

/// Une production cinématographique.
///
/// L'identifiant est un UUID généré côté client : il reste valable hors
/// connexion et lors de la future synchronisation avec PostgreSQL.
/// `version` augmente à chaque modification (détection des conflits).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub genre: Option<String>,
    pub estimated_duration_min: Option<u32>,
    pub status: ProjectStatus,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewProject {
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub genre: Option<String>,
    pub estimated_duration_min: Option<u32>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUpdate {
    pub id: Uuid,
    /// Version que l'interface a lue avant de modifier.
    pub expected_version: i64,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub genre: Option<String>,
    pub estimated_duration_min: Option<u32>,
    pub status: ProjectStatus,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

impl Project {
    pub fn create(input: NewProject, now: DateTime<Utc>) -> CoreResult<Self> {
        let title = required_text("Le titre", &input.title, TITLE_MAX)?;
        let estimated_duration_min =
            positive_duration("La durée estimée", input.estimated_duration_min)?;
        date_range(input.start_date, input.end_date)?;

        Ok(Self {
            id: Uuid::new_v4(),
            title,
            description: input.description.trim().to_string(),
            genre: optional_text(input.genre),
            estimated_duration_min,
            status: ProjectStatus::Preparation,
            start_date: input.start_date,
            end_date: input.end_date,
            created_at: now,
            updated_at: now,
            version: 1,
        })
    }

    /// Applique une modification. Refuse si la version lue est périmée.
    pub fn apply(&mut self, update: ProjectUpdate, now: DateTime<Utc>) -> CoreResult<()> {
        if self.version != update.expected_version {
            return Err(CoreError::Conflict);
        }
        let title = required_text("Le titre", &update.title, TITLE_MAX)?;
        let estimated_duration_min =
            positive_duration("La durée estimée", update.estimated_duration_min)?;
        date_range(update.start_date, update.end_date)?;

        self.title = title;
        self.description = update.description.trim().to_string();
        self.genre = optional_text(update.genre);
        self.estimated_duration_min = estimated_duration_min;
        self.status = update.status;
        self.start_date = update.start_date;
        self.end_date = update.end_date;
        self.updated_at = now;
        self.version += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_project(title: &str) -> NewProject {
        NewProject {
            title: title.into(),
            description: String::new(),
            genre: None,
            estimated_duration_min: None,
            start_date: None,
            end_date: None,
        }
    }

    fn update_of(p: &Project) -> ProjectUpdate {
        ProjectUpdate {
            id: p.id,
            expected_version: p.version,
            title: p.title.clone(),
            description: p.description.clone(),
            genre: p.genre.clone(),
            estimated_duration_min: p.estimated_duration_min,
            status: p.status,
            start_date: p.start_date,
            end_date: p.end_date,
        }
    }

    #[test]
    fn create_trims_title_and_starts_in_preparation() {
        let p = Project::create(new_project("  Le Dernier Métro  "), Utc::now()).unwrap();
        assert_eq!(p.title, "Le Dernier Métro");
        assert_eq!(p.status, ProjectStatus::Preparation);
        assert_eq!(p.version, 1);
    }

    #[test]
    fn create_rejects_blank_title() {
        let err = Project::create(new_project("   "), Utc::now()).unwrap_err();
        assert!(matches!(err, CoreError::Validation(_)));
    }

    #[test]
    fn create_rejects_inverted_dates() {
        let mut input = new_project("Film");
        input.start_date = NaiveDate::from_ymd_opt(2026, 12, 10);
        input.end_date = NaiveDate::from_ymd_opt(2026, 12, 1);
        assert!(matches!(
            Project::create(input, Utc::now()),
            Err(CoreError::Validation(_))
        ));
    }

    #[test]
    fn apply_bumps_version() {
        let mut p = Project::create(new_project("Film"), Utc::now()).unwrap();
        let mut u = update_of(&p);
        u.status = ProjectStatus::Tournage;
        p.apply(u, Utc::now()).unwrap();
        assert_eq!(p.version, 2);
        assert_eq!(p.status, ProjectStatus::Tournage);
    }

    #[test]
    fn apply_rejects_stale_version() {
        let mut p = Project::create(new_project("Film"), Utc::now()).unwrap();
        let mut u = update_of(&p);
        u.expected_version = 0;
        assert!(matches!(p.apply(u, Utc::now()), Err(CoreError::Conflict)));
        assert_eq!(p.version, 1);
    }

    #[test]
    fn status_round_trips_through_text() {
        for s in [
            ProjectStatus::Preparation,
            ProjectStatus::Tournage,
            ProjectStatus::Termine,
            ProjectStatus::Archive,
        ] {
            assert_eq!(s.as_str().parse::<ProjectStatus>().unwrap(), s);
        }
        assert!("inconnu".parse::<ProjectStatus>().is_err());
    }
}
