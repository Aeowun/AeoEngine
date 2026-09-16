use glam::{Mat4, Vec3};

use crate::world::{
    World,
    WorldCoord,
};

#[derive(Clone)]
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

        self.pitch =
            self.pitch.clamp(
                -limit,
                limit,
            );
    }

    pub fn zoom(&mut self, delta: f32) {
        self.distance -= delta * 1.5;

        self.distance =
            self.distance.clamp(
                2.0,
                100.0,
            );
    }

    pub fn get_position(&self) -> Vec3 {
        let cos_pitch = self.pitch.cos();

        Vec3::new(
            self.distance
                * cos_pitch
                * self.yaw.sin(),

            self.distance
                * self.pitch.sin(),

            self.distance
                * cos_pitch
                * self.yaw.cos(),
        ) + self.target
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        let pos =
            self.get_position();

        let (
            _right,
            up,
            _forward,
        ) = self.get_basis();

        Mat4::look_at_rh(
            pos,
            self.target,
            up,
        )
    }

    pub fn get_basis(&self) -> (Vec3, Vec3, Vec3) {
        let sin_yaw =
            self.yaw.sin();

        let cos_yaw =
            self.yaw.cos();

        let sin_pitch =
            self.pitch.sin();

        let cos_pitch =
            self.pitch.cos();

        let forward =
            Vec3::new(
                -cos_pitch * sin_yaw,
                -sin_pitch,
                -cos_pitch * cos_yaw,
            )
            .normalize();

        let right =
            Vec3::new(
                cos_yaw,
                0.0,
                -sin_yaw,
            )
            .normalize();

        let up =
            right
                .cross(forward)
                .normalize();

        (
            right,
            up,
            forward,
        )
    }

    pub fn move_target(
        &mut self,
        dx: f32,
        dy: f32,
    ) {
        let (
            right,
            up,
            _,
        ) = self.get_basis();

        let speed =
            self.distance * 0.01;

        self.target +=
            right * dx * speed
            + up * dy * speed;
    }
}

pub const ORBIT_SENSITIVITY: f32 = 0.005;

pub const MIN_PITCH: f32 =
    -85.0 * (std::f32::consts::PI / 180.0);

pub const MAX_PITCH: f32 =
    85.0 * (std::f32::consts::PI / 180.0);

pub const DEFAULT_DISTANCE: f32 = 6.0;

pub const DEFAULT_HEIGHT: f32 = 3.0;

pub const LOOK_HEIGHT: f32 = 1.0;

/// A dedicated runtime camera that follows an active character.
pub struct GameplayCamera {
    pub distance: f32,
    pub height: f32,
    pub look_height: f32,

    pub yaw: f32,
    pub pitch: f32,

    pub current_position: Vec3,
    pub current_target: Vec3,
}

impl GameplayCamera {
    pub fn new() -> Self {
        Self {
            distance: DEFAULT_DISTANCE,
            height: DEFAULT_HEIGHT,
            look_height: LOOK_HEIGHT,

            yaw: 45.0_f32.to_radians(),
            pitch: 20.0_f32.to_radians(),

            current_position: Vec3::ZERO,
            current_target: Vec3::ZERO,
        }
    }

    pub fn orbit(
        &mut self,
        dx: f32,
        dy: f32,
    ) {
        self.yaw -=
            dx * ORBIT_SENSITIVITY;

        self.pitch +=
            dy * ORBIT_SENSITIVITY;

        self.pitch =
            self.pitch.clamp(
                MIN_PITCH,
                MAX_PITCH,
            );
    }

    /// Provides horizontal basis vectors for camera-relative movement.
    pub fn get_horizontal_basis(
        &self,
    ) -> (Vec3, Vec3) {
        let sin_yaw =
            self.yaw.sin();

        let cos_yaw =
            self.yaw.cos();

        // Horizontal forward (ignores pitch)
        let forward =
            Vec3::new(
                -sin_yaw,
                0.0,
                -cos_yaw,
            )
            .normalize();

        // Horizontal right
        let right =
            Vec3::new(
                cos_yaw,
                0.0,
                -sin_yaw,
            )
            .normalize();

        (
            forward,
            right,
        )
    }

    /// Updates the camera matrices to follow the provided character.
    ///
    /// The player remains the fixed third person camera pivot. Edge scrolling
    /// changes yaw and pitch through orbit(), rather than moving this pivot.
    pub fn update(
        &mut self,
        character: &crate::character::Character,
        world: &World,
    ) {
        let char_pos =
            character.transform.position;

        self.current_target =
            char_pos
                + Vec3::Y
                    * self.look_height;

        let cos_pitch =
            self.pitch.cos();

        let follow_offset =
            Vec3::new(
                self.distance
                    * cos_pitch
                    * self.yaw.sin(),

                self.distance
                    * self.pitch.sin()
                    + self.height,

                self.distance
                    * cos_pitch
                    * self.yaw.cos(),
            );

        let desired_pos =
            char_pos
                + follow_offset;

        self.current_position =
            self.resolve_collision(
                self.current_target,
                desired_pos,
                world,
            );
    }

    fn resolve_collision(
        &self,
        target: Vec3,
        desired: Vec3,
        world: &World,
    ) -> Vec3 {
        let dir =
            desired - target;

        let dist =
            dir.length();

        if dist < 0.001 {
            return desired;
        }

        let dir_norm =
            dir / dist;

        // Preserve the existing camera collision behavior.
        let step_size = 0.2;

        let mut current_dist =
            0.0;

        while current_dist < dist {
            let point =
                target
                    + dir_norm
                        * current_dist;

            let coord =
                WorldCoord::new(
                    point.x.floor() as i32,
                    point.y.floor() as i32,
                    point.z.floor() as i32,
                );

            if let Some(cell) =
                world.get(coord)
            {
                if cell.solid {
                    // Pull the camera in front of the obstruction.
                    return target
                        + dir_norm
                            * (current_dist - 0.1)
                                .max(0.0);
                }
            }

            current_dist +=
                step_size;
        }

        desired
    }

    pub fn get_view_matrix(
        &self,
    ) -> Mat4 {
        Mat4::look_at_rh(
            self.current_position,
            self.current_target,
            Vec3::Y,
        )
    }

    /// Provides full 3D basis vectors.
    pub fn get_basis(
        &self,
    ) -> (Vec3, Vec3, Vec3) {
        let sin_yaw =
            self.yaw.sin();

        let cos_yaw =
            self.yaw.cos();

        let sin_pitch =
            self.pitch.sin();

        let cos_pitch =
            self.pitch.cos();

        let forward =
            Vec3::new(
                -cos_pitch * sin_yaw,
                -sin_pitch,
                -cos_pitch * cos_yaw,
            )
            .normalize();

        let right =
            Vec3::new(
                cos_yaw,
                0.0,
                -sin_yaw,
            )
            .normalize();

        let up =
            right
                .cross(forward)
                .normalize();

        (
            right,
            up,
            forward,
        )
    }
}