use cineplan_core::*;
use cineplan_database::SqliteStore;
use std::sync::Arc;
use uuid::Uuid;

async fn services() -> (ProjectService, SceneService) {
    let store = Arc::new(SqliteStore::in_memory().await.expect("base en mémoire"));
    (ProjectService::new(store.clone()), SceneService::new(store))
}

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

fn update_of(p: &Project, title: &str) -> ProjectUpdate {
    ProjectUpdate {
        id: p.id,
        expected_version: p.version,
        title: title.into(),
        description: p.description.clone(),
        genre: p.genre.clone(),
        estimated_duration_min: p.estimated_duration_min,
        status: p.status,
        start_date: p.start_date,
        end_date: p.end_date,
    }
}

fn new_scene(project_id: Uuid, number: u32, title: &str) -> NewScene {
    NewScene {
        project_id,
        number,
        title: title.into(),
        int_ext: IntExt::Int,
        time_of_day: TimeOfDay::Jour,
        location_name: Some("Appartement".into()),
        summary: String::new(),
        estimated_duration_min: Some(20),
    }
}

#[tokio::test]
async fn project_crud_round_trip() {
    let (projects, _) = services().await;

    let created = projects.create(new_project("Le Dernier Métro")).await.unwrap();
    assert_eq!(projects.list().await.unwrap().len(), 1);
    assert_eq!(projects.get(created.id).await.unwrap(), created);

    let updated = projects
        .update(update_of(&created, "Le Dernier Métro (v2)"))
        .await
        .unwrap();
    assert_eq!(updated.version, 2);
    assert_eq!(projects.get(created.id).await.unwrap().title, "Le Dernier Métro (v2)");

    projects.delete(created.id).await.unwrap();
    assert!(projects.list().await.unwrap().is_empty());
    assert!(matches!(projects.get(created.id).await, Err(CoreError::NotFound)));
}

#[tokio::test]
async fn concurrent_edit_is_detected() {
    let (projects, _) = services().await;
    let created = projects.create(new_project("Film")).await.unwrap();

    projects.update(update_of(&created, "Première modif")).await.unwrap();
    // Deuxième éditeur, resté sur la version 1.
    let stale = projects.update(update_of(&created, "Modif périmée")).await;
    assert!(matches!(stale, Err(CoreError::Conflict)));
}

#[tokio::test]
async fn scenes_are_sorted_and_numbers_are_unique_per_project() {
    let (projects, scenes) = services().await;
    let p = projects.create(new_project("Film")).await.unwrap();

    scenes.create(new_scene(p.id, 2, "Deuxième")).await.unwrap();
    scenes.create(new_scene(p.id, 1, "Première")).await.unwrap();

    let listed = scenes.list(p.id).await.unwrap();
    assert_eq!(listed.iter().map(|s| s.number).collect::<Vec<_>>(), vec![1, 2]);

    let duplicate = scenes.create(new_scene(p.id, 2, "Doublon")).await;
    assert!(matches!(duplicate, Err(CoreError::Validation(_))));
}

#[tokio::test]
async fn scene_requires_an_existing_project() {
    let (_, scenes) = services().await;
    let orphan = scenes.create(new_scene(Uuid::new_v4(), 1, "Orpheline")).await;
    assert!(matches!(orphan, Err(CoreError::NotFound)));
}

#[tokio::test]
async fn scene_update_and_delete() {
    let (projects, scenes) = services().await;
    let p = projects.create(new_project("Film")).await.unwrap();
    let s = scenes.create(new_scene(p.id, 1, "Le réveil")).await.unwrap();

    let updated = scenes
        .update(SceneUpdate {
            id: s.id,
            expected_version: s.version,
            number: 1,
            title: "Le réveil (nuit)".into(),
            int_ext: IntExt::Int,
            time_of_day: TimeOfDay::Nuit,
            location_name: None,
            summary: "Léa se réveille en sursaut.".into(),
            estimated_duration_min: Some(25),
        })
        .await
        .unwrap();
    assert_eq!(updated.version, 2);
    assert_eq!(updated.time_of_day, TimeOfDay::Nuit);

    scenes.delete(s.id).await.unwrap();
    assert!(scenes.list(p.id).await.unwrap().is_empty());
    assert!(matches!(scenes.delete(s.id).await, Err(CoreError::NotFound)));
}
