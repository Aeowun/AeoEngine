use std::collections::HashSet;

use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::editor::EditorTool;
use crate::world::{CellType, WorldCoord};

use super::{
    App, EditorMode, RMB_GUARD_EDGE_SCROLL_SPEED, RMB_GUARD_INNER_RADIUS_FACTOR,
    RMB_GUARD_MAX_RADIUS_FACTOR, View,
};

#[derive(Clone, Debug)]
pub struct PendingGrab {
    pub start_mouse: (f64, f64),
    pub anchor_coord: WorldCoord,
    pub source_coords: Vec<WorldCoord>,
}

impl App {
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

                            KeyCode::KeyC => {
                                if ctrl
                                    && self.view == View::Editor
                                    && self.editor.mode == EditorMode::Editor
                                {
                                    self.copy_selection();
                                }
                            }

                            KeyCode::KeyV => {
                                if ctrl
                                    && self.view == View::Editor
                                    && self.editor.mode == EditorMode::Editor
                                {
                                    self.paste_clipboard();
                                }
                            }

                            KeyCode::KeyZ => {
                                if ctrl
                                    && self.view == View::Editor
                                    && self.editor.mode == EditorMode::Editor
                                {
                                    if let Some((prev_cells, prev_bindings)) =
                                        self.editor.history.undo_stack.pop()
                                    {
                                        self.editor.history.redo_stack.push((
                                            self.world.cells.clone(),
                                            self.world.script_bindings.clone(),
                                        ));

                                        self.world.cells = prev_cells;
                                        self.world.script_bindings = prev_bindings;
                                        self.world.rebuild_id_mapping();
                                    }
                                }
                            }

                            KeyCode::KeyY => {
                                if ctrl
                                    && self.view == View::Editor
                                    && self.editor.mode == EditorMode::Editor
                                {
                                    if let Some((next_cells, next_bindings)) =
                                        self.editor.history.redo_stack.pop()
                                    {
                                        self.editor.history.undo_stack.push((
                                            self.world.cells.clone(),
                                            self.world.script_bindings.clone(),
                                        ));

                                        self.world.cells = next_cells;
                                        self.world.script_bindings = next_bindings;
                                        self.world.rebuild_id_mapping();
                                    }
                                }
                            }

