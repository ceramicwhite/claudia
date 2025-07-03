// This file acts as a drop-in replacement for the monolithic mcp.rs
// It re-exports all functionality from the modularized mcp/ directory

// Re-export all public items from the submodules for backward compatibility
pub use self::{
    models::*,
    server_management::*,
    import_export::*,
    project_config::*,
    server_operations::*,
};

// Module declarations (these point to the files in modules/mcp/)
#[path = "modules/mcp/models.rs"]
pub mod models;
#[path = "modules/mcp/utils.rs"]
pub mod utils;
#[path = "modules/mcp/server_management.rs"]
pub mod server_management;
#[path = "modules/mcp/import_export.rs"]
pub mod import_export;
#[path = "modules/mcp/project_config.rs"]
pub mod project_config;
#[path = "modules/mcp/server_operations.rs"]
pub mod server_operations;