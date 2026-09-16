use crate::world::WorldCoord;

/// Holds the state for the editor navigation controls.
///
/// This state is used to buffer text input for coordinate fields and track
/// whether the navigation interface should be displayed.
pub struct NavigationWindow {
    pub is_open: bool,
    pub x_buf: String,
    pub y_buf: String,
    pub z_buf: String,
}

impl NavigationWindow {
    pub fn new() -> Self {
        Self {
            is_open: true,
            x_buf: "0".to_string(),
            y_buf: "0".to_string(),
            z_buf: "0".to_string(),
        }
    }

    /// Synchronizes the text buffers with a given world coordinate.
    pub fn sync(&mut self, coord: WorldCoord) {
        self.x_buf = coord.x.to_string();
        self.y_buf = coord.y.to_string();
        self.z_buf = coord.z.to_string();
    }
}
