use glam::{Quat, Vec3};
use super::rig::Transform;

#[derive(Clone, Debug)]
pub struct Keyframe<T> {
    pub time: f32,
    pub value: T,
}

#[derive(Clone, Debug)]
pub struct TransformTrack {
    pub joint_index: usize,
    pub translations: Vec<Keyframe<Vec3>>,
    pub rotations: Vec<Keyframe<Quat>>,
    pub scales: Vec<Keyframe<Vec3>>,
}

impl TransformTrack {
    pub fn evaluate(&self, time: f32) -> Transform {
        let translation = sample_track(&self.translations, time, Vec3::ZERO, |a, b, t| a.lerp(*b, t));
        let rotation = sample_track(&self.rotations, time, Quat::IDENTITY, |a, b, t| a.slerp(*b, t));
        let scale = sample_track(&self.scales, time, Vec3::ONE, |a, b, t| a.lerp(*b, t));

        Transform { translation, rotation, scale }
    }
}

fn sample_track<T: Clone>(
    keyframes: &[Keyframe<T>],
    time: f32,
    default: T,
    lerp_fn: impl Fn(&T, &T, f32) -> T,
) -> T {
    if keyframes.is_empty() {
        return default;
    }
    if keyframes.len() == 1 || time <= keyframes[0].time {
        return keyframes[0].value.clone();
    }
    if time >= keyframes.last().unwrap().time {
        return keyframes.last().unwrap().value.clone();
    }

    for window in keyframes.windows(2) {
        if time >= window[0].time && time <= window[1].time {
            let duration = window[1].time - window[0].time;
            let t = if duration > 0.0 { (time - window[0].time) / duration } else { 0.0 };
            return lerp_fn(&window[0].value, &window[1].value, t);
        }
    }

    default
}

#[derive(Clone, Debug)]
pub struct AnimationClip {
    pub name: String,
    pub duration: f32,
    pub tracks: Vec<TransformTrack>,
}

impl AnimationClip {
    pub fn evaluate(&self, time: f32, out_transforms: &mut [Transform]) {
        // Clamp/wrap time depending on looping but here we sample within bounds or let the sampler handle limits.
        for track in &self.tracks {
            if track.joint_index < out_transforms.len() {
                out_transforms[track.joint_index] = track.evaluate(time);
            }
        }
    }
}

/// Creates a skeleton with fixed base metrics to anchor all clips.
pub fn create_default_skeleton() -> super::rig::Skeleton {
    use super::rig::Joint;

    // Joint Index Mapping:
    // 0: Root
    // 1: Hips (child of Root)
    // 2: Spine (child of Hips)
    // 3: Head (child of Spine)
    // 4: LeftLeg (child of Hips)
    // 5: LeftFoot (child of LeftLeg)
    // 6: RightLeg (child of Hips)
    // 7: RightFoot (child of RightLeg)
    // 8: LeftArm (child of Spine)
    // 9: RightArm (child of Spine)

    let joints = vec![
        Joint { name: "Root".to_string(), parent_index: None, local_transform: Transform::default() },
        Joint { name: "Hips".to_string(), parent_index: Some(0), local_transform: Transform { translation: Vec3::new(0.0, 0.9, 0.0), ..Default::default() } },
        Joint { name: "Spine".to_string(), parent_index: Some(1), local_transform: Transform { translation: Vec3::new(0.0, 0.4, 0.0), ..Default::default() } },
        Joint { name: "Head".to_string(), parent_index: Some(2), local_transform: Transform { translation: Vec3::new(0.0, 0.3, 0.0), ..Default::default() } },
        Joint { name: "LeftLeg".to_string(), parent_index: Some(1), local_transform: Transform { translation: Vec3::new(-0.2, 0.0, 0.0), ..Default::default() } },
        Joint { name: "LeftFoot".to_string(), parent_index: Some(4), local_transform: Transform { translation: Vec3::new(0.0, -0.45, 0.0), ..Default::default() } },
        Joint { name: "RightLeg".to_string(), parent_index: Some(1), local_transform: Transform { translation: Vec3::new(0.2, 0.0, 0.0), ..Default::default() } },
        Joint { name: "RightFoot".to_string(), parent_index: Some(6), local_transform: Transform { translation: Vec3::new(0.0, -0.45, 0.0), ..Default::default() } },
        Joint { name: "LeftArm".to_string(), parent_index: Some(2), local_transform: Transform { translation: Vec3::new(-0.35, 0.2, 0.0), ..Default::default() } },
        Joint { name: "RightArm".to_string(), parent_index: Some(2), local_transform: Transform { translation: Vec3::new(0.35, 0.2, 0.0), ..Default::default() } },
    ];

    super::rig::Skeleton::new(joints)
}

