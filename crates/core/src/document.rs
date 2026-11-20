use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Taille maximale d'un fichier importé (100 Mo).
pub const MAX_FILE_BYTES: u64 = 100 * 1024 * 1024;

str_enum! {
    /// Rangement de premier niveau (toujours présent, non modifiable).
    DocCategory {
        Scenario => "scenario",
        Storyboard => "storyboard",
        Preparation => "preparation",
        Planning => "planning",
        Decors => "decors",
        Costumes => "costumes",
        Materiel => "materiel",
        Administratif => "administratif",
        Autres => "autres",
    }
}

str_enum! {
    /// Type de fichier géré par la visionneuse intégrée.
    DocKind {
        Pdf => "pdf",
        Image => "image",
    }
}

/// Type de fichier reconnu à partir de son contenu (et non de son extension).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileType {
    pub kind: DocKind,
    pub ext: &'static str,
    pub mime: &'static str,
}

impl FileType {
    pub fn detect(head: &[u8]) -> Option<Self> {
        if head.starts_with(b"%PDF-") {
            return Some(Self { kind: DocKind::Pdf, ext: "pdf", mime: "application/pdf" });
        }
        if head.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            return Some(Self { kind: DocKind::Image, ext: "png", mime: "image/png" });
        }
        if head.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Some(Self { kind: DocKind::Image, ext: "jpg", mime: "image/jpeg" });
        }
        if head.len() >= 12 && &head[0..4] == b"RIFF" && &head[8..12] == b"WEBP" {
            return Some(Self { kind: DocKind::Image, ext: "webp", mime: "image/webp" });
        }
        None
    }
}

/// Résultat de la copie d'un fichier dans le stockage de l'application.
#[derive(Debug, Clone)]
pub struct StoredFile {
    pub stored_name: String,
    pub original_name: String,
    pub size_bytes: u64,
    pub file_type: FileType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub id: Uuid,
    pub project_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub category: DocCategory,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewFolder {
    pub project_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub category: DocCategory,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderRename {
    pub id: Uuid,
    pub expected_version: i64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentFile {
    pub id: Uuid,
    pub project_id: Uuid,
    pub folder_id: Option<Uuid>,
    pub category: DocCategory,
    /// Nom affiché (modifiable).
    pub name: String,
    pub original_name: String,
    /// Nom du fichier sur le disque (interne).
    pub stored_name: String,
    pub kind: DocKind,
    pub mime_type: String,
    pub size_bytes: u64,
    pub scene_id: Option<Uuid>,
    pub member_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportDocument {
    pub project_id: Uuid,
    pub source_path: String,
    pub category: DocCategory,
    pub folder_id: Option<Uuid>,
    pub scene_id: Option<Uuid>,
    pub member_id: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentUpdate {
    pub id: Uuid,
    pub expected_version: i64,
    pub name: String,
    pub category: DocCategory,
    pub folder_id: Option<Uuid>,
    pub scene_id: Option<Uuid>,
    pub member_id: Option<Uuid>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_supported_types_by_content() {
        assert_eq!(FileType::detect(b"%PDF-1.7\n").unwrap().kind, DocKind::Pdf);
        assert_eq!(FileType::detect(&[0x89, 0x50, 0x4E, 0x47, 0x0D]).unwrap().ext, "png");
        assert_eq!(FileType::detect(&[0xFF, 0xD8, 0xFF, 0xE0]).unwrap().mime, "image/jpeg");
        assert_eq!(FileType::detect(b"RIFF\x00\x00\x00\x00WEBPVP8 ").unwrap().ext, "webp");
    }

    #[test]
    fn rejects_other_content() {
        assert!(FileType::detect(b"MZ\x90\x00 programme").is_none());
        assert!(FileType::detect(b"Bonjour").is_none());
        assert!(FileType::detect(b"").is_none());
    }
}
