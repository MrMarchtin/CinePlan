//! Cœur métier de CinePlan.
//!
//! Règle d'architecture : ce crate ne dépend ni de Tauri, ni de Svelte, ni d'un
//! système d'exploitation, ni d'une base de données précise. Il définit les
//! entités, les règles de validation et les *traits* de stockage que les autres
//! crates (SQLite, PostgreSQL, serveur Axum, mobile) implémentent ou utilisent.

#[macro_use]
mod macros;

pub mod error;
pub mod project;
pub mod repository;
pub mod scene;
pub mod service;
pub mod validation;

pub use error::{CoreError, CoreResult};
pub use project::{NewProject, Project, ProjectStatus, ProjectUpdate};
pub use repository::{ProjectRepository, SceneRepository};
pub use scene::{IntExt, NewScene, Scene, SceneUpdate, TimeOfDay};
pub use service::{ProjectService, SceneService};
