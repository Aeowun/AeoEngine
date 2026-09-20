# Character Architecture

The `CharacterSystem` owns runtime character gameplay state.

Characters are simulated separately from generic dynamic physics bodies while interacting with the same authored World collision environment.

---

# 1. Character State

The runtime character maintains state for:

* Position.
* Movement.
* Velocity-related motion.
* Collision.
* Grounded state.
* Jumping.
* Orientation.
* Animation state.
* Visual appearance.

The CharacterSystem is responsible for updating this state during fixed-step gameplay simulation.

---

# 2. Character Spawning

Characters can be spawned from authored SpawnPoint Cells.

Spawn logic validates candidate spawn positions and searches nearby space when the initial location is not clear.

The goal is to avoid spawning the runtime character embedded in solid World geometry.

---

# 3. Fixed-Step Movement

Character movement is updated using the engine's fixed-step simulation timing.

This keeps movement, gravity, and collision behavior consistent with the physics update cadence.

---

# 4. Gravity and Grounding

Characters have runtime gravity and grounded state.

Grounded state controls whether jumping is allowed.

The character can respond to:

* Floor collision.
* Wall collision.
* Ceiling collision.

---

# 5. World Collision

Character collision is performed against solid World geometry.

The authored World therefore remains the source of static scene collision while CharacterSystem owns the runtime character position and motion.

~~~text
Authored World
      ↓
Solid collision geometry
      ↓
CharacterSystem
      ↓
Runtime character motion
~~~

---

# 6. Movement State

The character runtime tracks gameplay movement state including Idle and Walk behavior.

Character orientation follows movement direction independently from camera orientation.

---

# 7. Animation

The character system contains the runtime character rig, skeleton, and animation state.

Current character animation includes Idle and Walk clips with evaluated poses and blending.

Animation is updated on the same fixed runtime cadence as the character simulation.

---

# 8. Character Appearance

Character appearance is represented through runtime customization data and generated character geometry.

The visual implementation is owned by the character system rather than being hard-coded into the editor World representation.

---

# 9. Character Rendering

The runtime character is rendered using dynamic character meshes generated from the current character state and evaluated pose.

The temporary placeholder character representation was replaced by the integrated 3D character implementation.

---

# 10. Gameplay Camera Relationship

The GameplayCamera follows the active runtime character.

The camera can:

* Follow the character.
* Orbit using mouse input.
* Clamp pitch.
* Avoid obstruction from solid World geometry.

Camera orientation does not directly define character facing direction.

---

# 11. Editor Separation

Editor camera state and gameplay camera state are separate.

Entering Play preserves the editor camera.

Leaving Play restores the editor camera without using the gameplay camera as the editor camera state.

---

# 12. Runtime Boundary

Character state exists only during runtime.

The authored SpawnPoint and World geometry remain persistent scene data.

When Play mode stops, runtime character state is discarded.

