use super::physics_body::PhysicsBodyId;
use super::physics_world::PhysicsWorld;
use glam::Vec3;

// 60Hz fixed timestep keeps simulation stable and deterministic regardless of frame rate.
pub const SIMULATION_DT: f32 = 1.0 / 60.0;

// Limit catch up steps to 8 per frame to avoid the 'spiral of death' during stalls.
pub const MAX_PHYSICS_STEPS: u32 = 8;

// Velocity below which a body is considered "at rest" for sleep calculation.
pub const SLEEP_VELOCITY_THRESHOLD: f32 = 0.02;

// Time in seconds a body must stay below the threshold to fall asleep.
pub const SLEEP_TIME_THRESHOLD: f32 = 0.5;

pub struct PhysicsClock {
    pub accumulator: f32,
}

impl PhysicsClock {
    pub fn new() -> Self {
        Self { accumulator: 0.0 }
    }

    // Reset on exit to prevent simulation jumps when returning to a project.
    pub fn reset(&mut self) {
        self.accumulator = 0.0;
    }

    pub fn update<F>(&mut self, dt: f32, mut step_fn: F)
    where
        F: FnMut(f32),
    {
        // Clamp huge deltas (e.g. from window drags) to prevent tunneling.
        let dt = dt.min(0.25);
        self.accumulator += dt;

        let mut steps_processed = 0;
        while self.accumulator >= SIMULATION_DT {
            if steps_processed >= MAX_PHYSICS_STEPS {
                // Drop remaining time if we can't keep up; better to slow down than freeze.
                self.accumulator = 0.0;
                break;
            }

            step_fn(SIMULATION_DT);
            self.accumulator -= SIMULATION_DT;
            steps_processed += 1;
        }
    }
}

impl PhysicsWorld {
    // World gravity changes velocity but does not decide where the body ends up.
    // We cancel the gravity component pointing into a resting surface to ensure
    // stability and prevent "jittering" into blocks every step.
    pub fn apply_gravity(&mut self, gravity: Vec3, dt: f32) {
        self.step_count += 1;
        for body in &mut self.bodies {
            if body.is_sleeping || body.anchored || !body.gravity_participation {
                continue;
            }

            let mut effective_gravity = gravity;

            // Remove gravity pointing into an active contact normal if that normal
            // actually opposes the gravity vector.
            let mut combined_normal = Vec3::ZERO;
            if let Some(n) = body.static_contact_normal {
                combined_normal += n;
            }
            if let Some((_, n)) = body.dynamic_contact {
                combined_normal += n;
            }

            if combined_normal != Vec3::ZERO {
                let n = combined_normal.normalize();
                let gravity_into_surface = gravity.dot(n);
                // If gravity points into the surface (dot < 0), remove that component.
                if gravity_into_surface < 0.0 {
                    effective_gravity -= n * gravity_into_surface;
                }
            }

            body.velocity += effective_gravity * dt;
        }

        // Low frequency debug print (approx once per second at 60Hz)
        if self.step_count % 60 == 0 {
            for (i, body) in self.bodies.iter().enumerate() {
                println!(
                    "Physics Debug | Body[{}] Pos: ({:.3}, {:.3}, {:.3}) Vel: ({:.3}, {:.3}, {:.3}) Solid: {} Anchored: {} Sleep: {}",
                    i,
                    body.position.x,
                    body.position.y,
                    body.position.z,
                    body.velocity.x,
                    body.velocity.y,
                    body.velocity.z,
                    body.solid,
                    body.anchored,
                    body.is_sleeping,
                );
            }
        }
    }

    // Position integration moves dynamic bodies that are not sleeping.
    pub fn integrate_positions(&mut self, dt: f32) {
        for body in &mut self.bodies {
            if !body.anchored && !body.is_sleeping {
                body.position += body.velocity * dt;
            }
        }
    }

