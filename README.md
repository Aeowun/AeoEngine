# Worldkiln (formerly AeoEngine)

> **Build. Script. All in one.**
>
> Worldkiln is in active development.

<p align="center">
  <img src="SCREENSHOTS/thumbnail.png" alt="AeoEngine" width="100%">
</p>

Worldkiln is a 3D voxel game engine, editor, and scripting environment written in Rust.

Build worlds, add Actors, write gameplay with AeoScript, test everything in Play mode, and package the project as a standalone game.

**BUILD → SCRIPT → PLAY → RELEASE**

---

## Build the World

Place blocks. Shape spaces. Paint surfaces. Select, move, copy, and organize parts of the World.

Import textures and other project assets directly into the project.

<table>
<tr>
<td width="50%">
<img src="SCREENSHOTS/v%200.8.x%20editor.png" alt="AeoEngine 0.8 editor">
</td>
<td width="50%">
<img src="SCREENSHOTS/v%200.8.x%20block.png" alt="AeoEngine block editing">
</td>
</tr>
<tr>
<td align="center"><strong>THE EDITOR</strong></td>
<td align="center"><strong>VOXELS</strong></td>
</tr>
</table>

---

## Add Actors

Actors are game objects with their own transforms, packages, physics settings, Attributes, and scripts.

They can represent:

- Characters.
- Props.
- Weapons and items.
- Switches.
- Interactive objects.
- Other packaged game objects.

<p align="center">
  <img src="SCREENSHOTS/v%200.8.x%20actor.png" alt="Worldkiln Actor editing" width="100%">
</p>

Actors exist alongside the voxel World without needing to be represented as Blocks.

The Project Explorer keeps Worlds, Actors, scripts, UI, and project content organized as the project grows.

<p align="center">
  <img src="SCREENSHOTS/v%200.8.x%20explorer.png" alt="Worldkiln Project Explorer" width="100%">
</p>

---

## Write Gameplay with AeoScript

AeoScript is the gameplay language built for Worldkiln.

Write and edit `.aeo` scripts directly inside the engine.

<p align="center">
  <img src="SCREENSHOTS/v%200.8.x%20%20script_editor.png" alt="Worldkiln 0.8 AeoScript editor" width="100%">
</p>

AeoScript can control:

- Actors and Entities.
- Input.
- Events.
- UI.
- Audio.
- Animation.
- Attachments.
- Player behavior.
- Cameras.
- Collision events.
- Reusable Modules.
- Game state during Play mode.

```aeoscript
debug.log("Game script loaded")

gold: number = 0

on on_gold(amount) {
    gold += amount
}
```

Scripts can be attached to game objects or used as standalone systems that coordinate the game.

---

## Test in Play Mode

Run the project directly inside Worldkiln.

Play mode brings together the World, Actors, physics, scripts, UI, audio, animation, cameras, and input.

Worldkiln keeps script output visible inside the editor while the game runs.

<p align="center">
  <img src="SCREENSHOTS/v%200.8.x%20output.png" alt="Worldkiln Output dock" width="100%">
</p>

Changes made by ordinary gameplay scripts during Play mode do not overwrite the saved project.

---

## Build the Game

Worldkiln can package a project as a standalone game that runs independently from the editor.

```text
Create project
      ↓
Build World
      ↓
Add Actors
      ↓
Write gameplay
      ↓
Play
      ↓
Build game
```

Completed builds can also be found and launched from the Worldkiln Home screen.

---

# It Did Not Start Here

Worldkiln began much smaller.

### The First World

<p align="center">
  <img src="SCREENSHOTS/first_world.png" alt="Early AeoEngine world" width="80%">
</p>

### Early AeoScript

<p align="center">
  <img src="SCREENSHOTS/first_aeoscript.png" alt="Early AeoScript editor" width="100%">
</p>

### AeoEngine 0.7.x

<p align="center">
  <img src="SCREENSHOTS/v%200.7.x_menu.png" alt="AeoEngine 0.7.x editor" width="80%">
</p>

The direction has expanded from placing Blocks to building complete game projects.

**BUILD → SCRIPT → PLAY → RELEASE**

---

## Try Worldkiln

Worldkiln currently targets Windows 10 and 11.

**Public releases are beta builds.**

They are available for testing and experimentation and may contain bugs, unfinished features, compatibility issues, or breaking changes.

[**Download Worldkiln →**](https://aeowun.com/downloads/)

A release can also be downloaded directly from the repository's **Releases** tab.

---

## Support Worldkiln

Worldkiln is independently developed.

If you want to support continued development:

[Support Worldkiln on Ko-fi](https://ko-fi.com/aeowun/tip)

[Documentation](https://aeowun.com/docs/) · [Getting Started](https://aeowun.com/aeoengine/getting-started/) · [AeoScript](https://aeowun.com/aeoscript/)

---

<sub>Worldkiln is under active development. Features, APIs, editor behavior, project formats, and gameplay behavior may change between beta releases.</sub>
