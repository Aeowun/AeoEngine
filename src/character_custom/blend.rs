use super::animation::{
    AnimationClip, create_default_skeleton, create_idle_clip, create_walk_clip,
};
use super::rig::{EvaluatedPose, Skeleton, Transform};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetAnimation {
    Idle,
    Walk,
}

pub struct CharacterAnimationController {
    skeleton: Skeleton,
    idle_clip: AnimationClip,
    walk_clip: AnimationClip,

    current_time_idle: f32,
    current_time_walk: f32,
    playback_rate: f32,

    target_state: TargetAnimation,
    blend_weight: f32, // 0.0 = Idle, 1.0 = Walk
    blend_speed: f32,  // how fast weight changes per second

    // Cached collections to prevent heap reallocations per frame
    local_transforms_idle: Vec<Transform>,
    local_transforms_walk: Vec<Transform>,
    blended_transforms: Vec<Transform>,
}

impl CharacterAnimationController {
    pub fn new() -> Self {
        let skeleton = create_default_skeleton();
        let joint_count = skeleton.joints.len();

        let idle_clip = create_idle_clip();
        let walk_clip = create_walk_clip();

        Self {
            skeleton,
            idle_clip,
            walk_clip,
            current_time_idle: 0.0,
            current_time_walk: 0.0,
            playback_rate: 1.0,
            target_state: TargetAnimation::Idle,
            blend_weight: 0.0,
            blend_speed: 4.0, // Blends fully in 0.25 seconds
            local_transforms_idle: vec![Transform::default(); joint_count],
            local_transforms_walk: vec![Transform::default(); joint_count],
            blended_transforms: vec![Transform::default(); joint_count],
        }
    }

    /// Selects whether the character should be animating towards the Walk loop or Idle loop.
    pub fn select_animation(&mut self, anim: TargetAnimation) {
        self.target_state = anim;
    }

    /// Controls the animation playback speed multiplier.
    pub fn set_playback_rate(&mut self, rate: f32) {
        self.playback_rate = rate;
    }

    /// Advances the animation timelines and transitions the blend weights smoothly.
    pub fn update(&mut self, dt: f32) {
        let scaled_dt = dt * self.playback_rate;

        // Advance clip times with clean wrapping over durations
        self.current_time_idle = (self.current_time_idle + scaled_dt) % self.idle_clip.duration;
        self.current_time_walk = (self.current_time_walk + scaled_dt) % self.walk_clip.duration;

        // Blend target interpolation
        let target_weight = match self.target_state {
            TargetAnimation::Idle => 0.0,
            TargetAnimation::Walk => 1.0,
        };

        if self.blend_weight < target_weight {
            self.blend_weight = (self.blend_weight + self.blend_speed * dt).min(target_weight);
        } else if self.blend_weight > target_weight {
            self.blend_weight = (self.blend_weight - self.blend_speed * dt).max(target_weight);
        }
    }

    /// Evaluates the blended clip state, returning an opaque EvaluatedPose mapping out model matrices.
    pub fn evaluate_pose(&mut self) -> EvaluatedPose {
        // 1. Evaluate individual clips into their transform buffers
        // We supply default skeleton pose values beforehand so tracks layer accurately.
        for (i, joint) in self.skeleton.joints.iter().enumerate() {
            self.local_transforms_idle[i] = joint.local_transform;
            self.local_transforms_walk[i] = joint.local_transform;
        }

        self.idle_clip
            .evaluate(self.current_time_idle, &mut self.local_transforms_idle);
        self.walk_clip
            .evaluate(self.current_time_walk, &mut self.local_transforms_walk);

        // 2. Linear blend between the local joint poses
        for i in 0..self.skeleton.joints.len() {
            self.blended_transforms[i] = self.local_transforms_idle[i]
                .lerp(&self.local_transforms_walk[i], self.blend_weight);
        }

        // 3. Resolve parent-child transformations to global space matrices
        self.skeleton.evaluate_pose(&self.blended_transforms)
    }

    /// Exposed internal skeleton immutable getter for geometry binding.
    pub fn skeleton(&self) -> &Skeleton {
        &self.skeleton
    }

    /// Current blend weight getter for verification.
    pub fn blend_weight(&self) -> f32 {
        self.blend_weight
    }
}
