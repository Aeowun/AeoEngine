use super::ScriptHostBridge;

impl<'a> ScriptHostBridge<'a> {
    pub fn get_input_move_vector(&self) -> [f32; 2] {
        [self.move_input.x, self.move_input.y]
    }

    pub fn is_input_jump_pressed(&self) -> bool {
        self.jump_requested
    }

    pub fn get_input_orbit_delta(&self) -> [f32; 2] {
        self.orbit_delta
    }
}
