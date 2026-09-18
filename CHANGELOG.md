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
* Project-owned texture importing through the Build and Properties Texture controls.
* PNG texture file filtering in the project asset importer.
* Automatic creation of the project's `.assets/textures` directory when required.
* Project-owned texture copying into `.assets/textures`.
* Automatic texture identifier assignment after importing a texture.
* Dynamic renderer texture loading for imported project textures.
* Renderer texture caching with a fallback texture for missing assets.

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
* Texture fields can now reference project-owned imported textures by their asset identifier.

### Fixed

* Prevented egui menu and panel clicks from accidentally triggering Build, Erase, Select, Navigate, or editor camera input.
* Fixed Select drag behavior so selection does not invoke Build/Erase world editing.
* Fixed Top-Block Build placing new blocks inside the block being targeted.
* Added correct face-normal based placement for surface stacking.
* Added Shift-based immediate Build target updates.
* Prevented Play mode from depending on an uninitialized gameplay camera when no character exists.
* Preserved editor camera state when switching between Editor and Play modes.
* Preserved empty-space Build access through the existing grid fallback when Top-Block picking finds no authored target.
* Added a fallback texture path so missing or unavailable texture assets do not leave the renderer without a valid texture.

### Notes

Version `0.6.0` expands AeoEngine from a basic voxel editor into a more capable world-authoring workspace, with hierarchy navigation, multi-selection, depth-aware picking, surface construction, editor history, project-owned texture assets, and stronger separation between editor input and runtime behavior.

---

## Current Development

### In Progress

* Physics mass/density integration.
* Continued refinement and testing of Top-Block ray picking and face detection.
* Further definition of `FxBlock` authoring and rendering behavior.
* Determining the proper authored/runtime model for NPCs.
* Further third person player control refinement.
* Character interaction with dynamic PhysicsBodies.
* Texture import conflict handling and additional asset workflow validation.
* Visual and runtime testing of imported project textures.

### Planned

* Project-owned asset library using `.assets`.
* Controlled project asset importing.
* Additional supported project asset categories.
* Expanded rendering and material capabilities.
* Additional lighting and shadow refinement.
* Additional character gameplay states and systems.

### Planned Performance & Scalability

The current engine architecture is intentionally prototype-oriented. Several systems have known scalability limits that should be addressed as project size and runtime complexity increase.

#### World Rendering

The current World representation stores authored cells individually, and active block collection requires traversing the stored authored cells.

Future options include:

* Spatially chunked World storage.
* Visible-chunk selection based on camera position.
* Per-chunk render meshes.
* Face culling.
* Greedy meshing.
* Render batching.

A possible progression is:

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

The exact chunk size and meshing strategy will be determined through profiling and representative world workloads.

#### Dynamic Physics Broad Phase

The current dynamic collision system performs pairwise body checks.

Future options include:

* Uniform spatial grids.
* Spatial hashing.
* Sweep-and-prune.
* BVH or another dedicated broad-phase structure.

The existing narrow-phase collision system should remain separate from the broad phase so that candidate generation can be optimized independently.

#### Continuous Collision Detection

The current physics implementation uses discrete collision testing.

Future options include:

* Swept-volume collision testing.
* Selective continuous collision detection for high-speed bodies.
* Projectile-specific continuous collision handling.

Continuous collision detection should be introduced where tunneling becomes a meaningful gameplay or physics requirement rather than enabled indiscriminately.

#### Runtime Mesh Resources

Some runtime rendering paths currently create and destroy GPU vertex objects during mesh drawing.

Future options include:

* Persistent VAO/VBO resources.
* Reusable mesh resources.
* Shared geometry between identical renderables.
* Later batching or instanced rendering where appropriate.

The intended direction is to create stable GPU resources once and reuse them across frames whenever the geometry permits.

#### Performance Engineering

Performance improvements should be driven by profiling and representative workloads.

The engine should avoid replacing simple prototype systems with complex acceleration structures until measurements demonstrate a meaningful need.

The authored/runtime boundary remains unchanged:

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

Optimization systems should improve the performance of these paths without making runtime state authoritative over authored World data.

---

# Versioning

AeoEngine uses Semantic Versioning:

`MAJOR.MINOR.PATCH`

