# AeoEngine Changelog

0.1.0
Added
Initial AeoEngine project structure.
Initial voxel World representation.
Initial grid-based World editing.
Basic editor navigation and camera controls.
Basic block creation and erasing.
Notes

Version 0.1.0 established the initial AeoEngine project, voxel World, editor viewport, camera, and basic World editing foundation.

0.1.1
Added
World persistence foundation.
Initial project data persistence.
Initial Home screen and project management foundation.
Changed
World data can now survive application restarts.
Project state became the foundation for persistent AeoEngine projects.
Notes

Version 0.1.1 established the first persistent project and World workflow.

0.2.0
Added
Expanded editor tools and interaction.
Cell selection.
Cell Properties editing.
Docked World and Properties panels.
Multiple editor grid planes.
Expanded editor camera navigation.
Improved viewport interaction.
Changed
Editor UI expanded into a docked workspace.
Authored scene data remains stored in the World.
Notes

Version 0.2.0 expanded the basic editor into a usable World-authoring workspace.

0.2.1
Added
Authored Light cells.
World lighting data.
Lighting persistence foundation.
Changed
Lights became authored World data rather than purely runtime/editor state.
Notes

Version 0.2.1 introduced authored lighting and established lighting as persistent scene data.

0.3.0
Added
Play mode runtime physics.
Fixed-timestep physics simulation at 60 Hz.
Configurable World gravity using a full Vec3.
Dynamic PhysicsBodies for non-anchored cells.
Static collision detection against anchored solid cells.
Dynamic body collision detection and penetration resolution.
Support tracking between dynamic bodies.
Sleeping and waking behavior for supported dynamic bodies.
Runtime PhysicsBody rendering during Play mode.
Editor and Play mode separation.
World-owned gravity settings.
Changed
Physics runtime state is stored separately from authored World state.
Anchored cells remain part of the authored World.
Dynamic body positions are stored separately from authored World coordinates.
Runtime PhysicsBodies are rendered separately from authored World geometry.
Fixed
Improved resting contact stability.
Prevented gravity from continuously pushing supported bodies into surfaces.
Preserved tangential velocity during collision response.
Added wake behavior when physical support is lost.
Prevented Light cells from becoming physics geometry.
Notes

Version 0.3.0 introduced the runtime physics simulation and established the separation between authored World state and runtime simulation state.

0.3.1
Fixed
Improved dynamic support detection.
Improved sleeping and waking behavior for stacked bodies.
Fixed support-loss wake behavior.
Added regression coverage for physics support changes.
Preserved existing physics behavior while correcting stability issues.
Notes

Version 0.3.1 contains physics stability fixes without changing the runtime physics model.

0.4.0
Added
Renamed the generic Grass cell type to Block.
Added per-Block ColorRGB.
Added configurable Build properties.
Added numeric RGB editing.
Added clipboard RGB copying.
Added persistent Color Picker editing.
Changed
Build operations now use configured Block properties.
World persistence now stores Block color.
Legacy GRASS records load as Block.
Notes

Version 0.4.0 introduced configurable Block properties and established Blocks as the primary authored voxel building type.

0.4.1
Added
Drag-based Build behavior.
Drag-based Erase behavior.
Line building and erasing.
Plane building and erasing.
Ghost previews for drag Build and Erase operations.
Runtime PhysicsBody property snapshots.
Changed
Runtime dynamic rendering reads required properties directly from PhysicsBodies.
World is no longer queried to recover properties from moving runtime bodies.
PhysicsBodies retain required authored properties after entering Play mode.
Notes

Version 0.4.1 expanded World construction tools and improved authored-property preservation during runtime physics.

0.4.2
Fixed
Fixed runtime Block colors being lost during physics simulation.
Fixed runtime visibility being lost when Blocks become PhysicsBodies.
Fixed Color Picker popup lifetime and interaction behavior.
Notes

Version 0.4.2 contains runtime property and editor interaction fixes for the Block and Build systems.

