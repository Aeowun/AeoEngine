use egui::RichText;

use super::{
    App, COLOR_ACCENT_ORANGE, COLOR_ACCENT_RED, COLOR_BORDER, COLOR_BORDER_BRIGHT, COLOR_TEXT,
    COLOR_TEXT_BRIGHT, COLOR_TEXT_DIM, COLOR_VOID, COLOR_VOID_ELEVATED, COLOR_VOID_PANEL, View,
};

impl App {
    pub(crate) fn draw_splash_screen(&mut self, ctx: &egui::Context) {
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
                egui::FontId::monospace(60.0),
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
                let reaction = eased.powf(0.75);

                for dust_index in 0..20 {
                    let seed = index as f32 * 31.7 + dust_index as f32 * 17.3;

                    let spread_x = (seed.sin() * 7.0) + (dust_index as f32 % 5.0 - 2.0) * 1.8;
                    let spread_y = (seed.cos() * 5.0) + (dust_index as f32 % 4.0 - 1.5) * 2.0;

                    // Letter moves right -> dust is thrown backward/left.
                    let backward_distance =
                        reaction * (10.0 + dust_index as f32 * 3.4);

                    // Inertia carries the dust upward as the letter enters.
                    let upward_distance =
                        reaction * reaction * (8.0 + dust_index as f32 * 2.1);

                    let dust_x = x - backward_distance + spread_x;
                    let dust_y = y - upward_distance + spread_y;

                    let dust_alpha =
                        ((1.0 - progress) * 80.0).clamp(0.0, 255.0) as u8;

                    let dust_size =
                        0.35 + (dust_index % 3) as f32 * 0.15;

                    painter.circle_filled(
                        egui::pos2(dust_x, dust_y),
                        dust_size,
                        egui::Color32::from_rgba_unmultiplied(
                            210,
                            210,
                            210,
                            dust_alpha,
                        ),
                    );
                }
            }
        }

        if elapsed >= 4.15 {
            self.view = View::Home;
        }
    }

    pub(crate) fn draw_home_screen(&mut self, ctx: &egui::Context, next_view: &mut Option<View>) {
        Self::apply_home_style(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(COLOR_VOID))
            .show(ctx, |ui| {
                self.draw_home_background(ui);

                let available = ui.available_rect_before_wrap();
                let outer = available.shrink2(egui::vec2(20.0, 14.0));

                // Header spans the complete Home surface.
                let header_height = 42.0;
                let header_rect = egui::Rect::from_min_max(
                    outer.left_top(),
                    egui::pos2(outer.right(), outer.top() + header_height),
                );

                ui.allocate_ui_at_rect(header_rect, |ui| {
                    self.draw_home_header(ui);
                });

                let content_top = header_rect.bottom() + 14.0;

                // Use almost the entire viewport. No fixed-height card and
                // no centered document column leaving a dead area at the bottom.
                let workspace = egui::Rect::from_min_max(
                    egui::pos2(outer.left(), content_top),
                    outer.right_bottom(),
                );

                ui.painter().rect_filled(
                    workspace,
                    1.0,
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, 224),
                );

                ui.painter().rect_stroke(
                    workspace,
                    1.0,
                    egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 22),
                    ),
                );

                let rail_width = 236.0_f32.min(workspace.width() * 0.30);
                let rail_gap = 0.0;
                let main_width = (workspace.width() - rail_width - rail_gap).max(320.0);

                let main_rect = egui::Rect::from_min_max(
                    workspace.left_top(),
                    egui::pos2(workspace.left() + main_width, workspace.bottom()),
                );

                let rail_rect = egui::Rect::from_min_max(
                    egui::pos2(main_rect.right() + rail_gap, workspace.top()),
                    workspace.right_bottom(),
                );

                // Main authoring area.
                ui.allocate_ui_at_rect(main_rect.shrink2(egui::vec2(26.0, 22.0)), |ui| {
                    self.draw_home_hero(ui);

                    ui.add_space(22.0);

                    Self::draw_home_divider(ui);

                    ui.add_space(20.0);

                    self.draw_home_recent(ui, next_view);
                });

                // Navigation / learning rail lives on the RIGHT.
                ui.allocate_ui_at_rect(rail_rect, |ui| {
                    self.draw_home_right_rail(ui);
                });
            });

        self.draw_home_dialogs(ctx, next_view);
    }

    pub(crate) fn draw_home_divider(ui: &mut egui::Ui) {
        let y = ui.cursor().top();
        let left = ui.cursor().left();
        let right = ui.cursor().right();
        let accent_end = left + (right - left) * 0.20;

        ui.painter().line_segment(
            [egui::pos2(left, y), egui::pos2(right, y)],
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 18),
            ),
        );

        ui.painter().line_segment(
            [egui::pos2(left, y), egui::pos2(accent_end, y)],
            egui::Stroke::new(2.0, COLOR_ACCENT_ORANGE),
        );

        ui.add_space(1.0);
    }

    pub(crate) fn draw_home_background(&self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        let painter = ui.painter();

        // Dark technical grid.
        painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(3, 5, 7));

        let grid_x = 72.0;
        let grid_y = 72.0;

        let mut x = rect.left();
        while x <= rect.right() {
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, 7),
                ),
            );

            x += grid_x;
        }

        let mut y = rect.top();
        while y <= rect.bottom() {
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, 7),
                ),
            );

            y += grid_y;
        }

        // Very faint orange perspective structure. No orange circle.
        let vanishing = egui::pos2(rect.center().x, rect.bottom() - 250.0);

        for offset in [-840.0_f32, -640.0, -440.0, -240.0, 240.0, 440.0, 640.0, 840.0] {
            painter.line_segment(
                [
                    vanishing,
                    egui::pos2(rect.center().x + offset, rect.bottom()),
                ],
                egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_unmultiplied(228, 91, 36, 8),
                ),
            );
        }

        for index in 1..7 {
            let t = index as f32 / 7.0;
            let line_y = vanishing.y + (rect.bottom() - vanishing.y) * t * t;

            painter.line_segment(
                [egui::pos2(rect.left(), line_y), egui::pos2(rect.right(), line_y)],
                egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_unmultiplied(228, 91, 36, 7),
                ),
            );
        }

        // Dark black authoring surface behind the Home content.
        let center_card = egui::Rect::from_min_max(
            egui::pos2(rect.left() + 10.0, rect.top() + 6.0),
            egui::pos2(rect.right() - 10.0, rect.bottom() - 8.0),
        );

        painter.rect_filled(
            center_card,
            1.0,
            egui::Color32::from_rgba_unmultiplied(1, 2, 4, 245),
        );

        painter.rect_stroke(
            center_card,
            1.0,
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 18),
            ),
        );

        painter.line_segment(
            [
                center_card.left_top(),
                egui::pos2(center_card.left() + 190.0, center_card.top()),
            ],
            egui::Stroke::new(
                2.0,
                egui::Color32::from_rgba_unmultiplied(228, 91, 36, 115),
            ),
        );
    }

    pub(crate) fn draw_home_header(&mut self, ui: &mut egui::Ui) {
        let left = ui.cursor().left();
        let right = ui.cursor().right();

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;

            let mark_size = egui::vec2(22.0, 22.0);
            let (mark, _) = ui.allocate_exact_size(mark_size, egui::Sense::hover());
            let p = ui.painter();
            let cx = mark.center().x;

            // Upright A mark with orange crossbar.
            p.line_segment(
                [
                    egui::pos2(cx - 7.0, mark.bottom() - 3.0),
                    egui::pos2(cx, mark.top() + 3.0),
                ],
                egui::Stroke::new(2.0, COLOR_TEXT_BRIGHT),
            );

            p.line_segment(
                [
                    egui::pos2(cx, mark.top() + 3.0),
                    egui::pos2(cx + 7.0, mark.bottom() - 3.0),
                ],
                egui::Stroke::new(2.0, COLOR_TEXT_BRIGHT),
            );

            p.line_segment(
                [
                    egui::pos2(cx - 4.0, mark.center().y + 1.0),
                    egui::pos2(cx + 4.0, mark.center().y + 1.0),
                ],
                egui::Stroke::new(2.0, COLOR_ACCENT_ORANGE),
            );

            ui.vertical(|ui| {
                ui.label(
                    RichText::new("AEOWUN")
                        .monospace()
                        .strong()
                        .color(COLOR_TEXT_BRIGHT)
                        .size(16.0),
                );

                ui.label(
                    RichText::new("AEOENGINE / BY AEOWUN")
                        .monospace()
                        .color(COLOR_TEXT_DIM)
                        .size(8.0),
                );
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let exit_button = egui::Button::new(
                    RichText::new("×")
                        .strong()
                        .size(18.0)
                        .color(COLOR_TEXT_BRIGHT),
                )
                .min_size(egui::vec2(30.0, 30.0))
                .fill(COLOR_ACCENT_RED)
                .stroke(egui::Stroke::new(1.0, COLOR_ACCENT_ORANGE));

                if ui.add(exit_button).clicked() {
                    self.show_exit_confirmation_dialog = true;
                }

                ui.add_space(18.0);

                if self.draw_home_link(ui, "GITHUB").clicked() {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(
                        "https://github.com/Aeowun/AeoEngine",
                    ));
                }

                ui.add_space(18.0);

                if self.draw_home_link(ui, "DOCS").clicked() {
                    ui.ctx()
                        .open_url(egui::OpenUrl::new_tab("https://www.aeowun.com/docs/"));
                }
            });
        });

        ui.painter().line_segment(
            [
                egui::pos2(left, ui.cursor().top() + 30.0),
                egui::pos2(right, ui.cursor().top() + 30.0),
            ],
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20),
            ),
        );
    }

    pub(crate) fn draw_home_link(&self, ui: &mut egui::Ui, text: &str) -> egui::Response {
        let font_id = egui::FontId::monospace(10.0);

        let galley = ui
            .painter()
            .layout_no_wrap(text.to_owned(), font_id.clone(), COLOR_TEXT);

        let size = galley.size() + egui::vec2(10.0, 8.0);

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

        ui.painter().line_segment(
            [
                egui::pos2(rect.left() + 2.0, rect.bottom() - 1.0),
                egui::pos2(
                    rect.left()
                        + if response.hovered() {
                            rect.width() - 2.0
                        } else {
                            10.0
                        },
                    rect.bottom() - 1.0,
                ),
            ],
            egui::Stroke::new(
                1.0,
                if response.hovered() {
                    COLOR_ACCENT_ORANGE
                } else {
                    egui::Color32::from_rgba_unmultiplied(228, 91, 36, 35)
                },
            ),
        );

        response
    }

    pub(crate) fn draw_home_hero(&mut self, ui: &mut egui::Ui) {
        let width = ui.available_width();
        let height = 360.0_f32;

        let rect =
            egui::Rect::from_min_size(ui.cursor().min, egui::vec2(width, height));

        let p = ui.painter();

        // Sparse technical framing. No second content card.
        let technical_x = rect.left() + width * 0.70;
        let technical_right = rect.right() - 28.0;

        let faint = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 17);
        let orange = egui::Color32::from_rgba_unmultiplied(228, 91, 36, 70);

        p.line_segment(
            [
                egui::pos2(technical_x, rect.top()),
                egui::pos2(technical_x, rect.bottom()),
            ],
            egui::Stroke::new(1.0, faint),
        );

        p.line_segment(
            [
                egui::pos2(technical_right, rect.top() + 46.0),
                egui::pos2(technical_right, rect.bottom() - 28.0),
            ],
            egui::Stroke::new(1.0, faint),
        );

        for index in 0..6 {
            let y = rect.top() + 72.0 + index as f32 * 42.0;

            p.line_segment(
                [
                    egui::pos2(technical_x + 18.0, y),
                    egui::pos2(technical_right, y),
                ],
                egui::Stroke::new(1.0, if index == 0 { orange } else { faint }),
            );
        }

        p.line_segment(
            [
                egui::pos2(technical_x + 18.0, rect.top() + 72.0),
                egui::pos2(technical_x + 58.0, rect.top() + 72.0),
            ],
            egui::Stroke::new(2.0, COLOR_ACCENT_ORANGE),
        );

        ui.allocate_ui_with_layout(
            egui::vec2(width, height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.add_space(22.0);

                ui.horizontal(|ui| {
                    let (line, _) =
                        ui.allocate_exact_size(egui::vec2(42.0, 2.0), egui::Sense::hover());

                    ui.painter()
                        .rect_filled(line, 0.0, COLOR_ACCENT_ORANGE);

                    ui.add_space(8.0);

                    ui.label(
                        RichText::new(format!(
                            "AEOENGINE // {}",
                            env!("CARGO_PKG_VERSION")
                        ))
                        .monospace()
                        .strong()
                        .color(COLOR_ACCENT_ORANGE)
                        .size(11.0),
                    );
                });

                ui.add_space(18.0);

                ui.label(
                    RichText::new("AEO")
                        .monospace()
                        .strong()
                        .color(COLOR_TEXT_BRIGHT)
                        .size(72.0),
                );

                ui.label(
                    RichText::new("ENGINE")
                        .monospace()
                        .strong()
                        .color(COLOR_ACCENT_ORANGE)
                        .size(72.0),
                );

                ui.add_space(10.0);

                ui.label(
                    RichText::new("IMAGINE")
                        .monospace()
                        .strong()
                        .color(COLOR_TEXT_BRIGHT)
                        .size(20.0),
                );

                ui.label(
                    RichText::new("BUILD")
                        .monospace()
                        .strong()
                        .color(COLOR_TEXT)
                        .size(20.0),
                );

                ui.label(
                    RichText::new("PLAY")
                        .monospace()
                        .strong()
                        .color(COLOR_ACCENT_ORANGE)
                        .size(20.0),
                );

                ui.add_space(24.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;

                    let new_button = egui::Button::new(
                        RichText::new("+  NEW PROJECT")
                            .monospace()
                            .strong()
                            .color(COLOR_VOID),
                    )
                    .min_size(egui::vec2(178.0, 40.0))
                    .fill(COLOR_ACCENT_ORANGE)
                    .stroke(egui::Stroke::new(1.0, COLOR_ACCENT_ORANGE));

                    if ui.add(new_button).clicked() {
                        self.show_new_project_dialog = true;
                        self.show_open_project_dialog = false;
                    }

                    let open_button = egui::Button::new(
                        RichText::new("OPEN PROJECT  →")
                            .monospace()
                            .strong()
                            .color(COLOR_TEXT_BRIGHT),
                    )
                    .min_size(egui::vec2(178.0, 40.0))
                    .fill(egui::Color32::from_rgb(7, 10, 13))
                    .stroke(egui::Stroke::new(1.0, COLOR_BORDER_BRIGHT));

                    if ui.add(open_button).clicked() {
                        self.show_open_project_dialog = true;
                        self.show_new_project_dialog = false;
                    }
                });
            },
        );
    }

    pub(crate) fn draw_home_recent(
        &mut self,
        ui: &mut egui::Ui,
        next_view: &mut Option<View>,
    ) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("RECENT PROJECTS")
                    .monospace()
                    .strong()
                    .color(COLOR_TEXT_BRIGHT)
                    .size(13.0),
            );

            ui.add_space(10.0);

            ui.label(
                RichText::new("// WORKSPACES")
                    .monospace()
                    .color(COLOR_TEXT_DIM)
                    .size(9.0),
            );
        });

        ui.add_space(12.0);

        if self.project_manager.recent_projects.is_empty() {
            let rect = egui::Rect::from_min_size(
                ui.cursor().min,
                egui::vec2(ui.available_width(), 84.0),
            );

            ui.painter().rect_filled(
                rect,
                1.0,
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 5),
            );

            ui.painter().rect_stroke(
                rect,
                1.0,
                egui::Stroke::new(1.0, COLOR_BORDER_BRIGHT),
            );

            ui.painter().line_segment(
                [
                    rect.left_top(),
                    egui::pos2(rect.left() + 120.0, rect.top()),
                ],
                egui::Stroke::new(2.0, COLOR_ACCENT_ORANGE),
            );

            ui.painter().text(
                rect.left_top() + egui::vec2(16.0, 18.0),
                egui::Align2::LEFT_TOP,
                "NO RECENT WORKSPACES",
                egui::FontId::monospace(10.0),
                COLOR_TEXT_BRIGHT,
            );

            ui.painter().text(
                rect.left_top() + egui::vec2(16.0, 44.0),
                egui::Align2::LEFT_TOP,
                "Create a new project or open an existing project.",
                egui::FontId::proportional(11.0),
                COLOR_TEXT_DIM,
            );

            ui.allocate_rect(rect, egui::Sense::hover());
            return;
        }

        let recent = self.project_manager.recent_projects.clone();

        for (index, path) in recent.into_iter().enumerate() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let path_text = path.to_string_lossy();
            let row_height = 74.0_f32;

            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), row_height),
                egui::Sense::click(),
            );

            let hovered = response.hovered();

            ui.painter().rect_filled(
                rect,
                1.0,
                if hovered {
                    egui::Color32::from_rgb(12, 13, 15)
                } else {
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, 4)
                },
            );

            ui.painter().rect_stroke(
                rect,
                1.0,
                egui::Stroke::new(
                    1.0,
                    if hovered {
                        egui::Color32::from_rgba_unmultiplied(228, 91, 36, 145)
                    } else {
                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 32)
                    },
                ),
            );

            ui.painter().rect_filled(
                egui::Rect::from_min_max(
                    rect.left_top(),
                    egui::pos2(
                        rect.left() + if hovered { 4.0 } else { 2.0 },
                        rect.bottom(),
                    ),
                ),
                0.0,
                if hovered {
                    COLOR_ACCENT_ORANGE
                } else {
                    COLOR_BORDER_BRIGHT
                },
            );

            ui.painter().text(
                egui::pos2(rect.left() + 16.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                format!("{:02}", index + 1),
                egui::FontId::monospace(9.0),
                if hovered {
                    COLOR_ACCENT_ORANGE
                } else {
                    COLOR_TEXT_DIM
                },
            );

            ui.painter().text(
                egui::pos2(rect.left() + 52.0, rect.top() + 14.0),
                egui::Align2::LEFT_TOP,
                name.as_ref(),
                egui::FontId::proportional(15.0),
                COLOR_TEXT_BRIGHT,
            );

            ui.painter().text(
                egui::pos2(rect.left() + 52.0, rect.top() + 41.0),
                egui::Align2::LEFT_TOP,
                path_text.as_ref(),
                egui::FontId::monospace(9.0),
                COLOR_TEXT,
            );

            ui.painter().text(
                egui::pos2(rect.right() - 20.0, rect.center().y),
                egui::Align2::RIGHT_CENTER,
                if hovered { "OPEN  →" } else { "→" },
                egui::FontId::monospace(10.0),
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

    pub(crate) fn draw_home_right_rail(&self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        let p = ui.painter();

        // Strong vertical separation from the main workspace.
        p.line_segment(
            [
                egui::pos2(rect.left(), rect.top()),
                egui::pos2(rect.left(), rect.bottom()),
            ],
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 30),
            ),
        );

        p.rect_filled(
            rect,
            0.0,
            egui::Color32::from_rgba_unmultiplied(5, 7, 10, 210),
        );

        let left = rect.left() + 22.0;
        let right = rect.right() - 22.0;

        p.text(
            egui::pos2(left, rect.top() + 22.0),
            egui::Align2::LEFT_TOP,
            "LEARN",
            egui::FontId::monospace(11.0),
            COLOR_TEXT_BRIGHT,
        );

        p.text(
            egui::pos2(left, rect.top() + 48.0),
            egui::Align2::LEFT_TOP,
            "Start with the world editor.",
            egui::FontId::proportional(10.0),
            COLOR_TEXT_DIM,
        );

        p.text(
            egui::pos2(left, rect.top() + 64.0),
            egui::Align2::LEFT_TOP,
            "Then wire gameplay with AeoScript.",
            egui::FontId::proportional(10.0),
            COLOR_TEXT_DIM,
        );

        p.line_segment(
            [
                egui::pos2(left, rect.top() + 88.0),
                egui::pos2(right, rect.top() + 88.0),
            ],
            egui::Stroke::new(1.0, COLOR_BORDER),
        );

        let learn_links = [
            (
                "01",
                "FIRST WORLD",
                "https://www.aeowun.com/docs/tutorials/first-world/",
            ),
            (
                "02",
                "FIRST SCRIPT",
                "https://www.aeowun.com/docs/tutorials/first-script/",
            ),
        ];

        for (index, (num, label, url)) in learn_links.into_iter().enumerate() {
            let top = rect.top() + 104.0 + index as f32 * 42.0;

            let link_rect = egui::Rect::from_min_size(
                egui::pos2(left - 6.0, top),
                egui::vec2(right - left + 12.0, 30.0),
            );

            let response = ui.interact(
                link_rect,
                ui.id().with(("home-learn", label)),
                egui::Sense::click(),
            );

            if response.hovered() {
                p.rect_filled(
                    link_rect,
                    0.0,
                    egui::Color32::from_rgba_unmultiplied(228, 91, 36, 12),
                );

                p.rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(link_rect.left(), link_rect.top()),
                        egui::pos2(link_rect.left() + 2.0, link_rect.bottom()),
                    ),
                    0.0,
                    COLOR_ACCENT_ORANGE,
                );
            }

            p.text(
                egui::pos2(link_rect.left() + 10.0, link_rect.center().y),
                egui::Align2::LEFT_CENTER,
                num,
                egui::FontId::monospace(8.0),
                if response.hovered() {
                    COLOR_ACCENT_ORANGE
                } else {
                    COLOR_TEXT_DIM
                },
            );

            p.text(
                egui::pos2(link_rect.left() + 40.0, link_rect.center().y),
                egui::Align2::LEFT_CENTER,
                label,
                egui::FontId::monospace(9.0),
                if response.hovered() {
                    COLOR_TEXT_BRIGHT
                } else {
                    COLOR_TEXT
                },
            );

            if response.clicked() {
                ui.ctx().open_url(egui::OpenUrl::new_tab(url));
            }
        }

        let engine_top = rect.top() + 222.0;

        p.line_segment(
            [
                egui::pos2(left, engine_top),
                egui::pos2(right, engine_top),
            ],
            egui::Stroke::new(1.0, COLOR_BORDER),
        );

        p.text(
            egui::pos2(left, engine_top + 20.0),
            egui::Align2::LEFT_TOP,
            "ENGINE",
            egui::FontId::monospace(11.0),
            COLOR_TEXT_BRIGHT,
        );

        let engine_links = [
            ("AEOENGINE", "https://www.aeowun.com/aeoengine/"),
            ("AEOSCRIPT", "https://www.aeowun.com/aeoscript/"),
            ("DOCS", "https://www.aeowun.com/docs/"),
            ("SOURCE", "https://github.com/Aeowun/AeoEngine"),
        ];

        for (index, (label, url)) in engine_links.into_iter().enumerate() {
            let top = engine_top + 48.0 + index as f32 * 34.0;

            let link_rect = egui::Rect::from_min_size(
                egui::pos2(left - 6.0, top),
                egui::vec2(right - left + 12.0, 26.0),
            );

            let response = ui.interact(
                link_rect,
                ui.id().with(("home-engine", label)),
                egui::Sense::click(),
            );

            p.text(
                egui::pos2(link_rect.left() + 10.0, link_rect.center().y),
                egui::Align2::LEFT_CENTER,
                label,
                egui::FontId::monospace(9.0),
                if response.hovered() {
                    COLOR_TEXT_BRIGHT
                } else {
                    COLOR_TEXT_DIM
                },
            );

            if response.hovered() {
                p.line_segment(
                    [
                        egui::pos2(link_rect.left() + 10.0, link_rect.bottom()),
                        egui::pos2(link_rect.left() + 58.0, link_rect.bottom()),
                    ],
                    egui::Stroke::new(1.0, COLOR_ACCENT_ORANGE),
                );
            }

            if response.clicked() {
                ui.ctx().open_url(egui::OpenUrl::new_tab(url));
            }
        }

        // Rail branding belongs at the bottom, without duplicating the version.
        p.line_segment(
            [
                egui::pos2(left, rect.bottom() - 54.0),
                egui::pos2(right, rect.bottom() - 54.0),
            ],
            egui::Stroke::new(1.0, COLOR_BORDER),
        );

        p.text(
            egui::pos2(left, rect.bottom() - 22.0),
            egui::Align2::LEFT_BOTTOM,
            "AEOWUN // AEOENGINE",
            egui::FontId::monospace(9.0),
            COLOR_TEXT_DIM,
        );

        ui.allocate_rect(rect, egui::Sense::hover());
    }

    pub(crate) fn draw_home_footer(&self, _ui: &mut egui::Ui) {}

    pub(crate) fn draw_home_dialogs(
        &mut self,
        ctx: &egui::Context,
        next_view: &mut Option<View>,
    ) {
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
}