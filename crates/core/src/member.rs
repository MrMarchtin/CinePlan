use crate::error::{CoreError, CoreResult};
use crate::validation::{optional_text, required_text};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const NAME_MAX: usize = 100;

str_enum! {
    /// Fonction d'une personne sur le tournage.
    MemberRole {
        Realisateur => "realisateur",
        AssistantRealisateur => "assistant_realisateur",
        Cadreur => "cadreur",
        ChefOperateur => "chef_operateur",
        IngenieurSon => "ingenieur_son",
        Decorateur => "decorateur",
        Accessoiriste => "accessoiriste",
        Costumier => "costumier",
        Acteur => "acteur",
        Figurant => "figurant",
        Producteur => "producteur",
        Autre => "autre",
    }
}

/// Membre de l'équipe d'un projet (une fiche, pas un compte utilisateur).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamMember {
    pub id: Uuid,
    pub project_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub role: MemberRole,
    /// Précision libre de la fonction (surtout pour « Autre »).
    pub role_detail: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub notes: String,
    pub unavailable_dates: Vec<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewMember {
    pub project_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub role: MemberRole,
    pub role_detail: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub unavailable_dates: Vec<NaiveDate>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberUpdate {
    pub id: Uuid,
    pub expected_version: i64,
    pub first_name: String,
    pub last_name: String,
    pub role: MemberRole,
    pub role_detail: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub unavailable_dates: Vec<NaiveDate>,
}

struct Clean {
    first_name: String,
    last_name: String,
    role_detail: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    notes: String,
    unavailable_dates: Vec<NaiveDate>,
}

fn clean(
    first_name: &str,
    last_name: &str,
    role_detail: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    notes: &str,
    mut dates: Vec<NaiveDate>,
) -> CoreResult<Clean> {
    let first_name = required_text("Le prénom", first_name, NAME_MAX)?;
    let last_name = required_text("Le nom", last_name, NAME_MAX)?;
    let email = optional_text(email);
    if let Some(e) = &email {
        if !e.contains('@') {
            return Err(CoreError::Validation(
                "L'adresse e-mail n'est pas valide.".into(),
            ));
        }
    }
    dates.sort();
    dates.dedup();
    Ok(Clean {
        first_name,
        last_name,
        role_detail: optional_text(role_detail),
        phone: optional_text(phone),
        email,
        notes: notes.trim().to_string(),
        unavailable_dates: dates,
    })
}

impl TeamMember {
    pub fn create(input: NewMember, now: DateTime<Utc>) -> CoreResult<Self> {
        let c = clean(
            &input.first_name,
            &input.last_name,
            input.role_detail,
            input.phone,
            input.email,
            &input.notes,
            input.unavailable_dates,
        )?;
        Ok(Self {
            id: Uuid::new_v4(),
            project_id: input.project_id,
            first_name: c.first_name,
            last_name: c.last_name,
            role: input.role,
            role_detail: c.role_detail,
            phone: c.phone,
            email: c.email,
            notes: c.notes,
            unavailable_dates: c.unavailable_dates,
            created_at: now,
            updated_at: now,
            version: 1,
        })
    }

    pub fn apply(&mut self, update: MemberUpdate, now: DateTime<Utc>) -> CoreResult<()> {
        if self.version != update.expected_version {
            return Err(CoreError::Conflict);
        }
        let c = clean(
            &update.first_name,
            &update.last_name,
            update.role_detail,
            update.phone,
            update.email,
            &update.notes,
            update.unavailable_dates,
        )?;
        self.first_name = c.first_name;
        self.last_name = c.last_name;
        self.role = update.role;
        self.role_detail = c.role_detail;
        self.phone = c.phone;
        self.email = c.email;
        self.notes = c.notes;
        self.unavailable_dates = c.unavailable_dates;
        self.updated_at = now;
        self.version += 1;
        Ok(())
    }

    pub fn is_available_on(&self, date: NaiveDate) -> bool {
        !self.unavailable_dates.contains(&date)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_member(first: &str, last: &str) -> NewMember {
        NewMember {
            project_id: Uuid::new_v4(),
            first_name: first.into(),
            last_name: last.into(),
            role: MemberRole::Acteur,
            role_detail: None,
            phone: Some("  ".into()),
            email: None,
            notes: String::new(),
            unavailable_dates: vec![],
        }
    }

    #[test]
    fn create_requires_first_and_last_name() {
        assert!(TeamMember::create(new_member("", "Roche"), Utc::now()).is_err());
        assert!(TeamMember::create(new_member("Nina", " "), Utc::now()).is_err());
        assert!(TeamMember::create(new_member("Nina", "Roche"), Utc::now()).is_ok());
    }

    #[test]
    fn blank_phone_becomes_none_and_bad_email_is_refused() {
        let m = TeamMember::create(new_member("Nina", "Roche"), Utc::now()).unwrap();
        assert_eq!(m.phone, None);

        let mut input = new_member("Nina", "Roche");
        input.email = Some("pas-un-mail".into());
        assert!(matches!(
            TeamMember::create(input, Utc::now()),
            Err(CoreError::Validation(_))
        ));
    }

    #[test]
    fn unavailable_dates_are_sorted_deduplicated_and_checked() {
        let d1 = NaiveDate::from_ymd_opt(2026, 12, 6).unwrap();
        let d2 = NaiveDate::from_ymd_opt(2026, 12, 5).unwrap();
        let mut input = new_member("Nina", "Roche");
        input.unavailable_dates = vec![d1, d2, d1];
        let m = TeamMember::create(input, Utc::now()).unwrap();
        assert_eq!(m.unavailable_dates, vec![d2, d1]);
        assert!(!m.is_available_on(d1));
        assert!(m.is_available_on(NaiveDate::from_ymd_opt(2026, 12, 7).unwrap()));
    }

    #[test]
    fn role_round_trips_through_text() {
        assert_eq!(
            "assistant_realisateur".parse::<MemberRole>().unwrap(),
            MemberRole::AssistantRealisateur
        );
        assert_eq!(MemberRole::IngenieurSon.as_str(), "ingenieur_son");
    }
}
