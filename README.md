# AeoEngine

A voxel engine and editor for building authored worlds and running them with physics, lighting, characters, and scripts.

## What is it?

AeoEngine provides a grid-based editor for creating worlds and a runtime for simulating them.

World data saved in the project is kept separate from changes made while the game is running. Changes made during Play mode, such as moving a physics body or toggling a light from a script, are temporary and are discarded when the game stops.

## Current Features

* Voxel world editing
* Authored Block and Light cells
* World hierarchy and multi-selection
* Plane and Top-Block building modes
* Build, Erase, Undo, and Redo
* Project-owned texture assets
* Runtime physics simulation
* Dynamic PhysicsBodies
* Character spawning, movement, collision, and animation
* Third-person gameplay camera
* Directional and point lighting
* AeoScript gameplay scripting
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

The current development work is focused on AeoScript V1, physics, character interaction, asset handling, and editor behavior.

AeoScript currently supports:

* Variables and constants
* Numbers, strings, booleans, and `nil`
* Functions and returns
* Conditionals and loops
* Events
* Persistent script fibers
* `wait()`
* Cell and Entity handles
* Object discovery
* Runtime Cell property changes
* Structured runtime diagnostics
* Script Editor integration

See [CHANGELOG.md](CHANGELOG.md) for the development history and current work.

## Performance

Several systems currently have known scalability limits.

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
PhysicsWorld / Characters
```

Performance changes should not change which data is authoritative.

## Versioning

AeoEngine uses Semantic Versioning:

`MAJOR.MINOR.PATCH`

## Development Time

Estimated active development time represented by the current sprint:

**~50 hours 52 minutes**

*Not sponsored by Red Bull.*
