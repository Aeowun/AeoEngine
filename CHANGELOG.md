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

### Planned

* Character spawning system.
* Basic 3D character.
* Capsule character collision.
* Character movement.
* Character runtime state.
* Basic idle and walking animation.

### Notes

Version `0.5.0` begins the Character system while keeping character state, physics, rendering, and spawning separated into their own systems.

---

## Future

Planned engine work includes:

* Dynamic point lights.
* Expanded lighting controls.
* Shadow refinement.
* Character controller improvements.
* Additional character animation.
* Expanded rendering and material systems.

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

`0.5.0` — Character system.

### PATCH

Bug fixes, stability improvements, and small corrections within an existing feature milestone.

Example:

`0.3.1` — Physics stability fixes.