/// Creates a polished, breathing Idle loop.
pub fn create_idle_clip() -> AnimationClip {
    let mut tracks = Vec::new();
    let duration = 2.0;

    // Hips track - gentle breathing bob up and down
    tracks.push(TransformTrack {
        joint_index: 1,
        translations: vec![
            Keyframe { time: 0.0, value: Vec3::new(0.0, 0.9, 0.0) },
            Keyframe { time: 1.0, value: Vec3::new(0.0, 0.87, 0.0) },
            Keyframe { time: 2.0, value: Vec3::new(0.0, 0.9, 0.0) },
        ],
        rotations: vec![
            Keyframe { time: 0.0, value: Quat::IDENTITY },
            Keyframe { time: 2.0, value: Quat::IDENTITY },
        ],
        scales: vec![Keyframe { time: 0.0, value: Vec3::ONE }],
    });

    // Spine track - slight tilt forward/back
    tracks.push(TransformTrack {
        joint_index: 2,
        translations: vec![Keyframe { time: 0.0, value: Vec3::new(0.0, 0.4, 0.0) }],
        rotations: vec![
            Keyframe { time: 0.0, value: Quat::IDENTITY },
            Keyframe { time: 1.0, value: Quat::from_rotation_x(0.03) },
            Keyframe { time: 2.0, value: Quat::IDENTITY },
        ],
        scales: vec![Keyframe { time: 0.0, value: Vec3::ONE }],
    });

    // Head track - counters spine or nods slightly
    tracks.push(TransformTrack {
        joint_index: 3,
        translations: vec![Keyframe { time: 0.0, value: Vec3::new(0.0, 0.3, 0.0) }],
        rotations: vec![
            Keyframe { time: 0.0, value: Quat::IDENTITY },
            Keyframe { time: 1.0, value: Quat::from_rotation_x(-0.02) },
            Keyframe { time: 2.0, value: Quat::IDENTITY },
        ],
        scales: vec![Keyframe { time: 0.0, value: Vec3::ONE }],
    });

    // Left Arm and Right Arm - subtle out sways
    tracks.push(TransformTrack {
        joint_index: 8,
        translations: vec![Keyframe { time: 0.0, value: Vec3::new(-0.35, 0.2, 0.0) }],
        rotations: vec![
            Keyframe { time: 0.0, value: Quat::IDENTITY },
            Keyframe { time: 1.0, value: Quat::from_rotation_z(0.04) },
            Keyframe { time: 2.0, value: Quat::IDENTITY },
        ],
        scales: vec![Keyframe { time: 0.0, value: Vec3::ONE }],
    });

    tracks.push(TransformTrack {
        joint_index: 9,
        translations: vec![Keyframe { time: 0.0, value: Vec3::new(0.35, 0.2, 0.0) }],
        rotations: vec![
            Keyframe { time: 0.0, value: Quat::IDENTITY },
            Keyframe { time: 1.0, value: Quat::from_rotation_z(-0.04) },
            Keyframe { time: 2.0, value: Quat::IDENTITY },
        ],
        scales: vec![Keyframe { time: 0.0, value: Vec3::ONE }],
    });

    AnimationClip {
        name: "Idle".to_string(),
        duration,
        tracks,
    }
}

