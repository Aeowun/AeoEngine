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

    pub(crate) fn draw_home_screen(&mut self, ctx: &egui::Context, next_view: &mut Option<View>) {
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

    pub(crate) fn draw_home_divider(ui: &mut egui::Ui) {
        let y = ui.cursor().top();
        let left = ui.cursor().left();
        let right = ui.cursor().right();

        // Main hairline
        ui.painter().line_segment(
            [egui::pos2(left, y), egui::pos2(right, y)],
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 18),
            ),
        );

        // Small AEOWUN accent marker
        ui.painter().line_segment(
            [egui::pos2(left, y), egui::pos2(left + 42.0, y)],
            egui::Stroke::new(2.0, COLOR_ACCENT_ORANGE),
        );

        ui.add_space(1.0);
    }

    pub(crate) fn draw_home_background(&self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        let painter = ui.painter();

        // Very subtle elevated center field.
        let center = egui::Rect::from_min_max(
            egui::pos2(rect.left() + 18.0, rect.top()),
            egui::pos2(rect.right() - 18.0, rect.bottom()),
        );

        painter.rect_filled(
            center,
            0.0,
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 2),
        );

        // Top atmospheric band.
        let top_band = egui::Rect::from_min_max(
            rect.left_top(),
            egui::pos2(rect.right(), rect.top() + 140.0),
        );

        painter.rect_filled(
            top_band,
            0.0,
            egui::Color32::from_rgba_unmultiplied(158, 52, 29, 7),
        );

        // Thin top edge.
        painter.line_segment(
            [
                egui::pos2(rect.left(), rect.top()),
                egui::pos2(rect.right(), rect.top()),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(228, 91, 36, 28)),
        );

        // Subtle vertical framing rails.
        painter.line_segment(
            [
                egui::pos2(rect.left() + 18.0, rect.top()),
                egui::pos2(rect.left() + 18.0, rect.bottom()),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 7)),
        );

        painter.line_segment(
            [
                egui::pos2(rect.right() - 18.0, rect.top()),
                egui::pos2(rect.right() - 18.0, rect.bottom()),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 7)),
        );
    }

    pub(crate) fn draw_home_header(&mut self, ui: &mut egui::Ui) {
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

    pub(crate) fn draw_home_link(&self, ui: &mut egui::Ui, text: &str) -> egui::Response {
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

    pub(crate) fn draw_home_hero(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;

                let (rect, _) = ui.allocate_exact_size(egui::vec2(24.0, 2.0), egui::Sense::hover());

                ui.painter().rect_filled(rect, 0.0, COLOR_ACCENT_ORANGE);

                ui.label(
                    RichText::new("AEOENGINE")
                        .monospace()
                        .color(COLOR_TEXT_DIM)
                        .size(11.0),
                );
            });
            ui.label(
                RichText::new("AEOENGINE")
                    .monospace()
                    .color(COLOR_TEXT_DIM)
                    .size(11.0),
            );

            ui.add_space(10.0);

            ui.label(
                RichText::new("IMAGINE")
                    .strong()
                    .color(COLOR_TEXT_BRIGHT)
                    .size(40.0),
            );

            ui.label(RichText::new("BUILD").strong().color(COLOR_TEXT).size(40.0));

            ui.label(
                RichText::new("PLAY")
                    .strong()
                    .color(COLOR_ACCENT_ORANGE)
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

    pub(crate) fn draw_home_templates(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("START FRESH")
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

    pub(crate) fn draw_template_card(
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
        } else if coming_soon {
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 3)
        } else {
            COLOR_VOID_PANEL
        };

        let border = if hovered && !coming_soon {
            COLOR_ACCENT_ORANGE
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
            let accent_rect = egui::Rect::from_min_max(
                egui::pos2(rect.left(), rect.top()),
                egui::pos2(rect.left() + 3.0, rect.bottom()),
            );

            ui.painter().rect_filled(
                accent_rect,
                0.0,
                if coming_soon {
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, 14)
                } else {
                    COLOR_ACCENT_ORANGE
                },
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

    pub(crate) fn draw_home_recent(&mut self, ui: &mut egui::Ui, next_view: &mut Option<View>) {
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

    pub(crate) fn draw_home_tutorials(&self, ui: &mut egui::Ui) {
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

    pub(crate) fn draw_tutorial_link(
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

    pub(crate) fn draw_home_explore(&self, ui: &mut egui::Ui) {
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

    pub(crate) fn draw_home_utility_button(
        &self,
        ui: &mut egui::Ui,
        label: &str,
    ) -> egui::Response {
        let button = egui::Button::new(RichText::new(label).monospace().color(COLOR_TEXT))
            .min_size(egui::vec2(0.0, 28.0))
            .fill(COLOR_VOID)
            .stroke(egui::Stroke::new(1.0, COLOR_BORDER));

        ui.add(button)
    }

    pub(crate) fn draw_home_footer(&self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new(format!("AeoEngine {}", env!("CARGO_PKG_VERSION")))
                .color(COLOR_TEXT_DIM)
                .monospace()
                .size(10.0),
        );
    }

    pub(crate) fn draw_home_dialogs(&mut self, ctx: &egui::Context, next_view: &mut Option<View>) {
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
