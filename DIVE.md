# Worldkiln

> ## ACTIVE BETA
>
> **Worldkiln (formerly AeoEngine) is in active beta development.**
>
> Public releases are available for testing and experimentation. Features, APIs, project formats, and behavior may change between releases.

[Support Worldkiln on Ko-fi](https://ko-fi.com/aeowun/tip)

**Build a voxel world, make it playable, and turn it into a standalone game.**

Worldkiln is a Rust/OpenGL game engine and editor for building voxel worlds, creating gameplay with Actors and AeoScript, testing projects in Play mode, and building standalone games.

[Download Worldkiln](https://aeowun.com/downloads/) · [Worldkiln](https://aeowun.com/worldkiln/) · [Documentation](https://aeowun.com/docs/) · [Getting Started](https://aeowun.com/worldkiln/getting-started/)

---

## Features

- **Voxel world editing** — build, erase, paint, select, move, copy, paste, and organize worlds.
- **Actors** — add characters, props, interactive objects, and other packaged game objects with transforms, physics, Attributes, and scripts.
- **AeoScript** — create gameplay with input, events, UI, audio, animation, attachments, Modules, and game objects.
- **UI authoring** — build interface layouts with Panels, Text, Buttons, anchors, resizing, script keys, and responsive positioning.
- **Physics** — gravity, collision, moving objects, Characters, contact events, and Play-mode simulation.
- **Animation** — character animation, layered actions, and attachment points.
- **Audio** — import WAV files, place Audio Emitters, and control sounds through AeoScript.
- **Play mode** — run and test the game directly inside the editor.
- **Project assets** — manage textures, audio, character packages, scripts, UI, and other project files.
- **Standalone games** — package projects into games that can run separately from the editor.

---

## Install

Worldkiln currently supports Windows 10 and 11.

Download `Worldkiln.exe` from the [Worldkiln download page](https://aeowun.com/downloads/) or from the repository's Releases page.

On first launch, Worldkiln installs itself under:

```text
%LOCALAPPDATA%\Aeowun\
├── .assets\
├── UserData\
├── Worldkiln.exe
└── LICENSE
```

Projects are stored under `UserData/` by default.

Worldkiln can repair missing installation files without replacing existing projects or assets.

---

## Project Structure

A typical project contains:

```text
MyProject/
├── world.dat
├── scripts/
├── playerscripts/
└── .assets/
```

- `world.dat` stores the project World and its objects.
- `scripts/` contains standalone and reusable AeoScript Modules.
- `playerscripts/` contains scripts attached to the Player.
- `.assets/` contains textures, audio, packages, and other project assets.

---

## Actors

Actors are game objects that exist alongside the voxel World.

They can represent:

- Characters.
- Props.
- Weapons and items.
- Switches.
- Interactive objects.
- Other packaged game objects.

Actors can have their own transform, physics settings, Attributes, package, and AeoScript Modules.

During Play mode, Actors become active game objects that scripts, physics, animation, and other systems can interact with.

---

## AeoScript

AeoScript is the gameplay scripting language built for Worldkiln.

```aeoscript
debug.log("Game loaded")

gold: number = 0

on on_gold(amount) {
    gold += amount
}
```

AeoScript supports:

- Variables and functions.
- Events.
- Closures.
- `wait()`.
- Reusable Modules.
- Input.
- Actors and Entities.
- UI.
- Audio.
- Animation.
- Attachments.
- Collision events.
- Player and camera control.
- Creating and removing game objects during Play mode.

Scripts can run as standalone gameplay systems or be attached to Actors and other supported objects.

[AeoScript Documentation](https://aeowun.com/aeoscript/) · [Developer Reference](https://aeowun.com/docs/reference/)

---

## UI

Worldkiln includes UI authoring for game interfaces.

UI controls can use responsive anchors and can be accessed from AeoScript using script keys.

```aeoscript
const health_text = ui.find("health_text")
```

Scripts can also create Panels, Text, and Buttons during Play mode.

---

## Play Mode

Play mode runs the current project directly inside Worldkiln.

Gameplay can use:

- Character movement.
- Physics.
- Scripts.
- UI.
- Audio.
- Animation.
- Attachments.
- Collision events.
- Cameras.
- Input.

Changes made by ordinary gameplay scripts during Play mode are temporary and do not overwrite the saved project.

---

## Build a Game

Worldkiln can package a project as a standalone Windows game.

```text
Create project
      ↓
Build world
      ↓
Add Actors
      ↓
Write gameplay
      ↓
Play
      ↓
Build game
```

Completed builds are stored under:

```text
UserData/Builds/
```

Built games can also be discovered and launched from the Worldkiln Home screen.

---

## Workflow

1. Create or open a project.
2. Build the World.
3. Add Actors and assets.
4. Create UI where needed.
5. Write gameplay with AeoScript.
6. Test the game in Play mode.
7. Build the standalone game.

---

## Beta Software

Worldkiln is still under active development.

Beta releases may contain bugs, unfinished features, compatibility issues, or breaking changes.

Back up important projects before opening them in a newer release.

For current behavior and documentation, see the [documentation hub](https://aeowun.com/docs/).

---

## License

Worldkiln is **source-available software** and is not released under an open-source license.

See [LICENSE](LICENSE) for the complete terms.