                            KeyCode::Delete => {
                                if self.view == View::Editor
                                    && self.editor.mode == EditorMode::Editor
                                    && !self.editor.selected_coords.is_empty()
                                {
                                    self.push_undo_snapshot();

                                    for coord in &self.editor.selected_coords {
                                        self.world.set_cell(*coord, CellType::Empty);
                                    }

                                    self.editor.selected_coords.clear();

                                    self.editor.selected_coord = None;
                                    self.editor.needs_save = true;
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

                                    self.editor.anchor = WorldCoord::new(
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
                                if self.editor.grab_state.is_some() || self.pending_grab.is_some() {
                                    self.editor.grab_state = None;
                                    self.pending_grab = None;
                                } else if self.view == View::Splash {
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

                        let ctrl = self.keys_down.contains(&KeyCode::ControlLeft)
                            || self.keys_down.contains(&KeyCode::ControlRight);

                        if self.view == View::Editor
                            && self.editor.mode == EditorMode::Editor
                            && self.editor.current_tool == EditorTool::Select
                            && !ctrl
                        {
                            if let Some(hovered) = self.editor.hovered_cell {
                                if self.editor.selected_coords.contains(&hovered) {
                                    self.pending_grab = Some(PendingGrab {
                                        start_mouse: self.mouse_pos,
                                        anchor_coord: hovered,
                                        source_coords: self.editor.selected_coords.clone(),
                                    });
                                    self.drag_start_coord = None;
                                    return;
                                }
                            }
                        }

                        self.drag_start_coord = self.editor.hovered_cell;

                        if self.editor.current_tool == EditorTool::Navigate
                            || self.editor.current_tool == EditorTool::Select
                        {
                            self.on_click();
                        }
                    } else {
                        if let Some(grab) = self.editor.grab_state.take() {
                            if grab.valid && grab.delta != WorldCoord::new(0, 0, 0) {
                                self.push_undo_snapshot();
                                if let Ok(()) =
                                    self.world.move_cells(&grab.source_coords, grab.delta)
                                {
                                    self.editor.selected_coords = grab
                                        .source_coords
                                        .iter()
                                        .map(|c| {
                                            WorldCoord::new(
                                                c.x + grab.delta.x,
                                                c.y + grab.delta.y,
                                                c.z + grab.delta.z,
                                            )
                                        })
                                        .collect();

                                    if let Some(sel) = self.editor.selected_coord {
                                        self.editor.selected_coord = Some(WorldCoord::new(
                                            sel.x + grab.delta.x,
                                            sel.y + grab.delta.y,
                                            sel.z + grab.delta.z,
                                        ));
                                    }

                                    self.editor.needs_save = true;
                                }
                            }
                            self.pending_grab = None;
                        } else if let Some(_pending) = self.pending_grab.take() {
                            // Releasing after clicking an already-selected cell without dragging.
                        } else if self.is_left_mouse_down {
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

                    if let Some(ref pending) = self.pending_grab.clone() {
                        let pdx = position.x - pending.start_mouse.0;
                        let pdy = position.y - pending.start_mouse.1;
                        if pdx * pdx + pdy * pdy >= 25.0 {
                            let target = self
                                .calculate_placement_target()
                                .unwrap_or(pending.anchor_coord);
                            let delta = WorldCoord::new(
                                target.x - pending.anchor_coord.x,
                                target.y - pending.anchor_coord.y,
                                target.z - pending.anchor_coord.z,
                            );
                            let valid =
                                self.validate_grab_destination(&pending.source_coords, delta);

                            self.editor.grab_state = Some(crate::editor::GrabState {
                                source_coords: pending.source_coords.clone(),
                                anchor_coord: pending.anchor_coord,
                                current_target: target,
                                delta,
                                valid,
                            });
                            self.pending_grab = None;
                        }
                    } else if self.editor.grab_state.is_some() {
                        let target = self.calculate_placement_target().unwrap_or_else(|| {
                            self.editor.grab_state.as_ref().unwrap().anchor_coord
                        });

                        let (delta, source_coords) = {
                            let grab = self.editor.grab_state.as_ref().unwrap();
                            let delta = WorldCoord::new(
                                target.x - grab.anchor_coord.x,
                                target.y - grab.anchor_coord.y,
                                target.z - grab.anchor_coord.z,
                            );
                            (delta, grab.source_coords.clone())
                        };

                        let valid = self.validate_grab_destination(&source_coords, delta);

                        if let Some(ref mut grab) = self.editor.grab_state {
                            grab.current_target = target;
                            grab.delta = delta;
                            grab.valid = valid;
                        }
                    }
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

    pub(crate) fn push_undo_snapshot(&mut self) {
        self.editor
            .history
            .push(self.world.cells.clone(), self.world.script_bindings.clone());
    }

    pub(crate) fn calculate_placement_target(&self) -> Option<WorldCoord> {
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
                let shift_down = self.keys_down.contains(&KeyCode::ShiftLeft)
                    || self.keys_down.contains(&KeyCode::ShiftRight);

                if !shift_down {
                    return Some(WorldCoord::new(
                        coord.x + normal.x as i32,
                        coord.y + normal.y as i32,
                        coord.z + normal.z as i32,
                    ));
                } else {
                    return Some(coord);
                }
            }
        }

        crate::editor::grid::picking::update_hover(
            mouse_x,
            mouse_y,
            viewport_width,
            viewport_height,
            &self.editor.camera,
            self.editor.anchor,
        )
    }

    pub(crate) fn validate_grab_destination(
        &self,
        source_coords: &[WorldCoord],
        delta: WorldCoord,
    ) -> bool {
        let source_set: HashSet<_> = source_coords.iter().cloned().collect();
        for &src in source_coords {
            let dest = WorldCoord::new(src.x + delta.x, src.y + delta.y, src.z + delta.z);
            if self.world.get(dest).is_some() && !source_set.contains(&dest) {
                return false;
            }
        }
        true
    }

    pub(crate) fn find_copy_pivot(&self, coords: &[WorldCoord]) -> WorldCoord {
        if let Some(hovered) = self.editor.hovered_cell {
            if coords.contains(&hovered) {
                return hovered;
            }
        }

        // Default to bottom-most cell (min Y, then min X, min Z) so offsets extend upward
        let mut best = coords[0];
        for &c in &coords[1..] {
            if c.y < best.y
                || (c.y == best.y && c.x < best.x)
                || (c.y == best.y && c.x == best.x && c.z < best.z)
            {
                best = c;
            }
        }
        best
    }

    pub(crate) fn copy_selection(&mut self) {
        if self.editor.selected_coords.is_empty() && self.editor.selected_coord.is_none() {
            return;
        }

        let mut coords = Vec::new();
        for &c in &self.editor.selected_coords {
            if !coords.contains(&c) && self.world.get(c).is_some() {
                coords.push(c);
            }
        }

        if coords.is_empty() {
            if let Some(c) = self.editor.selected_coord {
                if self.world.get(c).is_some() {
                    coords.push(c);
                }
            }
        }

        if coords.is_empty() {
            return;
        }

        let pivot = self.find_copy_pivot(&coords);

        let mut clipboard_cells = Vec::new();

        for coord in &coords {
            if let Some(cell) = self.world.get(*coord) {
                let offset =
                    WorldCoord::new(coord.x - pivot.x, coord.y - pivot.y, coord.z - pivot.z);
                let script_binding = self
                    .world
                    .script_bindings
                    .iter()
                    .find(|b| b.target_identity == cell.id)
                    .cloned();

                clipboard_cells.push(crate::editor::ClipboardCell {
                    offset,
                    cell: cell.clone(),
                    script_binding,
                });
            }
        }

        if !clipboard_cells.is_empty() {
            self.editor.clipboard = Some(crate::editor::EditorClipboard {
                pivot_coord: pivot,
                cells: clipboard_cells,
            });
        }
    }

    pub(crate) fn paste_clipboard(&mut self) {
        let Some(clipboard) = self.editor.clipboard.clone() else {
            return;
        };

        if clipboard.cells.is_empty() {
            return;
        }

        let Some(target_pivot) = self
            .calculate_placement_target()
            .or(self.editor.hovered_cell)
        else {
            return;
        };

        self.push_undo_snapshot();

        match self.world.paste_cells(&clipboard.cells, target_pivot) {
            Ok(pasted_coords) => {
                self.editor.selected_coords = pasted_coords;
                self.editor.selected_coord = Some(target_pivot);
                self.editor.show_properties_window = true;
                self.editor.needs_save = true;
            }
            Err(_err) => {
                self.editor.history.undo_stack.pop();
            }
        }
    }

    pub(crate) fn update_hover(&mut self) {
        if self.editor.grab_state.is_some() || self.editor.current_tool == EditorTool::Build {
            self.editor.hovered_cell = self.calculate_placement_target();
            return;
        }

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

            if let Some((coord, _normal)) = hit {
                self.editor.hovered_cell = Some(coord);
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

    pub(crate) fn on_click(&mut self) {
        if self.view == View::Editor && self.editor.mode == EditorMode::Editor {
            if let Some(hovered) = self.editor.hovered_cell {
                match self.editor.current_tool {
                    EditorTool::Navigate => {
                        self.editor.set_anchor(hovered);
                    }

                    EditorTool::Select => {
                        let ctrl = self.keys_down.contains(&KeyCode::ControlLeft)
                            || self.keys_down.contains(&KeyCode::ControlRight);

                        if ctrl {
                            if self.editor.selected_coords.contains(&hovered) {
                                self.editor.selected_coords.retain(|c| *c != hovered);
                                if self.editor.selected_coord == Some(hovered) {
                                    self.editor.selected_coord =
                                        self.editor.selected_coords.last().cloned();
                                }
                            } else if self.world.get(hovered).is_some() {
                                self.editor.selected_coords.push(hovered);
                                self.editor.selected_coord = Some(hovered);
                            }
                        } else {
                            if self.world.get(hovered).is_some() {
                                self.editor.selected_coord = Some(hovered);
                                self.editor.selected_coords = vec![hovered];
                            } else {
                                self.editor.selected_coord = None;
                                self.editor.selected_coords.clear();
                            }
                        }

                        self.editor.show_properties_window = true;
                    }

                    _ => {}
                }
            }
        }
    }

    pub(crate) fn apply_tool_to_range(&mut self, start: WorldCoord, end: WorldCoord) {
        if self.view != View::Editor || self.editor.mode != EditorMode::Editor {
            return;
        }

        let ctrl = self.keys_down.contains(&KeyCode::ControlLeft)
            || self.keys_down.contains(&KeyCode::ControlRight);

        if self.editor.current_tool == EditorTool::Select {
            if !ctrl {
                self.editor.selected_coords.clear();
                self.editor.selected_coord = None;
            }
        } else {
            self.push_undo_snapshot();
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
                    let coord = WorldCoord::new(x, y, z);

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
                                if !self.editor.selected_coords.contains(&coord) {
                                    self.editor.selected_coords.push(coord);
                                }

                                self.editor.selected_coord = Some(coord);
                            }
                        }

                        _ => {}
                    }
                }
            }
        }
    }
}
