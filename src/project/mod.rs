pub mod project;
pub mod recent;

pub use project::{
    ProjectManager, discover_cameras, discover_characters, discover_controllers,
    ensure_project_cameras, ensure_project_characters, ensure_project_controllers,
};
