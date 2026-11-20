use cineplan_core::{
    CoreError, CoreResult, FileStorage, FileType, StoredFile, MAX_FILE_BYTES,
};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Stockage des fichiers dans un dossier de l'ordinateur :
/// `<racine>/<id du projet>/<uuid>.<ext>`.
pub struct LocalFileStorage {
    root: PathBuf,
}

fn io(context: &str, e: std::io::Error) -> CoreError {
    CoreError::Storage(format!("{context} : {e}"))
}

fn unreadable(e: std::io::Error) -> CoreError {
    CoreError::Validation(format!("Impossible de lire le fichier choisi : {e}"))
}

impl LocalFileStorage {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn project_dir(&self, project_id: Uuid) -> PathBuf {
        self.root.join(project_id.to_string())
    }

    /// Refuse tout nom qui ne serait pas un simple nom de fichier.
    fn stored_path(&self, project_id: Uuid, stored_name: &str) -> CoreResult<PathBuf> {
        let is_plain_name = Path::new(stored_name)
            .file_name()
            .and_then(|n| n.to_str())
            .map_or(false, |n| n == stored_name);
        if !is_plain_name {
            return Err(CoreError::Validation("Nom de fichier invalide.".into()));
        }
        Ok(self.project_dir(project_id).join(stored_name))
    }
}

impl FileStorage for LocalFileStorage {
    fn import(&self, project_id: Uuid, source: &Path) -> CoreResult<StoredFile> {
        let meta = fs::metadata(source).map_err(unreadable)?;
        if !meta.is_file() {
            return Err(CoreError::Validation("Ce n'est pas un fichier.".into()));
        }
        if meta.len() > MAX_FILE_BYTES {
            return Err(CoreError::Validation(
                "Le fichier dépasse la taille maximale de 100 Mo.".into(),
            ));
        }

        let mut head = [0u8; 16];
        let read = File::open(source)
            .and_then(|mut f| f.read(&mut head))
            .map_err(unreadable)?;
        let file_type = FileType::detect(&head[..read]).ok_or_else(|| {
            CoreError::Validation(
                "Format non pris en charge : seuls les PDF et les images (PNG, JPEG, WebP) sont acceptés."
                    .into(),
            )
        })?;

        let dir = self.project_dir(project_id);
        fs::create_dir_all(&dir).map_err(|e| io("Création du dossier de documents", e))?;
        let stored_name = format!("{}.{}", Uuid::new_v4(), file_type.ext);
        fs::copy(source, dir.join(&stored_name)).map_err(|e| io("Copie du fichier", e))?;

        let original_name = source
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| stored_name.clone());

        Ok(StoredFile {
            stored_name,
            original_name,
            size_bytes: meta.len(),
            file_type,
        })
    }

    fn read(&self, project_id: Uuid, stored_name: &str) -> CoreResult<Vec<u8>> {
        let path = self.stored_path(project_id, stored_name)?;
        fs::read(path).map_err(|e| io("Lecture du fichier", e))
    }

    fn export(&self, project_id: Uuid, stored_name: &str, dest: &Path) -> CoreResult<()> {
        let path = self.stored_path(project_id, stored_name)?;
        fs::copy(path, dest).map_err(|e| io("Enregistrement de la copie", e))?;
        Ok(())
    }

    fn remove(&self, project_id: Uuid, stored_name: &str) -> CoreResult<()> {
        let path = self.stored_path(project_id, stored_name)?;
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(io("Suppression du fichier", e)),
        }
    }
}
