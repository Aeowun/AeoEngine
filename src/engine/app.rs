use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Instant;

use egui::RichText;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::character::CharacterSystem;
use crate::editor::{Editor, EditorTool};
use crate::engine::entity::EntityManager;
use crate::engine::ui::RuntimeUi;
use crate::project::ProjectManager;
use crate::renderer::Renderer;
use crate::renderer::camera::{CameraController, GameplayCamera};
use crate::scripting::api::HostContext;
use crate::scripting::host::ScriptHostBridge;
use crate::scripting::scene::ScriptScene;
use crate::scripting::value::{HandleKind, Value};
use crate::world::{CellType, World};

use super::{EditorMode, View, physics};

const COLOR_VOID: egui::Color32 = egui::Color32::from_rgb(1, 1, 2);
const COLOR_VOID_ELEVATED: egui::Color32 = egui::Color32::from_rgb(6, 7, 9);
const COLOR_VOID_PANEL: egui::Color32 = egui::Color32::from_rgb(10, 12, 15);

const COLOR_BORDER: egui::Color32 = egui::Color32::from_rgba_premultiplied(255, 255, 255, 20);

const COLOR_BORDER_BRIGHT: egui::Color32 =
    egui::Color32::from_rgba_premultiplied(255, 255, 255, 51);

const COLOR_TEXT: egui::Color32 = egui::Color32::from_rgb(160, 164, 171);

const COLOR_TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(92, 98, 108);

const COLOR_TEXT_BRIGHT: egui::Color32 = egui::Color32::from_rgb(255, 255, 255);

const COLOR_ACCENT_RED: egui::Color32 = egui::Color32::from_rgb(158, 52, 29);

const COLOR_ACCENT_ORANGE: egui::Color32 = egui::Color32::from_rgb(228, 91, 36);

const COLOR_ACCENT_LIGHT: egui::Color32 = egui::Color32::from_rgb(255, 173, 99);

const COLOR_ACCENT_CREAM: egui::Color32 = egui::Color32::from_rgb(255, 225, 194);

const COLOR_ACCENT_COOL: egui::Color32 = egui::Color32::from_rgb(96, 115, 125);

const RMB_GUARD_MAX_RADIUS_FACTOR: f32 = 0.45;
const RMB_GUARD_INNER_RADIUS_FACTOR: f32 = 0.38;
const RMB_GUARD_EDGE_SCROLL_SPEED: f32 = 500.0;

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
}

impl App {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            view: View::Splash,

            renderer: Renderer::new(width, height),
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

            authored_disabled_scripts: Vec::new(),

            entity_manager: EntityManager::new(),

            runtime_ui: RuntimeUi::new(),
            controller_object: None,
            camera_object: None,

