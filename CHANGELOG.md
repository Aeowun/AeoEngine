# AeoEngine Changelog

## 0.1.0

### Added

* Initial AeoEngine project structure.
* Initial voxel World representation.
* Grid based world editing.
* Basic editor navigation and camera controls.
* Home screen and project management foundation.
* Basic block creation and erasing.
* World persistence foundation.

### Notes

Version `0.1.0` established the basic engine, editor, World, camera, and project structure.

---

## 0.2.0

### Added

* Expanded editor tools and interaction.
* Cell selection and Properties editing.
* Docked World and Properties panels.
* Multiple editor grid planes.
* Editor camera navigation and viewport interaction.
* Authored Light cells.
* World lighting data and persistence foundation.

### Changed

* Editor UI was expanded into a docked workspace.
* World remained authoritative for authored scene data.

### Notes

Version `0.2.0` established the main world editing workflow and the foundation for authored scene properties.

---

## 0.3.0

### Added

* Play mode runtime physics.
* Fixed timestep physics simulation at 60 Hz.
* Configurable world gravity using a full `Vec3`.
* Dynamic PhysicsBodies for non anchored cells.
* Static collision detection against anchored solid cells.
* Dynamic body collision detection and penetration resolution.
* Support tracking between dynamic bodies.
* Sleeping and waking behavior for supported dynamic bodies.
* Runtime PhysicsBody rendering during Play mode.
* Editor and Play mode separation.
* World owned gravity settings.

### Changed

* Physics runtime state is separate from authored World state.
* Anchored cells remain authoritative World geometry.
* Dynamic body positions are stored separately from authored World coordinates.
* Renderer draws runtime PhysicsBodies separately from authored World geometry.

### Fixed

* Improved resting contact stability.
* Prevented gravity from continuously pushing supported bodies into surfaces.
* Preserved tangential velocity during collision response.
* Added wake behavior when physical support is lost.
* Prevented Light cells from becoming physics geometry.

### Notes

Version `0.3.0` established the runtime physics foundation and the separation between authored World data and runtime simulation state.

---

## 0.3.1

### Fixed

* Improved dynamic support detection.
* Improved sleeping and waking behavior for stacked bodies.
* Fixed support loss wake behavior.
* Added regression coverage for physics support changes.
* Preserved existing physics behavior while correcting stability issues.

### Notes

Version `0.3.1` contains physics stability fixes without changing the overall runtime physics model.

---

## 0.4.0

### Added

* Renamed the generic `Grass` cell type to `Block`.
* Added per Block `ColorRGB`.
* Added configurable Build properties.
* Added numeric RGB editing.
* Added clipboard RGB copying.
* Added persistent Color Picker editing.
* Added drag based Build behavior.
* Added drag based Erase behavior.
* Added line building and erasing.
* Added plane building and erasing.
* Added ghost previews for drag Build and Erase operations.
* Added runtime PhysicsBody property snapshots.

### Changed

* Build operations now use configured Block properties.
* PhysicsBodies retain their required authored properties after entering Play mode.
* Runtime dynamic rendering reads properties directly from PhysicsBodies.
* World is no longer queried to recover properties from moving runtime bodies.
* World persistence now stores Block color.
* Legacy `GRASS` records load as `Block`.

### Fixed

* Fixed runtime Block colors being lost during physics simulation.
* Fixed runtime visibility being lost when Blocks become PhysicsBodies.
* Fixed Color Picker popup lifetime and interaction behavior.

### Notes

Version `0.4.0` expands the editor from basic block placement into configurable world authoring while strengthening the World → runtime property boundary.

---

## 0.5.0

### Added

* Character runtime system with dedicated character state.
* Character transform, movement, collision, and animation state.
* Character spawning from authored SpawnPoint cells.
* SpawnPoint validation and nearby clearance search.
* Fixed timestep character movement.
* Character gravity and grounded state.
* Character jumping with grounded jump gating.
* Character voxel collision against solid World cells.
* Character floor, wall, and ceiling collision handling.
* Character movement state switching between Idle and Walk.
* Character orientation based on movement direction.
* Isolated 3D character rig and skeleton implementation.
* Idle and Walk character animation clips.
* Character animation blending and evaluated poses.
* Character appearance customization data.
* Modular generated character geometry.
* Runtime integration of the custom character implementation.
* Dynamic character mesh rendering.
* Dedicated third person GameplayCamera.
* Gameplay camera follow behavior.
* Gameplay camera mouse orbit.
* Gameplay camera pitch limits.
* Gameplay camera collision against solid World geometry.
* Gameplay camera follow target derived from the runtime character.
* Editor camera preservation across Play mode transitions.

### Changed

