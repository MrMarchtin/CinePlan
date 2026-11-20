use chrono::NaiveDate;
use cineplan_core::*;
use cineplan_database::{LocalFileStorage, SqliteStore};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

struct World {
    projects: ProjectService,
    scenes: SceneService,
    members: MemberService,
    planning: PlanningService,
    documents: DocumentService,
    dir: PathBuf,
}

async fn world() -> World {
    let store = Arc::new(SqliteStore::in_memory().await.unwrap());
    let dir = std::env::temp_dir().join(format!("cineplan-test-{}", Uuid::new_v4()));
    let files = Arc::new(LocalFileStorage::new(dir.join("documents")));
    World {
        projects: ProjectService::new(store.clone()),
        scenes: SceneService::new(store.clone()),
        members: MemberService::new(store.clone()),
        planning: PlanningService::new(store.clone(), store.clone(), store.clone()),
        documents: DocumentService::new(store, files),
        dir,
    }
}

async fn project(w: &World) -> Project {
    w.projects
        .create(NewProject {
            title: "Film".into(),
            description: String::new(),
            genre: None,
            estimated_duration_min: None,
            start_date: None,
            end_date: None,
        })
        .await
        .unwrap()
}

async fn scene(w: &World, p: Uuid, number: u32, minutes: u32) -> Scene {
    w.scenes
        .create(NewScene {
            project_id: p,
            number,
            title: format!("Scène {number}"),
            int_ext: IntExt::Int,
            time_of_day: TimeOfDay::Jour,
            location_name: None,
            summary: String::new(),
            estimated_duration_min: Some(minutes),
        })
        .await
        .unwrap()
}

fn date(d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 12, d).unwrap()
}

async fn day(w: &World, p: Uuid, d: u32, members: Vec<Uuid>) -> ShootingDay {
    w.planning
        .create_day(NewDay {
            project_id: p,
            date: date(d),
            start_min: 480,
            end_min: 18 * 60,
            location: None,
            notes: String::new(),
            member_ids: members,
        })
        .await
        .unwrap()
}

#[tokio::test]
async fn team_member_crud_and_sorting() {
    let w = world().await;
    let p = project(&w).await;
    for (first, last) in [("Marc", "Dubois"), ("Léa", "Martin"), ("Nina", "Aït")] {
        w.members
            .create(NewMember {
                project_id: p.id,
                first_name: first.into(),
                last_name: last.into(),
                role: MemberRole::Acteur,
                role_detail: None,
                phone: None,
                email: Some("a@b.fr".into()),
                notes: String::new(),
                unavailable_dates: vec![date(6)],
            })
            .await
            .unwrap();
    }
    let list = w.members.list(p.id).await.unwrap();
    let names: Vec<_> = list.iter().map(|m| m.last_name.as_str()).collect();
    assert_eq!(names, vec!["Aït", "Dubois", "Martin"]);
    assert_eq!(list[0].unavailable_dates, vec![date(6)]);

    let m = &list[1];
    let updated = w
        .members
        .update(MemberUpdate {
            id: m.id,
            expected_version: m.version,
            first_name: m.first_name.clone(),
            last_name: m.last_name.clone(),
            role: MemberRole::Realisateur,
            role_detail: None,
            phone: Some("06 00 00 00 00".into()),
            email: None,
            notes: String::new(),
            unavailable_dates: vec![],
        })
        .await
        .unwrap();
    assert_eq!(updated.role, MemberRole::Realisateur);
    assert_eq!(updated.version, 2);

    w.members.delete(m.id).await.unwrap();
    assert_eq!(w.members.list(p.id).await.unwrap().len(), 2);
}

