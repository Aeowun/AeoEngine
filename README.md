# AeoEngine

> A voxel engine and editor for building authored worlds, defining gameplay with AeoScript, and running those worlds through a runtime with physics, lighting, characters, cameras, and interactive systems.

AeoEngine combines a visual World editor with a runtime simulation environment. Worlds are authored as persistent voxel-based scene data, while Play mode creates the temporary runtime state needed to simulate physics, characters, scripts, lighting, and other gameplay systems.

The engine is designed around a clear separation between **authored data** and **runtime behavior**. Persistent World content remains the source of truth for the project, while runtime systems operate on derived or temporary state that can be discarded when Play mode stops.

AeoEngine is being developed as a general-purpose foundation for building small games and interactive voxel worlds rather than as a single fixed gameplay framework.

## Contents

* [What is it?](#what-is-it)
* [Core Model](#core-model)
* [Current Features](#current-features)
* [Getting Started](#getting-started)

  * [Requirements](#requirements)
  * [Build and Run](#build-and-run)
* [Help](#help)

  * [Your First Project](#your-first-project)
  * [How Projects Are Organized](#how-projects-are-organized)
  * [How to Add a Controller](#how-to-add-a-controller)
  * [How to Add a Camera](#how-to-add-a-camera)
  * [How to Write AeoScript](#how-to-write-aeoscript)
  * [How to Add Textures](#how-to-add-textures)
  * [How to Customize a Character](#how-to-customize-a-character)
  * [Understanding Play Mode](#understanding-play-mode)
  * [Common Problems](#common-problems)
  * [Where to Go Next](#where-to-go-next)
* [Documentation](#documentation)

  * [Getting Started and Guides](#getting-started-and-guides)
  * [AeoScript](#aeoscript)
  * [Architecture](#architecture)
  * [Development](#development)
  * [Changelog](#changelog)
* [Project Status](#project-status)

  * [AeoScript Support](#aeoscript-support)
  * [Integration Testing](#integration-testing)
* [Performance](#performance)

  * [World Rendering](#world-rendering)
  * [Static Physics Synchronization](#static-physics-synchronization)
  * [Dynamic Physics](#dynamic-physics)
  * [Continuous Collision Detection](#continuous-collision-detection)
  * [Runtime Mesh Resources](#runtime-mesh-resources)
  * [Performance Work](#performance-work)
* [Versioning](#versioning)

---

## What is it?

AeoEngine provides a grid-based editor for creating authored worlds and a runtime for simulating and playing them.

The editor is responsible for creating and modifying persistent World content such as Blocks, Lights, textures, hierarchy data, attributes, and other authored scene information.

Play mode uses that authored World as its starting point and creates the runtime state required to simulate the game.

> [!IMPORTANT]
> Authored World data is kept separate from runtime state.

Changes made during Play mode—such as moving a physics body, changing a Cell property through AeoScript, creating or deleting runtime Cells, modifying runtime character state, or changing other temporary simulation state—are discarded when Play mode stops.

The authored World remains the source of truth for persistent scene data.

This separation allows the editor and runtime to evolve independently while keeping project data stable across repeated Play sessions.

---

## Core Model

AeoEngine broadly follows this flow:

```text
Authored World
      |
      +-------------------+
      |                   |
      v                   v
Render Representation   Runtime Conversion
      |                   |
      v                   +-------------------+
   Renderer               |        |          |
                          v        v          v
                     Physics   Characters   Scripts
```

The important boundary is between **what the project author saves** and **what the runtime temporarily derives from that data**.

### Authored State

Authored state includes persistent project content such as:

* World Cells
* Block properties
* Light properties
* World settings
* Cell attributes
* Script bindings
* Imported project assets
* Other persistent scene configuration

### Runtime State

Runtime state includes temporary systems such as:

* Physics bodies
* Runtime-created Cells
* Runtime property overrides
* Character movement state
* Character animation state
* Gameplay camera state
* Script execution state
* Runtime UI
* Other simulation-derived data

Stopping Play discards runtime state and leaves the authored World unchanged.

---

## Current Features

| Area              | Features                                                                                      |
| ----------------- | --------------------------------------------------------------------------------------------- |
| World Editing     | Voxel World editing, authored Block and Light Cells, World hierarchy, multi-selection         |
| Build Tools       | Plane and Top-Block building modes, Build, Erase, Undo, Redo                                  |
| Selection         | Cell selection, viewport multi-selection, persistent selection outlines, camera focus         |
| Assets            | Project-owned texture assets, imported PNG textures, runtime texture loading and caching      |
| Environment       | World sky environment and built-in sky presets                                                |
| Physics           | Runtime physics simulation, dynamic `PhysicsBody` instances, static collision synchronization |
| Characters        | Character spawning, movement, collision, gravity, jumping, animation                          |
| Camera            | Third-person gameplay camera, orbit, follow behavior, pitch limits, camera collision          |
| Lighting          | Directional and point lighting, authored Light Cells                                          |
| Audio             | Authored Audio Emitter Cells, WAV audio asset browser, runtime `AudioSystem` playback          |
| Scripting         | AeoScript gameplay scripting, top-level execution, lifecycle events, persistent script state  |
| Script Runtime    | Cooperative fibers, `wait()`, nested function calls, closures, runtime diagnostics            |
| Script Objects    | Cell and Entity handles, persistent Cell IDs, object discovery                                |
| Runtime World     | Authored Cell attributes, runtime property overrides, runtime Cell creation and deletion      |
| Runtime Input     | Movement input, jump input, mouse orbit input                                                 |
| Mouse Control     | World-authored mouse settings (`cursor_visible`, `screen_locked`), runtime mouse cursor API  |
| Scripted Gameplay | Script-defined controllers, script-defined cameras, generic player and camera APIs            |
| Events            | `on_touch(cell)`, per-Cell collision event control                                            |
| Script Control    | Runtime script enable/disable                                                                 |
| Runtime UI        | Runtime `Panel`, `Text`, and `Button` elements, properties, handles, callbacks                |
| Tooling           | AeoScript standard library, Script Editor, runtime terminal output                            |
| Testing           | In-game AeoScript integration testing                                                         |

---

# Getting Started

## Requirements

| Requirement      | Version          |
| ---------------- | ---------------- |
| Operating System | Windows 10 or 11 |
| Rust             | Stable toolchain |

## Build and Run

Open a terminal in the project root:

```text
cargo run
```

---

# Help

The technical documentation contains detailed engine and language information, but you do not need to understand all of it before making something in AeoEngine.

The recommended approach is:

1. Start with the Starter World.
2. Copy an existing working example.
3. Change one thing at a time.
4. Use the technical documentation when you need to understand a specific system.

AeoEngine is still under active development, so some systems are easier to customize than others.

---

## Your First Project

The easiest way to start is with the included Starter World.

Instead of building every project folder manually, copy the Starter World and use it as the starting point for your own game.

The Starter World provides simple working examples of:

* A third-person controller
* A third-person camera
* A first-person camera
* Character setup
* AeoScript files
* World data
* Project textures
* Runtime systems

> [!NOTE]
> The Starter World is primarily a reference and learning project. Some examples are still under active development and may have rough edges.


The Starter World is intended to be understandable and modifiable rather than treated as a black box.

A good first exercise is to make a copy and change one thing:

* Change camera sensitivity.
* Change movement speed.
* Change the camera distance.
* Change a texture.
* Add a simple scripted object.
* Add a runtime UI element.

Once that works, begin replacing the provided pieces with your own.

---

## How Projects Are Organized

A typical AeoEngine project looks roughly like this:

```text
UserData/
└── MyProject/
    ├── world.dat
    ├── camera.dat
    ├── scripts/
    ├── controllers/
    ├── cameras/
    ├── characters/
    └── .assets/
        ├── textures/
        └── ...
```

The important directories are:

| Directory      | Purpose                                 |
| -------------- | --------------------------------------- |
| `scripts/`     | General AeoScript gameplay and behavior |
| `controllers/` | Script-defined player controllers       |
| `cameras/`     | Script-defined gameplay cameras         |
| `characters/`  | Character packages and character data   |
| `.assets/`     | Project-owned textures and other assets |

For most beginners, the first places to work are:

```text
scripts/
controllers/
cameras/
```

---

## How to Add a Controller

The easiest way to create a new controller is to copy an existing one.

The Starter World contains:

```text
UserData/Starter World/controllers/thirdPerson_Controller/
```

Copy that entire directory into your own project:

```text
UserData/MyProject/controllers/
```

Then rename the copied controller directory to something descriptive, for example:

```text
UserData/MyProject/controllers/myController/
```

Inside the controller package, modify the existing `.aeo` script rather than starting from an empty file.

This gives you a known-working controller structure and keeps the package configuration correct.

The controller can then be changed to implement your own gameplay behavior.

For example, you can change:

* Movement speed
* Jump behavior
* Camera-relative movement
* Facing behavior
* Animation selection
* Input behavior

The engine still owns gravity, collision, grounded state, and physics integration.

### Selecting the Controller

The selected controller is referenced by the project configuration.

The Starter World uses:

```text
SELECTED_CONTROLLER thirdPerson_Controller
```

When creating a new controller, update the project selection to reference your new controller package.

The exact project data format is documented in the engine documentation and can be studied directly from the Starter World.

---

## How to Add a Camera

Cameras use the same general workflow as controllers.

The Starter World includes:

```text
UserData/Starter World/cameras/thirdPerson/
UserData/Starter World/cameras/firstPerson/
```

Copy one of these directories into your own project:

```text
UserData/MyProject/cameras/
```

Rename it and modify the included `.aeo` script.

For example:

```text
UserData/MyProject/cameras/myCamera/
```

Starting from an existing camera is recommended because the folder structure, package configuration, and script interface are already known to work.

A camera script can control things such as:

* Follow distance
* Camera height
* Look height
* Orbit sensitivity
* Yaw
* Pitch
* Camera positioning
* Camera collision behavior

The engine provides the underlying camera and rendering systems.

The script controls the gameplay behavior.

### Selecting the Camera

The selected camera is referenced by the project configuration.

The Starter World provides examples such as:

```text
SELECTED_CAMERA thirdPerson
```

or:

```text
SELECTED_CAMERA firstPerson
```

When creating a new camera, update the project selection to reference the new camera package.

---

## How to Write AeoScript

AeoScript is the gameplay language used by AeoEngine.

You do **not** need to learn the entire language before writing useful scripts.

Start by copying a small working script and changing it.

AeoScript supports:

* Variables
* Constants
* Functions
* Conditionals
* Loops
* Arrays / baskets
* Maps
* Functions as values
* Closures
* Runtime handles
* Events
* Input
* Runtime UI
* World interaction

A useful beginner workflow is:

```text
Copy working script
      ↓
Change one value
      ↓
Run the game
      ↓
Observe result
      ↓
Change behavior
      ↓
Repeat
```

For example, a controller might contain a value such as:

```aeoscript
speed = 5.0
```

Changing it to:

```aeoscript
speed = 8.0
```

changes the controller's movement speed without changing the Rust engine.

### Where Scripts Go

General gameplay scripts go into:

```text
UserData/MyProject/scripts/
```

Controllers go into:

```text
UserData/MyProject/controllers/
```

Cameras go into:

```text
UserData/MyProject/cameras/
```

The Script Editor can be used to inspect and modify AeoScript while working on the project.

### Where to Learn the Language

Start with:

* [AeoScript Overview](./docs/language/AeoScript.md)
* [AeoScript Examples](./docs/language/AeoScript_EXAMPLES.md)
* [AeoScript API](./docs/language/AeoScript_API.md)

Use the grammar, VM, lifecycle, and type documentation when you need deeper information about how the language works.

---

## How to Add Textures

Project textures are stored in the project's asset directory.

For example:

```text
UserData/MyProject/.assets/textures/
```

The current texture naming convention avoids spaces.

Examples:

```text
brick.png
brick_1.png
grass_1.png
roof_1.png
tile_1.png
wood_1.png
logo.png
debug.png
lightbulb.png
```

When creating new textures, use simple filenames without spaces.

This makes world data and asset references easier to work with.

After adding a texture, reference its filename from the appropriate World data or script system.

The engine loads project-owned textures at runtime and caches loaded texture resources.

---

## How to Customize a Character

**Full custom character authoring is not supported yet.**

Character behavior is currently further along than character customization, but the character system is still being expanded.

You can currently work with the existing character system and script-defined controller/camera systems.

A fully script-defined character workflow is planned and is not far off, but it should not be treated as a supported authoring workflow in this release.

Until that work is complete, use the existing character packages and focus customization work on:

* Controller behavior
* Camera behavior
* Input
* Gameplay scripts
* Animation selection
* Runtime interactions

This distinction is temporary and will change as the Character System work is completed.

---

## Understanding Play Mode

Play mode creates the temporary runtime state used to simulate the game.

During Play mode the engine can create or modify things such as:

* Physics bodies
* Character movement state
* Runtime Cell state
* Runtime property overrides
* Script state
* Camera state
* Runtime UI

When Play mode stops, those runtime changes are discarded.

The authored World remains unchanged.

This means:

> If you want something to become part of the saved project, author it in the World rather than relying on runtime changes.

Runtime scripting is intended for gameplay behavior, not as a replacement for authored project data.

---

## Common Problems

### My texture does not load

Check that:

1. The file exists inside the project's asset directory.
2. The filename matches the World reference exactly.
3. The filename follows the current naming convention.
4. The file does not accidentally contain spaces or an old filename.

For example:

```text
grass_4.png
```

is preferable to:

```text
Grass 4 - 128x128.png
```

---

### My controller does not appear

Check that:

1. The controller directory is inside the project's `controllers/` directory.
2. The copied package contains its required files.
3. The controller's name matches the project selection.
4. The `.aeo` script is inside the package.
5. You started from a known-working controller such as the Starter World third-person controller.

---

### My camera does not appear

Check the same things as the controller.

Start by copying:

```text
UserData/Starter World/cameras/thirdPerson/
```

or:

```text
UserData/Starter World/cameras/firstPerson/
```

Then modify the copy.

This is currently the safest way to create a new camera package.

---

### Why did my changes disappear?

Play mode changes are runtime changes.

They are discarded when Play mode stops.

This is expected behavior.

If the change needs to persist between sessions, it belongs in authored World data or another persistent project resource.

---

### The game is running slowly

The current voxel renderer is not yet optimized for large dense worlds.

Very dense Worlds can therefore become expensive.

Start with a smaller test area and avoid unnecessarily filling large regions with blocks until the planned voxel rendering improvements are available.

---

### A script is difficult to understand

Start with one of the examples in the Starter World rather than reading the entire AeoScript implementation documentation.

The technical documentation describes the engine in detail, but it is not intended to be the first thing a new game developer reads cover to cover.

Use it as a reference when a specific question comes up.

---

## Where to Go Next

A practical learning path is:

### Beginner

Start with:

1. Create or copy a Starter World.
2. Change a texture.
3. Change a controller value.
4. Change camera sensitivity.
5. Add a small script.
6. Run the game and observe the result.

### Intermediate

Next learn:

* Entity and Cell handles
* Input
* Runtime Cells
* Attributes
* Events
* Runtime UI
* Closures
* Script control

### Advanced

Then move into:

* AeoScript lifecycle behavior
* Runtime architecture
* Physics integration
* Character systems
* Rendering architecture
* Engine development

The technical documentation remains available for all of these systems.

The important thing is that you do not need to understand the entire engine before you can start building with it.

---

# Documentation

The `docs/` directory contains the engine, architecture, development, gameplay, and AeoScript documentation.

The documentation is intentionally detailed, but it is primarily technical reference material.

For a new developer, start with the **Help** section above and then move into the relevant guide.

## Getting Started and Guides

| Document                                            | Description                                      |
| --------------------------------------------------- | ------------------------------------------------ |
| [Getting Started](./docs/guides/GETTING_STARTED.md) | Initial engine setup and basic workflow          |
| [World Building](./docs/guides/WORLD_BUILDING.md)   | Creating and editing authored voxel worlds       |
| [Building a Game](./docs/guides/BUILDING_A_GAME.md) | Building a complete game workflow with AeoEngine |

## AeoScript

| Document                                                          | Description                                  |
| ----------------------------------------------------------------- | -------------------------------------------- |
| [AeoScript Overview](./docs/language/AeoScript.md)                | Language overview and core concepts          |
| [AeoScript API](./docs/language/AeoScript_API.md)                 | Engine-facing scripting API                  |
| [AeoScript Grammar](./docs/language/AeoScript_GRAMMAR.md)         | Language grammar and syntax                  |
| [AeoScript Types](./docs/language/AeoScript_TYPES.md)             | AeoScript values and runtime types           |
| [AeoScript Handles](./docs/language/AeoScript_HANDLES.md)         | Cell, Entity, and runtime handle behavior    |
| [AeoScript Standard Library](./docs/language/AeoScript_STDLIB.md) | `math`, `basket`, `string`, and related APIs |
| [AeoScript Lifecycle](./docs/language/AeoScript_LIFECYCLE.md)     | Script lifecycle and runtime execution       |
| [AeoScript Examples](./docs/language/AeoScript_EXAMPLES.md)       | AeoScript usage examples                     |
| [AeoScript VM](./docs/language/AeoScript_VM.md)                   | Script execution and virtual-machine model   |
| [AeoScript Editor](./docs/language/AeoScript_EDITOR.md)           | Integrated Script Editor documentation       |

## Architecture

| Document                                             | Description                                  |
| ---------------------------------------------------- | -------------------------------------------- |
| [Engine Architecture](./docs/architecture/ENGINE.md) | Overall engine structure                     |
| [World](./docs/architecture/WORLD.md)                | World representation and authored scene data |
| [Physics](./docs/architecture/PHYSICS.md)            | Physics runtime architecture                 |
| [Characters](./docs/architecture/CHARACTERS.md)      | Character system architecture                |
| [Rendering](./docs/architecture/RENDERING.md)        | Rendering architecture                       |
| [Runtime](./docs/architecture/RUNTIME.md)            | Runtime systems and Play mode                |
| [Editor](./docs/architecture/EDITOR.md)              | Editor architecture                          |
| [Persistence](./docs/architecture/PERSISTENCE.md)    | Authored data persistence                    |
| [Assets](./docs/architecture/ASSETS.md)              | Project asset architecture                   |

## Development

| Document                                           | Description                       |
| -------------------------------------------------- | --------------------------------- |
| [Building](./docs/development/BUILDING.md)         | Development build information     |
| [Testing](./docs/development/TESTING.md)           | Automated and integration testing |
| [Debugging](./docs/development/DEBUGGING.md)       | Development debugging workflows   |
| [Project Map](./docs/development/MAP.md)           | Source tree and system map        |
| [Contributing](./docs/development/CONTRIBUTING.md) | Contribution information          |

## Changelog

* [CHANGELOG.md](./CHANGELOG.md)

---

# Project Status

> [!NOTE]
> AeoEngine is under active development.

Version `0.7.2` is the current completed release.

Development is focused on using AeoEngine to build actual games, exercising the complete editor-to-runtime workflow, and addressing problems discovered through real gameplay and authoring.

Version `0.7.3` is currently under development.

Current 0.7.3 work includes:

* Expanding AeoScript interaction with gameplay systems
* Script-defined controller behavior
* Script-defined camera behavior
* Direct runtime input access
* Runtime UI
* Lexical closures
* Runtime script control
* Script Editor improvements
* Syntax highlighting
* Voxel rendering optimization

---

## AeoScript Support

AeoScript currently supports:

| Category         | Features                                                                               |
| ---------------- | -------------------------------------------------------------------------------------- |
| Core Values      | Variables and constants; Numbers, strings, booleans, and `nil`                         |
| Functions        | Functions, return values, nested function calls                                        |
| Control Flow     | Conditionals and loops                                                                 |
| Events           | Events and lifecycle functions                                                         |
| Execution        | Top-level executable statements, persistent script fibers, cooperative `wait(seconds)` |
| Closures         | Lexical closures with captured script scope                                            |
| Collections      | Maps and reference-backed baskets                                                      |
| Standard Library | `math`, `basket`, and `string` namespaces                                              |
| Strings          | Unicode-aware string operations                                                        |
| Randomness       | `math.random()` and `math.random(min, max)`                                            |
| World Objects    | Cell and Entity handles, persistent Cell IDs                                           |
| Discovery        | `find()` and `getAllCellsOfClass()`                                                    |
| Cell Data        | Authored Cell attributes and runtime Cell properties                                   |
| Runtime Cells    | Runtime Cell creation, deletion, position changes, and property changes                |
| Runtime State    | Runtime attribute overrides and authored/runtime state separation                      |
| Input            | Movement, jump, and mouse-orbit access                                                 |
| Gameplay         | Script-defined controllers and cameras                                                 |
| Collision Events | `on_touch(cell)`, `on_overlap(overlapping, cell)`, and per-Cell collision event control |
| Script Control   | Script enable/disable                                                                  |
| Runtime UI       | Panels, text, buttons, properties, handles, and callbacks                              |
| Audio Emitters   | `speaker.sound.play()`, `stop()`, `pause()`, `.playing`, `.looped`, `.volume` access    |
| Diagnostics      | Structured runtime diagnostics and terminal output                                     |
| Editor           | Script Editor integration                                                              |

Script bindings target the unique persistent `Cell.id` of an authored World Cell.

The authored `entity_identity` value remains a human-facing name and may be shared by multiple Cells.

---

## Integration Testing

AeoEngine contains an in-game AeoScript integration test suite.

A master `.aeo` test script runs individual tests covering:

* language features
* control flow
* arrays
* maps
* strings
* functions
* closures
* waits
* Cells
* attributes
* Entities
* Lights
* World behavior
* handles
* callbacks
* math

The master suite is intended for in-game integration verification rather than being treated as a normal unit-test replacement.

See [Testing](./docs/development/TESTING.md) for development testing information and [CHANGELOG.md](./CHANGELOG.md) for release verification history.

---

# Performance

> [!NOTE]
> Performance work should be driven by profiling and representative test worlds rather than added preemptively.

Several systems currently have known scalability limits.

<details>
<summary><strong>World Rendering</strong></summary>

The World currently stores authored Cells individually, and active Block collection requires traversing the stored Cells.

Planned rendering work includes:

* Spatially chunked World storage
* Visible-chunk selection
* Per-chunk render meshes
* Hidden-face culling
* Greedy meshing
* Render batching

A possible rendering path:

```text
World
  ↓
Spatial Chunks
  ↓
Visible Chunks
  ↓
Chunk Meshes
  ↓
GPU Rendering
```

Chunk size and meshing strategy will be selected based on profiling and representative test worlds.

</details>

<details>
<summary><strong>Static Physics Synchronization</strong></summary>

Static physics synchronization is change-driven rather than rebuilding the complete static collision set every physics frame.

Physics-relevant runtime changes mark affected Cells as dirty, allowing the physics system to reconcile only the Cells that changed.

This keeps idle static scenes inexpensive while preserving the separation between authored World data and runtime collision state.

</details>

<details>
<summary><strong>Dynamic Physics</strong></summary>

Dynamic collision detection currently performs pairwise body checks.

Possible future approaches include:

* Uniform spatial grids
* Spatial hashing
* Sweep-and-prune
* BVH or another broad-phase structure

The narrow-phase collision system can remain separate from candidate generation so the broad phase can be changed independently.

</details>

<details>
<summary><strong>Continuous Collision Detection</strong></summary>

The current physics implementation uses discrete collision testing.

Possible future approaches include:

* Swept-volume collision testing
* Selective continuous collision detection for high-speed bodies
* Projectile-specific continuous collision handling

Continuous collision detection should be added where tunneling becomes a demonstrated problem.

</details>

<details>
<summary><strong>Runtime Mesh Resources</strong></summary>

Some runtime rendering paths currently create and destroy GPU vertex objects during mesh drawing.

Possible future changes include:

* Persistent VAO/VBO resources
* Reusable mesh resources
* Shared geometry between identical renderables
* Batching or instanced rendering where appropriate

</details>

## Performance Work

Performance changes should be based on profiling and representative test workloads.

More complex acceleration structures should be added when measurements show that the current implementation is a bottleneck.

> [!IMPORTANT]
> Performance work should not change which data is authoritative.

The authored World remains separate from runtime state:

```text
Authored World
      ↓
Render Representation
      ↓
Renderer

Authored World
      ↓
Runtime Conversion
      ↓
PhysicsWorld / Characters / Script Runtime
```

Runtime systems may maintain their own state and derived representations, but authored World data remains authoritative for persistent scene content.

---

# Versioning

AeoEngine uses Semantic Versioning:

```text
MAJOR.MINOR.PATCH
```

Current development is in the `0.7.x` phase.

| Version | Summary                                                                                                                                                                                                      |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `0.6.x` | Established the advanced editor workflow and the first complete AeoScript runtime, including its standard library, handles, persistent Cell-ID bindings, fibers, nested yielding, and lifecycle persistence. |
| `0.7.0` | Expanded AeoScript with authored Cell attributes, top-level executable statements, a dedicated scripting host boundary, and ordered top-level execution.                                                     |
| `0.7.1` | Added runtime Cell creation, deletion, property mutation, runtime attribute overrides, and `math.random()` while preserving the separation between authored World data and temporary Play-mode state.        |
| `0.7.2` | Added the World sky environment, completed scripting lifecycle and nested-return fixes, and established the master in-game AeoScript integration test suite.                                                 |
| `0.7.3` | Current unreleased development cycle. Adds script-defined gameplay systems, runtime UI, direct runtime input, lexical closures, runtime script control, and related runtime/editor improvements.             |

See [CHANGELOG.md](./CHANGELOG.md) for the complete development history.
