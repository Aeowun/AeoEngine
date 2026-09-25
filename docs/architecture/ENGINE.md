# AeoEngine Architecture

AeoEngine is organized around a set of cooperating engine systems. The application layer coordinates those systems while each subsystem owns its own state and behavior.

The central architectural rule is:

> The authored World is the persistent source of truth for scene data. Runtime systems derive temporary simulation state from it.

Persistence, rendering, physics, characters, scripting, and editor interaction operate as separate layers around that authored World.

---

# 1. High-Level Structure

~~~text
Application
├── ProjectManager
├── World
├── WorldStorage
├── Editor
├── Renderer
├── PhysicsClock
├── PhysicsWorld
├── CharacterSystem
├── GameplayCamera
├── ScriptScene
└── EntityManager
~~~

`App` is the primary runtime coordinator. It owns or coordinates the active World, editor state, renderer, runtime physics state, character system, gameplay camera, script scene, and runtime entity manager.

`WorldStorage` is responsible for persistence of authored World data.

The major architectural boundaries are:

~~~text
                Authored Project State
                        │
                        ↓
                     World
                        │
          ┌─────────────┼─────────────┐
          │             │             │
          ↓             ↓             ↓
   WorldStorage      Renderer      Runtime Conversion
          │                            │
          ↓                     ┌──────┼──────┐
     Persistent               ↓      ↓      ↓
     World Data           Physics Characters Scripts
```

The renderer may maintain derived chunk and mesh state, while runtime systems maintain temporary simulation state. Neither becomes the persistent source of authored World data.

---

# 2. Application State

The application has two major views:

* Home
* Editor

The Editor can operate in two modes:

* Editor mode
* Play mode

Home provides project management, templates, and learning resources.

Editor mode is used to author persistent World data.

Play mode runs the authored World through the runtime systems.

The application coordinates transitions between these states while preserving the boundary between authored data and temporary simulation state.

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

Editor interaction state such as selection, active tools, previews, and editor camera state is separate from authored World content.

Runtime physics bodies and runtime character state are not the authoritative representation of the authored World.

The editor changes authored World state, while runtime systems consume derived or temporary representations of that data.

---

# 4. World and Persistence

The World represents the authored scene data used by the editor and runtime.

Persistent World data includes information such as:

* Cells.
* Cell properties.
* Light properties.
* Cell attributes.
* Script bindings.
* Other authored scene configuration.

`WorldStorage` is responsible only for persistence.

Conceptually:

~~~text
World
  ↕
WorldStorage
  ↕
Persistent project data
~~~

`WorldStorage` may:

* Load World data.
* Save World data.
* Determine whether stored World data exists.
* Remove stored World data when explicitly required.

Persistence does not own rendering, physics, characters, or scripting behavior.

The storage layer must not become responsible for:

* Rendering.
* Physics simulation.
* Character simulation.
* Script execution.
* Editor interaction.

This preserves the separation between persistent project data and the systems that consume it.

---

# 5. Entering Play Mode

When Play mode begins, runtime systems are initialized from authored World data.

~~~text
Authored World
      ↓
Runtime Conversion
      ├────────→ PhysicsWorld
      ├────────→ CharacterSystem
      ├────────→ ScriptScene
      └────────→ Runtime World State
~~~

The editor camera state is preserved before gameplay begins.

A dedicated `GameplayCamera` is used during Play when a runtime gameplay camera is active.

Runtime state is derived from the authored World rather than replacing it.

Changes made during Play remain temporary unless an explicit authored-data workflow records them.

---

# 6. Runtime Update

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
* GameplayCamera owns gameplay camera behavior.
* Renderer turns current state into graphics.

Runtime systems may exchange derived state as required for simulation, but they do not replace the authored World as the persistent source of truth.

---

# 7. Rendering Architecture

The renderer consumes World and runtime state and builds the representation required for GPU rendering.

The World representation and the render representation are separate.

~~~text
Authored World
      ↓
Render Representation
      ↓
Spatial Render Chunks
      ↓
Chunk Meshes
      ↓
GPU Rendering
~~~

The renderer uses chunk-based construction so localized World changes can rebuild affected render chunks instead of reconstructing unrelated render geometry.

Selective rebuilding is therefore a renderer concern rather than a change to the authored World model.

Renderer state may include:

* Render chunks.
* Chunk meshes.
* GPU buffers.
* Texture resources.
* Other graphics resources.

These are derived runtime resources and are not persistent authored World state.

---

# 8. Leaving Play Mode

Leaving Play mode restores the editor/runtime boundary.

Runtime systems are stopped or cleared as appropriate, including:

* Runtime script fibers.
* Runtime physics bodies.
* Runtime character state.
* Runtime Cell overrides.
* Gameplay camera state.
* Other temporary simulation state.

The saved editor camera is restored.

Runtime render and simulation resources may be released or rebuilt as necessary.

The authored World remains intact.

Stopping Play does not convert runtime state back into authored World data automatically.

---

# 9. System Ownership

### World

Owns authored scene data and the World state used by editor and runtime systems.

Runtime property overrides and other temporary changes are maintained separately from persistent authored values.

### WorldStorage

Owns persistence of authored World data.

It is responsible for loading and saving persistent World state and does not own rendering, physics, character, scripting, or editor behavior.

### Editor

Owns editing tools, selection, camera state, history, project interaction, and editor UI state.

### Renderer

Owns rendering resources and the derived graphics representation of the current World and runtime state.

This includes chunk-based render construction and selective rebuilding of affected render regions.

### PhysicsWorld

Owns runtime physics bodies and static collision representation.

### CharacterSystem

Owns runtime character state, movement, collision, and animation state.

### GameplayCamera

Owns the runtime gameplay camera.

### ScriptScene

Owns active script instances, lifecycle execution, and script runtime tasks/fibers.

### EntityManager

Owns live runtime entities used by gameplay and scripting.

---

# 10. Data Flow

The major authored-data path is:

~~~text
Persistent Project Data
        ↓
   WorldStorage
        ↓
      World
        ↓
   Authored State
~~~

The editor operates on that authored state:

~~~text
World
  ↓
Editor
  ↓
Authored World Changes
  ↓
WorldStorage
```