#[tokio::test]
async fn day_schedule_recomputes_after_a_delay() {
    let w = world().await;
    let p = project(&w).await;
    let d = day(&w, p.id, 5, vec![]).await;
    let s1 = scene(&w, p.id, 1, 60).await;
    let s2 = scene(&w, p.id, 2, 90).await;

    let i1 = w
        .planning
        .add_scene(NewSceneItem { day_id: d.id, scene_id: s1.id, duration_min: None })
        .await
        .unwrap();
    let i2 = w
        .planning
        .add_scene(NewSceneItem { day_id: d.id, scene_id: s2.id, duration_min: None })
        .await
        .unwrap();
    w.planning
        .add_break(NewBreakItem {
            day_id: d.id,
            label: "Déjeuner".into(),
            duration_min: 60,
            fixed_start_min: None,
        })
        .await
        .unwrap();

    let view = w.planning.view(d.id).await.unwrap();
    assert_eq!(view.items.len(), 3);
    assert_eq!(view.schedule.slots[1].planned_start, 540); // 09h00
    assert_eq!(view.schedule.summary.actual_end, 480 + 60 + 90 + 60);

    // La scène 1 prend 20 minutes de retard : tout ce qui suit est décalé.
    w.planning
        .update_item(ItemUpdate {
            id: i1.id,
            duration_min: 60,
            delay_min: 20,
            label: None,
            fixed_start_min: None,
        })
        .await
        .unwrap();
    let view = w.planning.view(d.id).await.unwrap();
    assert_eq!(view.schedule.slots[1].actual_start, 560);
    assert_eq!(view.schedule.slots[1].shift_min, 20);
    assert_eq!(view.schedule.summary.actual_end, 480 + 60 + 90 + 60 + 20);

    // Réordonner : la scène 2 passe avant la scène 1.
    let third = view.items[2].id;
    w.planning.reorder(d.id, vec![i2.id, i1.id, third]).await.unwrap();
    let view = w.planning.view(d.id).await.unwrap();
    assert_eq!(view.items[0].id, i2.id);

    // Une scène ne peut être planifiée qu'une fois par journée.
    let again = w
        .planning
        .add_scene(NewSceneItem { day_id: d.id, scene_id: s1.id, duration_min: None })
        .await;
    assert!(matches!(again, Err(CoreError::Validation(_))));

    // Un ordre incomplet est refusé.
    assert!(w.planning.reorder(d.id, vec![i1.id]).await.is_err());
}

#[tokio::test]
async fn one_day_per_date_and_unavailable_members_are_flagged() {
    let w = world().await;
    let p = project(&w).await;
    let nina = w
        .members
        .create(NewMember {
            project_id: p.id,
            first_name: "Nina".into(),
            last_name: "Roche".into(),
            role: MemberRole::Acteur,
            role_detail: None,
            phone: None,
            email: None,
            notes: String::new(),
            unavailable_dates: vec![date(6)],
        })
        .await
        .unwrap();

    let d5 = day(&w, p.id, 5, vec![nina.id]).await;
    assert!(w.planning.view(d5.id).await.unwrap().unavailable_member_ids.is_empty());

    let d6 = day(&w, p.id, 6, vec![nina.id]).await;
    assert_eq!(w.planning.view(d6.id).await.unwrap().unavailable_member_ids, vec![nina.id]);

    let duplicate = w
        .planning
        .create_day(NewDay {
            project_id: p.id,
            date: date(6),
            start_min: 480,
            end_min: 600,
            location: None,
            notes: String::new(),
            member_ids: vec![],
        })
        .await;
    assert!(matches!(duplicate, Err(CoreError::Validation(_))));

    let days = w.planning.list_days(p.id).await.unwrap();
    assert_eq!(days.iter().map(|d| d.date).collect::<Vec<_>>(), vec![date(5), date(6)]);
    assert_eq!(days[0].member_ids, vec![nina.id]);
}

#[tokio::test]
async fn day_update_detects_stale_version_and_replaces_members() {
    let w = world().await;
    let p = project(&w).await;
    let d = day(&w, p.id, 5, vec![]).await;
    let update = |version: i64| DayUpdate {
        id: d.id,
        expected_version: version,
        date: date(5),
        start_min: 420,
        end_min: 1000,
        location: Some("Quai de la Loire".into()),
        notes: String::new(),
        member_ids: vec![],
    };
    let updated = w.planning.update_day(update(1)).await.unwrap();
    assert_eq!(updated.start_min, 420);
    assert!(matches!(w.planning.update_day(update(1)).await, Err(CoreError::Conflict)));
}

