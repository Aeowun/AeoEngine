# AeoEngine

**Build a voxel world, make it playable, and turn it into a standalone game.**

AeoEngine is a Rust/OpenGL game engine and editor for building voxel worlds, creating gameplay with Actors and AeoScript, testing projects in Play mode, and packaging them as standalone games.

[Visit AeoEngine](https://aeowun.com/aeoengine/) · [Documentation](https://aeowun.com/docs/) · [Getting Started](https://aeowun.com/aeoengine/getting-started/)

## Features

- **Voxel world editing** — build, erase, paint, select, copy, paste, and organize world geometry.
- **Actors** — add packaged characters and objects with transforms, physics, attributes, and scripts.
- **AeoScript** — create gameplay logic, input handling, events, UI, audio, animation, attachments, and runtime behavior.
- **Play mode** — test projects directly inside the editor without modifying the authored world.
- **Project assets** — use project-owned textures, audio, character packages, cameras, and controllers.
- **Standalone builds** — package projects for the separate `aeogame` runtime.

## Install

AeoEngine currently targets Windows.

Download and launch `AeoEngine.exe`. The installer creates the engine installation under:

```text
%LOCALAPPDATA%\Aeowun\
├── .assets\
├── UserData\
├── AeoEngine.exe
└── LICENSE
```

Projects are stored under `UserData/` by default.

## Build from source

A stable Rust toolchain is required.

```powershell
git clone https://github.com/Aeowun/AeoEngine.git
cd AeoEngine
cargo run
```

Run the test suite with:

```powershell
cargo test
```

Build a release version of the editor with:

```powershell
cargo build --release
```

Build the standalone game runtime with:

```powershell
cargo build --release --no-default-features --features runtime --bin aeogame
```

## Project structure

A typical project contains:

```text
MyProject/
├── world.dat
├── scripts/
├── playerscripts/
└── .assets/
```

`scripts/` contains reusable and standalone AeoScript Modules.

`playerscripts/` contains Modules attached to the runtime Player Actor.

`.assets/` contains project-owned resources such as textures, audio, and packages.

## AeoScript

AeoScript is AeoEngine's gameplay scripting language.

It supports Actor and entity interaction, input, events, runtime UI, audio, animation, attachments, Modules, and engine handles.

Documentation:

- [AeoScript overview](docs/language/AeoScript.md)
- [Modules](docs/language/AeoScript_Modules.md)
- [API reference](docs/language/AeoScript_API.md)
- [Types](docs/language/AeoScript_TYPES.md)
- [Handles](docs/language/AeoScript_HANDLES.md)
- [Lifecycle](docs/language/AeoScript_LIFECYCLE.md)
- [Examples](docs/language/AeoScript_EXAMPLES.md)
- [Grammar](docs/language/AeoScript_GRAMMAR.md)
- [Standard library](docs/language/AeoScript_STDLIB.md)
- [Virtual machine](docs/language/AeoScript_VM.md)

## Workflow

1. Create or open a project.
2. Build the world and add Actors.
3. Attach AeoScript Modules to Actors or create standalone scripts.
4. Test the project in Play mode.
5. Fix problems, refine the project, and build the standalone game.

## Development status

AeoEngine is under active development.

Current work includes the editor, authored Actors, AeoScript, physics, character packages, standalone game builds, UI authoring, audio, lighting, and renderer development.

For current behavior and usage, see the [documentation hub](https://aeowun.com/docs/) and the source documentation in this repository.
