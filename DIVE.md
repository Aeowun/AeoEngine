# AeoEngine

> ## COMING SOON
>
> **AeoEngine is in active beta development.**
>
> Public releases are provided for testing and experimentation. All releases should be treated as **beta software** and may contain bugs, incomplete features, breaking changes, or platform-specific issues. A release is not a promise that every feature or project will work correctly.


[Support AeoEngine on Ko-fi](https://ko-fi.com/aeowun/tip)


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

Download the current Windows build from the AeoEngine download page, or download a release directly from the repository's Releases tab, then launch AeoEngine.exe.
On first launch, AeoEngine installs its required files under:

```text
%LOCALAPPDATA%\Aeowun\
├── .assets\
├── UserData\
├── AeoEngine.exe
└── LICENSE
```

Projects are stored under `UserData/` by default.

### Beta software

Every AeoEngine release is currently a **beta release**.

Beta builds are made available to test the engine as it develops. They may contain known or unknown bugs, unfinished systems, compatibility problems, performance issues, or changes that affect existing projects.

No beta release is guaranteed to work correctly on every system or with every project. Projects that matter should be backed up before being opened in a newer build.

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

AeoEngine is under active development.

Features, APIs, editor behavior, project formats, and runtime behavior may change between releases. Compatibility between beta versions is not guaranteed.

For current usage and behavior, use the [documentation hub](https://aeowun.com/docs/).

## License

AeoEngine is **source-available software** and is not released under an open-source license.

See [LICENSE](LICENSE) for the complete terms.
