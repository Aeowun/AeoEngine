//! Application state and frame/update orchestration.
//!
//! Owns the main application state shared by the editor, renderer, world,
//! gameplay systems, scripting runtime, input handling, and mode transitions.
pub mod app_home;
pub mod app_input;
pub mod app_project;
pub mod app_render;
pub mod app_runtime;
pub mod app_ui;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Instant;

use winit::keyboard::KeyCode;

use crate::character::CharacterSystem;
use crate::editor::Editor;
use crate::engine::entity::EntityManager;
use crate::engine::ui::RuntimeUi;
use crate::project::ProjectManager;
use crate::renderer::Renderer;
use crate::renderer::camera::{CameraController, GameplayCamera};
use crate::scripting::scene::ScriptScene;
use crate::scripting::value::{HandleKind, Value};
use crate::world::World;

use super::{EditorMode, View, physics};

pub use app_input::PendingGrab;

pub(crate) const COLOR_VOID: egui::Color32 = egui::Color32::from_rgb(1, 1, 2);
pub(crate) const COLOR_VOID_ELEVATED: egui::Color32 = egui::Color32::from_rgb(6, 7, 9);
pub(crate) const COLOR_VOID_PANEL: egui::Color32 = egui::Color32::from_rgb(10, 12, 15);

pub(crate) const COLOR_BORDER: egui::Color32 =
    egui::Color32::from_rgba_premultiplied(255, 255, 255, 20);

pub(crate) const COLOR_BORDER_BRIGHT: egui::Color32 =
    egui::Color32::from_rgba_premultiplied(255, 255, 255, 51);

pub(crate) const COLOR_TEXT: egui::Color32 = egui::Color32::from_rgb(160, 164, 171);

pub(crate) const COLOR_TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(92, 98, 108);

pub(crate) const COLOR_TEXT_BRIGHT: egui::Color32 = egui::Color32::from_rgb(255, 255, 255);

pub(crate) const COLOR_ACCENT_RED: egui::Color32 = egui::Color32::from_rgb(158, 52, 29);

pub(crate) const COLOR_ACCENT_ORANGE: egui::Color32 = egui::Color32::from_rgb(228, 91, 36);

pub(crate) const COLOR_ACCENT_LIGHT: egui::Color32 = egui::Color32::from_rgb(255, 173, 99);

pub(crate) const COLOR_ACCENT_CREAM: egui::Color32 = egui::Color32::from_rgb(255, 225, 194);

pub(crate) const COLOR_ACCENT_COOL: egui::Color32 = egui::Color32::from_rgb(96, 115, 125);

pub(crate) const RMB_GUARD_MAX_RADIUS_FACTOR: f32 = 0.45;
pub(crate) const RMB_GUARD_INNER_RADIUS_FACTOR: f32 = 0.38;
pub(crate) const RMB_GUARD_EDGE_SCROLL_SPEED: f32 = 500.0;

/// Application state shared by the window, editor, renderer, world,
/// scripting runtime, input system, and gameplay systems.
pub struct App {
    pub splash_started: Instant,

    pub view: View,
    pub renderer: Renderer,
    pub editor: Editor,
    pub world: World,
    pub project_manager: ProjectManager,

    pub last_frame_instant: Instant,
    pub physics_clock: physics::PhysicsClock,
    pub physics_world: physics::PhysicsWorld,
    pub last_mode: EditorMode,

    pub show_new_project_dialog: bool,
    pub new_project_name: String,
    pub show_open_project_dialog: bool,
    pub show_unsaved_scripts_dialog: bool,

    pub is_right_mouse_down: bool,
    pub is_middle_mouse_down: bool,
    pub last_cursor_pos: Option<(f64, f64)>,
    pub mouse_pos: (f64, f64),
    pub mouse: crate::engine::mouse::MouseController,

    pub rmb_edge_scroll_velocity: glam::Vec2,
    pub last_set_cursor_pos: Option<(f64, f64)>,

    pub is_left_mouse_down: bool,
    pub drag_start_coord: Option<crate::world::WorldCoord>,

    pub keys_down: HashSet<KeyCode>,
    pub jump_requested: bool,

    /// Runtime character simulation.
    pub character_system: CharacterSystem,

    /// Gameplay camera active during Play mode.
    pub gameplay_camera: GameplayCamera,

    /// Editor camera restored when leaving Play mode.
    pub saved_editor_camera: Option<CameraController>,

    pub script_scene: Option<ScriptScene>,

    pub script_dynamic_properties: HashMap<(HandleKind, u64), BTreeMap<String, Value>>,