0.5.0
Added
Character runtime system with dedicated character state.
Character transform, movement, collision, and animation state.
Character spawning from authored SpawnPoint cells.
SpawnPoint validation and nearby clearance search.
Fixed-timestep character movement.
Character gravity and grounded state.
Character jumping with grounded jump gating.
Character voxel collision against solid World cells.
Character floor, wall, and ceiling collision handling.
Changed
Character gameplay state is owned by CharacterSystem.
Notes

Version 0.5.0 introduced the runtime character gameplay system and basic third-person movement foundation.

0.5.1
Added
Character movement state switching between Idle and Walk.
Character orientation based on movement direction.
Isolated 3D character rig and skeleton implementation.
Idle and Walk character animation clips.
Character animation blending and evaluated poses.
Character appearance customization data.
Modular generated character geometry.
Runtime integration of the custom character implementation.
Dynamic character mesh rendering.
Changed
Character visual and animation implementation is provided by the character system.
Runtime animation updates use the existing fixed timestep.
Character orientation follows movement direction independently of camera orientation.
Character collision dimensions remain independent of visual character scaling.
Notes

Version 0.5.1 added the integrated 3D character, rig, animation, and runtime character rendering systems.

0.5.2
Added
Dedicated third-person GameplayCamera.
Gameplay camera follow behavior.
Gameplay camera mouse orbit.
Gameplay camera pitch limits.
Gameplay camera collision against solid World geometry.
Gameplay camera follow target derived from the runtime character.
Changed
Play mode now uses a dedicated GameplayCamera.
Gameplay camera state is separate from Editor camera state.
Notes

Version 0.5.2 added the dedicated third-person gameplay camera and camera-character integration.

0.5.3
Added
Editor camera preservation across Play mode transitions.
Regression coverage for character movement, collision, jumping, animation, spawning, and gameplay camera behavior.
Fixed
Prevented editor camera state from being lost across Play mode transitions.
Prevented editor camera controls from modifying the editor camera during Play.
Added gameplay camera obstruction handling against solid voxel geometry.
Prevented gameplay camera behavior from depending on invalid runtime camera state.
Notes

Version 0.5.3 stabilized the character and gameplay-camera systems and completed the Editor/Play camera separation.

0.6.0
Added
Expanded editor World hierarchy with grouped authored cell types and coordinate-based entries.
Hierarchy selection integrated with the existing editor selection and Properties system.
Multi-cell editor selection through viewport drag selection.
Persistent selection outlines for multiple selected cells.
Toggleable Plane and Top-Block picking modes.
Camera-ray-based picking for visible authored World cells.
Top-Block Build surface targeting.
Face-aware Build placement for stacking on existing surfaces.
Shift + Build overwrite behavior in Top-Block mode.
Editor keyboard shortcuts for Save, Delete, Undo, Redo, Focus, Escape, and tool switching.
Authored-world Undo/Redo support for Build, Erase, and Delete operations.
Camera focus on the active selection.
Light build type with dedicated Light cell defaults and editor ghost preview.
Improved editor center/anchor visibility.
Top-Block mode work-area grid visibility control.
Project-owned texture importing through the Build and Properties Texture controls.
PNG texture file filtering in the project asset importer.
Automatic creation of the project's .assets/textures directory when required.
Project-owned texture copying into .assets/textures.
Automatic texture identifier assignment after importing a texture.
Dynamic renderer texture loading for imported project textures.
Renderer texture caching with a fallback texture for missing assets.
Notes

Version 0.6.0 marked the transition into the advanced editor workflow and project asset pipeline.

0.6.1

### Added

* Initial AeoScript language runtime and execution pipeline.
* Lexer, parser, AST, interpreter, fibers, and scheduler integration.
* Numbers, strings, booleans, nil, variables, constants, arithmetic, and logic.
* Control flow: `if/else`, `while`, and `for` loops.
* Cooperative multitasking using the `wait(seconds)` keyword.
* Event handler syntax: `on EventName(args) { ... }`.
* Colon-based method call syntax (`object:method()`).
* Script Editor V1: Integrated workspace with Monaco-based editing and file management.
* Print Output terminal with severity-based coloring (Yellow for Warnings, Red for Errors).

### Notes

Version `0.6.1` introduced the foundation of AeoScript and the integrated script editor.

