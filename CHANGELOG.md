# Worldkiln Changelog

Development history for Worldkiln (formerly AeoEngine) and AeoScript.

> Worldkiln is the current product name. Historical release entries retain AeoEngine where that was the product name at the time.

## Contents

- [0.1.x — Foundation](#01x--foundation)
- [0.2.x — Editor Expansion](#02x--editor-expansion)
- [0.3.x — Physics](#03x--physics)
- [0.4.x — Blocks and Building](#04x--blocks-and-building)
- [0.5.x — Character and Gameplay](#05x--character-and-gameplay)
- [0.6.x — Editor and AeoScript Foundation](#06x--editor-and-aeoscript-foundation)
- [0.7.x — Gameplay and Scripting Expansion](#07x--gameplay-and-scripting-expansion)
- [0.8.0 — UI, Actors, Attachments, and Installation](#080--ui-actors-attachments-and-installation)

---

# 0.1.x — Foundation

## 0.1.0

### Added

- Initial voxel World.
- Basic World editing.
- Block building and erasing.
- Editor camera and navigation controls.

---

## 0.1.1

### Added

- Project and World saving.
- Initial Home screen and project management.

---

# 0.2.x — Editor Expansion

## 0.2.0

### Added

- Cell selection and Properties editing.
- Docked editor panels.
- Multiple building grid planes.
- Expanded editor camera and viewport controls.

---

## 0.2.1

### Added

- Light cells.
- Persistent World lighting.

---

# 0.3.x — Physics

## 0.3.0

### Added

- Play mode physics.
- Configurable World gravity.
- Dynamic physics bodies.
- Collision between moving bodies and solid World geometry.
- Sleeping and waking for supported physics bodies.
- Separation between editing and Play mode.

### Fixed

- Improved resting stability.
- Improved collision response.
- Fixed bodies being continuously pushed into supporting surfaces.
- Fixed Light cells being treated as physics geometry.

---

## 0.3.1

### Fixed

- Improved stacked-body stability.
- Improved sleeping and waking behavior.
- Fixed bodies failing to wake when support was removed.

---

# 0.4.x — Blocks and Building

## 0.4.0

### Added

- `Block` as the primary building cell.
- Per-Block color.
- Configurable Build properties.
- RGB editing and color picker.

---

## 0.4.1

### Added

- Drag building and erasing.
- Line building.
- Plane building.
- Build and erase previews.

---

## 0.4.2

### Fixed

- Fixed Block colors being lost during Play mode physics.
- Fixed Block visibility being lost when physics took control.
- Fixed Color Picker interaction issues.

---

# 0.5.x — Character and Gameplay

## 0.5.0

### Added

- Character system.
- Character movement and jumping.
- Gravity and grounded state.
- World collision.
- Spawn Points with clearance checking.

---

## 0.5.1

### Added

- 3D character rig and skeleton.
- Idle and Walk animations.
- Animation blending.
- Character appearance customization.
- Character facing based on movement direction.

---

## 0.5.2

### Added

- Third-person gameplay camera.
- Camera orbit and pitch controls.
- Camera collision with World geometry.
- Character-follow camera behavior.

---

## 0.5.3

### Fixed

- Editor camera state is preserved when entering and leaving Play mode.
- Editor camera controls no longer interfere with gameplay.
- Improved gameplay-camera obstruction handling.

---

# 0.6.x — Editor and AeoScript Foundation

## 0.6.0

### Added

- Expanded World hierarchy.
- Multi-cell selection.
- Plane and Top-Block selection modes.
- Surface-aware building.
- Editor shortcuts.
- Undo and Redo for World editing.
- Selection focus.
- Project texture importing.
- Project-owned texture assets.
- Texture loading and caching.

---

## 0.6.1

### Added

- First AeoScript language implementation.
- Variables, constants, numbers, strings, booleans, and `nil`.
- Arithmetic and logic.
- `if`, `while`, and `for`.
- Functions and event handlers.
- Cooperative `wait()` support.
- Integrated Script Editor.
- Output terminal with warnings and errors.

---

## 0.6.2

### Added

- Script access to Cells and Entities.
- Stable Cell IDs.
- `find()` and Cell discovery APIs.
- Script-controlled Cell properties during Play mode.
- Script bindings for World objects.

### Changed

- Scripts now reference Cells by stable IDs instead of coordinates.
- Changes made during Play mode no longer modify the saved World.

---

## 0.6.3

### Added

- AeoScript `math` library.
- AeoScript `basket` collection library.
- AeoScript `string` library.
- Unicode-aware string operations.

---

## 0.6.4

### Added

- Script bindings now target unique Cell IDs.
- Multiple Cells can share the same name without breaking script bindings.

### Fixed

- Fixed incorrect script-binding warnings on non-Entity Cells.

---

## 0.6.5

### Added

- Nested AeoScript function calls can now use `wait()`.
- Improved function call-stack handling.
- Improved Map behavior.

---

## 0.6.6

### Fixed

- Fixed script fields losing changes after a function yielded or completed.
- Script state now persists correctly between lifecycle callbacks and updates.

---

# 0.7.x — Gameplay and Scripting Expansion

## 0.7.0

### Added

- Custom Cell Attributes using Number, Bool, and String values.
- Script access to Cell Attributes.
- Top-level AeoScript statements.
- Top-level scripts can use `wait()`.
- Improved script error reporting.

---

## 0.7.1

### Added

- Scripts can change Cell Attributes during Play mode.
- `math.random()`.
- `cell.new()` for creating temporary Cells during gameplay.
- `cell.delete()`.
- Script control over temporary Cell position, name, visibility, solidity, anchoring, and color.

### Changed

- Play-mode Cell changes remain temporary and reset when Play mode ends.

---

## 0.7.2

### Added

- Sky environment system.
- OpenGL cubemap skyboxes.
- Temperate, Tropical, Desert, Snowy, and Mars sky presets.
- World controls for enabling and selecting skies.

### Fixed

- Fixed script state leaking across Play → Stop → Play.
- Fixed nested function return values reaching the wrong call site.
- Fixed release builds resolving `UserData` and `.assets` from the wrong location.

---

## 0.7.3

### Added

- Script-defined objects with fields, constructors, and methods.
- Lexical closures.
- Script access to keyboard and mouse input.
- Script-defined player controllers.
- Script-defined gameplay cameras.
- Script enable and disable controls.
- `on_touch` and `on_overlap` collision events.
- Runtime UI with Panels, Text, and Buttons.
- Script-created UI controls and button callbacks.
- Mouse cursor visibility and screen-lock controls.
- Audio Emitters.
- WAV audio assets.
- 3D audio playback.
- Script control over sound playback, looping, and volume.
- Improved AeoScript syntax highlighting.

### Changed

- Player movement and camera behavior can be controlled through AeoScript while AeoEngine continues handling physics and collision.
- UI is positioned relative to the playable viewport.
- Controller, camera, UI, and audio state reset when Play mode ends.

### Fixed

- Fixed editor projection and picking offsets caused by side panels.
- Fixed gameplay UI rendering over editor panels.
- Fixed script state not resetting correctly after Play mode.
- Fixed controller and camera state being lost between updates.

---

## 0.7.4

### Added

- Exposed-face voxel meshing.
- Chunked static World meshes.
- Dirty-chunk rebuilding.
- Greedy voxel meshing.
- Copy and Paste.
- Multi-selection drag movement.
- Additive selection with `Ctrl`.
- Drag previews with collision indicators.
- Generic scripted character controls.
- Scripted character spawning.
- Entity-based touch callbacks.
- Improved World hierarchy navigation for Lights, Audio Emitters, Spawn Points, FX Blocks, Players, and NPCs.
- Large Block collections are shown as a count instead of thousands of hierarchy entries.

### Changed

- Static World rendering now uses chunked meshes and selective rebuilding.
- Copying a multi-selection uses its lowest cells as the placement pivot.
- Undo and Redo now preserve script bindings with copied and moved World content.
- Character movement decisions can be scripted while AeoEngine continues handling physics, gravity, and collision.

### Fixed

- Fixed shadows becoming detached from Blocks as the camera moved.
- Fixed scripted Entity character controls.
- Fixed gameplay event dispatch.
- Fixed major World