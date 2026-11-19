use crate::error::{CoreError, CoreResult};
use crate::validation::{optional_text, positive_duration, required_text, TITLE_MAX};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

str_enum! {
    /// Intérieur ou extérieur.
    IntExt {
        Int => "int",
        Ext => "ext",
    }
}

str_enum! {
    /// Moment de la journée de la scène.
    TimeOfDay {
        Jour => "jour",
        Nuit => "nuit",
        Aube => "aube",
        Crepuscule => "crepuscule",
    }
}

/// Une scène du scénario, rattachée à un projet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    pub id: Uuid,
    pub project_id: Uuid,
    pub number: u32,
    pub title: String,
    pub int_ext: IntExt,
    pub time_of_day: TimeOfDay,
    pub location_name: Option<String>,
    pub summary: String,
    pub estimated_duration_min: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewScene {
    pub project_id: Uuid,
    pub number: u32,
    pub title: String,
    pub int_ext: IntExt,
    pub time_of_day: TimeOfDay,
    pub location_name: Option<String>,
    #[serde(default)]
    pub summary: String,
    pub estimated_duration_min: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneUpdate {
    pub id: Uuid,
    pub expected_version: i64,
    pub number: u32,
    pub title: String,
    pub int_ext: IntExt,
    pub time_of_day: TimeOfDay,
    pub location_name: Option<String>,
    #[serde(default)]
    pub summary: String,
    pub estimated_duration_min: Option<u32>,
}

fn check_number(number: u32) -> CoreResult<u32> {
    if number == 0 {
        return Err(CoreError::Validation(
            "Le numéro de scène doit être supérieur à 0.".into(),
        ));
    }
    Ok(number)
}

impl Scene {
    pub fn create(input: NewScene, now: DateTime<Utc>) -> CoreResult<Self> {
        let number = check_number(input.number)?;
        let title = required_text("Le titre", &input.title, TITLE_MAX)?;
        let estimated_duration_min =
            positive_duration("La durée estimée", input.estimated_duration_min)?;

        Ok(Self {
            id: Uuid::new_v4(),
            project_id: input.project_id,
            number,
            title,
            int_ext: input.int_ext,
            time_of_day: input.time_of_day,
            location_name: optional_text(input.location_name),
            summary: input.summary.trim().to_string(),
            estimated_duration_min,
            created_at: now,
            updated_at: now,
            version: 1,
        })
    }

    pub fn apply(&mut self, update: SceneUpdate, now: DateTime<Utc>) -> CoreResult<()> {
        if self.version != update.expected_version {
            return Err(CoreError::Conflict);
        }
        let number = check_number(update.number)?;
        let title = required_text("Le titre", &update.title, TITLE_MAX)?;
        let estimated_duration_min =
            positive_duration("La durée estimée", update.estimated_duration_min)?;

        self.number = number;
        self.title = title;
        self.int_ext = update.int_ext;
        self.time_of_day = update.time_of_day;
        self.location_name = optional_text(update.location_name);
        self.summary = update.summary.trim().to_string();
        self.estimated_duration_min = estimated_duration_min;
        self.updated_at = now;
        self.version += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_scene(number: u32, title: &str) -> NewScene {
        NewScene {
            project_id: Uuid::new_v4(),
            number,
            title: title.into(),
            int_ext: IntExt::Int,
            time_of_day: TimeOfDay::Jour,
            location_name: Some("  ".into()),
            summary: String::new(),
            estimated_duration_min: Some(20),
        }
    }

    #[test]
    fn create_drops_blank_location() {
        let s = Scene::create(new_scene(1, "Le réveil"), Utc::now()).unwrap();
        assert_eq!(s.location_name, None);
        assert_eq!(s.version, 1);
    }

    #[test]
    fn create_rejects_number_zero() {
        assert!(matches!(
            Scene::create(new_scene(0, "Scène"), Utc::now()),
            Err(CoreError::Validation(_))
        ));
    }

    #[test]
    fn create_rejects_zero_duration() {
        let mut input = new_scene(1, "Scène");
        input.estimated_duration_min = Some(0);
        assert!(Scene::create(input, Utc::now()).is_err());
    }

    #[test]
    fn apply_rejects_stale_version() {
        let mut s = Scene::create(new_scene(1, "Scène"), Utc::now()).unwrap();
        let u = SceneUpdate {
            id: s.id,
            expected_version: 7,
            number: 1,
            title: "Autre".into(),
            int_ext: IntExt::Ext,
            time_of_day: TimeOfDay::Nuit,
            location_name: None,
            summary: String::new(),
            estimated_duration_min: None,
        };
        assert!(matches!(s.apply(u, Utc::now()), Err(CoreError::Conflict)));
    }
}
