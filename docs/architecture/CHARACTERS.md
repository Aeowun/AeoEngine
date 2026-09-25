# Character Architecture

The `CharacterSystem` owns runtime character gameplay state.

Character simulation is separate from generic dynamic PhysicsBodies while using the same authored World collision environment.

---

# 1. CharacterSystem

`CharacterSystem` owns live runtime Characters and their gameplay state.

Character state includes:

* Position.
* Movement.
* Gravity.
* Collision.
* Grounded state.
* Jumping.
* Orientation.
* Animation state.
* Runtime visual state.

---

# 2. Character Spawning

Characters can be spawned from authored SpawnPoint Cells.

The spawn system:

* Locates available SpawnPoints.
* Evaluates candidate positions.
* Checks clearance.
* Selects a usable runtime position.

The objective is to avoid starting the runtime character inside solid World geometry.

---

# 3. Fixed-Step Movement

Character movement runs as part of the engine's fixed-step simulation.

This keeps:

* Movement.
* Gravity.
* Collision.
* Jumping.

synchronized with the physics simulation cadence.

---

# 4. Input

The application gathers runtime input.

AeoScript can also access gameplay input through the scripting API.

The general control path is:

```text
Player input
      ↓
App / script input access
      ↓
CharacterSystem
      ↓
Runtime character state
```

Input ownership remains separate from the character's physical simulation.

---

# 5. Movement

Character movement includes:

* Horizontal movement.
* Jumping.
* Gravity.
* Orientation.
* Movement state.

Movement constants and state live in the character movement subsystem.

Character-relative behavior can also be driven by AeoScript through the player APIs.

---

# 6. Grounding

Grounded state is determined from collision with supporting geometry.

Grounded state affects gameplay behavior such as jumping.

The character also responds to:

* Floor collision.
* Wall collision.
* Ceiling collision.

---

# 7. World Collision

Character collision uses solid World geometry.

```text
Authored World
      ↓
Effective collision geometry
      ↓
CharacterSystem
      ↓
Runtime character motion
```

The character's runtime position does not become an authored Cell position.

---

# 8. Dynamic Physics Interaction

Characters can interact with dynamic physics bodies.

Character-specific collision handling determines how runtime character motion responds to dynamic bodies.

The character remains owned by CharacterSystem while PhysicsWorld owns the generic physics bodies.

---

# 9. Animation

The runtime character contains animation state and evaluated pose information.

Current animation workflows include:

* Idle.
* Walk.
* Animation selection.
* Animation blending.
* Pose evaluation.

The high-level runtime state is owned by CharacterSystem.

Lower-level animation and rig data are implemented under:

```text
src/character_custom/
```

---

# 10. Custom Character Infrastructure

The custom character subsystem contains:

```text
animation.rs
appearance.rs
blend.rs
collision.rs
geometry.rs
rig.rs
```

Responsibilities include:

* Animation clips.
* Transform tracks.
* Skeleton/rig data.
* Pose evaluation.
* Animation blending.
* Character geometry.
* Appearance/material configuration.
* Custom collision representation.

---

# 11. Gameplay Camera

Gameplay camera behavior is separate from CharacterSystem.

The runtime relationship is:

```text
CharacterSystem
      ↓
Runtime character state
      ↓
GameplayCamera
      ↓
Renderer
```

Character logic can provide the runtime state needed by the gameplay camera, while the camera remains a separate subsystem.

---

# 12. Runtime Boundary

Characters are runtime objects.

```text
Authored SpawnPoint
      ↓
Character spawning
      ↓
Runtime Character
      ↓
Movement / collision / animation
      ↓
Temporary Play-mode state
```

When Play mode ends, runtime character state is discarded.

The authored SpawnPoint and other World data remain unchanged.