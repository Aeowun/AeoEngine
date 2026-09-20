# Runtime Architecture

AeoEngine separates authored scene data from the runtime simulation used during Play mode.

This boundary is fundamental to the engine's behavior.

---

# 1. Authored State

Authored state is created and edited by the developer.

Examples include:

* World Cells.
* Cell types.
* Cell positions.
* Persistent Cell IDs.
* Cell identities/names.
* Authored Cell properties.
* World gravity.
* World lighting.
* Script bindings.
* Project asset references.

Authored state is persistent project data.

---

# 2. Runtime State

Runtime state exists while Play mode is running.

Examples include:

* PhysicsBodies.
* Dynamic body positions and velocities.
* Sleeping and support state.
* Character state.
* Runtime Entity objects.
* Script fibers.
* Script execution state.
* Script lifecycle fields.
* Runtime Cell overrides.
* Gameplay camera state.

Runtime state is not automatically written back into the authored World.

---

# 3. Play Mode Initialization

Entering Play constructs runtime representations from authored World state.

~~~text
World
  ↓
Physics registration
  ↓
Character initialization
  ↓
Script loading
  ↓
Runtime simulation
~~~

The Editor camera state is kept separate from the GameplayCamera.

---

# 4. Runtime Simulation

During Play the major systems cooperate:

~~~text
ScriptScene
     ↓
PhysicsWorld
     ↓
CharacterSystem
     ↓
GameplayCamera
     ↓
Renderer
~~~

The systems have distinct responsibilities.

Scripts implement gameplay logic.

Physics simulates physical bodies and collision.

CharacterSystem manages character movement and animation state.

The GameplayCamera follows the runtime character when appropriate.

The Renderer displays the resulting state.

---

# 5. Runtime Cell Overrides

Scripts can modify supported Cell properties during Play.

~~~aeoscript
cell.visible = false
cell.solid = false
cell.anchored = true
cell.color = [1, 0, 0]
cell.offset = [0, 2, 0]
~~~

These are runtime overrides, not authored edits.

---

# 6. Runtime Physics State

Physics uses runtime state separate from authored World coordinates.

Dynamic bodies can move continuously during simulation without moving the authored Cell in the World grid.

The runtime can therefore simulate falling, collision, support, and sleeping without changing the saved authored position.

---

# 7. Runtime Character State

Characters are simulated by `CharacterSystem` and maintain runtime state for:

* Position.
* Movement.
* Collision.
* Grounded state.
* Jumping.
* Orientation.
* Animation.

The authored World provides the scene from which the runtime character is initialized.

---

# 8. Runtime Script State

`ScriptScene` manages script instances and execution fibers.

A lifecycle fiber contains active execution state such as:

* Call stack.
* Local scopes.
* Instruction position.
* Loop state.
* Wait state.

Persistent script fields are synchronized back to the owning `ScriptEntity` as lifecycle tasks advance so state survives across frames.

---

# 9. Leaving Play Mode

When Play mode stops, temporary runtime state is discarded.

This includes:

* Runtime physics bodies.
* Runtime character state.
* Script fibers.
* Runtime Cell overrides.
* Gameplay camera state.

The authored World remains available for continued editing.

---

# 10. Authoritative State Boundary

The runtime must not silently promote derived simulation state into authored state.

~~~text
Authored World
      ↓
Runtime initialization
      ↓
Temporary simulation state
      ↓
Play stops
      ↓
Temporary state discarded
~~~

This separation allows the editor and runtime to evolve independently.

