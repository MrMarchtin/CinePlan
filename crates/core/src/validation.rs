use crate::error::{CoreError, CoreResult};
use chrono::NaiveDate;

pub const TITLE_MAX: usize = 200;

fn invalid<T>(message: String) -> CoreResult<T> {
    Err(CoreError::Validation(message))
}

/// Texte obligatoire : rogné, non vide, longueur limitée.
pub fn required_text(label: &str, value: &str, max: usize) -> CoreResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return invalid(format!("{label} est obligatoire."));
    }
    if trimmed.chars().count() > max {
        return invalid(format!("{label} ne doit pas dépasser {max} caractères."));
    }
    Ok(trimmed.to_string())
}

/// Texte facultatif : rogné, `None` s'il est vide.
pub fn optional_text(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

pub fn positive_duration(label: &str, value: Option<u32>) -> CoreResult<Option<u32>> {
    match value {
        Some(0) => invalid(format!("{label} doit être supérieure à 0.")),
        other => Ok(other),
    }
}

pub fn date_range(start: Option<NaiveDate>, end: Option<NaiveDate>) -> CoreResult<()> {
    match (start, end) {
        (Some(s), Some(e)) if e < s => {
            invalid("La date de fin ne peut pas précéder la date de début.".into())
        }
        _ => Ok(()),
    }
}
