use crate::world::World;

use super::{App, EditorMode, View};

pub fn get_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let seconds = now.as_secs();

    let hours = (seconds / 3600) % 24;

    let minutes = (seconds / 60) % 60;

    let seconds = seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

impl App {
    pub fn exit_to_home(&mut self) {
        self.editor.mode = EditorMode::Editor;

        self.save_project();

        self.renderer.clear_chunk_cache();

        self.world = World::new();

        self.physics_world.bodies.clear();

        self.character_system.clear();

        self.project_manager.current_project = None;

        self.editor.history.undo_stack.clear();
        self.editor.history.redo_stack.clear();
        self.editor.clear_clipboard();

        self.view = View::Home;
    }

    pub fn load_project(&mut self) {
        if let Some(project_path) = &self.project_manager.current_project {
            self.renderer.clear_chunk_cache();

            self.editor
                .script_editor
                .refresh_scripts(&Some(project_path.clone()));

            let world_path = project_path.join("world.dat");

            if let Err(error) = crate::world::persistence::load_world(&mut self.world, &world_path)
            {
                eprintln!("Failed to load world: {}", error);
            }

            let camera_path = project_path.join("camera.dat");

            if camera_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&camera_path) {
                    let parts: Vec<&str> = content.split_whitespace().collect();

                    if parts.len() >= 6 {
                        self.editor.camera.yaw =
                            parts[0].parse().unwrap_or(self.editor.camera.yaw);

                        self.editor.camera.pitch =
                            parts[1].parse().unwrap_or(self.editor.camera.pitch);

                        self.editor.camera.distance =
                            parts[2].parse().unwrap_or(self.editor.camera.distance);

                        self.editor.camera.target.x =
                            parts[3].parse().unwrap_or(self.editor.camera.target.x);

                        self.editor.camera.target.y =
                            parts[4].parse().unwrap_or(self.editor.camera.target.y);

                        self.editor.camera.target.z =
                            parts[5].parse().unwrap_or(self.editor.camera.target.z);
                    }
                }
            }
        }
    }

    pub fn save_project(&mut self) {
        if let Some(project_path) = &self.project_manager.current_project {
            let world_path = project_path.join("world.dat");

            if let Err(error) = crate::world::persistence::save_world(&self.world, &world_path) {
                eprintln!("Failed to save world: {}", error);
            }

            let camera_path = project_path.join("camera.dat");

            let camera = &self.editor.camera;

            let content = format!(
                "{} {} {} {} {} {}",
                camera.yaw,
                camera.pitch,
                camera.distance,
                camera.target.x,
                camera.target.y,
                camera.target.z,
            );

            if let Err(error) = std::fs::write(&camera_path, content) {
                eprintln!("Failed to save camera: {}", error);
            }
        }
    }
}