    pub script_contacted_last_frame: HashSet<u64>,

    pub script_pending_events: Vec<(String, Vec<Value>)>,

    pub script_test_results: BTreeMap<String, bool>,

    pub script_pending_enable: Vec<String>,
    pub script_pending_disable: Vec<String>,

    pub script_pending_spawns: Vec<(String, u64)>,

    pub show_exit_confirmation_dialog: bool,
    pub exit_requested: bool,

    pub authored_disabled_scripts: Vec<String>,

    pub entity_manager: EntityManager,

    /// Script-created runtime UI.
    pub runtime_ui: RuntimeUi,

    /// Script-defined controller object instance.
    pub controller_object: Option<Value>,

    /// Script-defined camera object instance.
    pub camera_object: Option<Value>,

    /// Mouse orbit input consumed by the scripted camera/controller.
    pub orbit_delta: [f32; 2],

    /// Engine audio system.
    pub audio_system: crate::engine::audio::AudioSystem,

    pub pending_grab: Option<PendingGrab>,
}

impl App {
    pub fn new(width: f32, height: f32) -> Self {
        Self::with_renderer(Renderer::new(width, height))
    }

    #[cfg(test)]
    pub(crate) fn new_for_tests(width: f32, height: f32) -> Self {
        Self::with_renderer(Renderer::new_for_tests(width, height))
    }

    fn with_renderer(renderer: Renderer) -> Self {
        Self {
            view: View::Splash,

            renderer,
            editor: Editor::new(),
            world: World::new(),
            project_manager: ProjectManager::new(),

            splash_started: Instant::now(),
            last_frame_instant: Instant::now(),

            physics_clock: physics::PhysicsClock::new(),
            physics_world: physics::PhysicsWorld::new(),

            last_mode: EditorMode::default(),

            show_new_project_dialog: false,
            new_project_name: String::new(),
            show_open_project_dialog: false,
            show_unsaved_scripts_dialog: false,

            show_exit_confirmation_dialog: false,
            exit_requested: false,

            is_right_mouse_down: false,
            is_middle_mouse_down: false,
            last_cursor_pos: None,
            mouse_pos: (0.0, 0.0),
            mouse: crate::engine::mouse::MouseController::default(),

            rmb_edge_scroll_velocity: glam::Vec2::ZERO,
            last_set_cursor_pos: None,

            is_left_mouse_down: false,
            drag_start_coord: None,

            keys_down: HashSet::new(),
            jump_requested: false,

            character_system: CharacterSystem::new(),
            gameplay_camera: GameplayCamera::new(),
            saved_editor_camera: None,

            script_scene: None,
            script_dynamic_properties: HashMap::new(),
            script_contacted_last_frame: HashSet::new(),
            script_pending_events: Vec::new(),
            script_test_results: BTreeMap::new(),
            script_pending_enable: Vec::new(),
            script_pending_disable: Vec::new(),
            script_pending_spawns: Vec::new(),

            authored_disabled_scripts: Vec::new(),

            entity_manager: EntityManager::new(),

            runtime_ui: RuntimeUi::new(),
            controller_object: None,
            camera_object: None,

            orbit_delta: [0.0, 0.0],
            audio_system: crate::engine::audio::AudioSystem::new(),
            pending_grab: None,
        }
    }

