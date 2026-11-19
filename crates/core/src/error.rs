use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    /// Donnée refusée par une règle métier (message affichable à l'utilisateur).
    #[error("{0}")]
    Validation(String),
    #[error("Élément introuvable.")]
    NotFound,
    /// Modification concurrente : la version reçue n'est plus la version courante.
    #[error("Cet élément a été modifié entre-temps. Rechargez-le avant de réessayer.")]
    Conflict,
    #[error("Erreur de stockage : {0}")]
    Storage(String),
}

impl CoreError {
    /// Code stable utilisable par l'interface pour réagir à l'erreur.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Validation(_) => "validation",
            Self::NotFound => "not_found",
            Self::Conflict => "conflict",
            Self::Storage(_) => "storage",
        }
    }
}

pub type CoreResult<T> = Result<T, CoreError>;
