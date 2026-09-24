# Character Gameplay

AeoEngine includes a dedicated runtime character system with movement, collision, jumping, animation, and a third-person gameplay camera.

---

# 1. Character Spawn

Create a SpawnPoint in the authored World.

Enter Play mode and the runtime attempts to create the character at a valid location.

If the initial location is obstructed, nearby clearance search will be used to find a valid spawn location.

---

# 2. Movement

The runtime character supports gameplay movement using fixed-step simulation.

The character system tracks movement state including Idle and Walk behavior.

---

# 3. Gravity

Characters are affected by runtime gravity.

Grounded state is maintained separately so jumping can require valid floor contact.

---

# 4. Jumping

Jumping is gated by grounded state.

This prevents repeated mid-air jumps through ordinary input while the character is not supported by the ground.

---

# 5. Collision

The character can collide with solid World geometry.

The runtime handles:

* Floors.
* Walls.
* Ceilings.

The character's gameplay collision dimensions are kept separate from visual character scaling.

---

# 6. Character Orientation

Character facing follows movement direction.

It is not simply forced to match the camera's orientation.

This allows third-person camera orbit to remain independent from character movement direction.

---

# 7. Animation

The runtime character system includes a dedicated rig and skeleton.

Current animation includes:

* Idle.
* Walk.
* Pose evaluation.
* Animation blending.

Animation updates use the fixed runtime timestep.

---

# 8. Appearance

Character appearance uses generated character geometry and appearance customization data.

The character system owns this runtime visual representation.

---

# 9. Gameplay Camera

The GameplayCamera is a basic dedicated third-person camera.

It provides:

* Character follow.
* Mouse orbit.
* Pitch limits.
* Collision/obstruction handling against solid World geometry.

The GameplayCamera is separate from the Editor camera and can serve as an example for more complex setups

---

# 10. Testing a New Character

A small character test World should contain:

~~~text
SpawnPoint
Floor
Wall
Platform
Ceiling
~~~

Test:

1. Spawn.
2. Walk.
3. Stop at walls.
4. Walk off a platform.
5. Land on the floor.
6. Jump.
7. Test ceiling collision.
8. Orbit the camera.
9. Return to Editor mode.

---

# 11. Character and Scripts

AeoEngine 0.7.3 delegates character movement decisions and camera control to script-defined controller and camera profiles (`controllers/` and `cameras/`), while maintaining a strict responsibility boundary:

### Engine Responsibilities
* Physics simulation and 60Hz fixed timestep
* Gravity application and grounded detection
* AABB character voxel collision against floors, walls, and ceilings
* Rendering and mesh generation

### Script Responsibilities
* Input processing (`input.get_move_vector()`, `input.is_jump_pressed()`, `input.get_orbit_delta()`)
* Movement decisions (`player.set_horizontal_velocity(vx, vz)`, `player.apply_vertical_impulse(jump)`)
* Facing direction (`player.set_facing_direction(dx, dz)`)
* Animation selection (`player.select_animation("Walk")` / `"Idle"`)
* Camera orientation (`camera.set_position(...)`, `camera.set_target(...)`, `camera.set_orientation(...)`, `physics.resolve_camera_collision(...)`)


