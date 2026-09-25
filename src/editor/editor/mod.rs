pub mod inspector;
pub mod panels;
pub mod ui;

pub use inspector::{AttributeType, PropertyChange};

use super::navigation::NavigationWindow;
use crate::engine::EditorMode;
use crate::renderer::camera::CameraController;
use crate::scripting::binding::ScriptBinding;
use crate::world::{AttributeValue, Cell, WorldCoord};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GridPlane {
    Xz,
    Yz,
    Xy,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EditorTool {
    Navigate,
    Build,
    Erase,
    Select,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColorTarget {
    Build,
    PropertyColor(WorldCoord),
    PropertyLight(WorldCoord),
}

#[derive(Clone, Debug)]
pub struct ClipboardCell {
    pub offset: WorldCoord,
    pub cell: Cell,
    pub script_binding: Option<ScriptBinding>,
}

#[derive(Clone, Debug)]
pub struct EditorClipboard {
    pub pivot_coord: WorldCoord,
    pub cells: Vec<ClipboardCell>,
}

#[derive(Clone, Debug)]
pub struct GrabState {
    pub source_coords: Vec<WorldCoord>,
    pub anchor_coord: WorldCoord,
    pub current_target: WorldCoord,
    pub delta: WorldCoord,
    pub valid: bool,
}

pub struct History {
    pub undo_stack: Vec<(HashMap<WorldCoord, Cell>, Vec<ScriptBinding>)>,
    pub redo_stack: Vec<(HashMap<WorldCoord, Cell>, Vec<ScriptBinding>)>,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn push(&mut self, cells: HashMap<WorldCoord, Cell>, bindings: Vec<ScriptBinding>) {
        self.undo_stack.push((cells, bindings));
        self.redo_stack.clear();
    }
}

pub struct Editor {
    pub mode: EditorMode,
    pub camera: CameraController,
    pub anchor: WorldCoord,
    pub hovered_cell: Option<WorldCoord>,
    pub selected_coord: Option<WorldCoord>,
    pub selected_coords: Vec<WorldCoord>,
    pub current_tool: EditorTool,

    /// Resident streaming radius used while authoring and while playing.
    pub editor_max_distance_chunks: i32,
    pub play_max_distance_chunks: i32,

    // Windows
    pub navigation_window: NavigationWindow,
    pub show_properties_window: bool,
    pub show_world_window: bool,
    pub plane_picking: bool,

    // Layout
    pub right_panel_split: f32,

    pub show_script_workspace: bool,
    pub last_show_script_workspace: bool,
    pub script_editor: super::script_editor::ScriptEditor,

    // signals
    pub show_clear_confirmation: bool,
    pub needs_clear_world: bool,
    pub needs_save: bool,
    pub needs_exit: bool,

    // Build Settings (Template for new blocks)
    pub build_template: crate::world::Cell,

    // Active persistent color picker
    pub active_color_target: Option<ColorTarget>,

    /// The central area of the editor screen not covered by side or top/bottom panels.
    pub viewport_rect: egui::Rect,
    pub viewport_ppp: f32,

    pub history: History,

    pub clipboard: Option<EditorClipboard>,
    pub grab_state: Option<GrabState>,

    pub terminal_output: String,
    pub terminal_auto_scroll: bool,

    // Attribute add state
    pub attribute_add_name: String,
    pub attribute_add_type: AttributeType,
    pub attribute_add_value: AttributeValue,
    pub attribute_add_error: Option<String>,
    pub last_selected_coord: Option<WorldCoord>,

    // Cached World hierarchy data. Rebuilt only when the World render revision changes.
    world_hierarchy_revision: u64,
    world_hierarchy_blocks: Vec<WorldCoord>,
    world_hierarchy_lights: Vec<WorldCoord>,
    world_hierarchy_audio_emitters: Vec<WorldCoord>,
    world_hierarchy_spawn_points: Vec<WorldCoord>,
    world_hierarchy_fx_blocks: Vec<WorldCoord>,
    world_hierarchy_players: Vec<WorldCoord>,
    world_hierarchy_npcs: Vec<WorldCoord>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            mode: EditorMode::default(),
            camera: CameraController::new(),
            anchor: WorldCoord::new(0, 0, 0),
            hovered_cell: None,
            selected_coord: None,
            selected_coords: Vec::new(),
            current_tool: EditorTool::Select,
            editor_max_distance_chunks: 4,
            play_max_distance_chunks: 4,

            navigation_window: NavigationWindow::new(),
            show_properties_window: true,
            show_world_window: true,
            plane_picking: false,

            right_panel_split: 0.5,

            show_script_workspace: false,
            last_show_script_workspace: false,
            script_editor: super::script_editor::ScriptEditor::new(),

            show_clear_confirmation: false,
            needs_clear_world: false,
            needs_save: false,
            needs_exit: false,

            build_template: crate::world::Cell::new_block(),
            active_color_target: None,
            viewport_rect: egui::Rect::NOTHING,
            viewport_ppp: 1.0,
            history: History::new(),

            clipboard: None,
            grab_state: None,

            terminal_output: String::new(),
            terminal_auto_scroll: true,

            attribute_add_name: String::new(),
            attribute_add_type: AttributeType::String,
            attribute_add_value: AttributeValue::String(String::new()),
            attribute_add_error: None,
            last_selected_coord: None,

            world_hierarchy_revision: 0,
            world_hierarchy_blocks: Vec::new(),
            world_hierarchy_lights: Vec::new(),
            world_hierarchy_audio_emitters: Vec::new(),
            world_hierarchy_spawn_points: Vec::new(),
            world_hierarchy_fx_blocks: Vec::new(),
            world_hierarchy_players: Vec::new(),
            world_hierarchy_npcs: Vec::new(),
        }
    }

    pub fn clear_clipboard(&mut self) {
        self.clipboard = None;
    }

    pub fn set_anchor(&mut self, coord: WorldCoord) {
        self.anchor = coord;
        self.camera.target = glam::Vec3::new(coord.x as f32, coord.y as f32, coord.z as f32);
        self.navigation_window.sync(coord);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_script_workspace_toggle() {
        let mut editor = Editor::new();
        assert!(!editor.show_script_workspace);

        editor.show_script_workspace = true;
        assert!(editor.show_script_workspace);

        editor.show_script_workspace = false;
        assert!(!editor.show_script_workspace);
    }

    #[test]
    fn test_editor_script_refresh_on_transition() {
        let mut editor = Editor::new();
        let test_dir = std::path::PathBuf::from("TestProject_RefreshTransition");
        let scripts_dir = test_dir.join("scripts");
        if test_dir.exists() {
            let _ = std::fs::remove_dir_all(&test_dir);
        }
        std::fs::create_dir_all(&scripts_dir).unwrap();
        std::fs::write(scripts_dir.join("test.aeo"), "").unwrap();

        let ctx = egui::Context::default();
        let mut world = crate::world::World::new();

        // 1. Initial state: not showing, list empty
        assert!(!editor.show_script_workspace);
        assert!(editor.script_editor.scripts_list.is_empty());

        // 2. Toggle show_script_workspace to true
        editor.show_script_workspace = true;

        // 3. Call show_ui inside context.run, which should trigger refresh_scripts
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            editor.show_ui(ctx, &mut world, &Some(test_dir.clone()));
        });

        // 4. Verify list is now populated
        assert_eq!(editor.script_editor.scripts_list.len(), 1);

        let _ = std::fs::remove_dir_all(&test_dir);
    }
}
