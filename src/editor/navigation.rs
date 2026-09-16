use egui::Context;
use crate::world::WorldCoord;

pub struct NavigationWindow {
    pub is_open: bool,
    pub x_buf: String,
    pub y_buf: String,
    pub z_buf: String,
}

impl NavigationWindow {
    pub fn new() -> Self {
        Self {
            is_open: false,
            x_buf: "0".to_string(),
            y_buf: "0".to_string(),
            z_buf: "0".to_string(),
        }
    }

    pub fn sync(&mut self, coord: WorldCoord) {
        self.x_buf = coord.x.to_string();
        self.y_buf = coord.y.to_string();
        self.z_buf = coord.z.to_string();
    }

    pub fn show(&mut self, ctx: &Context) -> Option<WorldCoord> {
        let mut result = None;
        if self.is_open {
            egui::Window::new("Navigation")
                .open(&mut self.is_open)
                .resizable(false)
                .show(ctx, |ui| {
                    egui::Grid::new("nav_grid")
                        .spacing([10.0, 10.0])
                        .show(ui, |ui| {
                            ui.label("X:");
                            ui.text_edit_singleline(&mut self.x_buf);
                            ui.end_row();

                            ui.label("Y:");
                            ui.text_edit_singleline(&mut self.y_buf);
                            ui.end_row();

                            ui.label("Z:");
                            ui.text_edit_singleline(&mut self.z_buf);
                            ui.end_row();
                        });

                    ui.add_space(10.0);

                    if ui.button("Go").clicked() {
                        let x = self.x_buf.parse::<i32>().unwrap_or(0);
                        let y = self.y_buf.parse::<i32>().unwrap_or(0);
                        let z = self.z_buf.parse::<i32>().unwrap_or(0);
                        result = Some(WorldCoord::new(x, y, z));
                    }
                });
        }
        result
    }
}