    pub fn update(&mut self, egui_ctx: &egui::Context) {
        let now = Instant::now();

        let frame_time = now.duration_since(self.last_frame_instant).as_secs_f32();

        self.last_frame_instant = now;

        if self.view == View::Editor {
            if self.editor.mode == EditorMode::Play && self.last_mode == EditorMode::Editor {
                let has_dirty_scripts = self
                    .editor
                    .script_editor
                    .open_documents
                    .values()
                    .any(|document| document.dirty);

                if has_dirty_scripts && !self.show_unsaved_scripts_dialog {
                    self.editor.mode = EditorMode::Editor;
                    self.show_unsaved_scripts_dialog = true;

                    return;
                }

                self.saved_editor_camera = Some(self.editor.camera.clone());

                self.gameplay_camera
                    .set_mode_from_name(&self.world.selected_camera);

                self.physics_world.register_from_world(&self.world);
                self.world.physics_dirty_cells.clear();

                let spawned_id = self
                    .character_system
                    .spawn_player(&self.world, Some(self.world.selected_character.as_str()));

                self.mouse
                    .apply_world_defaults(self.world.cursor_visible, self.world.screen_locked);

                self.start_scripting();

                if let Some(character_id) = spawned_id {
                    let entity_id = self.entity_manager.create_entity("Player");

                    self.character_system
                        .associate_entity(entity_id, character_id);

                    self.sync_player_entity_from_character();

                    self.fire_player_spawned_event(entity_id.0);
                }
            }

            self.handle_mode_transition();

            if self.editor.mode == EditorMode::Play {
                self.update_gameplay(frame_time);
            }

            if egui_ctx.wants_keyboard_input() {
                return;
            }

            let mut move_vec = glam::Vec3::ZERO;

            if self.is_right_mouse_down
                && self.view == View::Editor
                && self.editor.mode == EditorMode::Editor
            {
                if self.keys_down.contains(&KeyCode::KeyW) {
                    move_vec.z += 1.0;
                }

                if self.keys_down.contains(&KeyCode::KeyS) {
                    move_vec.z -= 1.0;
                }

                if self.keys_down.contains(&KeyCode::KeyA) {
                    move_vec.x -= 1.0;
                }

                if self.keys_down.contains(&KeyCode::KeyD) {
                    move_vec.x += 1.0;
                }

                if self.keys_down.contains(&KeyCode::KeyE) {
                    move_vec.y += 1.0;
                }

                if self.keys_down.contains(&KeyCode::KeyQ) {
                    move_vec.y -= 1.0;
                }
            }

            if move_vec != glam::Vec3::ZERO {
                let speed = self.editor.camera.distance * 0.6 * frame_time;

                self.editor
                    .camera
                    .translate_free(move_vec.normalize() * speed);

                let target = self.editor.camera.target;

                self.editor.navigation_window.x_buf = target.x.floor().to_string();

                self.editor.navigation_window.y_buf = target.y.floor().to_string();

                self.editor.navigation_window.z_buf = target.z.floor().to_string();
            }

            if self.is_right_mouse_down
                && self.view == View::Editor
                && self.editor.mode == EditorMode::Editor
                && self.rmb_edge_scroll_velocity != glam::Vec2::ZERO
            {
                self.editor.camera.look(
                    self.rmb_edge_scroll_velocity.x * frame_time,
                    self.rmb_edge_scroll_velocity.y * frame_time,
                );
            }

            if self.editor.needs_clear_world {
                self.push_undo_snapshot();

                self.renderer.clear_chunk_cache();

                self.world = World::new();
                self.editor.clear_clipboard();
                self.editor.needs_clear_world = false;
            }

            if self.editor.needs_save {
                self.save_project();
                self.editor.needs_save = false;
            }

            if self.editor.needs_exit {
                self.exit_to_home();
                self.editor.needs_exit = false;
            }
        }
    }

    pub(crate) fn handle_mode_transition(&mut self) {
        if self.editor.mode == EditorMode::Editor && self.last_mode == EditorMode::Play {
            if let Some(saved) = self.saved_editor_camera.take() {
                self.editor.camera = saved;
            }

            self.physics_clock.reset();

            self.character_system.clear();

            self.physics_world.bodies.clear();

            self.audio_system.stop_all();

            self.stop_scripting();
            self.mouse.set_editor_defaults();

            self.world.clear_runtime_state();
        }

        self.last_mode = self.editor.mode;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::physics::{PhysicsBody, PhysicsBodyId};

    #[test]
    fn test_editor_idle_and_transition_cleanup() {
        let mut app = App::new_for_tests(800.0, 600.0);

        app.view = View::Editor;

        // Simulate that the previous frame was in Play mode.
        app.last_mode = EditorMode::Play;
        app.editor.mode = EditorMode::Editor;

        // Seed state that the transition is supposed to clear.
        app.physics_world.bodies.push(PhysicsBody::new(
            PhysicsBodyId(1),
            1,
            glam::Vec3::ZERO,
            glam::Vec3::ONE,
        ));

        app.physics_clock.update(0.1, |_| {});

        app.saved_editor_camera = Some(app.editor.camera.clone());

        // Exercise the actual transition handler.
        app.handle_mode_transition();

        assert!(
            app.physics_world.bodies.is_empty(),
            "Play -> Editor must clear physics bodies"
        );

        assert_eq!(
            app.physics_clock.accumulator, 0.0,
            "Play -> Editor must reset the physics clock"
        );

        assert!(
            app.saved_editor_camera.is_none(),
            "Play -> Editor must consume the saved gameplay camera"
        );

        assert_eq!(
            app.last_mode,
            EditorMode::Editor,
            "Transition handler must update last_mode"
        );
    }
}
