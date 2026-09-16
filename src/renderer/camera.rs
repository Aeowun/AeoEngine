use glam::{Mat4, Vec3};

pub struct CameraController {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target: Vec3,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            yaw: 45.0_f32.to_radians(),
            pitch: 35.0_f32.to_radians(),
            distance: 12.0,
            target: Vec3::ZERO,
        }
    }

    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.yaw -= dx * 0.005;
        self.pitch += dy * 0.005;

        // Clamp pitch to avoid gimbal lock and flipping.
        let limit = 89.0_f32.to_radians();
        self.pitch = self.pitch.clamp(-limit, limit);
    }

    pub fn zoom(&mut self, delta: f32) {
        self.distance -= delta * 1.5;
        self.distance = self.distance.clamp(2.0, 100.0);
    }

    pub fn get_position(&self) -> Vec3 {
        let cos_pitch = self.pitch.cos();
        Vec3::new(
            self.distance * cos_pitch * self.yaw.sin(),
            self.distance * self.pitch.sin(),
            self.distance * cos_pitch * self.yaw.cos(),
        ) + self.target
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        let pos = self.get_position();
        let (_right, up, _forward) = self.get_basis();
        Mat4::look_at_rh(pos, self.target, up)
    }

    pub fn get_basis(&self) -> (Vec3, Vec3, Vec3) {
        let sin_yaw = self.yaw.sin();
        let cos_yaw = self.yaw.cos();
        let sin_pitch = self.pitch.sin();
        let cos_pitch = self.pitch.cos();

        let forward = Vec3::new(
            -cos_pitch * sin_yaw,
            -sin_pitch,
            -cos_pitch * cos_yaw
        ).normalize();

        let right = Vec3::new(cos_yaw, 0.0, -sin_yaw).normalize();
        let up = right.cross(forward).normalize();
        (right, up, forward)
    }

    pub fn move_target(&mut self, dx: f32, dy: f32) {
        let (right, up, _) = self.get_basis();
        let speed = self.distance * 0.01;
        self.target += right * dx * speed + up * dy * speed;
    }
}
