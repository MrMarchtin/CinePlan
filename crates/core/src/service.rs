use crate::error::CoreResult;
use crate::project::{NewProject, Project, ProjectUpdate};
use crate::repository::{ProjectRepository, SceneRepository};
use crate::scene::{NewScene, Scene, SceneUpdate};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

/// Cas d'usage « projets » : valide, puis délègue au stockage.
#[derive(Clone)]
pub struct ProjectService {
    repo: Arc<dyn ProjectRepository>,
}

impl ProjectService {
    pub fn new(repo: Arc<dyn ProjectRepository>) -> Self {
        Self { repo }
    }

    pub async fn list(&self) -> CoreResult<Vec<Project>> {
        self.repo.list().await
    }

    pub async fn get(&self, id: Uuid) -> CoreResult<Project> {
        self.repo.get(id).await
    }

    pub async fn create(&self, input: NewProject) -> CoreResult<Project> {
        let project = Project::create(input, Utc::now())?;
        self.repo.insert(&project).await?;
        Ok(project)
    }

    pub async fn update(&self, input: ProjectUpdate) -> CoreResult<Project> {
        let mut project = self.repo.get(input.id).await?;
        project.apply(input, Utc::now())?;
        self.repo.update(&project).await?;
        Ok(project)
    }

    pub async fn delete(&self, id: Uuid) -> CoreResult<()> {
        self.repo.soft_delete(id, Utc::now()).await
    }
}

/// Cas d'usage « scènes ».
#[derive(Clone)]
pub struct SceneService {
    repo: Arc<dyn SceneRepository>,
}

impl SceneService {
    pub fn new(repo: Arc<dyn SceneRepository>) -> Self {
        Self { repo }
    }

    pub async fn list(&self, project_id: Uuid) -> CoreResult<Vec<Scene>> {
        self.repo.list_by_project(project_id).await
    }

    pub async fn create(&self, input: NewScene) -> CoreResult<Scene> {
        let scene = Scene::create(input, Utc::now())?;
        self.repo.insert(&scene).await?;
        Ok(scene)
    }

    pub async fn update(&self, input: SceneUpdate) -> CoreResult<Scene> {
        let mut scene = self.repo.get(input.id).await?;
        scene.apply(input, Utc::now())?;
        self.repo.update(&scene).await?;
        Ok(scene)
    }

    pub async fn delete(&self, id: Uuid) -> CoreResult<()> {
        self.repo.delete(id).await
    }
}
