use egui::RichText;

use super::{
    App, COLOR_ACCENT_ORANGE, COLOR_ACCENT_RED, COLOR_BORDER, COLOR_BORDER_BRIGHT, COLOR_TEXT,
    COLOR_VOID, COLOR_VOID_ELEVATED, COLOR_VOID_PANEL, EditorMode, View,
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
                    .fixed_pos(egui::pos2(
                        viewport.right() - 168.0,
                        viewport.top() + 12.0,
                    ))
                    .show(ctx, |ui| {
                        let frame_size = egui::vec2(168.0, 112.0);

                        let (frame_rect, _) =
                            ui.allocate_exact_size(frame_size, egui::Sense::hover());

                        // Very subtle depth layer.
                        ui.painter().rect_filled(
                            frame_rect.translate(egui::vec2(0.0, 3.0)),
                            4.0,
                            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 120),
                        );

                        // Main dark surface.
                        ui.painter().rect_filled(
                            frame_rect,
                            4.0,
                            egui::Color32::from_rgba_unmultiplied(7, 9, 12, 235),
                        );

                        // Fine border.
                        ui.painter().rect_stroke(
                            frame_rect,
                            4.0,
                            egui::Stroke::new(
                                1.0,
                                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 28),
                            ),
                        );

                        // Small orange structural highlight.
                        ui.painter().line_segment(
                            [
                                frame_rect.left_top(),
                                egui::pos2(frame_rect.left() + 42.0, frame_rect.top()),
                            ],
                            egui::Stroke::new(2.0, COLOR_ACCENT_ORANGE),
                        );

                        ui.allocate_ui_at_rect(
                            frame_rect.shrink2(egui::vec2(10.0, 7.0)),
                            |ui| {
                                ui.label(
                                    RichText::new("RMB + WASD   MOVE")
                                        .monospace()
                                        .size(10.0),
                                );

                                ui.label(
                                    RichText::new("RMB + Q/E    UP / DOWN")
                                        .monospace()
                                        .size(10.0),
                                );

                                ui.label(
                                    RichText::new("MMB + MOUSE  ORBIT")
                                        .monospace()
                                        .size(10.0),
                                );

                                ui.label(
                                    RichText::new("SCROLL       ZOOM")
                                        .monospace()
                                        .size(10.0),
                                );
                            },
                        );
                    });
            }

            self.render_runtime_ui(ctx);

            if self.show_unsaved_scripts_dialog {
                egui::Window::new("Unsaved Script Changes")
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .collapsible(false)
                    .frame(
                        egui::Frame::window(&ctx.style())
                            .fill(egui::Color32::from_rgb(7, 9, 12))
                            .stroke(egui::Stroke::new(1.0, COLOR_BORDER_BRIGHT))
                            .inner_margin(egui::Margin::symmetric(14.0, 12.0)),
                    )
                    .show(ctx, |ui| {
                        ui.label(
                            RichText::new("UNSAVED SCRIPT CHANGES")
                                .monospace()
                                .strong()
                                .color(COLOR_ACCENT_ORANGE)
                                .size(11.0),
                        );

                        ui.add_space(6.0);

                        ui.label(
                            RichText::new("You have unsaved changes in your scripts.")
                                .color(COLOR_TEXT)
                                .size(12.0),
                        );

                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Save & Play")
                                            .monospace()
                                            .strong()
                                            .color(COLOR_VOID),
                                    )
                                    .fill(COLOR_ACCENT_ORANGE)
                                    .stroke(egui::Stroke::new(1.0, COLOR_ACCENT_ORANGE)),
                                )
                                .clicked()
                            {
                                self.editor.script_editor.save_all();

                                self.editor.mode = EditorMode::Play;

                                self.show_unsaved_scripts_dialog = false;
                            }

                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Don't Save")
                                            .monospace()
                                            .color(COLOR_TEXT),
                                    )
                                    .fill(COLOR_VOID_PANEL)
                                    .stroke(egui::Stroke::new(1.0, COLOR_BORDER_BRIGHT)),
                                )
                                .clicked()
                            {
                                self.editor.mode = EditorMode::Play;

                                self.show_unsaved_scripts_dialog = false;
                            }

                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Cancel")
                                            .monospace()
                                            .color(COLOR_TEXT),
                                    )
                                    .fill(COLOR_VOID_PANEL)
                                    .stroke(egui::Stroke::new(1.0, COLOR_BORDER)),
                                )
                                .clicked()
                            {
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

                            let (response, _) =
                                ui.allocate_exact_size(size, egui::Sense::hover());

                            // Small shadow offset gives runtime panels a little depth.
                            ui.painter().rect_filled(
                                response.translate(egui::vec2(0.0, 3.0)),
                                4.0,
                                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 90),
                            );

                            // Actual panel.
                            ui.painter().rect_filled(response, 4.0, color);

                            // Fine edge separation.
                            ui.painter().rect_stroke(
                                response,
                                4.0,
                                egui::Stroke::new(
                                    1.0,
                                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, 22),
                                ),
                            );
                        });
                }

                crate::engine::ui::UiElement::Text(text) => {
                    egui::Area::new(area_id)
                        .fixed_pos(position)
                        .order(egui::Order::Middle)
                        .show(ctx, |ui| {
                            ui.set_clip_rect(rect);

                            ui.add(
                                egui::Label::new(
                                    RichText::new(&text.text)
                                        .color(color),
                                ),
                            );
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
                                egui::Button::new(
                                    RichText::new(&button.text)
                                        .strong()
                                        .color(color),
                                )
                                .fill(egui::Color32::from_rgba_unmultiplied(
                                    8, 10, 13, 225,
                                ))
                                .stroke(egui::Stroke::new(
                                    1.0,
                                    egui::Color32::from_rgba_unmultiplied(
                                        255, 255, 255, 26,
                                    ),
                                ))
                                .rounding(3.0),
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

        visuals.widgets.noninteractive.bg_fill =
            egui::Color32::from_rgb(7, 9, 12);

        visuals.widgets.noninteractive.fg_stroke =
            egui::Stroke::new(1.0, COLOR_TEXT);

        visuals.widgets.inactive.bg_fill = COLOR_VOID_PANEL;

        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(
            1.0,
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 18),
        );

        visuals.widgets.inactive.fg_stroke =
            egui::Stroke::new(1.0, COLOR_TEXT);

        visuals.widgets.hovered.bg_fill = COLOR_VOID_ELEVATED;

        visuals.widgets.hovered.bg_stroke =
            egui::Stroke::new(1.0, COLOR_ACCENT_ORANGE);

        visuals.widgets.hovered.fg_stroke =
            egui::Stroke::new(1.0, COLOR_ACCENT_ORANGE);

        visuals.widgets.active.bg_fill = COLOR_VOID_PANEL;

        visuals.widgets.active.bg_stroke =
            egui::Stroke::new(1.0, COLOR_ACCENT_ORANGE);

        visuals.widgets.active.fg_stroke =
            egui::Stroke::new(1.0, COLOR_ACCENT_ORANGE);

        ctx.set_visuals(visuals);
    }
}