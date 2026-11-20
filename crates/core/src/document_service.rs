use crate::document::{
    DocCategory, DocumentFile, DocumentUpdate, Folder, FolderRename, ImportDocument, NewFolder,
};
use crate::error::{CoreError, CoreResult};
use crate::repository::{DocumentRepository, FileStorage};
use crate::service::now;
use crate::validation::{required_text, TITLE_MAX};
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

/// Profondeur maximale de l'arborescence de dossiers (au sein d'une catégorie).
const MAX_FOLDER_DEPTH: usize = 4;

#[derive(Clone)]
pub struct DocumentService {
    repo: Arc<dyn DocumentRepository>,
    files: Arc<dyn FileStorage>,
}

impl DocumentService {
    pub fn new(repo: Arc<dyn DocumentRepository>, files: Arc<dyn FileStorage>) -> Self {
        Self { repo, files }
    }

    // ----- dossiers -----

    pub async fn list_folders(&self, project_id: Uuid) -> CoreResult<Vec<Folder>> {
        self.repo.list_folders(project_id).await
    }

    pub async fn create_folder(&self, input: NewFolder) -> CoreResult<Folder> {
        let name = required_text("Le nom du dossier", &input.name, TITLE_MAX)?;
        let mut category = input.category;

        if let Some(parent_id) = input.parent_id {
            let parent = self.repo.get_folder(parent_id).await?;
            if parent.project_id != input.project_id {
                return Err(CoreError::NotFound);
            }
            category = parent.category;

            // Profondeur : on remonte la chaîne des parents.
            let mut depth = 1;
            let mut cursor = parent.parent_id;
            while let Some(id) = cursor {
                depth += 1;
                if depth >= MAX_FOLDER_DEPTH {
                    return Err(CoreError::Validation(
                        "Trop de dossiers imbriqués : 4 niveaux au maximum.".into(),
                    ));
                }
                cursor = self.repo.get_folder(id).await?.parent_id;
            }
        }

        let ts = now();
        let folder = Folder {
            id: Uuid::new_v4(),
            project_id: input.project_id,
            parent_id: input.parent_id,
            category,
            name,
            created_at: ts,
            updated_at: ts,
            version: 1,
        };
        self.repo.insert_folder(&folder).await?;
        Ok(folder)
    }

    pub async fn rename_folder(&self, input: FolderRename) -> CoreResult<Folder> {
        let mut folder = self.repo.get_folder(input.id).await?;
        if folder.version != input.expected_version {
            return Err(CoreError::Conflict);
        }
        folder.name = required_text("Le nom du dossier", &input.name, TITLE_MAX)?;
        folder.updated_at = now();
        folder.version += 1;
        self.repo.update_folder(&folder).await?;
        Ok(folder)
    }

    /// Un dossier ne peut être supprimé que vide : on évite de perdre des documents.
    pub async fn delete_folder(&self, id: Uuid) -> CoreResult<()> {
        let folder = self.repo.get_folder(id).await?;
        let has_subfolders = self
            .repo
            .list_folders(folder.project_id)
            .await?
            .iter()
            .any(|f| f.parent_id == Some(id));
        let has_documents = self
            .repo
            .list_documents(folder.project_id)
            .await?
            .iter()
            .any(|d| d.folder_id == Some(id));
        if has_subfolders || has_documents {
            return Err(CoreError::Validation(
                "Ce dossier n'est pas vide : déplacez ou supprimez son contenu d'abord.".into(),
            ));
        }
        self.repo.delete_folder(id).await
    }

    // ----- documents -----

    pub async fn list_documents(&self, project_id: Uuid) -> CoreResult<Vec<DocumentFile>> {
        self.repo.list_documents(project_id).await
    }

    /// Vérifie le dossier cible ; dans un dossier, la catégorie est celle du dossier.
    async fn resolve_location(
        &self,
        project_id: Uuid,
        category: DocCategory,
        folder_id: Option<Uuid>,
    ) -> CoreResult<(DocCategory, Option<Uuid>)> {
        match folder_id {
            Some(id) => {
                let folder = self.repo.get_folder(id).await?;
                if folder.project_id != project_id {
                    return Err(CoreError::NotFound);
                }
                Ok((folder.category, Some(id)))
            }
            None => Ok((category, None)),
        }
    }

    pub async fn import(&self, input: ImportDocument) -> CoreResult<DocumentFile> {
        let (category, folder_id) = self
            .resolve_location(input.project_id, input.category, input.folder_id)
            .await?;
        let stored = self
            .files
            .import(input.project_id, Path::new(&input.source_path))?;

        let ts = now();
        let doc = DocumentFile {
            id: Uuid::new_v4(),
            project_id: input.project_id,
            folder_id,
            category,
            name: stored.original_name.clone(),
            original_name: stored.original_name.clone(),
            stored_name: stored.stored_name.clone(),
            kind: stored.file_type.kind,
            mime_type: stored.file_type.mime.to_string(),
            size_bytes: stored.size_bytes,
            scene_id: input.scene_id,
            member_id: input.member_id,
            created_at: ts,
            updated_at: ts,
            version: 1,
        };
        if let Err(e) = self.repo.insert_document(&doc).await {
            // Pas de fichier orphelin si l'enregistrement échoue.
            let _ = self.files.remove(doc.project_id, &doc.stored_name);
            return Err(e);
        }
        Ok(doc)
    }

    /// Renomme, déplace et/ou relie un document à une scène ou à un membre.
    pub async fn update_document(&self, input: DocumentUpdate) -> CoreResult<DocumentFile> {
        let mut doc = self.repo.get_document(input.id).await?;
        if doc.version != input.expected_version {
            return Err(CoreError::Conflict);
        }
        let name = required_text("Le nom du document", &input.name, TITLE_MAX)?;
        let (category, folder_id) = self
            .resolve_location(doc.project_id, input.category, input.folder_id)
            .await?;

        doc.name = name;
        doc.category = category;
        doc.folder_id = folder_id;
        doc.scene_id = input.scene_id;
        doc.member_id = input.member_id;
        doc.updated_at = now();
        doc.version += 1;
        self.repo.update_document(&doc).await?;
        Ok(doc)
    }

    pub async fn delete_document(&self, id: Uuid) -> CoreResult<()> {
        let doc = self.repo.get_document(id).await?;
        self.repo.delete_document(id).await?;
        let _ = self.files.remove(doc.project_id, &doc.stored_name);
        Ok(())
    }

    /// Contenu du fichier, pour la visionneuse intégrée.
    pub async fn read(&self, id: Uuid) -> CoreResult<Vec<u8>> {
        let doc = self.repo.get_document(id).await?;
        self.files.read(doc.project_id, &doc.stored_name)
    }

    /// Enregistre une copie du fichier à l'emplacement choisi.
    pub async fn export(&self, id: Uuid, dest: &str) -> CoreResult<()> {
        let doc = self.repo.get_document(id).await?;
        self.files
            .export(doc.project_id, &doc.stored_name, Path::new(dest))
    }
}