---

## 0.6.2

### Added

* Opaque engine handles for `Cell` and `Entity` objects.
* Generic object discovery: `getAllCellsOfClass(type)` and `find(identity)`.
* Stable 8-digit randomized Cell IDs for persistent instance tracking.
* Temporary runtime overrides for Cell properties: color, visibility, solidity, anchored, and offset.
* Persistent script binding system allowing scripts to be attached to authored world objects.

### Changed

* Scripting environment now uses stable IDs instead of world coordinates for object referencing.
* Runtime mutations no longer corrupt authored world data.

### Notes

Version `0.6.2` connected AeoScript to the engine world through handles, stable IDs, and runtime overrides.

---

## 0.6.3

### Added

* AeoScript Standard Library: `math`, `basket`, and `string` namespaces.
* `math`: Fully deterministic trigonometric, rounding, and range functions (`clamp`, `lerp`).
* `basket`: Reference-backed arrays with support for `sort`, `find`, `move`, `clone`, and `freeze`.
* `string`: Unicode-aware text manipulation including `len`, `reverse`, and character-based `split`.
* Automatic routing of basket namespace functions to method calls (e.g., `b.len()`).

### Changed

* String operations now operate on Unicode scalar values to prevent UTF-8 byte-slicing panics.
* Basket assignments now use reference semantics (aliasing).

### Notes

Version `0.6.3` provided the core utility library for AeoScript and ensured Unicode safety.

---

## 0.6.4

### Added

* Unique Instance Bindings: Script bindings now target the persistent numeric Cell ID rather than the identity name.
* Legacy script binding migration path for name-based bindings in older world files.
* O(1) runtime index for Cell ID to coordinate resolution.

### Changed

* Multiple authored cells can now share the same `entity_identity` (name) without conflict.
* `find()` remains name-based and can return multiple results for shared identities.

### Fixed

* Prevented stale script binding warnings when a binding targets a non-entity cell type (e.g., a Block with an identity).

### Notes

Version `0.6.4` stabilized the binding model, making script attachments specific to unique world instances.

---

## 0.6.5

### Added

* Fiber Call Stack: Implemented a true stack-based execution model for script fibers.
* Nested Yielding: Scripts can now call `wait()` from inside nested function calls.
* Improved Map semantics: Reading a missing key returns `nil`, and assigning `nil` deletes the key.

### Changed

* Instruction plan expanded with `CallUserFunction` to support resumable nested execution.
* Local variables and loop states are now isolated per-frame in the call stack.

### Notes

Version `0.6.5` completed the fiber execution model, enabling complex logic with yields across function boundaries.

---

## 0.6.6

### Added

* Persistent ScriptInstance synchronization: Fiber state is now merged back to the persistent entity instance on every tick.
* Lifecycle Persistence: mutations made in `on_spawn` or `on_ready` now correctly persist to the `update` loop.
* Added regression coverage for field persistence across multiple frame update tasks.

### Changed

* `ScriptScene` update loop now ensures that the authoritative instance state is updated before discarding completed or yielded fibers.

### Fixed

* Fixed a critical bug where entity fields modified during a script task were lost when the task yielded or finished.

### Notes

Version `0.6.6` finalized the script lifecycle and ensured state continuity between frames.

---

## 0.7.0

### Added

* AeoScript authored Cell Attributes with persistent `Number`, `Bool`, and `String` values.
* AeoScript access to authored Cell attributes through `cell.attributes["key"]`.
* Regression coverage for independent Cell attribute access through AeoScript handles.
* Dedicated AeoScript host bridge module at `src/scripting/host.rs`.
* Separation of the concrete AeoEngine scripting host implementation from application coordination.
* AeoScript top-level executable statements.
* Top-level script statements execute once in source order when the world loads.
* Each script file receives its own top-level execution fiber and `Global` script instance.
* Top-level statements support cooperative `wait()` using the existing fiber scheduler.
* Runtime error reporting for failures during top-level execution.
* Warnings for entity lifecycle declarations that have no matching script binding.
* Regression coverage for top-level execution, top-level yielding, top-level runtime errors, and unattached lifecycle warnings.

