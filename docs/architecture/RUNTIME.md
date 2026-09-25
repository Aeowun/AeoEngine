# Runtime Architecture

AeoEngine separates persistent authored World data from the temporary runtime state used during Play mode.

The runtime is the simulation layer. It derives state from the authored World and is discarded or rebuilt when Play mode ends.

---

# 1. Authored State

Authored state is persistent project data.

Examples include:

* World Cells.
* Cell types.
* Cell positions.
* Persistent Cell IDs.
* Cell identities.
* Cell properties.
* Cell attributes.
* World gravity.
* World lighting.
* Sky settings.
* Script bindings.
* Project asset references.
* Selected character/controller/camera configuration.

The editor is the primary system that modifies authored state.

---

# 2. Runtime State

Runtime state exists while the game is being simulated.

Examples include:

* PhysicsBodies.
* Dynamic positions and velocities.
* Support and sleeping state.
* Runtime Characters.
* Runtime Entities.
* Script fibers.
* Script execution state.
* Runtime Cell overrides.
* Runtime-created Cells.
* Gameplay camera state.
* Runtime UI.
* Audio playback state.

Runtime state is not automatically written back into authored World data.

---

# 3. Runtime Conversion

Entering Play converts or derives runtime representations from the authored World.

```text
Authored World
      ↓
Runtime initialization
      ├── PhysicsWorld
      ├── CharacterSystem
      ├── ScriptScene
      ├── EntityManager
      └── GameplayCamera
```

The renderer consumes authored and runtime-effective state separately from these simulation systems.

---

# 4. Play Mode Initialization

Play mode initializes the systems required to simulate the current project.

Typical initialization includes:

* Registering World collision with PhysicsWorld.
* Creating the runtime character.
* Initializing runtime Entities.
* Loading script bindings.
* Creating ScriptInstances.
* Starting lifecycle execution.
* Activating the gameplay camera.
* Preparing runtime rendering state.

The editor camera remains separate from the gameplay camera.

---

# 5. Runtime Update

The application coordinates runtime systems each frame.

The exact implementation order can evolve, but ownership remains distinct.

Conceptually:

```text
Frame
 ↓
Script runtime
 ↓
Physics synchronization
 ↓
Fixed-step simulation
 ↓
Character simulation
 ↓
Gameplay camera
 ↓
Rendering
```

Scripts own gameplay logic.

Physics owns physical simulation.

CharacterSystem owns character simulation.

ScriptScene owns script execution.

Renderer owns graphics representation.

---

# 6. Runtime World State

The World maintains temporary runtime state for supported runtime mutations.

For example:

```text
Authored Cell
      ↓
Runtime override
      ↓
Effective Cell value
```

Runtime values can include:

* Visibility.
* Solidity.
* Anchored state.
* Color.
* Offset.
* Light enabled state.
* Audio playback state.
* Attribute overrides.
* Temporary deletion state.

The authored value remains available as the baseline.

---

# 7. Runtime Physics

Physics uses runtime state separate from authored World coordinates.

A dynamic body can move continuously:

```text
Authored coordinate
      ↓
PhysicsBody
      ↓
Continuous runtime position
```

The body's movement does not rewrite the authored Cell position on every physics step.

---

# 8. Runtime Characters

Characters are owned by `CharacterSystem`.

Character runtime state includes:

* Position.
* Movement.
* Gravity.
* Collision.
* Grounded state.
* Jumping.
* Orientation.
* Animation.
* Runtime appearance state.

Character simulation interacts with World collision while remaining separate from authored Cell data.

---

# 9. Runtime Scripting

`ScriptScene` connects authored script bindings to runtime script instances.

A typical relationship is:

```text
Authored binding
      ↓
ScriptInstance
      ↓
ScriptFiber
      ↓
ScriptScheduler
      ↓
Engine host
      ↓
World / Runtime systems
```

Persistent script fields belong to the running script instance.

Temporary execution state belongs to the active fiber.

---

# 10. Runtime Entities

`EntityManager` owns live runtime Entities.

Entities are runtime objects and may represent:

* Players.
* NPCs.
* Other gameplay-managed objects.

AeoScript `Entity` handles reference these runtime objects.

The runtime Entity system does not replace the authored World Cell representation.

---

# 11. Runtime Rendering

The renderer consumes current effective state and builds a graphics representation.

```text
Authored World
      ↓
Effective state
      ↓
Render representation
      ↓
GPU
```

Static voxel geometry uses chunk-based rendering.

Dynamic runtime objects are represented separately from persistent authored World geometry.

---

# 12. Leaving Play Mode

When Play mode stops, temporary runtime systems are cleared or stopped.

This includes:

* Runtime script fibers.
* Runtime physics bodies.
* Runtime character state.
* Runtime-created Cells.
* Runtime Cell overrides.
* Gameplay camera state.
* Runtime UI.
* Other simulation-derived state.

The editor camera is restored.

The authored World remains available for continued editing.

---

# 13. Authority Boundary

The runtime must not silently promote derived state into authored state.

```text
Authored World
      ↓
Runtime initialization
      ↓
Temporary simulation
      ↓
Play stops
      ↓
Runtime state discarded
```

This separation allows Play mode to be started repeatedly without replacing persistent project data with temporary simulation state.