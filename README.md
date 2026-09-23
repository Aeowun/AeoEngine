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
* [Documentation](#documentation)
    * [Getting Started and Guides](#getting-started-and-guides)
    * [AeoScript](#aeoscript)
    * [Architecture](#architecture)
    * [Development](#development)
    * [Changelog](#changelog)
* [Project Status](#project-status)
    * [AeoScript](#aeoscript-1)
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
| Scripting         | AeoScript gameplay scripting, top-level execution, lifecycle events, persistent script state  |
| Script Runtime    | Cooperative fibers, `wait()`, nested function calls, closures, runtime diagnostics            |
| Script Objects    | Cell and Entity handles, persistent Cell IDs, object discovery                                |
| Runtime World     | Authored Cell attributes, runtime property overrides, runtime Cell creation and deletion      |
| Runtime Input     | Movement input, jump input, mouse orbit input                                                 |
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

# Documentation

The `docs/` directory contains the engine, architecture, development, gameplay, and AeoScript documentation.

## Getting Started and Guides

| Document                                                  | Description                                      |
| --------------------------------------------------------- | ------------------------------------------------ |
| [Getting Started](./docs/guides/GETTING_STARTED.md)       | Initial engine setup and basic workflow          |
| [World Building](./docs/guides/WORLD_BUILDING.md)         | Creating and editing authored voxel worlds       |
| [Building a Game](./docs/guides/BUILDING_A_GAME.md)       | Building a complete game workflow with AeoEngine |
| [Physics Gameplay](./docs/guides/PHYSICS_GAMEPLAY.md)     | Using runtime physics in gameplay                |
| [Character Gameplay](./docs/guides/CHARACTER_GAMEPLAY.md) | Character movement and gameplay behavior         |
| [Scripting](./docs/guides/SCRIPTING.md)                   | Using AeoScript in gameplay                      |
| [Textures](./docs/guides/TEXTURES.md)                     | Project-owned texture workflows                  |
| [Debugging](./docs/guides/DEBUGGING.md)                   | Debugging engine and gameplay issues             |

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

## AeoScript

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
| Collision Events | `on_touch(cell)` and per-Cell collision event control                                  |
| Script Control   | Script enable/disable                                                                  |
| Runtime UI       | Panels, text, buttons, properties, handles, and callbacks                              |
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
