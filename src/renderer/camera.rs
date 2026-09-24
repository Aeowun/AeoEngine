use glam::{Mat4, Vec3};

use crate::world::{World, WorldCoord};

const ORBIT_SENSITIVITY: f32 = 0.005;

const EDITOR_MIN_PITCH: f32 = -85.0_f32.to_radians();
const EDITOR_MAX_PITCH: f32 = 85.0_f32.to_radians();

const GAMEPLAY_MIN_PITCH: f32 = -85.0_f32.to_radians();
const GAMEPLAY_MAX_PITCH: f32 = 85.0_f32.to_radians();

pub const DEFAULT_DISTANCE: f32 = 6.0;
pub const DEFAULT_HEIGHT: f32 = 3.0;
pub const LOOK_HEIGHT: f32 = 1.0;

const CAMERA_COLLISION_STEP: f32 = 0.2;
const CAMERA_COLLISION_MARGIN: f32 = 0.1;
const FIRST_PERSON_HEIGHT: f32 = 1.5;

#[derive(Clone)]
pub struct CameraController {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target: Vec3,
}

impl Default for CameraController {
    fn default() -> Self {
        Self::new()
    }
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
        self.yaw -= dx * ORBIT_SENSITIVITY;
        self.pitch += dy * ORBIT_SENSITIVITY;
        self.pitch = self.pitch.clamp(EDITOR_MIN_PITCH, EDITOR_MAX_PITCH);
    }

    /// Rotates the camera while keeping its current world position fixed.
    pub fn look(&mut self, dx: f32, dy: f32) {
        let position = self.get_position();

        self.yaw -= dx * ORBIT_SENSITIVITY;
        self.pitch += dy * ORBIT_SENSITIVITY;
        self.pitch = self.pitch.clamp(EDITOR_MIN_PITCH, EDITOR_MAX_PITCH);

        self.target = position - orbit_offset(self.distance, self.yaw, self.pitch);
    }

    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance - delta * 1.5).clamp(2.0, 100.0);
    }

    pub fn get_position(&self) -> Vec3 {
        self.target + orbit_offset(self.distance, self.yaw, self.pitch)
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        let position = self.get_position();
        let (_, up, _) = self.get_basis();

        Mat4::look_at_rh(position, self.target, up)
    }

    pub fn get_basis(&self) -> (Vec3, Vec3, Vec3) {
        camera_basis(self.yaw, self.pitch)
    }

    pub fn move_target(&mut self, dx: f32, dy: f32) {
        let (right, up, _) = self.get_basis();
        let speed = self.distance * 0.01;

        self.target += right * dx * speed + up * dy * speed;
    }

    pub fn translate_free(&mut self, movement: Vec3) {
        let (right, up, forward) = self.get_basis();

        self.target += right * movement.x + up * movement.y + forward * movement.z;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraMode {
    ThirdPerson,
    FirstPerson,
}

/// Runtime camera attached to the active character.
pub struct GameplayCamera {
    pub mode: CameraMode,
    pub distance: f32,
    pub height: f32,
    pub look_height: f32,

    pub yaw: f32,
    pub pitch: f32,

    pub current_position: Vec3,
    pub current_target: Vec3,
}

impl Default for GameplayCamera {
    fn default() -> Self {
        Self::new()
    }
}

impl GameplayCamera {
    pub fn new() -> Self {
        Self {
            mode: CameraMode::ThirdPerson,
            distance: DEFAULT_DISTANCE,
            height: DEFAULT_HEIGHT,
            look_height: LOOK_HEIGHT,

            yaw: 45.0_f32.to_radians(),
            pitch: 20.0_f32.to_radians(),

            current_position: Vec3::ZERO,
            current_target: Vec3::ZERO,
        }
    }

    pub fn set_mode_from_name(&mut self, name: &str) {
        self.mode = match name {
            "firstPerson" => CameraMode::FirstPerson,
            _ => CameraMode::ThirdPerson,
        };
    }

    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.yaw -= dx * ORBIT_SENSITIVITY;
        self.pitch += dy * ORBIT_SENSITIVITY;
        self.pitch = self.pitch.clamp(GAMEPLAY_MIN_PITCH, GAMEPLAY_MAX_PITCH);
    }

    /// Returns horizontal forward and right vectors for camera-relative movement.
    pub fn get_horizontal_basis(&self) -> (Vec3, Vec3) {
        let sin_yaw = self.yaw.sin();
        let cos_yaw = self.yaw.cos();

        let forward = Vec3::new(-sin_yaw, 0.0, -cos_yaw);
        let right = Vec3::new(cos_yaw, 0.0, -sin_yaw);

        (forward, right)
    }

    /// Updates the camera to follow the active character.
    pub fn update(&mut self, character: &crate::character::Character, world: &World) {
        let character_position = character.transform.position;

        match self.mode {
            CameraMode::ThirdPerson => {
                self.current_target = character_position + Vec3::Y * self.look_height;

                let desired_position = self.current_target
                    + orbit_offset(self.distance, self.yaw, self.pitch)
                    + Vec3::Y * (self.height - self.look_height);

                self.current_position =
                    self.resolve_collision(self.current_target, desired_position, world);
            }

            CameraMode::FirstPerson => {
                self.current_position = character_position + Vec3::Y * FIRST_PERSON_HEIGHT;

                let forward = first_person_forward(self.yaw, self.pitch);
                self.current_target = self.current_position + forward;
            }
        }
    }

    /// Moves the third-person camera toward the target when a solid cell
    /// obstructs the camera's desired position.
    pub fn resolve_collision(&self, target: Vec3, desired: Vec3, world: &World) -> Vec3 {
        let offset = desired - target;
        let distance = offset.length();

        if distance <= f32::EPSILON {
            return desired;
        }

        let direction = offset / distance;
        let mut current_distance = CAMERA_COLLISION_STEP;

        while current_distance < distance {
            let point = target + direction * current_distance;

            let coord = WorldCoord::new(
                point.x.floor() as i32,
                point.y.floor() as i32,
                point.z.floor() as i32,
            );

            if world.get(coord).is_some() && world.is_cell_solid(coord) {
                return target + direction * (current_distance - CAMERA_COLLISION_MARGIN).max(0.0);
            }

            current_distance += CAMERA_COLLISION_STEP;
        }

        desired
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.current_position, self.current_target, Vec3::Y)
    }

    /// Returns camera-relative right, up, and forward vectors.
    pub fn get_basis(&self) -> (Vec3, Vec3, Vec3) {
        camera_basis(self.yaw, self.pitch)
    }
}

fn orbit_offset(distance: f32, yaw: f32, pitch: f32) -> Vec3 {
    let cos_pitch = pitch.cos();

    Vec3::new(
        distance * cos_pitch * yaw.sin(),
        distance * pitch.sin(),
        distance * cos_pitch * yaw.cos(),
    )
}

fn camera_basis(yaw: f32, pitch: f32) -> (Vec3, Vec3, Vec3) {
    let sin_yaw = yaw.sin();
    let cos_yaw = yaw.cos();
    let sin_pitch = pitch.sin();
    let cos_pitch = pitch.cos();

    let forward = Vec3::new(-cos_pitch * sin_yaw, -sin_pitch, -cos_pitch * cos_yaw).normalize();

    let right = Vec3::new(cos_yaw, 0.0, -sin_yaw).normalize();
    let up = right.cross(forward).normalize();

    (right, up, forward)
}

fn first_person_forward(yaw: f32, pitch: f32) -> Vec3 {
    let cos_pitch = pitch.cos();
    let sin_pitch = pitch.sin();

    Vec3::new(-cos_pitch * yaw.sin(), sin_pitch, -cos_pitch * yaw.cos()).normalize()
}
