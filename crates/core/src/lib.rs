//! Cœur métier de CinePlan.
//!
//! Règle d'architecture : ce crate ne dépend ni de Tauri, ni de Svelte, ni d'un
//! système d'exploitation, ni d'une base de données précise. Il définit les
//! entités, les règles de validation et les *traits* de stockage que les autres
//! crates (SQLite, PostgreSQL, serveur Axum, mobile) implémentent ou utilisent.

#[macro_use]
mod macros;

pub mod document;
pub mod document_service;
pub mod error;
pub mod member;
pub mod member_service;
pub mod planning;
pub mod planning_service;
pub mod project;
pub mod repository;
pub mod scene;
pub mod schedule;
pub mod service;
pub mod validation;

pub use document::{
    DocCategory, DocKind, DocumentFile, DocumentUpdate, FileType, Folder, FolderRename,
    ImportDocument, NewFolder, StoredFile, MAX_FILE_BYTES,
};
pub use document_service::DocumentService;
pub use error::{CoreError, CoreResult};
pub use member::{MemberRole, MemberUpdate, NewMember, TeamMember};
pub use member_service::MemberService;
pub use planning::{
    DayItem, DayUpdate, ItemKind, ItemUpdate, NewBreakItem, NewDay, NewSceneItem, ShootingDay,
};
pub use planning_service::{DayView, PlanningService};
pub use project::{NewProject, Project, ProjectStatus, ProjectUpdate};
pub use repository::{
    DocumentRepository, FileStorage, MemberRepository, PlanningRepository, ProjectRepository,
    SceneRepository,
};
pub use scene::{IntExt, NewScene, Scene, SceneUpdate, TimeOfDay};
pub use schedule::{compute as compute_schedule, Schedule, Slot, SlotConflict, Summary};
pub use service::{ProjectService, SceneService};