* Character gameplay state is owned by `CharacterSystem`.
* Character visual and animation implementation is provided by the isolated character system.
* Runtime animation updates use the existing fixed timestep.
* Play mode now renders the integrated 3D character instead of the temporary Magenta cube.
* Play mode uses a dedicated GameplayCamera instead of the Editor camera.
* Editor and gameplay camera state are explicitly separated.
* Character orientation follows actual movement direction and remains independent of camera orientation.
* Character collision dimensions remain independent from visual character scaling.

### Fixed

* Prevented editor camera state from being lost across Play mode transitions.
* Prevented editor camera controls from continuing to mutate the editor camera during Play.
* Added gameplay camera obstruction handling against solid voxel geometry.
* Added regression coverage for character movement, collision, jumping, animation, spawning, and gameplay camera behavior.

### Notes

Version `0.5.0` establishes the first complete runtime character and third person gameplay foundation while preserving the separation between World authority, runtime character state, physics, camera systems, rendering, and editor state.

---

## 0.6.0

### Added

* Expanded editor World hierarchy with grouped authored cell types and coordinate-based entries.
* Hierarchy selection integrated with the existing editor selection and Properties system.
* Multi-cell editor selection through viewport drag selection.
* Persistent selection outlines for multiple selected cells.
* Toggleable Plane and Top-Block picking modes.
* Camera-ray based depth-aware picking for visible authored world cells.
* Top-Block Build surface targeting.
* Face-aware Build placement for stacking on existing surfaces.
* Shift + Build overwrite behavior in Top-Block mode.
* Editor keyboard shortcuts for Save, Delete, Undo, Redo, Focus, Escape, and tool switching.
* Authored-world Undo/Redo support for Build, Erase, and Delete operations.
* Camera focus on the active selection.
* Light build type with dedicated Light cell defaults and editor ghost preview.
* Improved editor center/anchor visibility.
* Top-Block mode work-area grid visibility control.

### Changed

* Select and Navigate can now choose the top visible authored block instead of always using grid-plane depth.
* Build and Erase can use the same Plane/Top-Block picking toggle while retaining grid-plane behavior in Plane mode.
* Top-Block Build places new cells outward from the face being targeted instead of replacing the hit cell by default.
* Shift + Build restores intentional overwrite behavior for the hovered cell.
* Escape behavior is context-sensitive in Editor mode while preserving Stop Play and Main Menu behavior.
* Editor world interaction is blocked when the mouse is interacting with egui controls outside the 3D viewport.
* Play mode falls back to the normal editor camera when no runtime character is active.
* Cursor capture follows the presence of an active gameplay character rather than Play mode alone.
* The work-area grid is hidden when Plane picking is disabled.
* The WORLD panel now functions as an authored-world hierarchy rather than only a lighting/physics settings panel.

### Fixed

* Prevented egui menu and panel clicks from accidentally triggering Build, Erase, Select, Navigate, or editor camera input.
* Fixed Select drag behavior so selection does not invoke Build/Erase world editing.
* Fixed Top-Block Build placing new blocks inside the block being targeted.
* Added correct face-normal based placement for surface stacking.
* Added Shift-based immediate Build target updates.
* Prevented Play mode from depending on an uninitialized gameplay camera when no character exists.
* Preserved editor camera state when switching between Editor and Play modes.
* Preserved empty-space Build access through the existing grid fallback when Top-Block picking finds no authored target.

### Notes

Version `0.6.0` expands AeoEngine from a basic voxel editor into a more capable world-authoring workspace, with hierarchy navigation, multi-selection, depth-aware picking, surface construction, editor history, and stronger separation between editor input and runtime behavior.

---

## Current Development

### In Progress

* Physics mass/density integration.
* Continued refinement and testing of Top-Block ray picking and face detection.
* Further definition of `FxBlock` authoring and rendering behavior.
* Determining the proper authored/runtime model for NPCs.
* Further third person player control refinement.
* Character interaction with dynamic PhysicsBodies.

### Planned

* Project-owned asset library using `.assets`.
* Controlled project asset importing.
* Project texture support for authored Blocks.
* Expanded rendering and material capabilities.
* Additional lighting and shadow refinement.
* Additional character gameplay states and systems.

---

# Versioning

AeoEngine uses Semantic Versioning:

`MAJOR.MINOR.PATCH`

### MAJOR

A breaking architectural or project level change requiring existing projects or systems to migrate.

Example:

`1.0.0`

### MINOR

A new engine capability or major feature milestone.

Example:

`0.6.0` — Expanded editor authoring, hierarchy, selection, picking, surface building, and editor history.

### PATCH

Bug fixes, stability improvements, and small corrections within an existing feature milestone.

Example:

`0.3.1` — Physics stability fixes.
