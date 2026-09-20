# AeoEngine

A voxel engine and editor for building authored worlds and running them with physics, lighting, characters, and gameplay scripts.

## What is it?

AeoEngine provides a grid-based editor for creating authored worlds and a runtime for simulating and playing them.

Authored World data is kept separate from runtime state. Changes made during Play mode—such as moving a physics body, changing a Cell property through AeoScript, or modifying runtime character state—are temporary and are discarded when Play mode stops.

The authored World remains the source of truth for scene data.

## Current Features

* Voxel World editing
* Authored Block and Light cells
* World hierarchy and multi-selection
* Plane and Top-Block building modes
* Build, Erase, Undo, and Redo
* Project-owned texture assets
* Runtime physics simulation
* Dynamic PhysicsBodies
* Change-driven static physics synchronization
* Character spawning, movement, collision, and animation
* Third-person gameplay camera
* Directional and point lighting
* AeoScript gameplay scripting
* Script bindings using persistent Cell IDs
* Cell and Entity handles
* Object discovery through `find()` and `getAllCellsOfClass()`
* Runtime Cell property overrides
* AeoScript standard library
* Persistent script lifecycle state
* Cooperative script fibers and `wait()`
* Runtime script diagnostics and terminal output

## Getting Started

### Requirements

* Windows 10 or 11
* Rust stable toolchain

### Build and Run

Open a terminal in the project root:

```text
cargo run
```

## Documentation

The `docs/` directory contains the engine and AeoScript documentation.

### AeoScript

* [AeoScript Overview](docs/language/AeoScript.md)
* [AeoScript API](docs/language/AeoScript_API.md)
* [AeoScript Grammar](docs/language/AeoScript_GRAMMAR.md)
* [AeoScript VM](docs/language/AeoScript_VM.md)
* [AeoScript Editor](docs/language/AeoScript_EDITOR.md)

### Project Documentation

* [Changelog](CHANGELOG.md)

## Project Status

AeoEngine is under active development.

The current development phase is focused on using the engine to build actual games, exercising the complete editor-to-runtime workflow, and addressing problems discovered through real gameplay and authoring.

AeoScript currently supports:

* Variables and constants
* Numbers, strings, booleans, and `nil`
* Functions and return values
* Conditionals and loops
* Events and lifecycle functions
* Nested function calls
* Persistent script fibers
* Cooperative `wait(seconds)`
* Maps and reference-backed baskets
* Math, basket, and string standard-library namespaces
* Unicode-aware string operations
* Cell and Entity handles
* Persistent Cell IDs
* Name-based object discovery through `find()`
* Class-based object discovery through `getAllCellsOfClass()`
* Runtime Cell property changes
* Separation between authored state and runtime overrides
* Persistent lifecycle state between script updates
* Structured runtime diagnostics
* Script Editor integration

Script bindings target the unique persistent `Cell.id` of an authored World Cell. The authored `entity_identity` value remains a human-facing name and may be shared by multiple Cells.

See [CHANGELOG.md](CHANGELOG.md) for the development history and current work.

## Performance

Several systems currently have known scalability limits.

Performance work should be driven by profiling and representative test worlds rather than added preemptively.

### World Rendering

The World currently stores authored cells individually, and active block collection requires traversing the stored cells.

Possible future changes include:

* Spatially chunked World storage
* Visible-chunk selection
* Per-chunk render meshes
* Face culling
* Greedy meshing
* Render batching

A possible rendering path:

```text
World
  ↓
Spatial chunks
  ↓
Visible chunks
  ↓
Chunk meshes
  ↓
GPU rendering
```

Chunk size and meshing strategy will be selected based on profiling and test worlds.

### Static Physics Synchronization

Static physics synchronization is change-driven rather than rebuilding the complete static collision set every physics frame.

Physics-relevant runtime changes mark affected Cells as dirty, allowing the physics system to reconcile only the Cells that changed.

This keeps idle static scenes inexpensive while preserving the separation between authored World data and runtime collision state.

### Dynamic Physics

Dynamic collision detection currently performs pairwise body checks.

Possible future approaches include:

* Uniform spatial grids
* Spatial hashing
* Sweep-and-prune
* BVH or another broad-phase structure

The narrow-phase collision system can remain separate from candidate generation so the broad phase can be changed independently.

### Continuous Collision Detection

The current physics implementation uses discrete collision testing.

Possible future approaches include:

* Swept-volume collision testing
* Selective continuous collision detection for high-speed bodies
* Projectile-specific continuous collision handling

Continuous collision detection should be added where tunneling becomes a demonstrated problem.

### Runtime Mesh Resources

Some runtime rendering paths currently create and destroy GPU vertex objects during mesh drawing.

Possible future changes include:

* Persistent VAO/VBO resources
* Reusable mesh resources
* Shared geometry between identical renderables
* Batching or instanced rendering where appropriate

### Performance Work

Performance changes should be based on profiling and test workloads.

More complex acceleration structures should be added when measurements show that the current implementation is a bottleneck.

Performance work should not change which data is authoritative.

The authored World remains separate from runtime state:

```text
Authored World
      ↓
Render representation
      ↓
Renderer

Authored World
      ↓
Runtime conversion
      ↓
PhysicsWorld / Characters / Script Runtime
```

Runtime systems may maintain their own state and derived representations, but authored World data remains authoritative for persistent scene content.

## Versioning

AeoEngine uses Semantic Versioning:

`MAJOR.MINOR.PATCH`

Current development is in the `0.7.x` phase.

Version `0.6.x` established the advanced editor workflow and the first complete AeoScript runtime, including its standard library, handles, persistent Cell-ID bindings, fibers, nested yielding, and lifecycle persistence.

Version `0.7.x` begins the next development phase: building actual games with AeoEngine and using those games to expose and fix the first real workflow, gameplay, and engine-integration problems.

## Development Time

Estimated active development time represented by the current sprint:

**~50 hours 52 minutes**

*Not sponsored by Red Bull.*
