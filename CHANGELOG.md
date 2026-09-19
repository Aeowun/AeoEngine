# AeoEngine Changelog

## 0.1.0

### Added

* Initial AeoEngine project structure.
* Initial voxel World representation.
* Grid-based world editing.
* Basic editor navigation and camera controls.
* Home screen and project management foundation.
* Basic block creation and erasing.
* World persistence foundation.

### Notes

Version `0.1.0` added the initial engine, editor, World, camera, and project systems.

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
* Authored scene data remains stored in the World.

### Notes

Version `0.2.0` added the main world editing workflow and authored scene properties.

---

## 0.3.0

### Added

* Play mode runtime physics.
* Fixed-timestep physics simulation at 60 Hz.
* Configurable world gravity using a full `Vec3`.
* Dynamic PhysicsBodies for non-anchored cells.
* Static collision detection against anchored solid cells.
* Dynamic body collision detection and penetration resolution.
* Support tracking between dynamic bodies.
* Sleeping and waking behavior for supported dynamic bodies.
* Runtime PhysicsBody rendering during Play mode.
* Editor and Play mode separation.
* World-owned gravity settings.

### Changed

* Physics runtime state is stored separately from authored World state.
* Anchored cells remain part of the authored World.
* Dynamic body positions are stored separately from authored World coordinates.
* Runtime PhysicsBodies are rendered separately from authored World geometry.

### Fixed

* Improved resting contact stability.
* Prevented gravity from continuously pushing supported bodies into surfaces.
* Preserved tangential velocity during collision response.
* Added wake behavior when physical support is lost.
* Prevented Light cells from becoming physics geometry.

### Notes

Version `0.3.0` added runtime physics and separate runtime simulation state.

---

## 0.3.1

### Fixed

* Improved dynamic support detection.
* Improved sleeping and waking behavior for stacked bodies.
* Fixed support-loss wake behavior.
* Added regression coverage for physics support changes.
* Preserved existing physics behavior while correcting stability issues.

### Notes

Version `0.3.1` contains physics stability fixes without changing the runtime physics model.

---

## 0.4.0

### Added

* Renamed the generic `Grass` cell type to `Block`.
* Added per-Block `ColorRGB`.
* Added configurable Build properties.
* Added numeric RGB editing.
* Added clipboard RGB copying.
* Added persistent Color Picker editing.
* Added drag-based Build behavior.
* Added drag-based Erase behavior.
* Added line building and erasing.
* Added plane building and erasing.
* Added ghost previews for drag Build and Erase operations.
* Added runtime PhysicsBody property snapshots.

### Changed

* Build operations now use configured Block properties.
* PhysicsBodies retain required authored properties after entering Play mode.
* Runtime dynamic rendering reads properties directly from PhysicsBodies.
* World is no longer queried to recover properties from moving runtime bodies.
* World persistence now stores Block color.
* Legacy `GRASS` records load as `Block`.

### Fixed

* Fixed runtime Block colors being lost during physics simulation.
* Fixed runtime visibility being lost when Blocks become PhysicsBodies.
* Fixed Color Picker popup lifetime and interaction behavior.

### Notes

Version `0.4.0` added configurable Block properties and expanded the Build and Erase tools.

---

## 0.5.0

### Added

* Character runtime system with dedicated character state.
* Character transform, movement, collision, and animation state.
* Character spawning from authored SpawnPoint cells.
* SpawnPoint validation and nearby clearance search.
* Fixed-timestep character movement.
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
* Dedicated third-person GameplayCamera.
* Gameplay camera follow behavior.
* Gameplay camera mouse orbit.
* Gameplay camera pitch limits.
* Gameplay camera collision against solid World geometry.
* Gameplay camera follow target derived from the runtime character.
* Editor camera preservation across Play mode transitions.

### Changed

* Character gameplay state is owned by `CharacterSystem`.
* Character visual and animation implementation is provided by the character system.
* Runtime animation updates use the existing fixed timestep.
* Play mode now renders the integrated 3D character instead of the temporary Magenta cube.
* Play mode uses a dedicated GameplayCamera instead of the Editor camera.
* Editor and gameplay camera state are stored separately.
* Character orientation follows movement direction independently of camera orientation.
* Character collision dimensions remain independent of visual character scaling.

### Fixed

* Prevented editor camera state from being lost across Play mode transitions.
* Prevented editor camera controls from modifying the editor camera during Play.
* Added gameplay camera obstruction handling against solid voxel geometry.
* Added regression coverage for character movement, collision, jumping, animation, spawning, and gameplay camera behavior.

### Notes

Version `0.5.0` added the runtime character, third-person camera, movement, collision, and animation systems.

---

## 0.6.0

### Added

* Expanded editor World hierarchy with grouped authored cell types and coordinate-based entries.
* Hierarchy selection integrated with the existing editor selection and Properties system.
* Multi-cell editor selection through viewport drag selection.
* Persistent selection outlines for multiple selected cells.
* Toggleable Plane and Top-Block picking modes.
* Camera-ray-based picking for visible authored World cells.
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