fn write_file(dir: &PathBuf, name: &str, content: &[u8]) -> String {
    std::fs::create_dir_all(dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, content).unwrap();
    path.to_string_lossy().into_owned()
}

#[tokio::test]
async fn documents_folders_import_move_read_and_delete() {
    let w = world().await;
    let p = project(&w).await;
    let src = w.dir.join("sources");
    let pdf = write_file(&src, "scenario.pdf", b"%PDF-1.4\ncontenu de test");
    let text = write_file(&src, "notes.txt", b"juste du texte");

    // Les fichiers qui ne sont pas des PDF ou des images sont refusés.
    let refused = w
        .documents
        .import(ImportDocument {
            project_id: p.id,
            source_path: text,
            category: DocCategory::Autres,
            folder_id: None,
            scene_id: None,
            member_id: None,
        })
        .await;
    assert!(matches!(refused, Err(CoreError::Validation(_))));

    let folder = w
        .documents
        .create_folder(NewFolder {
            project_id: p.id,
            parent_id: None,
            category: DocCategory::Scenario,
            name: "Versions".into(),
        })
        .await
        .unwrap();

    let doc = w
        .documents
        .import(ImportDocument {
            project_id: p.id,
            source_path: pdf,
            category: DocCategory::Autres, // ignorée : le dossier impose la catégorie
            folder_id: Some(folder.id),
            scene_id: None,
            member_id: None,
        })
        .await
        .unwrap();
    assert_eq!(doc.category, DocCategory::Scenario);
    assert_eq!(doc.kind, DocKind::Pdf);
    assert_eq!(doc.name, "scenario.pdf");
    assert_eq!(w.documents.read(doc.id).await.unwrap(), b"%PDF-1.4\ncontenu de test");

    // Un dossier non vide ne peut pas être supprimé.
    assert!(matches!(
        w.documents.delete_folder(folder.id).await,
        Err(CoreError::Validation(_))
    ));

    // Renommer et déplacer à la racine d'une autre catégorie.
    let moved = w
        .documents
        .update_document(DocumentUpdate {
            id: doc.id,
            expected_version: doc.version,
            name: "Scénario v1".into(),
            category: DocCategory::Storyboard,
            folder_id: None,
            scene_id: None,
            member_id: None,
        })
        .await
        .unwrap();
    assert_eq!(moved.category, DocCategory::Storyboard);
    assert_eq!(moved.folder_id, None);
    assert_eq!(moved.name, "Scénario v1");

    // Exporter une copie.
    let copy = w.dir.join("copie.pdf");
    w.documents.export(doc.id, copy.to_str().unwrap()).await.unwrap();
    assert!(std::fs::read(&copy).unwrap().starts_with(b"%PDF-"));

    // Le dossier est maintenant vide : suppression possible.
    w.documents.delete_folder(folder.id).await.unwrap();

    w.documents.delete_document(doc.id).await.unwrap();
    assert!(w.documents.list_documents(p.id).await.unwrap().is_empty());
    assert!(w.documents.read(doc.id).await.is_err());

    let _ = std::fs::remove_dir_all(&w.dir);
}

#[tokio::test]
async fn folder_depth_is_limited() {
    let w = world().await;
    let p = project(&w).await;
    let mut parent = None;
    let mut last = Ok(());
    for level in 0..6 {
        match w
            .documents
            .create_folder(NewFolder {
                project_id: p.id,
                parent_id: parent,
                category: DocCategory::Decors,
                name: format!("Niveau {level}"),
            })
            .await
        {
            Ok(f) => parent = Some(f.id),
            Err(e) => {
                last = Err(e);
                break;
            }
        }
    }
    assert!(matches!(last, Err(CoreError::Validation(_))));
}