            orbit_delta: [0.0, 0.0],
            audio_system: crate::engine::audio::AudioSystem::new(),
        }
    }

    /// Publishes the authoritative CharacterSystem player transform to the
    /// script-facing Player entity.
    ///
    /// This is deliberately one-way.
    ///
    /// A position difference NEVER means "teleport".
    /// Explicit teleports are handled by ScriptHostBridge::set_position().
    fn sync_player_entity_from_character(&mut self) {
        let Some(player_id) = self.entity_manager.lookup_entity("Player") else {
            return;
        };

        let Some(position) = self
            .character_system
            .get_active_player()
            .map(|player| player.transform.position)
        else {
            return;
        };

        self.entity_manager.set_position(player_id, position);
    }

    pub fn on_window_event(
        &mut self,
        event: &WindowEvent,
        egui_ctx: &egui::Context,
        window: &winit::window::Window,
    ) {
        match event {
            WindowEvent::Resized(size) => {
                self.renderer.resize(size.width as f32, size.height as f32);
            }

            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        physical_key,
                        state,
                        ..
                    },
                ..
            } => {
                if let PhysicalKey::Code(key) = physical_key {
                    let was_pressed = *state == ElementState::Pressed;

                    if was_pressed {
                        self.keys_down.insert(*key);
                    } else {
                        self.keys_down.remove(key);
                    }

                    if egui_ctx.wants_keyboard_input() {
                        let ctrl = self.keys_down.contains(&KeyCode::ControlLeft)
                            || self.keys_down.contains(&KeyCode::ControlRight);

                        if !(*key == KeyCode::KeyS && ctrl) && *key != KeyCode::KeyG {
                            return;
                        }
                    }

                    if (*key == KeyCode::ShiftLeft || *key == KeyCode::ShiftRight)
                        && self.view == View::Editor
                        && self.editor.mode == EditorMode::Editor
                    {
                        let pixels_per_point = egui_ctx.pixels_per_point();

                        let mouse_logical = egui::pos2(
                            self.mouse_pos.0 as f32 / pixels_per_point,
                            self.mouse_pos.1 as f32 / pixels_per_point,
                        );

                        let in_viewport = self.editor.viewport_rect.contains(mouse_logical);

                        if in_viewport && !egui_ctx.wants_pointer_input() {
                            self.update_hover();
                        }
                    }

                    if was_pressed {
                        let ctrl = self.keys_down.contains(&KeyCode::ControlLeft)
                            || self.keys_down.contains(&KeyCode::ControlRight);

                        match key {
                            KeyCode::F5 => {
                                if self.view == View::Editor
                                    && self.editor.mode == EditorMode::Editor
                                {
                                    self.editor.mode = EditorMode::Play;
                                }
                            }

                            KeyCode::KeyG => {
                                self.view = if self.view == View::Home {
                                    View::Editor
                                } else {
                                    View::Home
                                };
                            }

                            KeyCode::KeyS => {
                                if ctrl && self.view == View::Editor {
                                    if self.editor.show_script_workspace {
                                        let shift = self.keys_down.contains(&KeyCode::ShiftLeft)
                                            || self.keys_down.contains(&KeyCode::ShiftRight);

                                        if shift {
                                            self.editor.script_editor.save_all();
                                        } else {
                                            self.editor.script_editor.save_active();
                                        }
                                    } else {
                                        self.save_project();
                                    }
                                }
                            }

                            KeyCode::KeyN => {
                                if ctrl
                                    && self.view == View::Editor
                                    && self.editor.show_script_workspace
                                {
                                    self.editor.script_editor.show_new_script_dialog = true;
                                }
                            }

                            KeyCode::KeyZ => {
                                if ctrl && self.view == View::Editor {
                                    if let Some(prev) = self.editor.history.undo_stack.pop() {
                                        self.editor
                                            .history
                                            .redo_stack
                                            .push(self.world.cells.clone());

                                        self.world.cells = prev;
                                        self.world.rebuild_id_mapping();
                                    }
                                }
                            }

                            KeyCode::KeyY => {
                                if ctrl && self.view == View::Editor {
                                    if let Some(next) = self.editor.history.redo_stack.pop() {
                                        self.editor
                                            .history
                                            .undo_stack
                                            .push(self.world.cells.clone());

                                        self.world.cells = next;
                                        self.world.rebuild_id_mapping();
                                    }
                                }
                            }

                            KeyCode::Delete => {
                                if self.view == View::Editor
                                    && !self.editor.selected_coords.is_empty()
                                {
                                    self.editor.history.push(self.world.cells.clone());

                                    for coord in &self.editor.selected_coords {
                                        self.world.set_cell(*coord, CellType::Empty);
                                    }

                                    self.editor.selected_coords.clear();

                                    self.editor.selected_coord = None;
                                }
                            }

                            KeyCode::KeyF => {
                                if self.view == View::Editor
                                    && !self.editor.selected_coords.is_empty()
                                {
                                    let mut min = glam::Vec3::new(f32::MAX, f32::MAX, f32::MAX);

                                    let mut max = glam::Vec3::new(f32::MIN, f32::MIN, f32::MIN);

                                    for coord in &self.editor.selected_coords {
                                        let position = glam::Vec3::new(
                                            coord.x as f32,
                                            coord.y as f32,
                                            coord.z as f32,
                                        );

                                        min = min.min(position);
                                        max = max.max(position);
                                    }

                                    let center = (min + max) / 2.0;

                                    self.editor.camera.target = center;

                                    self.editor.anchor = crate::world::WorldCoord::new(
                                        center.x.round() as i32,
                                        center.y.round() as i32,
                                        center.z.round() as i32,
                                    );
                                }
                            }

                            KeyCode::Digit1 => {
                                if self.view == View::Editor {
                                    self.editor.current_tool = EditorTool::Navigate;
                                }
                            }

                            KeyCode::Digit2 => {
                                if self.view == View::Editor {
                                    self.editor.current_tool = EditorTool::Select;
                                }
                            }

                            KeyCode::Digit3 => {
                                if self.view == View::Editor {
                                    self.editor.current_tool = EditorTool::Build;
                                }
                            }

                            KeyCode::Digit4 => {
                                if self.view == View::Editor {
                                    self.editor.current_tool = EditorTool::Erase;
                                }
                            }

                            KeyCode::Escape => {
                                if self.view == View::Splash {
                                    self.view = View::Home;
                                } else if self.view == View::Home {
                                    self.show_exit_confirmation_dialog = true;
                                } else if self.editor.mode == EditorMode::Play {
                                    self.editor.mode = EditorMode::Editor;
                                } else if self.view == View::Editor {
                                    if self.is_left_mouse_down {
                                        self.is_left_mouse_down = false;

                                        self.drag_start_coord = None;
                                    } else if !self.editor.selected_coords.is_empty() {
                                        self.editor.selected_coords.clear();

                                        self.editor.selected_coord = None;
                                    } else {
                                        self.exit_to_home();
                                    }
                                }
                            }

                            KeyCode::Enter => {
                                if self.view == View::Splash {
                                    self.view = View::Home;
                                }
                            }

                            KeyCode::Space => {
                                if self.view == View::Editor && self.editor.mode == EditorMode::Play
                                {
                                    self.jump_requested = true;
                                }
                            }

                            _ => {}
                        }
                    }
                }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                let pixels_per_point = egui_ctx.pixels_per_point();

                let mouse_logical = egui::pos2(
                    self.mouse_pos.0 as f32 / pixels_per_point,
                    self.mouse_pos.1 as f32 / pixels_per_point,
                );

                let in_viewport = self.editor.viewport_rect.contains(mouse_logical);

                let wants_pointer = egui_ctx.wants_pointer_input();

                if *button == MouseButton::Right {
                    if *state == ElementState::Pressed {
                        if !in_viewport || wants_pointer {
                            return;
                        }

                        self.is_right_mouse_down = true;
                    } else {
                        self.is_right_mouse_down = false;
                        self.rmb_edge_scroll_velocity = glam::Vec2::ZERO;
                        self.last_cursor_pos = None;
                    }

                    return;
                }

                if *button == MouseButton::Middle {
                    if *state == ElementState::Pressed {
                        if !in_viewport || wants_pointer {
                            return;
                        }

                        self.is_middle_mouse_down = true;
                    } else {
                        self.is_middle_mouse_down = false;
                        self.last_cursor_pos = None;
                    }

                    return;
                }

                if *button == MouseButton::Left {
                    if *state == ElementState::Pressed {
                        if !in_viewport || wants_pointer {
                            return;
                        }

                        self.is_left_mouse_down = true;
                        self.drag_start_coord = self.editor.hovered_cell;

                        if self.editor.current_tool == EditorTool::Navigate
                            || self.editor.current_tool == EditorTool::Select
                        {
                            self.on_click();
                        }
                    } else {
                        if self.is_left_mouse_down {
                            if self.editor.current_tool == EditorTool::Build
                                || self.editor.current_tool == EditorTool::Erase
                                || self.editor.current_tool == EditorTool::Select
                            {
                                if let (Some(start), Some(end)) =
                                    (self.drag_start_coord, self.editor.hovered_cell)
                                {
                                    self.apply_tool_to_range(start, end);
                                }
                            }
                        }

                        self.is_left_mouse_down = false;
                        self.drag_start_coord = None;
                    }

                    return;
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                let dx = position.x - self.mouse_pos.0;
                let dy = position.y - self.mouse_pos.1;

                self.mouse_pos = (position.x, position.y);

                let pixels_per_point = egui_ctx.pixels_per_point();

                let mouse_logical = egui::pos2(
                    position.x as f32 / pixels_per_point,
                    position.y as f32 / pixels_per_point,
                );

                let in_viewport = self.editor.viewport_rect.contains(mouse_logical);

                if in_viewport && self.editor.mode == EditorMode::Editor {
                    self.update_hover();
                } else {
                    self.editor.hovered_cell = None;
                }

                if self.view == View::Editor
                    && self.editor.mode == EditorMode::Editor
                    && self.is_right_mouse_down
                {
                    if let Some((last_x, last_y)) = self.last_set_cursor_pos {
                        if (position.x - last_x).abs() < 0.1 && (position.y - last_y).abs() < 0.1 {
                            self.last_set_cursor_pos = None;
                            return;
                        }
                    }

                    let pixels_per_point = egui_ctx.pixels_per_point();

                    let viewport_rect = self.editor.viewport_rect;

                    let center_logical = viewport_rect.center();

                    let center_physical = egui::pos2(
                        center_logical.x * pixels_per_point,
                        center_logical.y * pixels_per_point,
                    );

                    let min_side =
                        viewport_rect.width().min(viewport_rect.height()) * pixels_per_point;

                    let max_radius = min_side * RMB_GUARD_MAX_RADIUS_FACTOR;

                    let inner_radius = min_side * RMB_GUARD_INNER_RADIUS_FACTOR;

                    let offset = egui::pos2(
                        position.x as f32 - center_physical.x,
                        position.y as f32 - center_physical.y,
                    );

                    let distance = (offset.x * offset.x + offset.y * offset.y).sqrt();

                    self.editor.camera.look(dx as f32, dy as f32);

                    if distance > inner_radius {
                        let denominator = (max_radius - inner_radius).max(0.001);

                        let strength = ((distance - inner_radius) / denominator).clamp(0.0, 1.0);

                        let direction = egui::vec2(offset.x / distance, offset.y / distance);

                        self.rmb_edge_scroll_velocity = glam::Vec2::new(direction.x, direction.y)
                            * strength
                            * RMB_GUARD_EDGE_SCROLL_SPEED;
                    } else {
                        self.rmb_edge_scroll_velocity = glam::Vec2::ZERO;
                    }

                    if distance > max_radius {
                        let clamped_offset = egui::vec2(
                            offset.x / distance * max_radius,
                            offset.y / distance * max_radius,
                        );

                        let clamped_pos = egui::pos2(
                            center_physical.x + clamped_offset.x,
                            center_physical.y + clamped_offset.y,
                        );

                        let _ = window.set_cursor_position(winit::dpi::PhysicalPosition::new(
                            clamped_pos.x as f64,
                            clamped_pos.y as f64,
                        ));

                        self.last_set_cursor_pos =
                            Some((clamped_pos.x as f64, clamped_pos.y as f64));

                        self.mouse_pos = (clamped_pos.x as f64, clamped_pos.y as f64);
                    }
                } else if self.is_middle_mouse_down
                    && self.view == View::Editor
                    && self.editor.mode == EditorMode::Editor
                {
                    self.editor.camera.orbit(dx as f32, dy as f32);
                } else {
                    self.rmb_edge_scroll_velocity = glam::Vec2::ZERO;
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let pixels_per_point = egui_ctx.pixels_per_point();

                let mouse_logical = egui::pos2(
                    self.mouse_pos.0 as f32 / pixels_per_point,
                    self.mouse_pos.1 as f32 / pixels_per_point,
                );

                let in_viewport = self.editor.viewport_rect.contains(mouse_logical);

                if !in_viewport || egui_ctx.wants_pointer_input() {
                    return;
                }

                let y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(pos) => (pos.y / 100.0) as f32,
                };

                if self.editor.mode == EditorMode::Editor {
                    self.editor.camera.zoom(y);
                    self.update_hover();
                }
            }

            _ => {}
        }
    }

    pub fn on_mouse_motion(&mut self, dx: f64, dy: f64) {
        if self.view == View::Editor && self.editor.mode == EditorMode::Play {
            self.orbit_delta = [dx as f32, dy as f32];

            self.gameplay_camera.orbit(dx as f32, dy as f32);
        }
    }

    fn update_hover(&mut self) {
        let pixels_per_point = self.editor.viewport_ppp.max(1.0);

        let rect = self.editor.viewport_rect;

        let (mouse_x, mouse_y, viewport_width, viewport_height) =
            if rect.width() > 1.0 && rect.height() > 1.0 {
                let viewport_x = rect.min.x * pixels_per_point;

                let viewport_y = rect.min.y * pixels_per_point;

                let viewport_width = rect.width() * pixels_per_point;

                let viewport_height = rect.height() * pixels_per_point;

                let mouse_x = self.mouse_pos.0 as f32 - viewport_x;

                let mouse_y = self.mouse_pos.1 as f32 - viewport_y;

                (mouse_x, mouse_y, viewport_width, viewport_height)
            } else {
                (
                    self.mouse_pos.0 as f32,
                    self.mouse_pos.1 as f32,
                    self.renderer.width(),
                    self.renderer.height(),
                )
            };

        if !self.editor.plane_picking {
            let hit = crate::editor::grid::picking::raycast_world(
                mouse_x,
                mouse_y,
                viewport_width,
                viewport_height,
                &self.editor.camera,
                &self.world,
                self.editor.mode == crate::engine::EditorMode::Editor,
            );

            if let Some((coord, normal)) = hit {
                if self.editor.current_tool == EditorTool::Build {
                    let shift_down = self.keys_down.contains(&KeyCode::ShiftLeft)
                        || self.keys_down.contains(&KeyCode::ShiftRight);

                    if !shift_down {
                        self.editor.hovered_cell = Some(crate::world::WorldCoord::new(
                            coord.x + normal.x as i32,
                            coord.y + normal.y as i32,
                            coord.z + normal.z as i32,
                        ));
                    } else {
                        self.editor.hovered_cell = Some(coord);
                    }
                } else {
                    self.editor.hovered_cell = Some(coord);
                }

                return;
            }

            if self.editor.current_tool == EditorTool::Select
                || self.editor.current_tool == EditorTool::Erase
            {
                self.editor.hovered_cell = None;
                return;
            }
        }

        self.editor.hovered_cell = crate::editor::grid::picking::update_hover(
            mouse_x,
            mouse_y,
            viewport_width,
            viewport_height,
            &self.editor.camera,
            self.editor.anchor,
        );
    }

    fn on_click(&mut self) {
        if self.view == View::Editor && self.editor.mode == EditorMode::Editor {
            if let Some(hovered) = self.editor.hovered_cell {
                match self.editor.current_tool {
                    EditorTool::Navigate => {
                        self.editor.set_anchor(hovered);
                    }

                    EditorTool::Select => {
                        self.editor.selected_coord = Some(hovered);

                        self.editor.selected_coords = vec![hovered];

                        self.editor.show_properties_window = true;
                    }

                    _ => {}
                }
            }
        }
    }

    fn apply_tool_to_range(
        &mut self,
        start: crate::world::WorldCoord,
        end: crate::world::WorldCoord,
    ) {
        if self.view != View::Editor || self.editor.mode != EditorMode::Editor {
            return;
        }

        if self.editor.current_tool == EditorTool::Select {
            self.editor.selected_coords.clear();
            self.editor.selected_coord = None;
        } else {
            self.editor.history.push(self.world.cells.clone());
        }

        let x_min = start.x.min(end.x);
        let x_max = start.x.max(end.x);
        let y_min = start.y.min(end.y);
        let y_max = start.y.max(end.y);
        let z_min = start.z.min(end.z);
        let z_max = start.z.max(end.z);

        for x in x_min..=x_max {
            for y in y_min..=y_max {
                for z in z_min..=z_max {
                    let coord = crate::world::WorldCoord::new(x, y, z);

                    match self.editor.current_tool {
                        EditorTool::Build => {
                            let cell = self.editor.build_template.clone();

                            self.world.set_cell(coord, cell.cell_type);

                            if let Some(target) = self.world.get_mut(coord) {
                                let id = target.id;

                                *target = cell;
                                target.id = id;
                            }
                        }

                        EditorTool::Erase => {
                            self.world.set_cell(coord, CellType::Empty);
                        }

                        EditorTool::Select => {
                            if self.world.get(coord).is_some() {
                                self.editor.selected_coords.push(coord);

                                self.editor.selected_coord = Some(coord);
                            }
                        }

                        _ => {}
                    }
                }
            }
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

                let spawned_id = self
                    .character_system
                    .spawn_player(&self.world, self.project_manager.current_project.as_deref());

                self.mouse
                    .apply_world_defaults(self.world.cursor_visible, self.world.screen_locked);

                self.start_scripting();

                if let Some(_character_id) = spawned_id {
                    let entity_id = self.entity_manager.create_entity("Player");

                    self.sync_player_entity_from_character();

                    self.fire_player_spawned_event(entity_id.0);
                }
            }

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

            if self.editor.mode == EditorMode::Play {
                /*
                 * ----------------------------------------------------------
                 * SCRIPT PHASE
                 * ----------------------------------------------------------
                 *
                 * CharacterSystem is authoritative for the player's current
                 * transform. Publish that transform to the script-facing
                 * Player entity before scripts run.
                 *
                 * There is NO reverse position comparison here.
                 *
                 * When AeoScript explicitly calls:
                 *
                 *   get_entity("Player").set_position(...)
                 *
                 * the existing EngineHost::set_position() implementation
                 * handles the deliberate teleport immediately.
                 */
                self.sync_player_entity_from_character();

                if let Some(scene) = self.script_scene.as_mut() {
                    let mut bridge = ScriptHostBridge {
                        entity_manager: &mut self.entity_manager,

                        world: &mut self.world,

                        mouse: &mut self.mouse,

                        dynamic_properties: &mut self.script_dynamic_properties,

                        pending_events: &mut self.script_pending_events,

                        test_results: &mut self.script_test_results,

                        pending_enable_scripts: &mut self.script_pending_enable,

                        pending_disable_scripts: &mut self.script_pending_disable,

                        runtime_ui: &mut self.runtime_ui,

                        viewport_size: [self.renderer.width(), self.renderer.height()],

                        move_input: glam::Vec2::ZERO,

                        jump_requested: false,

                        orbit_delta: [0.0, 0.0],

                        character_system: Some(&mut self.character_system),

                        gameplay_camera: Some(&mut self.gameplay_camera),
                    };

                    let mut context = HostContext {
                        delta_time: frame_time as f64,

                        engine: &mut bridge,
                    };

                    if let Err(error) = scene.update(frame_time, &mut context) {
                        self.editor.terminal_output.push_str(&format!(
                            "[{}] [ERROR] Scripting runtime error: {}\n",
                            get_timestamp(),
                            error
                        ));
                    }

                    for record in scene.drain_output() {
                        let timestamp = get_timestamp();

                        let formatted = if record.script_path.is_none()
                            && record.entity_name.is_none()
                        {
                            let engine_context = match record.severity {
                                crate::scripting::log::LogSeverity::Error => "Engine Error",

                                crate::scripting::log::LogSeverity::Warning => "Engine Warning",

                                _ => "Engine",
                            };

                            format!(
                                "[{}] [{}] {} | {}\n",
                                timestamp,
                                record.severity.name(),
                                engine_context,
                                record.message
                            )
                        } else {
                            let script_part = if let Some(path) = &record.script_path {
                                format!("{} | ", path)
                            } else {
                                String::new()
                            };

                            let entity_part = if let Some(name) = &record.entity_name {
                                if name == "Global" {
                                    String::new()
                                } else {
                                    format!(
                                        "Entity: {} | ID: {} | ",
                                        name,
                                        record.entity_id.unwrap_or(0)
                                    )
                                }
                            } else {
                                String::new()
                            };

                            let context_part = if let Some(context_name) = &record.context_name {
                                format!("{} | ", context_name)
                            } else {
                                String::new()
                            };

                            format!(
                                "[{}] [{}] {}{}{}{}\n",
                                timestamp,
                                record.severity.name(),
                                script_part,
                                entity_part,
                                context_part,
                                record.message
                            )
                        };

                        self.editor.terminal_output.push_str(&formatted);
                    }
                }

                self.physics_world.sync_with_world(&mut self.world);

                let gravity = self.world.gravity;

                let mut raw_input = glam::Vec2::ZERO;

                if self.keys_down.contains(&KeyCode::KeyW) {
                    raw_input.y += 1.0;
                }

                if self.keys_down.contains(&KeyCode::KeyS) {
                    raw_input.y -= 1.0;
                }

                if self.keys_down.contains(&KeyCode::KeyA) {
                    raw_input.x -= 1.0;
                }

                if self.keys_down.contains(&KeyCode::KeyD) {
                    raw_input.x += 1.0;
                }

                let mut contacted_this_frame = HashSet::new();

                let scene_opt = &mut self.script_scene;

                let controller_opt = &self.controller_object;

                let camera_opt = &self.camera_object;

                let entity_manager = &mut self.entity_manager;

                let world = &mut self.world;

                let mouse = &mut self.mouse;

                let dynamic_properties = &mut self.script_dynamic_properties;

                let pending_events = &mut self.script_pending_events;

                let test_results = &mut self.script_test_results;

                let pending_enable = &mut self.script_pending_enable;

                let pending_disable = &mut self.script_pending_disable;

                let runtime_ui = &mut self.runtime_ui;

                let character_system = &mut self.character_system;

                let gameplay_camera = &mut self.gameplay_camera;

                /*
                 * Input requests are edge-triggered per render frame.
                 *
                 * A fixed-timestep loop may execute several simulation steps
                 * during one render frame, but a single key press should not
                 * become several jump requests.
                 */
                let mut step_jump = self.jump_requested;

                let mut step_orbit = self.orbit_delta;

                self.orbit_delta = [0.0, 0.0];

                self.physics_clock.update(frame_time, |dt| {
                    /*
                     * --------------------------------------------------
                     * SCRIPTED CONTROLLER
                     * --------------------------------------------------
                     *
                     * The controller writes player movement state
                     * directly through EngineHost.
                     */
                    if let (Some(scene), Some(controller)) = (scene_opt.as_mut(), controller_opt) {
                        let mut bridge = ScriptHostBridge {
                            entity_manager,
                            world,
                            mouse,

                            dynamic_properties,
                            pending_events,
                            test_results,

                            pending_enable_scripts: pending_enable,

                            pending_disable_scripts: pending_disable,

                            runtime_ui,

                            move_input: raw_input,

                            jump_requested: step_jump,

                            viewport_size: [self.renderer.width(), self.renderer.height()],

                            orbit_delta: step_orbit,

                            character_system: Some(character_system),

                            gameplay_camera: Some(gameplay_camera),
                        };

                        let mut context = HostContext {
                            delta_time: dt as f64,

                            engine: &mut bridge,
                        };

                        let _ = scene.call_object_method(
                            controller,
                            "update",
                            vec![Value::Number(dt as f64)],
                            &mut context,
                        );
                    }

                    // CharacterSystem consumes the horizontal velocity supplied by the
                    // scripted controller above. It owns gravity, integration, collision,
                    // grounded state, and animation.

                    let p_world = &mut self.physics_world;
                    p_world.apply_gravity(gravity, dt);
                    p_world.integrate_positions(dt);
                    p_world.resolve_dynamic_collisions();
                    p_world.resolve_static_collisions();
                    p_world.refresh_dynamic_support();
                    p_world.update_sleeping(gravity);

                    let contacted = character_system.update_scripted(world, p_world, dt);

                    contacted_this_frame.extend(contacted);

                    // Update the scripted camera after the character has completed this
                    // fixed simulation step so it follows the latest authoritative transform.
                    if let (Some(scene), Some(camera)) = (scene_opt.as_mut(), camera_opt) {
                        let mut bridge = ScriptHostBridge {
                            entity_manager,
                            world,
                            mouse,
                            dynamic_properties,
                            pending_events,
                            test_results,
                            pending_enable_scripts: pending_enable,
                            pending_disable_scripts: pending_disable,
                            runtime_ui,
                            move_input: raw_input,
                            jump_requested: false,
                            viewport_size: [self.renderer.width(), self.renderer.height()],
                            orbit_delta: step_orbit,
                            character_system: Some(character_system),
                            gameplay_camera: Some(gameplay_camera),
                        };

                        let mut context = HostContext {
                            delta_time: dt as f64,
                            engine: &mut bridge,
                        };

                        let _ = scene.call_object_method(
                            camera,
                            "update",
                            vec![Value::Number(dt as f64)],
                            &mut context,
                        );
                    }

                    // These inputs are render-frame requests, not continuous fixed-step state.
                    // Consuming them here prevents one key press from becoming multiple jumps
                    // or repeated camera orbit operations during catch-up steps.
                    step_jump = false;
                    step_orbit = [0.0, 0.0];
                });

                /*
                 * ----------------------------------------------------------
                 * FINAL SCRIPT-FACING PLAYER SYNC
                 * ----------------------------------------------------------
                 *
                 * Physics is authoritative. Publish the final transform
                 * after the fixed simulation has completed.
                 */
                self.sync_player_entity_from_character();

                /*
                 * Collision/event processing remains outside the fixed
                 * movement implementation so overlap transitions are evaluated
                 * once per rendered frame.
                 */
                if let Some(scene) = &mut self.script_scene {
                    let mut bridge = ScriptHostBridge {
                        entity_manager: &mut self.entity_manager,

                        world: &mut self.world,

                        mouse: &mut self.mouse,

                        viewport_size: [self.renderer.width(), self.renderer.height()],

                        dynamic_properties: &mut self.script_dynamic_properties,

                        pending_events: &mut self.script_pending_events,

                        test_results: &mut self.script_test_results,

                        pending_enable_scripts: &mut self.script_pending_enable,

                        pending_disable_scripts: &mut self.script_pending_disable,

                        runtime_ui: &mut self.runtime_ui,

                        move_input: raw_input,

                        jump_requested: false,

                        orbit_delta: [0.0, 0.0],

                        character_system: Some(&mut self.character_system),

                        gameplay_camera: Some(&mut self.gameplay_camera),
                    };

                    let mut context = HostContext {
                        delta_time: frame_time as f64,

                        engine: &mut bridge,
                    };

                    for &cell_id in &contacted_this_frame {
                        if !self.script_contacted_last_frame.contains(&cell_id) {
                            if let Err(error) = scene.on_player_overlap(cell_id, true, &mut context)
                            {
                                self.editor.terminal_output.push_str(&format!(
                                    "[{}] [ERROR] Overlap start error: {}\n",
                                    get_timestamp(),
                                    error
                                ));
                            }
                        }
                    }

                    for &cell_id in &self.script_contacted_last_frame {
                        if !contacted_this_frame.contains(&cell_id) {
                            if let Err(error) =
                                scene.on_player_overlap(cell_id, false, &mut context)
                            {
                                self.editor.terminal_output.push_str(&format!(
                                    "[{}] [ERROR] Overlap end error: {}\n",
                                    get_timestamp(),
                                    error
                                ));
                            }
                        }
                    }

                    if !contacted_this_frame.is_empty() {
                        if let Err(error) =
                            scene.on_player_contact(&contacted_this_frame, &mut context)
                        {
                            self.editor.terminal_output.push_str(&format!(
                                "[{}] [ERROR] Scripting contact event error: {}\n",
                                get_timestamp(),
                                error
                            ));
                        }
                    }
                }

                self.audio_system
                    .update(&self.world, &self.project_manager.current_project);

                self.script_contacted_last_frame = contacted_this_frame;

                if let Some(player) = self.character_system.get_active_player() {
                    self.gameplay_camera.update(player, &self.world);
                }

                self.jump_requested = false;
            } else {
                self.physics_clock.reset();
                self.character_system.clear();
                self.physics_world.bodies.clear();
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
                self.editor.history.push(self.world.cells.clone());

                self.world = World::new();
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

    fn start_scripting(&mut self) {
        self.script_contacted_last_frame.clear();
        self.runtime_ui.clear();
        self.controller_object = None;
        self.camera_object = None;

        self.authored_disabled_scripts = self.world.disabled_scripts.clone();

        let Some(project_path) = &self.project_manager.current_project else {
            return;
        };

        let controller_name = self.world.selected_controller.clone();
        let camera_name = self.world.selected_camera.clone();

        match ScriptScene::load_from_bindings(
            project_path,
            &self.world,
            &self.world.script_bindings,
            &mut self.entity_manager,
            0.0,
        ) {
            Ok(mut scene) => {
                let mut bridge = ScriptHostBridge {
                    entity_manager: &mut self.entity_manager,
                    world: &mut self.world,
                    mouse: &mut self.mouse,
                    dynamic_properties: &mut self.script_dynamic_properties,
                    pending_events: &mut self.script_pending_events,
                    test_results: &mut self.script_test_results,
                    pending_enable_scripts: &mut self.script_pending_enable,
                    pending_disable_scripts: &mut self.script_pending_disable,
                    runtime_ui: &mut self.runtime_ui,
                    viewport_size: [self.renderer.width(), self.renderer.height()],
                    move_input: glam::Vec2::ZERO,
                    jump_requested: false,
                    orbit_delta: [0.0, 0.0],
                    character_system: Some(&mut self.character_system),
                    gameplay_camera: Some(&mut self.gameplay_camera),
                };

                let mut context = HostContext {
                    delta_time: 0.0,
                    engine: &mut bridge,
                };

                if let Err(error) = scene.start(&mut context) {
                    self.editor.terminal_output.push_str(&format!(
                        "[{}] [ERROR] Scripting startup error: {}\n",
                        get_timestamp(),
                        error
                    ));
                } else {
                    let player_value = Value::Namespace("player".to_string());
                    let camera_value = Value::Namespace("camera".to_string());
                    let input_value = Value::Namespace("input".to_string());

                    let controller_class = match controller_name.as_str() {
                        "thirdPerson_Controller" => "ThirdPersonController",
                        other => other,
                    };

                    if let Ok(object) = scene.instantiate_script_object(
                        controller_class,
                        vec![
                            player_value.clone(),
                            camera_value.clone(),
                            input_value.clone(),
                        ],
                        &mut context,
                    ) {
                        self.controller_object = Some(object);
                    }

                    let camera_class = match camera_name.as_str() {
                        "thirdPerson" => "ThirdPersonCamera",
                        "firstPerson" => "FirstPersonCamera",
                        "topDown" => "TopDownCamera",
                        other => other,
                    };

                    if let Ok(object) = scene.instantiate_script_object(
                        camera_class,
                        vec![player_value],
                        &mut context,
                    ) {
                        self.camera_object = Some(object);
                    }

                    self.script_scene = Some(scene);
                }
            }
            Err(error) => {
                self.editor.terminal_output.push_str(&format!(
                    "[{}] [ERROR] Scripting load error: {}\n",
                    get_timestamp(),
                    error
                ));
            }
        }
    }

    fn stop_scripting(&mut self) {
        let scene_opt = self.script_scene.take();

        if let Some(mut scene) = scene_opt {
            let mut bridge = ScriptHostBridge {
                entity_manager: &mut self.entity_manager,

                world: &mut self.world,

                mouse: &mut self.mouse,

                dynamic_properties: &mut self.script_dynamic_properties,

                pending_events: &mut self.script_pending_events,

                test_results: &mut self.script_test_results,

                pending_enable_scripts: &mut self.script_pending_enable,

                pending_disable_scripts: &mut self.script_pending_disable,

                runtime_ui: &mut self.runtime_ui,

                viewport_size: [self.renderer.width(), self.renderer.height()],

                move_input: glam::Vec2::ZERO,

                jump_requested: false,

                orbit_delta: [0.0, 0.0],

                character_system: Some(&mut self.character_system),

                gameplay_camera: Some(&mut self.gameplay_camera),
            };

            let mut context = HostContext {
                delta_time: 0.0,
                engine: &mut bridge,
            };

            scene.stop(&mut context);
        }

        self.script_contacted_last_frame.clear();
        self.script_pending_events.clear();
        self.script_test_results.clear();
        self.script_pending_enable.clear();
        self.script_pending_disable.clear();
        self.script_dynamic_properties.clear();

        self.runtime_ui.clear();

        self.controller_object = None;
        self.camera_object = None;

        self.world.disabled_scripts = std::mem::take(&mut self.authored_disabled_scripts);

        self.entity_manager.clear();
    }

    fn fire_player_spawned_event(&mut self, player_entity_id: u64) {
        if let Some(scene) = &mut self.script_scene {
            let mut bridge = ScriptHostBridge {
                entity_manager: &mut self.entity_manager,

                world: &mut self.world,

                mouse: &mut self.mouse,

                viewport_size: [self.renderer.width(), self.renderer.height()],

                dynamic_properties: &mut self.script_dynamic_properties,

                pending_events: &mut self.script_pending_events,

                test_results: &mut self.script_test_results,

                pending_enable_scripts: &mut self.script_pending_enable,

                pending_disable_scripts: &mut self.script_pending_disable,

                runtime_ui: &mut self.runtime_ui,

                move_input: glam::Vec2::ZERO,

                jump_requested: false,

                orbit_delta: [0.0, 0.0],

                character_system: Some(&mut self.character_system),

                gameplay_camera: Some(&mut self.gameplay_camera),
            };

            let mut context = HostContext {
                delta_time: 0.0,
                engine: &mut bridge,
            };

            let arguments = vec![Value::Handle {
                kind: HandleKind::Entity,
                id: player_entity_id,
            }];

            if let Err(error) = scene.dispatch_event("PlayerSpawned", arguments, &mut context) {
                self.editor.terminal_output.push_str(&format!(
                    "[{}] [ERROR] Event PlayerSpawned error: {}\n",
                    get_timestamp(),
                    error
                ));
            }
        }
    }

    pub fn exit_to_home(&mut self) {
        self.editor.mode = EditorMode::Editor;

        self.save_project();

        self.world = World::new();

        self.physics_world.bodies.clear();

        self.character_system.clear();

        self.project_manager.current_project = None;

        self.editor.history.undo_stack.clear();
        self.editor.history.redo_stack.clear();

        self.view = View::Home;
    }

    fn draw_splash_screen(&mut self, ctx: &egui::Context) {
        ctx.request_repaint();

        let elapsed = self.splash_started.elapsed().as_secs_f32();

        let rect = ctx.screen_rect();

        let painter = ctx.layer_painter(egui::LayerId::background());

        painter.rect_filled(rect, 0.0, COLOR_VOID);

        let presents_start = 0.25;
        let presents_fade_in = 0.35;
        let presents_fade_out_start = 1.15;
        let presents_fade_out = 0.45;

        let mut presents_alpha = 0.0;

        if elapsed >= presents_start {
            presents_alpha = ((elapsed - presents_start) / presents_fade_in).clamp(0.0, 1.0);
        }

        if elapsed >= presents_fade_out_start {
            presents_alpha *=
                (1.0 - (elapsed - presents_fade_out_start) / presents_fade_out).clamp(0.0, 1.0);
        }

        if presents_alpha > 0.0 {
            let color = egui::Color32::from_rgba_unmultiplied(
                COLOR_TEXT_DIM.r(),
                COLOR_TEXT_DIM.g(),
                COLOR_TEXT_DIM.b(),
                (presents_alpha * 255.0) as u8,
            );

            painter.text(
                rect.center() + egui::vec2(0.0, -18.0),
                egui::Align2::CENTER_CENTER,
                "AEOWUN PRESENTS...",
                egui::FontId::monospace(14.0),
                color,
            );
        }

        let title_start = 1.45;
        let stagger = 0.12;
        let reveal_duration = 0.38;

        let title = "AEOENGINE";

        let font = egui::FontId::monospace(52.0);

        let char_width = painter
            .layout_no_wrap("M".to_string(), font.clone(), COLOR_TEXT_BRIGHT)
            .size()
            .x;

        let total_width = char_width * title.chars().count() as f32;

        let start_x = rect.center().x - total_width * 0.5 + char_width * 0.5;

        let base_y = rect.center().y + 20.0;

        for (index, character) in title.chars().enumerate() {
            let char_start = title_start + index as f32 * stagger;

            let progress = ((elapsed - char_start) / reveal_duration).clamp(0.0, 1.0);

            if progress <= 0.0 {
                continue;
            }

            let eased = 1.0 - (1.0 - progress).powi(3);

            let x = start_x + index as f32 * char_width - (1.0 - eased) * 28.0;

            let y = base_y;

            let alpha = (eased * 255.0) as u8;

            let color = egui::Color32::from_rgba_unmultiplied(
                COLOR_TEXT_BRIGHT.r(),
                COLOR_TEXT_BRIGHT.g(),
                COLOR_TEXT_BRIGHT.b(),
                alpha,
            );

            painter.text(
                egui::pos2(x, y),
                egui::Align2::CENTER_CENTER,
                character.to_string(),
                font.clone(),
                color,
            );

            if progress < 1.0 {
                for dust_index in 0..4 {
                    let seed = index as f32 * 17.0 + dust_index as f32 * 9.0;

                    let drift = (elapsed * 7.0 + seed).sin() * 8.0;

                    let dust_x = x - (1.0 - eased) * 24.0 - dust_index as f32 * 4.0;

                    let dust_y = y + drift + (dust_index as f32 - 1.5) * 3.0;

                    let dust_alpha = ((1.0 - progress) * 45.0) as u8;

                    painter.circle_filled(
                        egui::pos2(dust_x, dust_y),
                        1.0 + dust_index as f32 * 0.25,
                        egui::Color32::from_rgba_unmultiplied(180, 180, 180, dust_alpha),
                    );
                }
            }
        }

        if elapsed >= 4.15 {
            self.view = View::Home;
        }
    }

    pub fn update_ui(&mut self, ctx: &egui::Context) {
        let mut next_view = None;

        if self.view == View::Splash {
            self.draw_splash_screen(ctx);
        } else if self.view == View::Home {
            self.draw_home_screen(ctx, &mut next_view);
        } else if self.view == View::Editor {
            self.editor
                .show_ui(ctx, &mut self.world, &self.project_manager.current_project);

            if self.editor.mode == EditorMode::Editor && !self.editor.show_script_workspace {
                let viewport = self.editor.viewport_rect;

                egui::Area::new("editor_navigation_help".into())
                    .order(egui::Order::Foreground)
                    .fixed_pos(egui::pos2(viewport.right() - 156.0, viewport.top() + 12.0))
                    .show(ctx, |ui| {
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgba_unmultiplied(10, 12, 15, 210))
                            .stroke(egui::Stroke::new(
                                1.0,
                                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 25),
                            ))
                            .rounding(4.0)
                            .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new("RMB + WASD   MOVE").monospace().size(10.0));

                                ui.label(
                                    RichText::new("RMB + Q/E    UP / DOWN")
                                        .monospace()
                                        .size(10.0),
                                );

                                ui.label(
                                    RichText::new("MMB + MOUSE  ORBIT").monospace().size(10.0),
                                );

                                ui.label(RichText::new("SCROLL       ZOOM").monospace().size(10.0));
                            });
                    });
            }

            self.render_runtime_ui(ctx);

            if self.show_unsaved_scripts_dialog {
                egui::Window::new("Unsaved Script Changes")
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.label("You have unsaved changes in your scripts.");

                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            if ui.button("Save & Play").clicked() {
                                self.editor.script_editor.save_all();

                                self.editor.mode = EditorMode::Play;

                                self.show_unsaved_scripts_dialog = false;
                            }

                            if ui.button("Don't Save").clicked() {
                                self.editor.mode = EditorMode::Play;

                                self.show_unsaved_scripts_dialog = false;
                            }

                            if ui.button("Cancel").clicked() {
                                self.show_unsaved_scripts_dialog = false;
                            }
                        });
                    });
            }
        }

        if let Some(view) = next_view {
            self.view = view;
        }
    }

    fn render_runtime_ui(&mut self, ctx: &egui::Context) {
        if self.editor.mode != EditorMode::Play {
            return;
        }

        let rect = self.editor.viewport_rect;

        if rect.width() <= 1.0 || rect.height() <= 1.0 {
            return;
        }

        for (&id, element) in &self.runtime_ui.elements {
            let common = element.common();

            if !common.visible {
                continue;
            }

            let position = egui::pos2(
                rect.min.x + common.position[0],
                rect.min.y + common.position[1],
            );

            let size = egui::vec2(common.size[0], common.size[1]);

            let color = egui::Color32::from_rgba_unmultiplied(
                (common.color[0] * 255.0).clamp(0.0, 255.0) as u8,
                (common.color[1] * 255.0).clamp(0.0, 255.0) as u8,
                (common.color[2] * 255.0).clamp(0.0, 255.0) as u8,
                (common.color[3] * 255.0).clamp(0.0, 255.0) as u8,
            );

            let area_id = egui::Id::new("runtime_ui_element").with(id);

            match element {
                crate::engine::ui::UiElement::Panel(_) => {
                    egui::Area::new(area_id)
                        .fixed_pos(position)
                        .order(egui::Order::Middle)
                        .show(ctx, |ui| {
                            ui.set_clip_rect(rect);

                            let (response, _) = ui.allocate_exact_size(size, egui::Sense::hover());

                            ui.painter().rect_filled(response, 4.0, color);
                        });
                }

                crate::engine::ui::UiElement::Text(text) => {
                    egui::Area::new(area_id)
                        .fixed_pos(position)
                        .order(egui::Order::Middle)
                        .show(ctx, |ui| {
                            ui.set_clip_rect(rect);

                            ui.add(egui::Label::new(RichText::new(&text.text).color(color)));
                        });
                }

                crate::engine::ui::UiElement::Button(button) => {
                    let mut clicked = false;

                    egui::Area::new(area_id)
                        .fixed_pos(position)
                        .order(egui::Order::Middle)
                        .show(ctx, |ui| {
                            ui.set_clip_rect(rect);

                            let response = ui.add_enabled(
                                common.enabled,
                                egui::Button::new(RichText::new(&button.text).color(color)),
                            );

                            if response.clicked() {
                                clicked = true;
                            }
                        });

                    if clicked {
                        self.runtime_ui.pending_clicks.push(id);
                    }
                }
            }
        }
    }

    /*
     * ----------------------------------------------------------------------
     * Existing Home/UI implementation.
     *
     * These functions intentionally remain independent of the gameplay
     * synchronization changes above.
     * ----------------------------------------------------------------------
     */

    fn apply_home_style(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();

        visuals.window_rounding = 4.0.into();

        visuals.widgets.noninteractive.rounding = 4.0.into();

        visuals.widgets.inactive.rounding = 4.0.into();

        visuals.widgets.hovered.rounding = 4.0.into();

        visuals.widgets.active.rounding = 4.0.into();

        visuals.widgets.open.rounding = 4.0.into();

        visuals.extreme_bg_color = COLOR_VOID;

        visuals.window_fill = COLOR_VOID_ELEVATED;

        visuals.panel_fill = COLOR_VOID;

        visuals.selection.bg_fill = COLOR_ACCENT_ORANGE;

        visuals.widgets.inactive.bg_fill = COLOR_VOID_PANEL;

        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, COLOR_TEXT);

        visuals.widgets.hovered.bg_fill = COLOR_VOID_ELEVATED;

        visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, COLOR_ACCENT_ORANGE);

        visuals.widgets.active.bg_fill = COLOR_VOID_PANEL;

        visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, COLOR_ACCENT_ORANGE);

        ctx.set_visuals(visuals);
    }

    fn draw_home_screen(&mut self, ctx: &egui::Context, next_view: &mut Option<View>) {
        Self::apply_home_style(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(COLOR_VOID))
            .show(ctx, |ui| {
                self.draw_home_background(ui);

                egui::ScrollArea::vertical()
                    .id_source("home_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let available_width = ui.available_width();

                        let content_width = if available_width >= 320.0 {
                            (available_width - 48.0).min(1200.0)
                        } else {
                            available_width
                        };

                        ui.vertical_centered(|ui| {
                            ui.set_width(content_width);

                            ui.add_space(24.0);

                            self.draw_home_header(ui);

                            ui.add_space(40.0);

                            self.draw_home_hero(ui);

                            ui.add_space(40.0);

                            Self::draw_home_divider(ui);

                            ui.add_space(28.0);

                            self.draw_home_templates(ui);

                            ui.add_space(40.0);

                            Self::draw_home_divider(ui);

                            ui.add_space(28.0);

                            self.draw_home_recent(ui, next_view);

                            ui.add_space(40.0);

                            Self::draw_home_divider(ui);

                            ui.add_space(28.0);

                            self.draw_home_tutorials(ui);

                            ui.add_space(40.0);

                            Self::draw_home_divider(ui);

                            ui.add_space(28.0);

                            self.draw_home_explore(ui);

                            ui.add_space(32.0);

                            self.draw_home_footer(ui);

                            ui.add_space(24.0);
                        });
                    });
            });

        self.draw_home_dialogs(ctx, next_view);
    }

    fn draw_home_divider(ui: &mut egui::Ui) {
        let y = ui.cursor().top();
        let left = ui.cursor().left();
        let right = ui.cursor().right();

        ui.painter().line_segment(
            [egui::pos2(left, y), egui::pos2(right, y)],
            egui::Stroke::new(1.0, COLOR_BORDER),
        );

        ui.add_space(1.0);
    }

    fn draw_home_background(&self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        let painter = ui.painter();

        let top_band =
            egui::Rect::from_min_max(rect.left_top(), egui::pos2(rect.right(), rect.top() + 96.0));

        painter.rect_filled(
            top_band,
            0.0,
            egui::Color32::from_rgba_unmultiplied(158, 52, 29, 3),
        );

        painter.line_segment(
            [
                egui::pos2(rect.left(), rect.top()),
                egui::pos2(rect.right(), rect.top()),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(228, 91, 36, 18)),
        );
    }

    fn draw_home_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("AEOWUN")
                    .monospace()
                    .strong()
                    .color(COLOR_TEXT_BRIGHT)
                    .size(15.0),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let exit_button = egui::Button::new(
                    RichText::new("×")
                        .strong()
                        .size(18.0)
                        .color(COLOR_TEXT_BRIGHT),
                )
                .min_size(egui::vec2(28.0, 28.0))
                .fill(COLOR_ACCENT_RED)
                .stroke(egui::Stroke::new(1.0, COLOR_ACCENT_ORANGE));

                if ui.add(exit_button).clicked() {
                    self.show_exit_confirmation_dialog = true;
                }

                ui.add_space(14.0);

                if self.draw_home_link(ui, "GITHUB").clicked() {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(
                        "https://github.com/Aeowun/AeoEngine",
                    ));
                }

                ui.add_space(20.0);

                if self.draw_home_link(ui, "DOCUMENTATION").clicked() {
                    ui.ctx()
                        .open_url(egui::OpenUrl::new_tab("https://www.aeowun.com/docs/"));
                }
            });
        });
    }

    fn draw_home_link(&self, ui: &mut egui::Ui, text: &str) -> egui::Response {
        let font_id = egui::FontId::monospace(11.0);

        let galley = ui
            .painter()
            .layout_no_wrap(text.to_owned(), font_id.clone(), COLOR_TEXT);

        let size = galley.size() + egui::vec2(6.0, 6.0);

        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

        let color = if response.hovered() {
            COLOR_TEXT_BRIGHT
        } else {
            COLOR_TEXT
        };

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            font_id,
            color,
        );

        if response.hovered() {
            ui.painter().line_segment(
                [
                    egui::pos2(rect.left(), rect.bottom() - 1.0),
                    egui::pos2(rect.right(), rect.bottom() - 1.0),
                ],
                egui::Stroke::new(1.0, COLOR_ACCENT_ORANGE),
            );
        }

        response
    }

    fn draw_home_hero(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label(
                RichText::new("AEOENGINE")
                    .monospace()
                    .color(COLOR_TEXT_DIM)
                    .size(11.0),
            );

            ui.add_space(10.0);

            ui.label(
                RichText::new("BUILD WORLDS.")
                    .strong()
                    .color(COLOR_TEXT_BRIGHT)
                    .size(40.0),
            );

            ui.label(
                RichText::new("WRITE LOGIC.")
                    .strong()
                    .color(COLOR_TEXT_BRIGHT)
                    .size(40.0),
            );

            ui.label(
                RichText::new("SHAPE GAMES.")
                    .strong()
                    .color(COLOR_TEXT_BRIGHT)
                    .size(40.0),
            );

            ui.add_space(26.0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;

                let new_button = egui::Button::new(
                    RichText::new("NEW PROJECT")
                        .monospace()
                        .strong()
                        .color(COLOR_VOID),
                )
                .min_size(egui::vec2(154.0, 34.0))
                .fill(COLOR_TEXT_BRIGHT)
                .stroke(egui::Stroke::new(1.0, COLOR_TEXT_BRIGHT));

                if ui.add(new_button).clicked() {
                    self.show_new_project_dialog = true;

                    self.show_open_project_dialog = false;
                }

                let open_button = egui::Button::new(
                    RichText::new("OPEN PROJECT")
                        .monospace()
                        .strong()
                        .color(COLOR_TEXT_BRIGHT),
                )
                .min_size(egui::vec2(154.0, 34.0))
                .fill(COLOR_VOID_PANEL)
                .stroke(egui::Stroke::new(1.0, COLOR_BORDER_BRIGHT));

                if ui.add(open_button).clicked() {
                    self.show_open_project_dialog = true;

                    self.show_new_project_dialog = false;
                }
            });
        });
    }

    fn draw_home_templates(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("START WITH A WORLD")
                .monospace()
                .color(COLOR_TEXT_DIM)
                .size(11.0),
        );

        ui.add_space(14.0);

        let available_width = ui.available_width();

        let columns = if available_width >= 900.0 {
            3usize
        } else if available_width >= 600.0 {
            2usize
        } else {
            1usize
        };

        let spacing = 16.0_f32;

        let card_width = (available_width - spacing * (columns as f32 - 1.0)) / columns as f32;

        let templates = [
            ("BLANK", "Empty World", false),
            ("FIELD", "Open World", true),
            ("CASTLE", "Stone World", true),
            ("SPACE", "Space World", true),
        ];

        for chunk in templates.chunks(columns) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = spacing;

                for (title, description, coming_soon) in chunk {
                    let clicked =
                        self.draw_template_card(ui, title, description, card_width, *coming_soon);

                    if clicked && !*coming_soon {
                        self.show_new_project_dialog = true;

                        self.show_open_project_dialog = false;
                    }
                }
            });

            ui.add_space(spacing);
        }
    }

    fn draw_template_card(
        &self,
        ui: &mut egui::Ui,
        title: &str,
        description: &str,
        width: f32,
        coming_soon: bool,
    ) -> bool {
        let height = 164.0_f32;

        let sense = if coming_soon {
            egui::Sense::hover()
        } else {
            egui::Sense::click()
        };

        let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), sense);

        let hovered = response.hovered();

        let background = if hovered && !coming_soon {
            COLOR_VOID_ELEVATED
        } else {
            COLOR_VOID_PANEL
        };

        let border = if hovered {
            COLOR_BORDER_BRIGHT
        } else {
            COLOR_BORDER
        };

        ui.painter()
            .rect(rect, 4.0, background, egui::Stroke::new(1.0, border));

        if hovered && !coming_soon {
            ui.painter()
                .rect_stroke(rect, 4.0, egui::Stroke::new(1.0, COLOR_ACCENT_ORANGE));

            let glow_rect = egui::Rect::from_min_max(
                egui::pos2(rect.left(), rect.top()),
                egui::pos2(rect.right(), rect.top() + 2.0),
            );

            ui.painter()
                .rect_filled(glow_rect, 0.0, COLOR_ACCENT_ORANGE);
        }

        let inner = rect.shrink(16.0);

        ui.painter().text(
            egui::pos2(inner.left(), inner.top()),
            egui::Align2::LEFT_TOP,
            title,
            egui::FontId::monospace(14.0),
            COLOR_TEXT_BRIGHT,
        );

        ui.painter().text(
            egui::pos2(inner.left(), inner.top() + 26.0),
            egui::Align2::LEFT_TOP,
            description,
            egui::FontId::proportional(12.0),
            COLOR_TEXT,
        );

        let start_rect = egui::Rect::from_min_size(
            egui::pos2(inner.left(), inner.bottom() - 28.0),
            egui::vec2(inner.width(), 26.0),
        );

        let start_fill = if coming_soon {
            COLOR_VOID_ELEVATED
        } else {
            COLOR_VOID
        };

        let start_border = if hovered && !coming_soon {
            COLOR_ACCENT_ORANGE
        } else {
            COLOR_BORDER_BRIGHT
        };

        ui.painter().rect(
            start_rect,
            3.0,
            start_fill,
            egui::Stroke::new(1.0, start_border),
        );

        ui.painter().text(
            start_rect.center(),
            egui::Align2::CENTER_CENTER,
            "START",
            egui::FontId::monospace(10.0),
            if coming_soon {
                COLOR_TEXT_DIM
            } else {
                COLOR_TEXT_BRIGHT
            },
        );

        if coming_soon && hovered {
            ui.painter().rect_filled(
                rect,
                4.0,
                egui::Color32::from_rgba_unmultiplied(1, 1, 2, 218),
            );

            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "COMING SOON",
                egui::FontId::monospace(11.0),
                COLOR_TEXT_BRIGHT,
            );
        }

        response.clicked()
    }

    fn draw_home_recent(&mut self, ui: &mut egui::Ui, next_view: &mut Option<View>) {
        ui.label(
            RichText::new("RECENT PROJECTS")
                .monospace()
                .color(COLOR_TEXT_DIM)
                .size(11.0),
        );

        ui.add_space(14.0);

        if self.project_manager.recent_projects.is_empty() {
            ui.label(
                RichText::new("No recent projects.")
                    .color(COLOR_TEXT)
                    .size(12.0),
            );

            ui.add_space(4.0);

            ui.label(
                RichText::new("Create a new project or open an existing project.")
                    .color(COLOR_TEXT_DIM)
                    .size(11.0),
            );

            return;
        }

        let recent = self.project_manager.recent_projects.clone();

        for path in recent {
            let name = path.file_name().unwrap_or_default().to_string_lossy();

            let path_text = path.to_string_lossy();

            let row_height = 58.0_f32;

            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), row_height),
                egui::Sense::click(),
            );

            let hovered = response.hovered();

            ui.painter().rect(
                rect,
                4.0,
                if hovered {
                    COLOR_VOID_ELEVATED
                } else {
                    egui::Color32::TRANSPARENT
                },
                egui::Stroke::new(
                    1.0,
                    if hovered {
                        COLOR_BORDER_BRIGHT
                    } else {
                        COLOR_BORDER
                    },
                ),
            );

            if hovered {
                ui.painter().line_segment(
                    [
                        egui::pos2(rect.left(), rect.top()),
                        egui::pos2(rect.left(), rect.bottom()),
                    ],
                    egui::Stroke::new(2.0, COLOR_ACCENT_ORANGE),
                );
            }

            let inner = rect.shrink2(egui::vec2(16.0, 8.0));

            ui.painter().text(
                egui::pos2(inner.left(), inner.top()),
                egui::Align2::LEFT_TOP,
                name.as_ref(),
                egui::FontId::proportional(13.0),
                COLOR_TEXT_BRIGHT,
            );

            ui.painter().text(
                egui::pos2(inner.left(), inner.top() + 22.0),
                egui::Align2::LEFT_TOP,
                path_text.as_ref(),
                egui::FontId::monospace(10.0),
                COLOR_TEXT_DIM,
            );

            ui.painter().text(
                egui::pos2(inner.right(), rect.center().y),
                egui::Align2::RIGHT_CENTER,
                ">",
                egui::FontId::proportional(16.0),
                if hovered {
                    COLOR_ACCENT_ORANGE
                } else {
                    COLOR_TEXT_DIM
                },
            );

            if response.clicked() {
                if self.project_manager.open_project(path) {
                    self.load_project();
                    *next_view = Some(View::Editor);
                } else {
                    self.project_manager.load_recent();
                }
            }

            ui.add_space(8.0);
        }
    }

    fn draw_home_tutorials(&self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("LEARN AEOENGINE")
                .monospace()
                .color(COLOR_TEXT_DIM)
                .size(11.0),
        );

        ui.add_space(14.0);

        let available_width = ui.available_width();

        let columns = if available_width >= 760.0 {
            3usize
        } else if available_width >= 500.0 {
            2usize
        } else {
            1usize
        };

        let spacing = 24.0_f32;

        let item_width = (available_width - spacing * (columns as f32 - 1.0)) / columns as f32;

        let tutorials = [
            (
                "FIRST WORLD",
                "Build a world.",
                "https://www.aeowun.com/docs/tutorials/first-world/",
            ),
            (
                "FIRST SCRIPT",
                "Bind a script.",
                "https://www.aeowun.com/docs/tutorials/first-script/",
            ),
            (
                "FLASH BEHAVIOR",
                "Make it behave.",
                "https://www.aeowun.com/docs/tutorials/flash-behavior/",
            ),
        ];

        for chunk in tutorials.chunks(columns) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = spacing;

                for (title, description, url) in chunk {
                    self.draw_tutorial_link(ui, item_width, title, description, url);
                }
            });

            ui.add_space(12.0);
        }

        ui.add_space(8.0);

        if self
            .draw_home_utility_button(ui, "VIEW ALL TUTORIALS")
            .clicked()
        {
            ui.ctx().open_url(egui::OpenUrl::new_tab(
                "https://www.aeowun.com/docs/tutorials/",
            ));
        }
    }

    fn draw_tutorial_link(
        &self,
        ui: &mut egui::Ui,
        width: f32,
        title: &str,
        description: &str,
        url: &str,
    ) {
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(width, 54.0), egui::Sense::click());

        let hovered = response.hovered();

        ui.painter().text(
            egui::pos2(rect.left(), rect.top()),
            egui::Align2::LEFT_TOP,
            title,
            egui::FontId::monospace(11.0),
            if hovered {
                COLOR_TEXT_BRIGHT
            } else {
                COLOR_TEXT
            },
        );

        ui.painter().text(
            egui::pos2(rect.left(), rect.top() + 23.0),
            egui::Align2::LEFT_TOP,
            description,
            egui::FontId::proportional(11.0),
            COLOR_TEXT_DIM,
        );

        if hovered {
            ui.painter().line_segment(
                [
                    egui::pos2(rect.left(), rect.bottom() - 2.0),
                    egui::pos2((rect.left() + 64.0).min(rect.right()), rect.bottom() - 2.0),
                ],
                egui::Stroke::new(1.0, COLOR_ACCENT_ORANGE),
            );
        }

        if response.clicked() {
            ui.ctx().open_url(egui::OpenUrl::new_tab(url));
        }
    }

    fn draw_home_explore(&self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 28.0;

            ui.spacing_mut().item_spacing.y = 8.0;

            let links = [
                ("AEOENGINE", "https://www.aeowun.com/aeoengine/"),
                ("AEOSCRIPT", "https://www.aeowun.com/aeoscript/"),
                ("DOCS", "https://www.aeowun.com/docs/"),
                ("SOURCE", "https://github.com/Aeowun/AeoEngine"),
            ];

            for (label, url) in links {
                if self.draw_home_link(ui, label).clicked() {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(url));
                }
            }
        });
    }

    fn draw_home_utility_button(&self, ui: &mut egui::Ui, label: &str) -> egui::Response {
        let button = egui::Button::new(RichText::new(label).monospace().color(COLOR_TEXT))
            .min_size(egui::vec2(0.0, 28.0))
            .fill(COLOR_VOID)
            .stroke(egui::Stroke::new(1.0, COLOR_BORDER));

        ui.add(button)
    }

    fn draw_home_footer(&self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new(format!("AeoEngine {}", env!("CARGO_PKG_VERSION")))
                .color(COLOR_TEXT_DIM)
                .monospace()
                .size(10.0),
        );
    }

    fn draw_home_dialogs(&mut self, ctx: &egui::Context, next_view: &mut Option<View>) {
        if self.show_new_project_dialog {
            egui::Window::new("New Project")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .frame(
                    egui::Frame::window(&ctx.style())
                        .fill(COLOR_VOID_ELEVATED)
                        .stroke(egui::Stroke::new(1.0, COLOR_BORDER)),
                )
                .show(ctx, |ui| {
                    ui.add_space(8.0);

                    ui.label(
                        RichText::new("PROJECT NAME")
                            .monospace()
                            .color(COLOR_TEXT_DIM)
                            .size(10.0),
                    );

                    ui.add_space(6.0);

                    ui.add_sized(
                        egui::vec2(320.0, 28.0),
                        egui::TextEdit::singleline(&mut self.new_project_name),
                    );

                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("CREATE PROJECT")
                                        .monospace()
                                        .strong()
                                        .color(COLOR_VOID),
                                )
                                .min_size(egui::vec2(140.0, 30.0))
                                .fill(COLOR_TEXT_BRIGHT),
                            )
                            .clicked()
                        {
                            if self
                                .project_manager
                                .create_project(&self.new_project_name)
                                .is_some()
                            {
                                self.load_project();

                                *next_view = Some(View::Editor);

                                self.show_new_project_dialog = false;

                                self.new_project_name.clear();
                            }
                        }

                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("CANCEL").monospace().color(COLOR_TEXT),
                                )
                                .min_size(egui::vec2(90.0, 30.0))
                                .fill(COLOR_VOID_PANEL)
                                .stroke(egui::Stroke::new(1.0, COLOR_BORDER_BRIGHT)),
                            )
                            .clicked()
                        {
                            self.show_new_project_dialog = false;
                        }
                    });

                    ui.add_space(8.0);
                });
        }

        if self.show_open_project_dialog {
            egui::Window::new("Open Project")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .frame(
                    egui::Frame::window(&ctx.style())
                        .fill(COLOR_VOID_ELEVATED)
                        .stroke(egui::Stroke::new(1.0, COLOR_BORDER)),
                )
                .show(ctx, |ui| {
                    ui.add_space(8.0);

                    let projects = self.project_manager.list_projects();

                    if projects.is_empty() {
                        ui.label(
                            RichText::new("No projects found in UserData.")
                                .color(COLOR_TEXT_DIM)
                                .size(12.0),
                        );
                    } else {
                        egui::ScrollArea::vertical()
                            .max_height(300.0)
                            .show(ui, |ui| {
                                for path in projects {
                                    let name =
                                        path.file_name().unwrap_or_default().to_string_lossy();

                                    if ui
                                        .add(
                                            egui::Button::new(
                                                RichText::new(name.to_string())
                                                    .strong()
                                                    .color(COLOR_TEXT_BRIGHT),
                                            )
                                            .min_size(egui::vec2(320.0, 32.0))
                                            .fill(COLOR_VOID_PANEL)
                                            .stroke(egui::Stroke::new(1.0, COLOR_BORDER)),
                                        )
                                        .clicked()
                                    {
                                        if self.project_manager.open_project(path) {
                                            self.load_project();

                                            *next_view = Some(View::Editor);

                                            self.show_open_project_dialog = false;
                                        }
                                    }

                                    ui.add_space(6.0);
                                }
                            });
                    }

                    ui.add_space(16.0);

                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("CANCEL").monospace().color(COLOR_TEXT),
                            )
                            .min_size(egui::vec2(90.0, 30.0))
                            .fill(COLOR_VOID_PANEL)
                            .stroke(egui::Stroke::new(1.0, COLOR_BORDER_BRIGHT)),
                        )
                        .clicked()
                    {
                        self.show_open_project_dialog = false;
                    }

                    ui.add_space(8.0);
                });
        }

        if self.show_exit_confirmation_dialog {
            egui::Window::new("Exit AeoEngine?")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .frame(
                    egui::Frame::window(&ctx.style())
                        .fill(COLOR_VOID_ELEVATED)
                        .stroke(egui::Stroke::new(1.0, COLOR_BORDER)),
                )
                .show(ctx, |ui| {
                    ui.label(
                        RichText::new("Are you sure you want to exit?")
                            .monospace()
                            .color(COLOR_TEXT),
                    );

                    ui.add_space(14.0);

                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("YES").monospace().strong().color(COLOR_VOID),
                                )
                                .min_size(egui::vec2(70.0, 28.0))
                                .fill(COLOR_ACCENT_ORANGE),
                            )
                            .clicked()
                        {
                            self.show_exit_confirmation_dialog = false;

                            self.exit_requested = true;
                        }

                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("NO").monospace().color(COLOR_TEXT),
                                )
                                .min_size(egui::vec2(70.0, 28.0))
                                .fill(COLOR_VOID_PANEL)
                                .stroke(egui::Stroke::new(1.0, COLOR_BORDER)),
                            )
                            .clicked()
                        {
                            self.show_exit_confirmation_dialog = false;
                        }
                    });
                });
        }
    }

    pub fn render(&self) {
        match self.view {
            View::Splash => {
                self.renderer.render_home();
            }

            View::Home => {
                self.renderer.render_home();
            }

            View::Editor => {
                self.renderer.render_editor(
                    &self.editor,
                    &self.world,
                    &self.physics_world,
                    &self.character_system,
                    &self.gameplay_camera,
                    if self.is_left_mouse_down {
                        self.drag_start_coord
                    } else {
                        None
                    },
                );
            }
        }
    }

    pub fn load_project(&mut self) {
        if let Some(project_path) = &self.project_manager.current_project {
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
                        self.editor.camera.yaw = parts[0].parse().unwrap_or(self.editor.camera.yaw);

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

fn get_timestamp() -> String {
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
