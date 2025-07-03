// This file acts as a drop-in replacement for the monolithic agents.rs
// It re-exports all functionality from the modularized agents/ directory

// Re-export all public items from the submodules for backward compatibility
pub use self::{
    models::*,
    database::*,
    crud::*,
    execution::*,
    import_export::*,
    github::*,
    scheduling::*,
    settings::*,
};

// Module declarations (these point to the files in modules/agents/)
#[path = "modules/agents/models.rs"]
pub mod models;
#[path = "modules/agents/database.rs"]
pub mod database;
#[path = "modules/agents/utils.rs"]
pub mod utils;
#[path = "modules/agents/crud.rs"]
pub mod crud;
#[path = "modules/agents/execution.rs"]
pub mod execution;
#[path = "modules/agents/import_export.rs"]
pub mod import_export;
#[path = "modules/agents/github.rs"]
pub mod github;
#[path = "modules/agents/scheduling.rs"]
pub mod scheduling;
#[path = "modules/agents/settings.rs"]
pub mod settings;