use crate::error::{CoreError, CoreResult};
use crate::validation::{minutes_range, optional_text, required_text};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

str_enum! {
    /// Nature d'un élément de la journée.
    ItemKind {
        Scene => "scene",
        Break => "break",
    }
}

/// Une journée de tournage. Horaires en minutes depuis minuit (480 = 08h00).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShootingDay {
    pub id: Uuid,
    pub project_id: Uuid,
    pub date: NaiveDate,
    pub start_min: u16,
    pub end_min: u16,
    pub location: Option<String>,
    pub notes: String,
    /// Membres convoqués ce jour-là.
    pub member_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewDay {
    pub project_id: Uuid,
    pub date: NaiveDate,
    pub start_min: u16,
    pub end_min: u16,
    pub location: Option<String>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub member_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DayUpdate {
    pub id: Uuid,
    pub expected_version: i64,
    pub date: NaiveDate,
    pub start_min: u16,
    pub end_min: u16,
    pub location: Option<String>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub member_ids: Vec<Uuid>,
}

fn dedup(mut ids: Vec<Uuid>) -> Vec<Uuid> {
    ids.sort();
    ids.dedup();
    ids
}

impl ShootingDay {
    pub fn create(input: NewDay, now: DateTime<Utc>) -> CoreResult<Self> {
        minutes_range(input.start_min, input.end_min)?;
        Ok(Self {
            id: Uuid::new_v4(),
            project_id: input.project_id,
            date: input.date,
            start_min: input.start_min,
            end_min: input.end_min,
            location: optional_text(input.location),
            notes: input.notes.trim().to_string(),
            member_ids: dedup(input.member_ids),
            created_at: now,
            updated_at: now,
            version: 1,
        })
    }

    pub fn apply(&mut self, update: DayUpdate, now: DateTime<Utc>) -> CoreResult<()> {
        if self.version != update.expected_version {
            return Err(CoreError::Conflict);
        }
        minutes_range(update.start_min, update.end_min)?;
        self.date = update.date;
        self.start_min = update.start_min;
        self.end_min = update.end_min;
        self.location = optional_text(update.location);
        self.notes = update.notes.trim().to_string();
        self.member_ids = dedup(update.member_ids);
        self.updated_at = now;
        self.version += 1;
        Ok(())
    }
}

/// Élément ordonné d'une journée : une scène à tourner ou une pause.
///
/// `duration_min` est la durée prévue ; `delay_min` l'écart constaté
/// (positif = retard, négatif = avance). `fixed_start_min` n'a de sens que
/// pour une pause à heure fixe (ex. déjeuner à 12h00).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DayItem {
    pub id: Uuid,
    pub day_id: Uuid,
    pub position: u32,
    pub kind: ItemKind,
    pub scene_id: Option<Uuid>,
    pub label: String,
    pub duration_min: u32,
    pub delay_min: i32,
    pub fixed_start_min: Option<u16>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSceneItem {
    pub day_id: Uuid,
    pub scene_id: Uuid,
    pub duration_min: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewBreakItem {
    pub day_id: Uuid,
    pub label: String,
    pub duration_min: u32,
    pub fixed_start_min: Option<u16>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemUpdate {
    pub id: Uuid,
    pub duration_min: u32,
    pub delay_min: i32,
    pub label: Option<String>,
    pub fixed_start_min: Option<u16>,
}

fn check_duration(minutes: u32) -> CoreResult<u32> {
    if minutes == 0 || minutes > 1440 {
        return Err(CoreError::Validation(
            "La durée doit être comprise entre 1 minute et 24 heures.".into(),
        ));
    }
    Ok(minutes)
}

fn check_delay(minutes: i32) -> CoreResult<i32> {
    if minutes.abs() > 1440 {
        return Err(CoreError::Validation(
            "Le retard ou l'avance ne peut pas dépasser 24 heures.".into(),
        ));
    }
    Ok(minutes)
}

fn check_fixed(start: Option<u16>) -> CoreResult<Option<u16>> {
    match start {
        Some(s) if s >= 1440 => Err(CoreError::Validation(
            "L'heure fixe doit être comprise entre 00h00 et 23h59.".into(),
        )),
        other => Ok(other),
    }
}

impl DayItem {
    pub fn new_scene(
        day_id: Uuid,
        scene_id: Uuid,
        duration_min: u32,
        position: u32,
    ) -> CoreResult<Self> {
        Ok(Self {
            id: Uuid::new_v4(),
            day_id,
            position,
            kind: ItemKind::Scene,
            scene_id: Some(scene_id),
            label: String::new(),
            duration_min: check_duration(duration_min)?,
            delay_min: 0,
            fixed_start_min: None,
        })
    }

    pub fn new_break(input: NewBreakItem, position: u32) -> CoreResult<Self> {
        Ok(Self {
            id: Uuid::new_v4(),
            day_id: input.day_id,
            position,
            kind: ItemKind::Break,
            scene_id: None,
            label: required_text("Le nom de la pause", &input.label, 100)?,
            duration_min: check_duration(input.duration_min)?,
            delay_min: 0,
            fixed_start_min: check_fixed(input.fixed_start_min)?,
        })
    }

    pub fn apply(&mut self, update: ItemUpdate) -> CoreResult<()> {
        let duration = check_duration(update.duration_min)?;
        let delay = check_delay(update.delay_min)?;
        if self.kind == ItemKind::Break {
            let label = update.label.unwrap_or_else(|| self.label.clone());
            self.label = required_text("Le nom de la pause", &label, 100)?;
            self.fixed_start_min = check_fixed(update.fixed_start_min)?;
        }
        self.duration_min = duration;
        self.delay_min = delay;
        Ok(())
    }
}