    /// Resolves overlaps between dynamic bodies and static blocks.
    pub fn resolve_static_collisions(&mut self) {
        for i in 0..self.bodies.len() {
            if !self.bodies[i].solid || self.bodies[i].anchored {
                continue;
            }

            let mut latest_contact_normal: Option<Vec3> = None;

            for _ in 0..4 {
                let body = &self.bodies[i];
                let b_min = body.min_corner();
                let b_max = body.max_corner();
                let b_center = (b_min + b_max) * 0.5;

                let mut best_collision: Option<(Vec3, f32)> = None;
                let mut min_overlap = f32::MAX;

                let candidates = self.query_static_colliders_in_aabb(
                    b_min - Vec3::splat(0.5),
                    b_max + Vec3::splat(0.5),
                );

                for (_cell_id, v_min) in &candidates {
                    let v_max = v_min + Vec3::ONE;
                    let v_center = v_min + Vec3::new(0.5, 0.5, 0.5);

                    if Self::aabb_overlap_static(b_min, b_max, *v_min, v_max) {
                        let overlap_x = b_max.x.min(v_max.x) - b_min.x.max(v_min.x);
                        let overlap_y = b_max.y.min(v_max.y) - b_min.y.max(v_min.y);
                        let overlap_z = b_max.z.min(v_max.z) - b_min.z.max(v_min.z);

                        let (axis_normal, depth) = if overlap_x < overlap_y && overlap_x < overlap_z
                        {
                            let n = if b_center.x < v_center.x {
                                Vec3::NEG_X
                            } else {
                                Vec3::X
                            };
                            (n, overlap_x)
                        } else if overlap_y < overlap_z {
                            let n = if b_center.y < v_center.y {
                                Vec3::NEG_Y
                            } else {
                                Vec3::Y
                            };
                            (n, overlap_y)
                        } else {
                            let n = if b_center.z < v_center.z {
                                Vec3::NEG_Z
                            } else {
                                Vec3::Z
                            };
                            (n, overlap_z)
                        };

                        if depth < min_overlap {
                            min_overlap = depth;
                            best_collision = Some((axis_normal, depth));
                        }
                    }
                }

                if let Some((normal, penetration)) = best_collision {
                    if penetration > 0.0001 {
                        let body = &mut self.bodies[i];
                        let epsilon = 0.0001;
                        // Correct position along normal (points from voxel to body).
                        body.position += normal * (penetration + epsilon);

                        // Remove only inward velocity using dot product.
                        let v_in = body.velocity.dot(normal);
                        if v_in < 0.0 {
                            body.velocity -= normal * v_in;
                        }

                        latest_contact_normal = Some(normal);
                        body.wake();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            if let Some(n) = latest_contact_normal {
                self.bodies[i].static_contact_normal = Some(n);
            } else if let Some(n) = self.bodies[i].static_contact_normal {
                if !self.check_body_touching_static(&self.bodies[i], n) {
                    self.bodies[i].static_contact_normal = None;
                }
            }
        }
    }

    /// Resolves overlaps between dynamic bodies.
    pub fn resolve_dynamic_collisions(&mut self) {
        let n = self.bodies.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let (body_a, body_b) = if i < j {
                    let (left, right) = self.bodies.split_at_mut(j);
                    (&mut left[i], &mut right[0])
                } else {
                    continue;
                };

                if !body_a.solid || !body_b.solid || (body_a.anchored && body_b.anchored) {
                    continue;
                }

                let min_a = body_a.min_corner();
                let max_a = body_a.max_corner();
                let center_a = (min_a + max_a) * 0.5;

                let min_b = body_b.min_corner();
                let max_b = body_b.max_corner();
                let center_b = (min_b + max_b) * 0.5;

                // Rule: We only resolve actual geometric overlap.
                if Self::aabb_overlap_static(min_a, max_a, min_b, max_b) {
                    let overlap_x = max_a.x.min(max_b.x) - min_a.x.max(min_b.x);
                    let overlap_y = max_a.y.min(max_b.y) - min_a.y.max(min_b.y);
                    let overlap_z = max_a.z.min(max_b.z) - min_a.z.max(min_b.z);

                    // Normal points from B toward A.
                    let (normal, penetration) = if overlap_x < overlap_y && overlap_x < overlap_z {
                        let n = if center_a.x < center_b.x {
                            Vec3::NEG_X
                        } else {
                            Vec3::X
                        };
                        (n, overlap_x)
                    } else if overlap_y < overlap_z {
                        let n = if center_a.y < center_b.y {
                            Vec3::NEG_Y
                        } else {
                            Vec3::Y
                        };
                        (n, overlap_y)
                    } else {
                        let n = if center_a.z < center_b.z {
                            Vec3::NEG_Z
                        } else {
                            Vec3::Z
                        };
                        (n, overlap_z)
                    };

                    if penetration > 0.0001 {
                        let epsilon = 0.0001;
                        let correction = normal * (penetration * 0.5 + epsilon);

                        if body_a.anchored {
                            body_b.position -= normal * (penetration + epsilon);
                        } else if body_b.anchored {
                            body_a.position += normal * (penetration + epsilon);
                        } else {
                            body_a.position += correction;
                            body_b.position -= correction;
                        }

                        // Remove only inward velocity using dot product with normal.
                        let v_a_in = body_a.velocity.dot(normal);
                        if v_a_in < 0.0 {
                            body_a.velocity -= normal * v_a_in;
                        }

                        let v_b_in = body_b.velocity.dot(-normal);
                        if v_b_in < 0.0 {
                            body_b.velocity -= (-normal) * v_b_in;
                        }

                        // Overlap resolution indicates a physical disturbance; wake both.
                        body_a.wake();
                        body_b.wake();
                    }
                }
            }
        }
    }

    /// Identifies exact touching face contacts and registers support relationships.
    pub fn refresh_dynamic_support(&mut self) {
        let n = self.bodies.len();

        // We use indices to safely manage support assignment.
        let mut new_supports: Vec<Option<(PhysicsBodyId, Vec3)>> = vec![None; n];

        for i in 0..n {
            for j in (i + 1)..n {
                let min_a = self.bodies[i].min_corner();
                let max_a = self.bodies[i].max_corner();
                let center_a = (min_a + max_a) * 0.5;

                let min_b = self.bodies[j].min_corner();
                let max_b = self.bodies[j].max_corner();
                let center_b = (min_b + max_b) * 0.5;

                // Touching is defined as zero overlap on one axis within a tolerance
                // and real overlap on the others.
                let epsilon = 0.01;
                let overlap_x = max_a.x.min(max_b.x) - min_a.x.max(min_b.x);
                let overlap_y = max_a.y.min(max_b.y) - min_a.y.max(min_b.y);
                let overlap_z = max_a.z.min(max_b.z) - min_a.z.max(min_b.z);

                let ax = overlap_x.abs() < epsilon && overlap_y > 0.001 && overlap_z > 0.001;
                let ay = overlap_y.abs() < epsilon && overlap_x > 0.001 && overlap_z > 0.001;
                let az = overlap_z.abs() < epsilon && overlap_x > 0.001 && overlap_y > 0.001;

                if ax || ay || az {
                    // Normal points from support toward supported.
                    let (supported_idx, supporting_idx, support_normal) = if ax {
                        if center_a.x < center_b.x {
                            (j, i, Vec3::X)
                        } else {
                            (i, j, Vec3::NEG_X)
                        }
                    } else if ay {
                        if center_a.y < center_b.y {
                            (j, i, Vec3::Y)
                        } else {
                            (i, j, Vec3::NEG_Y)
                        }
                    } else {
                        if center_a.z < center_b.z {
                            (j, i, Vec3::Z)
                        } else {
                            (i, j, Vec3::NEG_Z)
                        }
                    };

                    new_supports[supported_idx] =
                        Some((self.bodies[supporting_idx].id, support_normal));
                }
            }
        }

        // Apply new supports and wake if the support relationship changed or was lost.
        for i in 0..n {
            let old = self.bodies[i].dynamic_contact;
            let new = new_supports[i];

            if old != new {
                self.bodies[i].wake();
            }

            self.bodies[i].dynamic_contact = new;
        }
    }

    /// Evaluates linear velocity and support to manage body sleep state.
    pub fn update_sleeping(&mut self, gravity: Vec3) {
        for i in 0..self.bodies.len() {
            let mut should_wake = false;

            // Supporting Body Check.
            if let Some((support_id, _normal)) = self.bodies[i].dynamic_contact {
                if let Some(support) = self.bodies.iter().find(|b| b.id == support_id) {
                    // Rule: supported body wakes if support body wakes.
                    if !support.is_sleeping {
                        should_wake = true;
                    }
                } else {
                    should_wake = true; // Support disappeared.
                }
            }

            if should_wake {
                self.bodies[i].wake();
            }
        }

        for body in &mut self.bodies {
            if body.anchored {
                continue;
            }

            if body.is_sleeping {
                body.velocity = Vec3::ZERO;
                continue;
            }

            // Sleep Rule: must stay slow for SLEEP_TIME_THRESHOLD.
            if body.velocity.length() < SLEEP_VELOCITY_THRESHOLD {
                // Rule: Under nonzero gravity, a body can only sleep if it has valid physical
                // support opposing the gravity vector.
                let mut is_supported = false;
                if body.gravity_participation && gravity.length_squared() > 1e-6 {
                    if let Some(n) = body.static_contact_normal {
                        if gravity.dot(n) < -1e-3 {
                            is_supported = true;
                        }
                    }
                    if !is_supported {
                        if let Some((_, n)) = body.dynamic_contact {
                            if gravity.dot(n) < -1e-3 {
                                is_supported = true;
                            }
                        }
                    }
                } else {
                    is_supported = true;
                }

                if is_supported {
                    body.sleep_timer += SIMULATION_DT;
                    if body.sleep_timer >= SLEEP_TIME_THRESHOLD {
                        body.is_sleeping = true;
                        body.velocity = Vec3::ZERO;
                    }
                } else {
                    body.sleep_timer = 0.0;
                }
            } else {
                body.sleep_timer = 0.0;
            }
        }
    }
}