* Select and Navigate can choose the top visible authored block instead of always using grid-plane depth.
* Build and Erase can use the same Plane/Top-Block picking toggle while retaining grid-plane behavior in Plane mode.
* Top-Block Build places new cells outward from the targeted face instead of replacing the hit cell by default.
* Shift + Build restores overwrite behavior for the hovered cell.
* Escape behavior is context-sensitive in Editor mode while preserving Stop Play and Main Menu behavior.
* Editor world interaction is blocked when the mouse is interacting with egui controls outside the 3D viewport.
* Play mode falls back to the normal editor camera when no runtime character exists.
* Cursor capture follows the presence of an active gameplay character rather than Play mode alone.
* The work-area grid is hidden when Plane picking is disabled.
* The WORLD panel now functions as an authored-world hierarchy.
* Texture fields can reference project-owned imported textures by asset identifier.

### Fixed

* Prevented egui menu and panel clicks from triggering Build, Erase, Select, Navigate, or editor camera input.
* Fixed Select drag behavior so selection does not invoke Build/Erase world editing.
* Fixed Top-Block Build placing new blocks inside the targeted block.
* Added face-normal-based placement for surface stacking.
* Added Shift-based immediate Build target updates.
* Prevented Play mode from depending on an uninitialized gameplay camera when no character exists.
* Preserved editor camera state when switching between Editor and Play modes.
* Preserved empty-space Build access through the existing grid fallback when Top-Block picking finds no authored target.
* Added a fallback texture path for missing or unavailable texture assets.

### Notes

Version `0.6.0` added hierarchy navigation, multi-selection, depth-aware picking, surface building, editor history, and project texture assets.

---

# Current Development

## AeoScript V1

### Added

* Initial AeoScript language runtime and execution pipeline.
* Lexer, parser, AST, interpreter, fibers, scheduler, and script runtime integration.
* Script source spans and runtime diagnostics.
* Numbers, strings, booleans, nil, variables, constants, arithmetic, comparisons, conditionals, loops, functions, and returns.
* Cooperative `wait()` execution through persistent script fibers.
* Event handler syntax using `on EventName(args)`.
* Colon-based method calls.
* Generic object and property access foundations.
* Runtime `Cell` and `Entity` handles.
* Generic Basket values with `.len()` and index access.
* Generic object discovery through `getAllCellsOfClass()`.
* Generic `find()` object discovery.
* Parent and child object discovery.
* `Cell:getObject()` runtime object resolution.
* Player spawning events through the runtime `CharacterSystem` player spawn.
* Global and unattached event handlers.
* Existing legacy `get_entity()` compatibility.
* Existing legacy `get_light()` compatibility.
* Script-to-engine light control through light handles.
* Light enable/disable scripting workflow.
* Script Editor V1.
* Script workspace and script management.
* Script creation, opening, editing, saving, and attachment.
* Structured `LogRecord` output with Script, Warning, Error, and System severities.
* Script path, event/function, entity, and object identity logging context.
* Editor Print Output terminal integration using runtime script output.
* Selectable terminal text with mouse interaction and keyboard copy/select support.
* Severity-based terminal presentation with yellow warnings and red errors.
* Engine-originated warning and error presentation without synthetic `UNKNOWN` or `UNATTACHED` labels.
* Runtime error propagation from script fibers into structured terminal diagnostics.
* Non-fatal stale script binding warnings.
* Generic runtime collection discovery returning script-visible Cell references.

### Changed

* Script runtime state remains separate from authored World state.
* Global script handlers operate without requiring an authored entity attachment.
* Script-visible engine objects are represented through opaque runtime handles.
* Runtime script output is routed through the structured logging system.
* Print Output displays runtime diagnostics from the same structured logging path used by the scripting runtime.

### Fixed

* Fixed duplicate PlayerSpawned handler execution.
* Fixed silent failures from global and unattached script fibers being discarded.
* Fixed stale script bindings preventing the scripting scene from loading.
* Fixed script runtime errors failing to reach the editor output path.
* Fixed generic light discovery progressing through the AeoScript runtime.
* Fixed Basket length access for discovered object collections.
* Improved runtime error context for failing script operations.

## In Progress

* Stable 8-digit randomized Cell identity (WO-00102).
* Temporary runtime overrides for Cell properties.
* Runtime modification of Cell color, visibility, solidity, and visual offset.
* Script-to-engine discovery using stable Cell IDs instead of world coordinates.
* AeoScript string concatenation (`+`) with automatic string coercion.
* Unified terminal output through the structured `LogRecord` pipeline.
* Terminal severity coloring: red for Errors and yellow for Warnings.
* Removal of `UNKNOWN` and `UNATTACHED` placeholders from terminal diagnostics.
* Continued AeoScript API and Script Editor integration work.
* Continued language, API, and editor documentation updates.