### Changed

* `src/engine/app.rs` now acts as the application coordinator for scripting rather than containing the concrete `ScriptHostBridge` implementation.
* `EngineHost` remains the scripting-side host contract while `ScriptHostBridge` provides the concrete AeoEngine implementation.
* AeoScript programs now distinguish executable top-level statements from declarations such as functions, events, and entities.
* Unattached scripts can execute top-level code while continuing to provide global event handlers.
* Entity lifecycle functions remain attachment-driven and now produce warnings when declared without a matching binding.

### Notes

Version `0.7.0` expanded AeoScript from a primarily declaration- and event-driven system into a general-purpose script execution model with authored Cell metadata, a dedicated engine host boundary, and ordered top-level execution.

Verification: `cargo check` passed and the full test suite passed with **303 tests, 0 failures**.

---

## 0.7.1

### Added

* Runtime Cell Attributes: AeoScript can now mutate Cell attributes during Play mode.
* Runtime Attribute Overrides: Script writes to attributes are stored as temporary overrides in `RuntimeCellState` and do not modify authored World data.
* Effective Attribute Resolution: Attribute reads return the runtime override if it exists, falling back to the authored baseline.
* Attribute Reset Semantics: Assigning `nil` to a runtime attribute removes the override and restores the authored value.
* `math.random()`: Generates a random Number in the range [0.0, 1.0).
* `math.random(min, max)`: Generates a random integer-valued Number in the inclusive range [min, max].
* `cell.new(type)`: Creates a runtime-only Cell that exists only during the Play session and is never persisted.
* Mutable Cell Position: Runtime-created Cells can be relocated in the grid using the `position` property.
* `cell.delete(handle)`: Removes a Cell from the active Play-time world. Authored Cells are preserved and return upon stopping Play, while runtime Cells are destroyed.
* Expanded mutable Cell properties for runtime-only instances including `name`, `visible`, `solid`, `anchored`, and `color`.
* Regression coverage for runtime cell creation, authored-state preservation, and `math.random` behavior.

### Changed

* `ScriptHostBridge::get_property` for "attributes" now returns the effective merged attribute map.
* `Interpreter::assign_target` now intercepts attribute-index assignments to route them to the engine's runtime state.
* World rendering and physics now use the "effective" view of the world, combining authored and runtime-created Cells.

### Notes

Version `0.7.1` introduces temporary runtime data and dynamic object creation for AeoScript, while maintaining the absolute invariant that Play-mode execution never modifies authored project data.

---

## 0.7.2

### Added

* World Sky Environment system foundation.
* Native OpenGL `GL_TEXTURE_CUBE_MAP` skybox rendering.
* Support for single-asset horizontal cross layout cubemaps (4:3 aspect ratio).
* Camera-locked skybox positioning to eliminate translation parallax.
* 5 built-in sky presets: `Temperate`, `Tropical`, `Desert`, `Snowy`, `Mars`.
* Persistent `SKY` configuration in world data.
* Editor UI in the World panel for enabling sky and switching presets.
* Asset routing for dedicated `.assets/skybox/` directory.

### Changed

* Optimized texture fallback system to support multiple asset subdirectories.
* Updated world persistence format to include single-asset sky configuration.

### Notes

Version `0.7.2` establishes the foundation for environment rendering in AeoEngine with a seamless skybox system.

---

## 0.7.3

### Added

* `on_touch(cell)` event: AeoScript lifecycle/event function triggered when the player character contacts a scripted object.
* Cell Collision Event Toggle: Persistent `collision_events_enabled` property to control whether a Cell dispatches contact events.
* Script Enable/Disable: Ability to temporarily deactivate scripts from the Script Editor Inspector.
* Persistence for new collision and script state settings.

### Changed

* `ScriptScene` now checks for script enablement before spawning entities or top-level fibers.
* Global event handlers for `on_touch` are filtered based on script enablement.
* Collision event dispatch is gated by the per-cell `collision_events_enabled` flag.

### Notes

Version `0.7.3` provides fine-grained control over script execution and object-level interactivity, enabling more complex gameplay scenarios and better debugging tools.