The rendering path is derived from the World:

~~~text
World
  ↓
Render Representation
  ↓
Spatial Chunks
  ↓
Chunk Meshes
  ↓
Renderer / GPU
~~~

The runtime path is also derived from the World:

~~~text
World
  ↓
Runtime Conversion
  ├────────→ PhysicsWorld
  ├────────→ CharacterSystem
  ├────────→ ScriptScene
  ├────────→ GameplayCamera
  └────────→ Runtime Entity State
~~~

Runtime systems may then produce temporary state:

~~~text
Runtime Systems
      ↓
Temporary Runtime State
      ↓
Renderer / Gameplay
~~~

None of these derived paths changes which data is authoritative.

Runtime systems may maintain derived or temporary state. They must not silently become the persistent source of authored scene data.

---

# 11. Runtime World State

Runtime systems may require temporary representations of authored World data.

Examples include:

* Physics bodies.
* Static collision representations.
* Runtime-created Cells.
* Runtime property overrides.
* Character state.
* Script execution state.
* Gameplay camera state.
* Runtime UI.
* Derived renderer chunks and meshes.

These systems may mutate their own runtime state during Play.

Those mutations do not automatically modify the authored World.

The relationship is:

~~~text
Authored World
      ↓
Derived Runtime State
      ↓
Temporary Simulation
```

The authored World remains the source of truth when runtime state is cleared.

---

# 12. Editor and Runtime Boundary

The editor and runtime operate on different responsibilities.

The editor answers:

> What has the developer authored?

The runtime answers:

> What is happening to that authored World right now?

The renderer answers:

> How should the current World and runtime state be represented visually?

The persistence layer answers:

> How is authored World data stored and restored?

Keeping these responsibilities separate prevents simulation, rendering, or editor interaction state from silently becoming persistent scene state.

---

# 13. Architectural Principle

The central architecture can be summarized as:

~~~text
                 Authored World
                       │
          ┌────────────┼────────────┐
          │            │            │
          ↓            ↓            ↓
   WorldStorage     Editor       Runtime
          │            │            │
          ↓            ↓       ┌────┼────────────┐
   Persistent Data  Authoring   ↓    ↓      ↓     ↓
                           Physics Characters Scripts Camera
                                  │
                                  ↓
                              Runtime State
                                  │
                                  ↓
                               Renderer
~~~

The persistent authored World remains authoritative.

`WorldStorage` persists it.

The editor modifies it.

Runtime systems derive temporary state from it.

The renderer builds a visual representation from the current state.

This separation allows the editor, persistence layer, runtime systems, and renderer to evolve independently while preserving a stable authored-data boundary.

---

# Change Summary

*Added: `WorldStorage` to the high-level architecture.
*Added: A dedicated persistence section defining the responsibilities of `WorldStorage`.
*Added: Explicit persistence data flow between `World` and stored project data.
*Added: Explicit renderer data flow from World → render representation → spatial chunks → chunk meshes → GPU.
*Added: Selective render-chunk rebuilding as part of renderer architecture.
*Added: Explicit distinction between authored World data and derived renderer resources.
*Added: Explicit runtime conversion boundary between authored World state and PhysicsWorld, CharacterSystem, ScriptScene, GameplayCamera, and runtime entities.
*Added: Runtime World State section describing temporary simulation representations.
*Changed: `World` ownership to distinguish authored scene data from temporary runtime overrides.
*Changed: `Renderer` ownership to include chunk-based render construction and selective rebuilding.
*Changed: `Data Flow` to separate persistence, editor authoring, rendering, and runtime conversion paths.
*Changed: `Leaving Play Mode` to explicitly state that runtime state is discarded rather than converted back into authored World data.
*Changed: `Architectural Principle` to include the persistence layer and renderer as separate architectural responsibilities.