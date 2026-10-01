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
  * [0.7.3](#073)
  * [0.7.4](#074)
  * [0.7.5](#075)
* [0.8.x — In Progress](#08x--in-progress)
* [Future Work Backlog](#future-work-backlog)

---

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

## 0.7.3

### Added

* Script-defined entity objects with persistent fields, constructors, methods, and independent object instances
* Lexical closures with captured script scopes retained by function values
* Direct AeoScript access to Play-mode input for movement, jump, and mouse orbit
* Generic player movement and camera control APIs
* Runtime script enable/disable control
* `on_touch(cell)` and `on_overlap(overlapping, cell)` collision events
* Persistent `collision_events_enabled` Cell property for controlling collision event dispatch
* Script-defined controller and camera profiles
* Runtime UI with `Panel`, `Text`, and `Button` elements
* Runtime UI properties for position, size, visibility, enabled state, color, and text
* `UiHandle` runtime values
* `ui.new()` and `ui.delete()` for runtime UI creation and removal
* Runtime UI button callbacks using AeoScript closures
* Runtime UI click scheduling through the script runtime
* World-authored mouse settings: `cursor_visible` and `screen_locked`
* Runtime AeoScript mouse control through `get.mouse()`, `mouse.setCursorVisible`, and `mouse.setScreenLocked`
* Authored `AudioEmitter` Cells with persistent audio, playback, looping, and volume properties
* WAV audio asset support under `.assets/audio/`
* Audio asset browsing, importing, searching, and inline path suggestions
* 3D Audio Emitter editor markers and World Inspector controls
* AeoScript `sound` objects on Cell handles with `.play()`, `.stop()`, `.pause()`, `.playing`, `.looped`, and `.volume`
* Runtime audio playback through the engine `AudioSystem`
* Runtime Cell creation, deletion, movement, and property mutation
* Integration coverage for scripting, runtime UI, audio, controllers, cameras, input, callbacks, and runtime state
* Syntax Highlighting: Improve AeoScript syntax highlighting using the existing lexer and language grammar

### Changed

* `ScriptScene` now loads `.aeo` files from `scripts/`, `controllers/`, and `cameras/`
* Script-defined controllers and cameras retain their runtime state between updates
* Controller scripts now provide movement decisions while AeoEngine retains ownership of gravity, physics, collision, grounded resolution, and movement integration
* Camera scripts now provide camera behavior while AeoEngine retains ownership of the underlying camera and rendering systems
* Repeated controller and camera calls now use direct runtime method dispatch
* Runtime UI is rendered as a foreground gameplay overlay clipped to the central 3D viewport
* Runtime UI coordinates are relative to the central 3D viewport
* Editor 3D rendering, OpenGL clearing, and hover raycasting now use the exact central viewport bounds
* Editor workspace now maintains the intended `[P | V | P]` layout
* Controller and camera selection is now project-configurable through controller and camera profiles
* Runtime mouse state is initialized from World defaults when Play starts and restored when Play exits
* Runtime UI and runtime audio state are reset with the Play-mode lifecycle
* Runtime audio state is maintained separately from authored World audio state
* Website and documentation materials were updated to reflect the current AeoEngine and AeoScript implementation

### Fixed

* Redundant script-side gravity handling in controller scripts
* Repeated controller and camera dispatch overhead
* Temporary controller diagnostic logging after controller and input behavior were verified
* Incorrect editor projection caused by using the full application window aspect ratio
* Hover picking offsets caused by editor side-panel dimensions
* Runtime UI coordinates being interpreted relative to the full application window
* Runtime UI rendering over editor side panels
* Default runtime Panel styling that made white text unreadable
* Runtime script state not being reset correctly when Play mode stops
* Runtime UI state persisting beyond the Play session
* Controller and camera runtime state not persisting correctly between updates

### Notes

> Version 0.7.3 expands AeoScript beyond scene and object scripting into script-defined gameplay controllers, cameras, input handling, runtime UI, closures, runtime script control, collision events, mouse control, and spatial audio while keeping AeoEngine responsible for physics, gravity, collision, rendering, audio playback, and simulation.

> Runtime UI, runtime audio state, runtime-created Cells, and other temporary Play-mode state are discarded when Play mode stops. Authored World data remains the source of truth for persistent project content.

---

## 0.7.4 — Released 

### Added

* Exposed-Face Meshing. Static voxel geometry submission now culls occluded internal faces.
* Spatial Chunking & Batched Static Voxel Meshes.
* Dirty Chunk / Dirty-Region Rebuilding.
* Greedy Meshing for static voxel geometry.
* Editor Copy & Paste (`Ctrl+C` / `Ctrl+V`). Copies selected authored cells, relative offsets, cell data, and script bindings into an in-memory clipboard. Pasting assigns new persistent unique IDs and duplicates script bindings to the new IDs.
* Transactional Grab / Drag in Select Mode. Click and hold an already-selected cell to drag the entire multi-selection as a group. Uses a 5-pixel drag threshold to distinguish clicks from moves.
* Additive Multi-Selection (`Ctrl + Click` / `Ctrl + Drag`). Holding `Ctrl` while clicking adds/toggles individual blocks. Holding `Ctrl` while dragging out a selection box adds all blocks in the range to the active selection group without clearing existing selections.
* Translucent Grab Ghost Preview. Renders translucent preview blocks and outlines at target positions during Grab mode, displaying red highlight indicators if destination coordinates collide with unrelated cells.
* Generic Runtime Entity Character Controls. AeoScript entities can now expose position, velocity, facing, grounded state, animation, health, maximum health, and alive state, along with jump, damage, heal, destroy, and pathfinding operations.
* Scripted Runtime Character Spawning. AeoScript can spawn generic runtime character entities and control them through the generic Entity API.
* Scripted Enemy Orchestration. Enemy behavior can now be organized across multiple AeoScript files using shared events for spawning, AI, combat, damage, death, and respawning.
* Runtime Character Contact Identity. Character collision processing now preserves the identity of the character that touched a World cell so AeoScript touch handlers can receive the touching Entity.
* Entity-Based `on_touch(thing)` Callbacks. Runtime touch events now pass the Entity that physically touched the object, allowing scripts to distinguish Player, enemies, and other runtime characters.
* Persistent World Hierarchy Cache. The editor now caches grouped World hierarchy data and reuses it until the World render revision changes.
* Category Navigation for non-Block World objects. Lights, Audio Emitters, Spawn Points, FX Blocks, Players, and NPCs can be navigated with `PREV` / `NEXT` controls instead of rendering one UI entry per Cell.
* Count-only Block hierarchy display. Large Block collections are represented by a single `BLOCKS (count)` entry instead of thousands of individual UI rows.
* Physics static collider bulk registration for large authored Worlds.

### Changed

* Static voxel rendering now uses persistent spatially chunked meshes with texture batching and selective chunk rebuilding.
* Multi-Cell Copy Pivot Alignment. Copying a multi-selection defaults to the bottom-most cell ($\min(Y)$, $\min(X)$, $\min(Z)$) as the pivot, ensuring pasted structures sit cleanly on top of target surfaces without floor collisions.
* History & Undo/Redo. History now snapshots both `world.cells` and `world.script_bindings` together, keeping script binding associations synchronized on undo/redo.
* Character gameplay responsibilities now separate engine-owned movement, gravity, collision, and grounded state from script-owned movement decisions and behavior.
* AeoScript gameplay control now favors generic Entity APIs rather than specialized NPC-specific namespaces.
* World-panel hierarchy construction no longer creates one egui widget per authored Block.
* Static physics registration now builds the static collider chunk index once after bulk registration instead of rebuilding it for every anchored solid Cell.
* Character contact handling now separates the touching Entity from the touched World cell, allowing collision-driven scripts to act on the actual object that made contact.
* Starter World interaction scripts now use `on_touch(thing)` for collision-driven behavior. Traps and Coins can verify that the Player was the thing that touched them before applying their behavior.
* Enemy combat now keeps chase/attack decision-making in the enemy AI flow while the combat script handles the resulting damage event without duplicating character-distance collision calculations.
* World hierarchy cache invalidation is now tied to World replacement when projects are loaded or closed, preventing data from a previous project from remaining visible in the World panel.

### Fixed

* Fixed a long-standing issue where block shadows could become detached from their source blocks as the camera moved.
* Fixed runtime Entity method dispatch for scripted character controls.
* Fixed script event queuing and dispatch for runtime gameplay events.
* Fixed severe World-panel frame-rate degradation in large Worlds caused by creating tens of thousands of Block UI entries every frame.
* Fixed severe Play-mode startup cost for large anchored Worlds caused by repeatedly rebuilding the static physics chunk index during Cell registration.
* Fixed World hierarchy data remaining from a closed project after opening a different project.
* Fixed World hierarchy cache state becoming stale when a newly loaded World happened to use the same render revision as the previous World.
* Fixed voxel-face shadow artifacts caused by insufficient shadow-map self-shadowing bias.
* Fixed character contact events losing the identity of the character that caused the contact.
* Fixed touch-driven interaction scripts from receiving only the touched Cell instead of the touching Entity.
* Fixed Coin and Trap interaction logic so they can explicitly verify that the Player was the object that touched them.
* Fixed redundant enemy combat distance checks that duplicated contact decisions already made by the enemy behavior script.

### Performance

* Large Worlds can now remain at 60 FPS while displaying the World panel without constructing a UI row for every Block.
* Large anchored Worlds can now enter Play mode without rebuilding the complete static physics index once per Cell.
* 60K-Block stress testing now maintains 60 FPS in Editor mode and Play mode across repeated Play/Stop cycles and project switching.
* Selective renderer chunk rebuilding remains localized after the large-World performance changes.
* Character contact processing preserves per-character contact identity without changing the existing collision processing model.

### Notes

> Version 0.7.4 continues the static voxel renderer work while removing major non-rendering frame and startup costs exposed by large-World testing.
>
> The World panel now treats Blocks as aggregate authored data and provides individual navigation controls only for World object categories where selecting a specific Cell is useful.
>
> Physics registration now separates bulk World initialization from incremental runtime synchronization: initial static colliders are registered first and the static chunk index is rebuilt once, while later mutations continue to use the existing dirty-cell synchronization path.
>
> Character collision events now preserve both sides of a contact: the touched World Cell remains the event target, while the Entity that caused the contact is passed to `on_touch(thing)`.
>
> This allows interaction scripts such as Coins and Traps to make their own decisions about which Entity should trigger the interaction without introducing specialized gameplay APIs.
>
> Enemy gameplay keeps detection and attack-range decisions in the enemy behavior flow while combat remains responsible for applying damage, avoiding duplicate spatial contact calculations.
>
> World hierarchy caching is invalidated explicitly across project lifecycle transitions because World revisions are local to a World instance and cannot by themselves identify a newly loaded project.
>
> 60K-Block stress testing completed the following lifecycle without an FPS drop:
>
> `Editor → Play → Stop → Play → Close Project → Open Project → Play`
>
> Static voxel renderer benchmarking continues to show localized selective rebuild timings independent of total World size within the tested cases.
>
> The renderer shadow artifact affecting voxel faces was corrected with slope-dependent shadow bias and normal-offset protection.

---
## 0.7.5 — Current Unreleased

### Added

* Editor Input Settings:

  * Added persistent editor camera navigation bindings.
  * Added configurable Orbit Camera and Free Look mouse bindings.
  * Added configurable Select and Delete editor bindings.
  * Added Reset Bindings for restoring the default editor input layout.

* Editor Settings:

  * Added configurable editor camera movement speed and camera sensitivity.
  * Added configurable undo-history limits.
  * Added Settings controls for existing Output and Problems panel visibility.
  * Added beta Autosave intervals for authored project and open script changes.
  * Added a beta preference for bypassing existing destructive-action confirmation dialogs.
  * Added persistent AeoEngine editor preferences across application sessions.
  * Added Home-screen access to the existing Settings workspace.

* Editor Theme Settings:

  * Added functional Theme controls for AeoEngine editor appearance.
  * Added `Aeo Dark`, `Light`, and configurable theme presentation modes.
  * Added editor accent selection with Orange, Red, and Cool accent presets.
  * Added interface scaling at 75%, 100%, 125%, and 150%.
  * Added independent proportional and monospace font-size controls.
  * Added configurable panel rounding.
  * Added configurable panel contrast.
  * Added configurable separator visibility.
  * Added editor UI animation preferences.
  * Added tooltip visibility control.
  * Added status-bar visibility control.
  * Added persistent AeoEngine editor preferences so Theme settings survive application restarts.

* Editor Lighting Settings UI:

  * Added functional UI controls for beta lighting pipeline options, including radiance cache, radiance cascades, clipmap shadows, GI resolution, and diagnostics.
  * Positioned the beta lighting performance overlay within the viewport bounds to prevent panel overlap.

* Generic Keyboard Input Events & Polling:

  * `on.keypress(key, down)` script event generated for physical keyboard press and release events.
  * Auto-repeat events are suppressed so each physical keypress produces exactly one press and one release event.
  * Window focus loss automatically dispatches release events for all currently held keys and drains held input state.
  * Canonical AeoScript key naming such as `"A"`, `"1"`, `"Space"`, `"ArrowUp"`, and `"Escape"`.
  * `input.is_key_down(key_name)` standard library function for AeoScript keyboard polling.

* Script Runtime Hardening & Dynamic Property Ownership (0.7.5D):

  * Assignment-time script ownership tracking for dynamic properties (`DynamicProperty { value, owner_script }`).
  * Per-script dynamic property teardown (`remove_dynamic_properties_by_owner`) automatically cleaning up assigned callbacks on script disable.
  * Synchronous `on_destroy()` teardown error collection in `ScriptScene::stop()`.
  * Runtime teardown errors are reported through engine terminal output.
  * Negative zero normalization (`-0.0` → `0.0`) and `NaN` rejection in `Value::as_map_key()`.
  * Expanded regression coverage for handle validation, queued event propagation, arity checking, task scheduler purging, dynamic property lifecycle, `on_destroy()` error logging, and map key normalization.

* First-Class Authored Actors:

  * Introduced `Actor` as a distinct authored World object rather than representing Actors as `Cell` instances.
  * Authored Actors store identity, transform, visibility, enabled state, package selection, physics defaults, and persistent attributes.
  * Authored Actor IDs use a dedicated allocator separate from authored Cell IDs.
  * Added dedicated Actor persistence through `world.dat`.
  * Added persistent Actor script bindings separate from ordinary Cell script bindings.
  * Added migration support for legacy worlds where Actors were stored as Cells.
  * Added runtime `RuntimeActor` bookkeeping that maps authored Actor IDs to runtime Entity IDs.
  * Character spawning remains optional: a Character is created only when the Actor's package represents a character package.
  * Actor runtime bookkeeping remains separate from scripting, rendering, character control, and physics ownership.

* Generic AeoScript Modules:

  * Added generic script-to-script module references through `scripts.someScript.aeo`.
  * Module references create and cache runtime module instances for the lifetime of the scripting runtime.
  * Module instances maintain persistent top-level module state.
  * Different module paths maintain isolated module state.
  * Module functions use normal AeoScript arguments and return values.
  * Nested module calls use the existing interpreter call-frame and execution model.
  * Module calls remain compatible with resumable fibers and `wait()`.
  * Modules can be called from top-level scripts, event handlers, Entity scripts, and Actor scripts.
  * Missing modules and missing module functions use the existing AeoScript runtime error system.
  * The module mechanism is generic and does not introduce gameplay-specific module types.
  * Added Actor-scoped Module instances with `self.parent` bound to the owning Actor.
  * The same Module file can be attached to multiple Actors without sharing instance state.
  * Added `player.controller` and `player.find(...)` access to PlayerScripts attached to the runtime Player Actor.
  * Added project-scoped `proj.find(...)`, `proj.find.all(...)`, and hierarchical Actor lookup paths so project queries do not rely on ambiguous global names.
  * Preserved Actor identity and attached Modules when an Actor is renamed.

* Authoritative Architecture Documentation:

  * Documented `Cell` as authored voxel/world data.
  * Documented `Actor` as an independent authored complex gameplay object.
  * Documented `Entity` as temporary runtime state.
  * Documented Actor → Entity conversion during Play mode.
  * Documented CharacterSystem as an optional runtime capability rather than the definition of Actor.
  * Documented persistent Actor IDs and Actor script bindings.
  * Added dedicated AeoScript module documentation covering references, state, calls, nesting, fibers, events, and runtime lifetime.

* Runtime UI system hardening and additional practical functionality built on the existing `Panel`, `Text`, and `Button` APIs.

* Additional useful capabilities within existing engine systems where current workflows expose clear gaps.

* Expanded regression coverage for mature runtime and editor systems.

* Standalone Game Build Pipeline:

  * Built-in Release build target for the standalone `aeogame` runtime executable.
  * Runtime-only Cargo build using `--no-default-features --features runtime --bin aeogame`.
  * Per-project build output under `UserData/Builds/<Project>/<Version>/`.
  * Standalone executable packaging together with the required game data and assets.
  * Build metadata and runtime project data are copied into the standalone build directory.

* Built Game Discovery and Launching:

  * Home screen discovery of completed standalone game builds under `UserData/Builds/`.
  * Built games are grouped by project and version.
  * Built games can be launched directly from the AeoEngine Home screen.
  * Launching a built game uses the packaged executable's directory as its working directory.

* Standalone Runtime Controls:

  * Standalone built games can exit with `Escape`.
  * Runtime input handling remains available to AeoScript while the standalone runtime owns application-level exit behavior.

### Changed

* Editor input controls now use configurable bindings instead of fixed keyboard and mouse assignments.

* Removed unsupported mouse-behavior and input-diagnostics placeholders from the Input Settings page.

* Editor settings now expose only supported editor preferences rather than placeholder viewport and render-distance controls.

* Editor preference changes are stored separately from authored project data.

* Editor visual styling is now driven by Theme settings instead of the Theme page displaying placeholder controls.

* Theme changes apply to the AeoEngine editor without modifying authored game UI or runtime project presentation.

* Theme preferences are stored as AeoEngine-level preferences rather than individual project state.

* Status-bar visibility now follows the configured editor preference.

* Editor interface scale and typography can be adjusted independently of authored runtime UI layout.

* Lighting UI dropdown labels now display beta metadata explicitly in the Settings workspace without altering renderer state labels.

* Point Light Emitter Collection & Indexing:

  * Enforced that every candidate point light must satisfy `cell.cell_type == CellType::Light && world.is_light_enabled(coord)`.
  * Persisted `Block` cells with `light_enabled == true` are ignored and no longer create invalid point lights.
  * Updated `World::iter_light_ids()` and `rebuild_light_and_marker_indices` to filter out deleted or non-Light cells from the light registry.
  * Point light source positions are uploaded using the voxel cell center (`coord + 0.5`) for uniform spatial lighting.

* `ScriptScheduler::remove()` purges terminal tasks from both `tasks` and `ready_queue`, keeping task queue sizes bounded across long-running script updates.

* `ScriptRuntime::remove_fiber()` returns `Result<Option<ScriptFiber>, String>` and atomically removes scheduler tasks.

* Enforced strict arity (`!= 2`) on `test.complete()` and `complete_test()`.

* Re-routed `entity.is_valid()` to check handle validity prior to entity manager lookup.

* Runtime UI behavior is being matured around properties, callbacks, handle lifetime, cleanup, and Play-mode lifecycle.

* Audio runtime behavior is being matured around playback state, emitter control, cleanup, and script interaction.

* Existing AeoScript APIs are being refined for consistency, reliability, diagnostics, and predictable runtime behavior.

* Character, Entity, Physics, World, and scripting systems are being hardened without introducing unnecessary new architecture.

* Editor workflows are being matured with a focus on consistency, usability, state management, and recovery from invalid or incomplete operations.

* Runtime build reporting now surfaces build progress and completion information through the engine terminal.

* Standalone runtime builds compile without the editor feature and use the dedicated runtime executable target.

* Built game launch paths use packaged runtime-relative assets and project data rather than the editor working directory.

* Documentation is being updated to match current implemented systems and terminology, including the Cell / Actor / Entity architecture and generic AeoScript modules.

### Fixed

* Fixed `entity.is_valid()` panicking or erroring on destroyed entity handles.

* Fixed `cell.delete()` accepting non-cell/light handle kinds such as `Entity`, `Ui`, `Sound`, or `Mouse`.

* Fixed queued event dispatch errors being dropped silently in `ScriptScene::update()`.

* Fixed Entity destruction leaving independently scheduled contact callbacks alive after the owning Entity was removed.

* Fixed AeoScript resolving the built-in `player` as `nil` when no Player Actor is active, matching the documented optional-player behavior.

* Fixed individual Entity destruction suppressing `on_destroy()` failures; errors now reach the caller after runtime state is cleaned up, without dropping later queued destruction requests.

* Fixed deleted Cells leaving bound script instances, dynamic properties, or suspended Cell callbacks alive; Cell-bound `on_destroy()` now runs when a Cell disappears.

* Fixed two-argument AeoScript `spawn(name, position)` creating a generic Entity instead of the documented default runtime Character; explicit third-argument packages remain supported.

* Fixed Actor hierarchy entries drifting out of sync when Actors are created or removed outside Project Explorer controls.

* Fixed disabled Actor Modules falling through to standalone execution, and corrected enable/disable lookup and re-instantiation for attached Modules and PlayerScripts.

* Fixed map key retrieval treating `-0.0` and `0.0` as distinct keys.

* Fixed `NaN` values causing invalid map-key behavior.

* Fixed invalid point lights being created from ordinary `Block` cells containing `light_enabled == true` in persisted chunk data.

* Fixed stale or deleted light IDs remaining in `World::iter_light_ids()`.

* Fixed standalone runtime project loading so built games correctly reproduce the required runtime initialization sequence:

  * World loading
  * Gameplay camera selection
  * Physics registration
  * Player character spawning
  * Runtime mouse defaults
  * Script startup
  * Player entity creation and association
  * `PlayerSpawned` event dispatch

* Fixed standalone runtime rendering paths so game rendering does not depend on editor-only `EditorMode` state.

* Fixed runtime chunk/cache rendering mode separation between editor rendering and standalone game rendering.

* Fixed standalone built games launching without their expected authored World/runtime setup.

* Fixed Home built-game discovery so completed packaged executables can be identified by project and version.

* Fixed built-game launching so the standalone executable starts with its packaged build directory as the working directory.

* Fixed standalone runtime `Escape` handling so `Escape` closes the built game instead of attempting to use editor-only exit handling.

* Fixed legacy Actor-as-Cell persistence migration so existing worlds can transition to first-class Actors without losing Actor identity, package, physics defaults, attributes, or script bindings.

* Fixed runtime Actor ownership so authored Actor IDs are no longer stored or tracked as authored Cell IDs.

* Fixed Actor runtime mappings to use authoritative authored Actor IDs rather than script-facing names.

* Fixed Actor persistence parsing for transform and physics fields, including persistent physics mass.

* Removed the requirement for authored Actor names/identities to be globally unique.

* Fixed AeoScript module call context restoration so nested module calls correctly restore the caller execution context after the callee returns.

* Fixed nested module execution across resumable calls so module calls preserve the correct caller/callee instance context while fibers yield and resume.

* Added a dedicated resizable, hideable Output dock that retains Play logs after returning to the Editor and remains visible while the Script Editor is open; Script Editor mode uses a compact dock height.

### Performance

* Eliminating duplicate world rasterization passes reduces rendering overhead during Play and Editor modes.
* Static runtime rendering uses dedicated game rendering paths without depending on editor mode state.
* Standalone games use the runtime-only Cargo feature set without compiling the editor executable target.
* Continued focused performance profiling uses measured AeoEngine hot paths.
* Full test/build verification includes the standalone runtime build path.

### Verification

| Verification                               |                     Result |
| ------------------------------------------ | -------------------------: |
| Library Rust tests                         | **738 passing, 1 ignored** |
| Standalone runtime build integration tests |          **2 / 2 passing** |
| Character workspace integration tests      |          **7 / 7 passing** |
| Doc-tests                                  |   **0 tests, no failures** |
| Full `cargo test` run                      |             **0 failures** |

### Notes

> Version 0.7.5 remains a hardening and maturity release.
>
> The primary goal is not to expand AeoEngine with another large collection of unrelated systems, but to make the systems introduced and expanded through 0.7.x more reliable, consistent, and complete.
>
> Runtime UI, Audio, AeoScript, Entity and Character control, Physics, World interaction, and editor workflows continue to receive focused reliability improvements.
>
> The 0.7.5 cycle also introduces the first complete standalone game workflow:
>
> `AeoEngine Editor → Build Game → Standalone aeogame → Home → Built Games → Launch`
>
> Built games are packaged separately from the editor and can be launched without opening the editor project itself. The runtime initializes the authored World, selected gameplay camera, physics, character system, scripting, runtime input, and required project-relative data before entering gameplay.
>
> The standalone runtime remains intentionally separate from the editor. Editor-only rendering and interaction state are not required by the shipped game executable.
>
> The Home screen now provides a Built Games section that discovers completed packaged builds under `UserData/Builds/` and launches them directly.
>
> Standalone runtime application controls are also separated from gameplay scripting. `Escape` closes the standalone game while gameplay input events remain available to AeoScript.
>
> The Actor model is now formally separated from the Cell model. Authored Actors persist independently, become runtime Entities during Play, and can represent gameplay objects beyond Characters without redesigning the existing Cell/voxel architecture.
>
> AeoScript now also has a generic module model for reusable script-to-script communication. Modules use normal AeoScript functions, arguments, return values, persistent runtime state, nested calls, and fiber-compatible execution rather than introducing gameplay-specific module systems.
>
> The documentation now defines the Cell / Actor / Entity architecture and the AeoScript module model as authoritative engine concepts.
>
> Warning output is currently suppressed locally during development so the increasingly large automated test suite remains readable while the codebase is being hardened. Warning cleanup remains a separate maintenance task.

---

## 0.8.x — In Progress

### Added

* Added a beta Settings workspace control surface for experimental lighting methods, shadow methods, GI quality, bounce and temporal behavior, scene-query selection, cache updates, denoising, debug views, performance views, and apply behavior. The workspace labels beta features as experimental and preserves Legacy lighting as the default.
* Added the first inspectable Two-Level Radiance Cache research slice: an incremental static voxel scene, six-direction world radiance cache, diffuse forward-shader resolve, single/multiple bounce modes, temporal reuse, update-rate and resolution controls, and isolated indirect-light debug output. Screen probes remain pending.
* Added live Chunk Grid DDA and Occupied-Chunk BVH query backends for GI rays, with Automatic selection for sparse occupied-chunk bounds and first-hit regression coverage.
* Added the first dynamic GI query tier for visible solid physics-body AABBs through the `Bodies Only` setting. Static and dynamic hits share nearest-hit selection; Actor and Character GI remains pending.
* Added a non-destructive bounded World render-change journal so renderer caches can maintain independent invalidation cursors.
* Added `dev-build.ps1` to build the editor and update `%LOCALAPPDATA%\Aeowun\AeoEngine.exe` for development installs.

* Added responsive UI anchors for all parent edges, corners, and center points, plus stretch anchors with margins. Authored and runtime UI layout now resolve against the playable viewport so anchored elements adapt to window and resolution changes.
* Added UI Authoring resize handles, editable display names and script keys, anchor selection, and Undo/Redo history.
* Added script lookup for keyed UI controls through `ui.find(key)`, `ui.get(key)`, and their document-qualified forms, `ui.find(document_key, key)` and `ui.get(document_key, key)`. Lookup returns the existing `Ui` handle and property workflow.
* Added stable script keys to authored UI nodes. Keys are serialized, unique within each document, and default to the top-left anchor when loading older UI documents.
* Added UI anchor and key properties to the runtime UI API so authored and script-created controls share the same property model.
* Added standalone Windows AeoEngine installation from a single `AeoEngine.exe`. On first launch, the release executable installs AeoEngine under `%LOCALAPPDATA%\Aeowun`.
* Added the first-run installer window with the AeoEngine splash, animated installation state, package checks, and live installation status.
* Added automatic installation and repair of the required AeoEngine layout:
  * `AeoEngine.exe`
  * `.assets/`
  * `UserData/`
  * `LICENSE`
* Added missing-package restoration without overwriting existing `UserData`, `.assets`, or `LICENSE` content.
* Added automatic relaunch from the installed `%LOCALAPPDATA%\Aeowun\AeoEngine.exe` after first-run installation completes.
* Added `debug-startup.ps1` for isolated first-run installation verification using the already-built `target/release/AeoEngine.exe` and a temporary `LOCALAPPDATA`.
* Added stale-release detection to the startup simulation so it refuses to test an outdated release executable.
* Added runtime-only named rig attachment-point lookup for CharacterPackage-backed Entities. It resolves points from the latest evaluated pose into a world-space transform matrix without creating an attachment or changing authored data.
* Added a Play-session-only AttachmentSystem with named-point transform solving, cycle rejection, mounted physics/collision suspension, and physics/movement restoration on detach. Exposed generic `attach(...)` and `detach(...)` AeoScript operations.
* Added one-shot animation action layers with independent playback and joint-subtree masks, exposed as `entity.animation.play_layer(slot, clip, joint_root)`. Dungeon's Hero slash playtest is triggered by F and layers over locomotion; G detaches the held sword and restores its physical World state.
* Fixed Actor Module contact dispatch so handlers assigned through `self.parent.on_touch` receive runtime Entity contacts. Legacy `entity { ... }` lifecycle code inside an attached Actor Module now emits an actionable warning instead of the misleading generic “no script binding” warning.
* Registered `attach(...)` and `detach(...)` as first-class global AeoScript built-ins, so their engine bridge is reachable from ordinary scripts and Actor Module callbacks without an unknown-variable failure.
* Replaced the Output dock's transient framework resize state with explicit Editor-owned heights for the normal and Script Editor workspaces. The dock now preserves the released height across frames and hide/show cycles while retaining the compact Script Editor default.
* Editor selection bounds for package-backed Actors now follow their rendered package geometry, keeping item outlines such as `hand_sword` aligned with the visible asset.

### Fixed

* Development `cargo run` no longer enters Play mode because a stale `target/debug/GameData` directory exists; debug editor builds always start in Editor mode.
* Project discovery now uses `%LOCALAPPDATA%\Aeowun\UserData` on Windows for both development and installed editor launches. Legacy `UserData\Project` recent entries continue to resolve, while new recent-project entries are stored relative to the canonical UserData directory.

* Authored UI now loads before script startup, allowing startup scripts to find authored controls while preserving controls created dynamically during startup.
* Deleting an authored UI parent now preserves its children's resolved layout, and moving a node no longer changes its parent implicitly.
* Fixed the release startup path so a downloaded AeoEngine executable installs its required files before launching the installed copy.
* Fixed installation verification so it exercises the actual release executable and real installer flow instead of rebuilding or pre-populating the simulated install directory.
* Character packages now resolve by package folder name, canonical `package.json` name, and optional manifest-declared `aliases`. Renamed packages can preserve existing authored Actor references without hardcoded engine mappings; the `hand_coin`, `hand_key`, and `hand_switch` packages retain their legacy `character_*` IDs this way.
* Unknown explicit character package IDs no longer silently substitute the default robot package; they fail with the missing package ID in the error.
* Character package selectors list each package once using its manifest name, while duplicate identifiers are reported and resolved deterministically.
* CharacterSystem-owned Actor transforms are synchronized to their runtime Entities before contact evaluation, so the visible sword and its `on_touch` contact body cannot drift apart. Generic Entity physics no longer integrates the same Character-backed Actor a second time.
* Actor Module startup failures are quarantined to the failing Actor for the current Play session. One broken attachment now reports one actionable error while unrelated scripts, player movement, camera setup, and runtime UI continue starting.
* `on_touch` now fires on contact entry instead of every fixed-physics step, preventing repeated callbacks and Output spam while two bodies remain overlapped.
* First-person camera proximity hiding now applies only to the explicitly active Player. Nearby package-backed Actors such as swords, keys, switches, and coins remain visible.
* Normalized `hand_coin`, `hand_key`, `hand_switch`, and `hand_sword` geometry to a grounded one-cell local footprint, aligned their collision dimensions with the rendered meshes, and updated package manifest names to the `hand_*` convention.
* Split `hand_sword`'s visual joint from its `Grip` attachment point. The authored Grip now mounts at the handle and projects the blade forward instead of letting the attachment solve cancel the intended orientation.
* Simplified the Hero slash to a clean, connected RightArm shoulder arc with lift, cut, and recovery while locomotion continues underneath it. The sword mount also rolls the blade into a vertical cutting plane instead of holding it flat like a paddle.
* Reserved bare `G` for gameplay input during Play mode. Dropping an attached item no longer triggers the Editor/Home navigation shortcut.
* Runtime UI layout now uses the actual playable viewport rather than the complete application window, preventing HUD controls from shrinking or sliding beneath editor panels.
* Moved Script Editor parser diagnostics out of the permanently reserved bottom strip into a resizable, closable Problems window. The Project/World header and View menu now toggle it, with the active script's issue count shown in the label.
* Added end-to-end regression coverage proving AeoScript contact-to-attachment dispatch, Grip-to-RightHand transform alignment and blade direction during simultaneous Walk and upper-body Slash animation, detach physics restoration, Play-mode gameplay-key ownership, hand-package grid bounds, player-only camera identity, and Output dock height persistence.

---

## Future Work Backlog

> These are follow-on maturity tasks, not claims that the listed systems are wholly absent and not evidence of a 0.7.5 feature being broken.

* Harden AudioEmitter runtime controls
* Expand useful capabilities within existing systems
* Mature generic Entity capabilities
* Expand Actor and Entity physics capabilities
* Add generic Entity appearance and animation capabilities
* Expand AeoScript runtime reliability and diagnostics
* Expand AeoScript Entity interaction and world-query APIs
* Expand script-to-script communication beyond the current generic module system
* Expand regression coverage
* Mature Entity and Character workflows
* Harden Physics and runtime World synchronization
* Continue maturing the editor
* Audit inconsistent or prototype-level editor workflows
* Continue documentation correction and alignment with implemented behavior
* Continue hardening the standalone game runtime
* Improve packaged game distribution and build management

> Advanced lighting and shadow experiments were removed from the active 0.7.5 scope due to implementation cost. They are deferred future work; 0.7.5 does not claim a compute-lighting or advanced-shadow pipeline.

---

> **Current release:** `0.7.5`
> **Development cycle:** `0.7.x`
> **Next release:** `0.7.6`
