# AeoEngine

**Build a voxel world, make it playable, and turn it into a standalone game.**

AeoEngine is a Rust/OpenGL game engine and editor for building voxel worlds, creating gameplay with Actors and AeoScript, testing projects in Play mode, and packaging them as standalone games.

[Download AeoEngine](https://aeowun.com/downloads/) · [AeoEngine](https://aeowun.com/aeoengine/) · [Documentation](https://aeowun.com/docs/) · [Getting Started](https://aeowun.com/aeoengine/getting-started/)

## Features

- **Voxel world editing** — build, erase, paint, select, copy, paste, and organize worlds.
- **Actors** — add packaged characters and objects with transforms, physics, attributes, and scripts.
- **AeoScript** — write gameplay logic for input, events, UI, audio, animation, attachments, and runtime behavior.
- **Play mode** — test projects directly inside the editor.
- **Project assets** — use project-owned textures, audio, character packages, cameras, and controllers.
- **Standalone builds** — package projects for the separate `aeogame` runtime.

## Install

AeoEngine currently targets Windows 10 and 11.

Download the current Windows build from the [AeoEngine download page](https://aeowun.com/downloads/) and launch `AeoEngine.exe`.

On first launch, AeoEngine installs its required files under:

```text
%LOCALAPPDATA%\Aeowun\
├── .assets\
├── UserData\
├── AeoEngine.exe
└── LICENSE
```

Projects are stored under `UserData/` by default.

The current public download is **AeoEngine 0.7.5 Beta**. The **0.8.x** development series is currently in development and is not yet offered as the public Windows download.

## Project structure

A typical project contains:

```text
MyProject/
├── world.dat
├── scripts/
├── playerscripts/
└── .assets/
```

- `scripts/` contains reusable and standalone AeoScript Modules.
- `playerscripts/` contains Modules attached to the runtime Player Actor.
- `.assets/` contains project-owned textures, audio, packages, and other resources.

## AeoScript

AeoScript is AeoEngine's gameplay scripting language.

It supports Actor and entity interaction, input, events, runtime UI, audio, animation, attachments, Modules, and engine handles.

- [AeoScript documentation](https://aeowun.com/aeoscript/)
- [Documentation hub](https://aeowun.com/docs/)
- [Developer reference](https://aeowun.com/docs/reference/)

## Workflow

1. Create or open a project.
2. Build the world and add Actors.
3. Attach AeoScript Modules or create standalone scripts.
4. Test the project in Play mode.
5. Refine the project and build the standalone game.

## Development status

AeoEngine is under active development. Features, APIs, editor behavior, and file formats may continue to change.

For current usage and behavior, use the [documentation hub](https://aeowun.com/docs/).

## License

AeoEngine is **source-available software** and is not released under an open-source license.

See [LICENSE](LICENSE) for the complete terms.
