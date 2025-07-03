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

// Module declarations (these point to the files in agents/)
pub mod models;
pub mod database;
pub mod utils;
pub mod crud;
pub mod execution;
pub mod import_export;
pub mod github;
pub mod scheduling;
pub mod settings;