use egui::RichText;

use super::{
    App, COLOR_ACCENT_ORANGE, COLOR_TEXT, COLOR_VOID, COLOR_VOID_ELEVATED, COLOR_VOID_PANEL,
    EditorMode, View,
};

impl App {
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

    pub(crate) fn render_runtime_ui(&mut self, ctx: &egui::Context) {
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

    pub(crate) fn apply_home_style(ctx: &egui::Context) {
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
}