/// Creates a full coordinated walking cycle.
pub fn create_walk_clip() -> AnimationClip {
    let mut tracks = Vec::new();
    let duration = 1.0;

    // Hips bob up and down twice per full cycle (at each step)
    tracks.push(TransformTrack {
        joint_index: 1,
        translations: vec![
            Keyframe { time: 0.0, value: Vec3::new(0.0, 0.88, 0.0) },
            Keyframe { time: 0.25, value: Vec3::new(0.0, 0.91, 0.0) },
            Keyframe { time: 0.5, value: Vec3::new(0.0, 0.88, 0.0) },
            Keyframe { time: 0.75, value: Vec3::new(0.0, 0.91, 0.0) },
            Keyframe { time: 1.0, value: Vec3::new(0.0, 0.88, 0.0) },
        ],
        rotations: vec![
            Keyframe { time: 0.0, value: Quat::IDENTITY },
            Keyframe { time: 0.5, value: Quat::from_rotation_y(0.04) },
            Keyframe { time: 1.0, value: Quat::IDENTITY },
        ],
        scales: vec![Keyframe { time: 0.0, value: Vec3::ONE }],
    });

    // Left Leg swing forward/backward
    tracks.push(TransformTrack {
        joint_index: 4,
        translations: vec![Keyframe { time: 0.0, value: Vec3::new(-0.2, 0.0, 0.0) }],
        rotations: vec![
            Keyframe { time: 0.0, value: Quat::from_rotation_x(0.3) },
            Keyframe { time: 0.5, value: Quat::from_rotation_x(-0.3) },
            Keyframe { time: 1.0, value: Quat::from_rotation_x(0.3) },
        ],
        scales: vec![Keyframe { time: 0.0, value: Vec3::ONE }],
    });

    // Right Leg swing opposite to Left Leg
    tracks.push(TransformTrack {
        joint_index: 6,
        translations: vec![Keyframe { time: 0.0, value: Vec3::new(0.2, 0.0, 0.0) }],
        rotations: vec![
            Keyframe { time: 0.0, value: Quat::from_rotation_x(-0.3) },
            Keyframe { time: 0.5, value: Quat::from_rotation_x(0.3) },
            Keyframe { time: 1.0, value: Quat::from_rotation_x(-0.3) },
        ],
        scales: vec![Keyframe { time: 0.0, value: Vec3::ONE }],
    });

    // Left Arm swing opposite to Left Leg
    tracks.push(TransformTrack {
        joint_index: 8,
        translations: vec![Keyframe { time: 0.0, value: Vec3::new(-0.35, 0.2, 0.0) }],
        rotations: vec![
            Keyframe { time: 0.0, value: Quat::from_rotation_x(-0.25) },
            Keyframe { time: 0.5, value: Quat::from_rotation_x(0.25) },
            Keyframe { time: 1.0, value: Quat::from_rotation_x(-0.25) },
        ],
        scales: vec![Keyframe { time: 0.0, value: Vec3::ONE }],
    });

    // Right Arm swing opposite to Right Leg
    tracks.push(TransformTrack {
        joint_index: 9,
        translations: vec![Keyframe { time: 0.0, value: Vec3::new(0.35, 0.2, 0.0) }],
        rotations: vec![
            Keyframe { time: 0.0, value: Quat::from_rotation_x(0.25) },
            Keyframe { time: 0.5, value: Quat::from_rotation_x(-0.25) },
            Keyframe { time: 1.0, value: Quat::from_rotation_x(0.25) },
        ],
        scales: vec![Keyframe { time: 0.0, value: Vec3::ONE }],
    });

    AnimationClip {
        name: "Walk".to_string(),
        duration,
        tracks,
    }
}
