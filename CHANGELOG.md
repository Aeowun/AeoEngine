# AeoEngine Changelog

> Development history for AeoEngine and AeoScript.

## Contents

* [0.1.x — Foundation](#01x--foundation)

    * [0.1.0](#010)
    * [0.1.1](#011)
* [0.2.x — Editor Expansion](#02x--editor-expansion)

    * [0.2.0](#020)
    * [0.2.1](#021)
* [0.3.x — Physics](#03x--physics)

    * [0.3.0](#030)
    * [0.3.1](#031)
* [0.4.x — Blocks and Building](#04x--blocks-and-building)

    * [0.4.0](#040)
    * [0.4.1](#041)
    * [0.4.2](#042)
* [0.5.x — Character and Gameplay](#05x--character-and-gameplay)

    * [0.5.0](#050)
    * [0.5.1](#051)
    * [0.5.2](#052)
    * [0.5.3](#053)
* [0.6.x — Editor and AeoScript Foundation](#06x--editor-and-aeoscript-foundation)

    * [0.6.0](#060)
    * [0.6.1](#061)
    * [0.6.2](#062)
    * [0.6.3](#063)
    * [0.6.4](#064)
    * [0.6.5](#065)
    * [0.6.6](#066)
* [0.7.x — Runtime Scripting Expansion](#07x--runtime-scripting-expansion)

    * [0.7.0](#070)
    * [0.7.1](#071)
    * [0.7.2](#072)
    * [0.7.3 — Unreleased](#073--unreleased)
* [TODO](#todo)

---

# 0.1.x — Foundation

## 0.1.0

### Added

* Initial AeoEngine project structure
* Initial voxel World representation
* Initial grid-based World editing
* Basic editor navigation and camera controls
* Basic block creation and erasing

### Notes

> Version 0.1.0 established the initial AeoEngine project, voxel World, editor viewport, camera, and basic World editing foundation.

---

## 0.1.1

### Added

* World persistence foundation
* Initial project data persistence
* Initial Home screen and project management foundation

### Changed

* World data can now survive application restarts
* Project state became the foundation for persistent AeoEngine projects

### Notes

> Version 0.1.1 established the first persistent project and World workflow.

---

# 0.2.x — Editor Expansion

## 0.2.0

### Added

* Expanded editor tools and interaction
* Cell selection
* Cell Properties editing
* Docked World and Properties panels
* Multiple editor grid planes
* Expanded editor camera navigation
* Improved viewport interaction

### Changed

* Editor UI expanded into a docked workspace
* Authored scene data remains stored in the World

### Notes

> Version 0.2.0 expanded the basic editor into a usable World-authoring workspace.

---

## 0.2.1

### Added

* Authored Light cells
* World lighting data
* Lighting persistence foundation

### Changed

* Lights became authored World data rather than purely runtime/editor state

### Notes

> Version 0.2.1 introduced authored lighting and established lighting as persistent scene data.

---

# 0.3.x — Physics

## 0.3.0

### Added

* Play mode runtime physics
* Fixed-timestep physics simulation at 60 Hz
* Configurable World gravity using a full `Vec3`
* Dynamic `PhysicsBody` instances for non-anchored cells
* Static collision detection against anchored solid cells
* Dynamic body collision detection and penetration resolution
* Support tracking between dynamic bodies
* Sleeping and waking behavior for supported dynamic bodies
* Runtime `PhysicsBody` rendering during Play mode
* Editor and Play mode separation
* World-owned gravity settings

### Changed

* Physics runtime state is stored separately from authored World state
* Anchored cells remain part of the authored World
* Dynamic body positions are stored separately from authored World coordinates
* Runtime `PhysicsBody` instances are rendered separately from authored World geometry

### Fixed

* Improved resting contact stability
* Prevented gravity from continuously pushing supported bodies into surfaces
* Preserved tangential velocity during collision response
* Added wake behavior when physical support is lost
* Prevented Light cells from becoming physics geometry

### Notes

> Version 0.3.0 introduced the runtime physics simulation and established the separation between authored World state and runtime simulation state.

---

## 0.3.1

### Fixed

* Improved dynamic support detection
* Improved sleeping and waking behavior for stacked bodies
* Fixed support-loss wake behavior
* Added regression coverage for physics support changes
* Preserved existing physics behavior while correcting stability issues

### Notes

> Version 0.3.1 contains physics stability fixes without changing the runtime physics model.

---

# 0.4.x — Blocks and Building

## 0.4.0

### Added

* Renamed the generic Grass cell type to `Block`
* Added per-Block `ColorRGB`
* Added configurable Build properties
* Added numeric RGB editing
* Added clipboard RGB copying
* Added persistent Color Picker editing

### Changed

* Build operations now use configured Block properties
* World persistence now stores Block color
* Legacy `GRASS` records load as `Block`

### Notes

> Version 0.4.0 introduced configurable Block properties and established Blocks as the primary authored voxel building type.

---

## 0.4.1

### Added

* Drag-based Build behavior
* Drag-based Erase behavior
* Line building and erasing
* Plane building and erasing
* Ghost previews for drag Build and Erase operations
* Runtime `PhysicsBody` property snapshots

### Changed

* Runtime dynamic rendering reads required properties directly from `PhysicsBody` instances
* World is no longer queried to recover properties from moving runtime bodies
* `PhysicsBody` instances retain required authored properties after entering Play mode

### Notes

> Version 0.4.1 expanded World construction tools and improved authored-property preservation during runtime physics.

---

## 0.4.2

### Fixed

* Fixed runtime Block colors being lost during physics simulation
* Fixed runtime visibility being lost when Blocks become `PhysicsBody` instances
* Fixed Color Picker popup lifetime and interaction behavior

### Notes

> Version 0.4.2 contains runtime property and editor interaction fixes for the Block and Build systems.

---

# 0.5.x — Character and Gameplay

## 0.5.0

### Added

* Character runtime system with dedicated character state
* Character transform, movement, collision, and animation state
* Character spawning from authored `SpawnPoint` cells
* `SpawnPoint` validation and nearby clearance search
* Fixed-timestep character movement
* Character gravity and grounded state
* Character jumping with grounded jump gating
* Character voxel collision against solid World cells
* Character floor, wall, and ceiling collision handling

### Changed

* Character gameplay state is owned by `CharacterSystem`

### Notes

> Version 0.5.0 introduced the runtime character gameplay system and basic third-person movement foundation.

---

## 0.5.1

### Added

* Character movement state switching between `Idle` and `Walk`
* Character orientation based on movement direction
* Isolated 3D character rig and skeleton implementation
* `Idle` and `Walk` character animation clips
* Character animation blending and evaluated poses
* Character appearance customization data
* Modular generated character geometry
* Runtime integration of the custom character implementation
* Dynamic character mesh rendering

### Changed

* Character visual and animation implementation is provided by the character system
* Runtime animation updates use the existing fixed timestep
* Character orientation follows movement direction independently of camera orientation
* Character collision dimensions remain independent of visual character scaling

### Notes

> Version 0.5.1 added the integrated 3D character, rig, animation, and runtime character rendering systems.

---

## 0.5.2

### Added

* Dedicated third-person `GameplayCamera`
* Gameplay camera follow behavior
* Gameplay camera mouse orbit
* Gameplay camera pitch limits
* Gameplay camera collision against solid World geometry
* Gameplay camera follow target derived from the runtime character

### Changed

* Play mode now uses a dedicated `GameplayCamera`
* Gameplay camera state is separate from Editor camera state

### Notes

> Version 0.5.2 added the dedicated third-person gameplay camera and camera-character integration.

---

## 0.5.3

### Added

* Editor camera preservation across Play mode transitions
* Regression coverage for character movement, collision, jumping, animation, spawning, and gameplay camera behavior

### Fixed

* Prevented editor camera state from being lost across Play mode transitions
* Prevented editor camera controls from modifying the editor camera during Play
* Added gameplay camera obstruction handling against solid voxel geometry
* Prevented gameplay camera behavior from depending on invalid runtime camera state

### Notes

> Version 0.5.3 stabilized the character and gameplay-camera systems and completed the Editor/Play camera separation.

---

# 0.6.x — Editor and AeoScript Foundation

## 0.6.0

### Added

* Expanded editor World hierarchy with grouped authored cell types and coordinate-based entries
* Hierarchy selection integrated with the existing editor selection and Properties system
* Multi-cell editor selection through viewport drag selection
* Persistent selection outlines for multiple selected cells
* Toggleable Plane and Top-Block picking modes
* Camera-ray-based picking for visible authored World cells
* Top-Block Build surface targeting
* Face-aware Build placement for stacking on existing surfaces
* `Shift` + Build overwrite behavior in Top-Block mode
* Editor keyboard shortcuts for Save, Delete, Undo, Redo, Focus, Escape, and tool switching
* Authored-world Undo/Redo support for Build, Erase, and Delete operations
* Camera focus on the active selection
* Light build type with dedicated Light cell defaults and editor ghost preview
* Improved editor center/anchor visibility
* Top-Block mode work-area grid visibility control
* Project-owned texture importing through the Build and Properties Texture controls
* PNG texture file filtering in the project asset importer
* Automatic creation of the project's `.assets/textures` directory when required
* Project-owned texture copying into `.assets/textures`
* Automatic texture identifier assignment after importing a texture
* Dynamic renderer texture loading for imported project textures
* Renderer texture caching with a fallback texture for missing assets

### Notes

> Version 0.6.0 marked the transition into the advanced editor workflow and project asset pipeline.

---

## 0.6.1

### Added

* Initial AeoScript language runtime and execution pipeline
* Lexer, parser, AST, interpreter, fibers, and scheduler integration
* Numbers, strings, booleans, `nil`, variables, constants, arithmetic, and logic
* Control flow: `if/else`, `while`, and `for` loops
* Cooperative multitasking using the `wait(seconds)` keyword
* Event handler syntax: `on EventName(args) { ... }`
* Colon-based method call syntax: `object:method()`
* Script Editor V1 with Monaco-based editing and file management
* Print Output terminal with severity-based coloring:

    * Yellow for warnings
    * Red for errors

### Notes

> Version 0.6.1 introduced the foundation of AeoScript and the integrated script editor.

---

## 0.6.2

### Added

* Opaque engine handles for `Cell` and `Entity` objects
* Generic object discovery:

    * `getAllCellsOfClass(type)`
    * `find(identity)`
* Stable 8-digit randomized Cell IDs for persistent instance tracking
* Temporary runtime overrides for Cell properties:

    * `color`
    * `visibility`
    * `solidity`
    * `anchored`
    * `offset`
* Persistent script binding system allowing scripts to be attached to authored World objects

### Changed

* Scripting environment now uses stable IDs instead of World coordinates for object referencing
* Runtime mutations no longer corrupt authored World data

### Notes

> Version 0.6.2 connected AeoScript to the engine World through handles, stable IDs, and runtime overrides.

---

## 0.6.3

### Added

* AeoScript Standard Library:

    * `math`
    * `basket`
    * `string`
* `math` namespace with deterministic trigonometric, rounding, and range functions including `clamp` and `lerp`
* `basket` namespace with reference-backed arrays and support for:

    * `sort`
    * `find`
    * `move`
    * `clone`
    * `freeze`
* `string` namespace with Unicode-aware text manipulation including:

    * `len`
    * `reverse`
    * character-based `split`
* Automatic routing of basket namespace functions to method calls such as `b.len()`

### Changed

* String operations now operate on Unicode scalar values to prevent UTF-8 byte-slicing panics
* Basket assignments now use reference semantics (aliasing)

### Notes

> Version 0.6.3 provided the core utility library for AeoScript and ensured Unicode safety.

---

## 0.6.4

### Added

* Unique Instance Bindings: script bindings now target the persistent numeric Cell ID rather than the identity name
* Legacy script binding migration path for name-based bindings in older World files
* O(1) runtime index for Cell ID to coordinate resolution

### Changed

* Multiple authored cells can now share the same `entity_identity` without conflict
* `find()` remains name-based and can return multiple results for shared identities

### Fixed

* Prevented stale script binding warnings when a binding targets a non-entity cell type such as a Block with an identity

### Notes

> Version 0.6.4 stabilized the binding model, making script attachments specific to unique World instances.

---

## 0.6.5

### Added

* Fiber Call Stack: implemented a true stack-based execution model for script fibers
* Nested Yielding: scripts can now call `wait()` from inside nested function calls
* Improved Map semantics:

    * Reading a missing key returns `nil`
    * Assigning `nil` deletes the key

### Changed

* Instruction plan expanded with `CallUserFunction` to support resumable nested execution
* Local variables and loop states are now isolated per-frame in the call stack

### Notes

> Version 0.6.5 completed the fiber execution model, enabling complex logic with yields across function boundaries.

---

## 0.6.6

### Added

* Persistent `ScriptInstance` synchronization: Fiber state is now merged back to the persistent entity instance on every tick
* Lifecycle Persistence: mutations made in `on_spawn` or `on_ready` now correctly persist to the `update` loop
* Regression coverage for field persistence across multiple frame update tasks

### Changed

* `ScriptScene` update loop now ensures that authoritative instance state is updated before discarding completed or yielded fibers

### Fixed

* Fixed a critical bug where entity fields modified during a script task were lost when the task yielded or finished

### Notes

> Version 0.6.6 finalized the script lifecycle and ensured state continuity between frames.

---

# 0.7.x — Runtime Scripting Expansion

## 0.7.0

### Added

* AeoScript authored Cell Attributes with persistent `Number`, `Bool`, and `String` values
* AeoScript access to authored Cell attributes through `cell.attributes["key"]`
* Regression coverage for independent Cell attribute access through AeoScript handles
* Dedicated AeoScript host bridge module at `src/scripting/host.rs`
* Separation of the concrete AeoEngine scripting host implementation from application coordination
* AeoScript top-level executable statements
* Top-level script statements execute once in source order when the World loads
* Each script file receives its own top-level execution fiber and `Global` script instance
* Top-level statements support cooperative `wait()` using the existing fiber scheduler
* Runtime error reporting for failures during top-level execution
* Warnings for entity lifecycle declarations that have no matching script binding
* Regression coverage for:

    * top-level execution
    * top-level yielding
    * top-level runtime errors
    * unattached lifecycle warnings

### Changed

* `src/engine/app.rs` now acts as the application coordinator for scripting rather than containing the concrete `ScriptHostBridge` implementation
* `EngineHost` remains the scripting-side host contract while `ScriptHostBridge` provides the concrete AeoEngine implementation
* AeoScript programs now distinguish executable top-level statements from declarations such as functions, events, and entities
* Unattached scripts can execute top-level code while continuing to provide global event handlers
* Entity lifecycle functions remain attachment-driven and now produce warnings when declared without a matching binding

### Notes

> Version 0.7.0 expanded AeoScript from a primarily declaration- and event-driven system into a general-purpose script execution model with authored Cell metadata, a dedicated engine host boundary, and ordered top-level execution.

| Verification         |                   Result |
| -------------------- | -----------------------: |
| `cargo check`        |                   Passed |
| Full Rust test suite | **303 passed, 0 failed** |

---

## 0.7.1

### Added

* Runtime Cell Attributes: AeoScript can now mutate Cell attributes during Play mode
* Runtime Attribute Overrides: script writes to attributes are stored as temporary overrides in `RuntimeCellState` and do not modify authored World data
* Effective Attribute Resolution: attribute reads return the runtime override if it exists, falling back to the authored baseline
* Attribute Reset Semantics: assigning `nil` to a runtime attribute removes the override and restores the authored value
* `math.random()`: generates a random Number in the range `[0.0, 1.0)`
* `math.random(min, max)`: generates a random integer-valued Number in the inclusive range `[min, max]`
* `cell.new(type)`: creates a runtime-only Cell that exists only during the Play session and is never persisted
* Mutable Cell Position: runtime-created Cells can be relocated in the grid using the `position` property
* `cell.delete(handle)`: removes a Cell from the active Play-time World
* Expanded mutable Cell properties for runtime-only instances including:

    * `name`
    * `visible`
    * `solid`
    * `anchored`
    * `color`
* Regression coverage for runtime Cell creation, authored-state preservation, and `math.random()` behavior

### Changed

* `ScriptHostBridge::get_property` for `"attributes"` now returns the effective merged attribute map
* `Interpreter::assign_target` now intercepts attribute-index assignments to route them to the engine's runtime state
* World rendering and physics now use the effective view of the World, combining authored and runtime-created Cells

### Notes

> Version 0.7.1 introduces temporary runtime data and dynamic object creation for AeoScript while maintaining the absolute invariant that Play-mode execution never modifies authored project data.

---

## 0.7.2

### Added

* World Sky Environment system foundation
* Native OpenGL `GL_TEXTURE_CUBE_MAP` skybox rendering
* Support for single-asset horizontal cross layout cubemaps with a `4:3` aspect ratio
* Camera-locked skybox positioning to eliminate translation parallax
* Five built-in sky presets:

    * `Temperate`
    * `Tropical`
    * `Desert`
    * `Snowy`
    * `Mars`
* Persistent `SKY` configuration in World data
* Editor UI in the World panel for enabling sky and switching presets
* Asset routing for the dedicated `.assets/skybox/` directory

### Changed

* Optimized texture fallback system to support multiple asset subdirectories
* Updated World persistence format to include single-asset sky configuration

### Fixed

* Fixed Play → Stop → Play scripting lifecycle state leaking between sessions
* Preserved authored disabled-script state across Play sessions
* Cleared transient scripting state when stopping Play mode
* Fixed nested AeoScript function return values being consumed by the wrong call site
* Fixed release executable relative paths so `UserData` and `.assets` resolve from the executable directory

### Notes

> Version 0.7.2 establishes the sky/environment foundation and closes the release with scripting lifecycle, interpreter, and release-runtime stability fixes.

> Version 0.7.2 also introduced the master `.aeo` integration test script, which runs and verifies individual scripts for language features, control flow, arrays, maps, strings, functions, closures, waits, Cells, attributes, Entities, Lights, World behavior, handles, callbacks, and math as a single in-game test suite.

| Verification                |                Result |
| --------------------------- | --------------------: |
| Rust tests                  | **402 / 402 passing** |
| In-game integration tests   |   **16 / 16 passing** |
| Consecutive full-suite runs |                 **5** |

---

## 0.7.3 — Unreleased

### AeoScript

#### Added

* Script-defined entity objects with persistent fields, constructors, methods, and independent object instances
* Lexical closures with captured script scopes retained by function values
* Direct AeoScript access to Play-mode input:

  * movement
  * jump
  * mouse orbit
* Generic player movement and camera control APIs
* Runtime Script Control for enabling and disabling scripts while the game is running
* `on_touch(cell)` event for player contact with scripted Cells
* Persistent `collision_events_enabled` Cell property for controlling contact event dispatch
* Script enable/disable state persistence
* Continued lexer and parser work with corresponding regression coverage

#### Changed

* `ScriptScene` now loads `.aeo` files from:

  * `scripts/`
  * `controllers/`
  * `cameras/`
* Script-defined controller and camera objects retain state between runtime updates
* Repeated controller and camera calls use direct runtime method dispatch rather than constructing temporary call expressions
* Controller scripts now provide movement decisions while AeoEngine continues to own:

  * gravity
  * physics
  * collision
  * grounded resolution
  * movement integration
* Camera scripts now provide camera behavior while AeoEngine continues to own the underlying camera and rendering systems

#### Fixed

* Removed redundant script-side gravity handling from controller scripts
* Reduced repeated controller and camera dispatch overhead
* Removed temporary controller diagnostic logging after controller execution and input access were verified

---

### Runtime UI

#### Added

* Runtime UI system with:

  * `Panel`
  * `Text`
  * `Button`
* Runtime UI properties for:

  * position
  * size
  * visibility
  * enabled state
  * color
* `UiHandle` runtime values
* `ui.new()` for runtime UI creation
* `ui.delete()` for runtime UI removal
* Runtime UI button callbacks using AeoScript closures
* Runtime UI click scheduling through the script runtime
* Integration coverage for:

  * creation
  * property access
  * callbacks
  * deletion
  * handle validation

#### Changed

* Runtime UI is reset with Play mode lifecycle state
* Runtime UI is rendered as a foreground gameplay overlay inside the central 3D viewport
* Runtime UI coordinates are relative to the central 3D viewport
* Runtime UI rendering is clipped to the central 3D viewport

#### Fixed

* Runtime UI coordinates being interpreted relative to the full application window
* Runtime UI rendering over editor side panels
* Default runtime Panel styling making white text unreadable

---

### Editor and Viewport

#### Changed

* Editor 3D rendering now uses the exact central viewport bounds and aspect ratio
* Editor OpenGL rendering and clearing are clipped to the central viewport
* Editor hover raycasting now accounts for central viewport bounds
* Editor workspace now maintains the intended `[P | V | P]` layout

#### Fixed

* Incorrect editor projection caused by using the full application window aspect ratio
* Hover picking offsets caused by side-panel dimensions

---

### Gameplay Controllers and Cameras

#### Added

* Project controller profiles for selecting script-defined controller implementations
* Project camera profiles for selecting script-defined camera implementations
* Generic APIs for scripted player movement and facing
* Generic APIs for scripted camera behavior
* Script-driven third-person controller implementation
* Script-driven gameplay camera implementation

#### Changed

* Controller logic is now delegated to AeoScript while runtime physics remains engine-owned
* Camera logic is now delegated to AeoScript while rendering remains engine-owned

---

### Runtime Stability

#### Fixed

* Script runtime control now supports enabling and disabling scripts without restarting Play mode
* Runtime script state is correctly reset with Play mode lifecycle state
* Runtime UI state is discarded when Play mode stops
* Controller and camera runtime state persists correctly between updates

---

### Notes

> Version 0.7.3 expands AeoScript beyond scene and object scripting into script-defined gameplay controllers, cameras, input handling, runtime UI, closures, and runtime script control while keeping AeoEngine responsible for physics, gravity, collision, rendering, and simulation.

> Runtime UI is session-only and is discarded when Play mode stops.

| Verification              |                          Result |
| ------------------------- | ------------------------------: |
| Rust tests                |                  **402 passed** |
| Rust test failures        |                           **0** |
| In-game master test suite | Runtime UI integration coverage |

### Planned

<details>
<summary><strong>Character System</strong></summary>

Characters will become project-owned data rather than being directly tied to the current `src/character_custom` implementation.

The first implementation will prove a concrete project-owned character workflow before expanding the system further.

</details>

<details>
<summary><strong>Syntax Highlighting</strong></summary>

The Script Editor will gain proper AeoScript syntax highlighting based on the engine's existing lexer.

This will make keywords, strings, numbers, functions, comments, and other language elements visually distinct while keeping the editor aligned with the actual language grammar.

</details>

<details>
<summary><strong>Voxel Rendering</strong></summary>

The voxel renderer will avoid generating faces that are completely hidden by neighboring blocks.

This will significantly reduce unnecessary geometry while continuing to use GPU back-face culling for the faces that remain.

</details>

---

# TODO

* [ ] Perform a full sweep of AeoEngine documentation
* [ ] Perform a full sweep of the AEOWUN website
* [ ] Bring documentation and website terminology into alignment with the current AeoEngine and AeoScript implementation

---

> **Current release:** `0.7.2`
> **Development branch:** `0.7.3 — Unreleased`
