# AeoEngine Architecture

AeoEngine is organized around a set of cooperating engine systems. The application layer coordinates those systems while each subsystem owns its own state and behavior.

The central architectural rule is:

> The authored World is the persistent source of truth for scene data. Runtime systems derive temporary simulation state from it.

---

# 1. High-Level Structure

~~~text
Application
├── ProjectManager
├── World
├── Editor
├── Renderer
├── PhysicsClock
├── PhysicsWorld
├── CharacterSystem
├── GameplayCamera
├── ScriptScene
└── EntityManager
~~~

`App` is the primary runtime coordinator. It owns the active World, editor state, renderer, runtime physics state, character system, gameplay camera, script scene, and runtime entity manager.

---

# 2. Application State

The application has two major views:

* Home
* Editor

The Editor can operate in two modes:

* Editor mode
* Play mode

Home provides project management. Editor mode is used to author persistent World data. Play mode runs the authored World through the runtime systems.

---

# 3. Editor Mode

Editor mode primarily operates on authored data.

The editor can:

* Navigate the World.
* Select Cells.
* Build Cells.
* Erase Cells.
* Edit Cell properties.
* Manage hierarchy and selection.
* Edit lighting properties.
* Import project textures.
* Attach scripts.
* Modify script bindings.
* Undo and redo authored edits.
* Save the project.

Runtime physics bodies and runtime character state are not the authoritative representation of the authored World.

---

# 4. Entering Play Mode

When Play mode begins, runtime systems are initialized from authored World data.

~~~text
Authored World
      ↓
Register physics state
      ↓
Spawn runtime character
      ↓
Load/start ScriptScene
      ↓
Begin runtime simulation
~~~

The editor camera state is preserved before gameplay begins. A dedicated GameplayCamera is used during Play when a runtime character exists.

---

# 5. Runtime Update

During Play mode the application coordinates the major runtime systems each frame.

~~~text
Frame
 ↓
Script runtime update
 ↓
Physics synchronization
 ↓
Fixed-step physics
 ↓
Character simulation
 ↓
Gameplay camera
 ↓
Rendering
~~~

The exact implementation order may evolve, but ownership remains separated:

* Scripts control gameplay logic.
* Physics owns physical simulation.
* CharacterSystem owns runtime character state.
* Renderer turns current state into graphics.

---

# 6. Leaving Play Mode

Leaving Play mode restores the editor/runtime boundary.

Runtime systems are stopped or cleared as appropriate, including:

* Runtime script fibers.
* Runtime physics bodies.
* Runtime character state.
* Runtime Cell overrides.
* Gameplay camera state.

The saved editor camera is restored.

The authored World remains intact.

---

# 7. System Ownership

### World

Owns authored scene data and runtime Cell overrides.

### Editor

Owns editing tools, selection, camera state, history, project interaction, and editor UI state.

### Renderer

Owns OpenGL rendering resources and converts current engine state into graphics.

### PhysicsWorld

Owns runtime physics bodies and static collision representation.

### CharacterSystem

Owns runtime character state, movement, collision, and animation state.

### GameplayCamera

Owns the runtime third-person gameplay camera.

### ScriptScene

Owns active script instances, lifecycle execution, and script runtime tasks/fibers.

### EntityManager

Owns live runtime entities used by gameplay and scripting.

---

# 8. Data Flow

~~~text
Authored World
      │
      ├──────────────→ Renderer
      │
      ├──────────────→ PhysicsWorld
      │
      ├──────────────→ CharacterSystem
      │
      └──────────────→ ScriptScene
                              │
                              ↓
                       Runtime changes
~~~

Runtime systems may maintain derived or temporary state. They must not silently become the persistent source of authored scene data.

---

# 9. Architectural Principle

The editor answers:

> What has the developer authored?

The runtime answers:

> What is happening to that authored world right now?

Keeping those responsibilities separate prevents simulation state from silently becoming authored scene state.

