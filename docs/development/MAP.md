# AeoEngine Project Map

**Repository:** `https://github.com/Aeowun/AeoEngine`
**Branch:** `main`
**Purpose:** Source-code phone book for finding the system/file/location to change.

Line numbers below are based on the current `main` source inspected for this map. They will move as code changes. Keep this file updated whenever systems are moved, split, or renamed.

---

# 1. Fast Lookup

Use this section first.

| What you want to change | Start here | Important locations |
|---|---|---|
| Cell data / Cell fields | `src/world/cell.rs` | `Cell` at lines 19-37 |
| Cell creation/defaults | `src/world/cell.rs` | `Cell::default`, `new_block`, `new_light`, `new_spawn_point` |
| World storage | `src/world/world.rs` | `World` at lines 28-50 |
| Find Cell by coordinate | `src/world/world.rs` | `get`, `get_mut` lines 64-70 |
| Find Cell by persistent ID | `src/world/world.rs` | `resolve_cell_id` lines 72-75 |
| Create/delete Cell | `src/world/world.rs` | `set_cell` lines 223-250 |
| Cell ID index | `src/world/world.rs` | `update_id_mapping`, `rebuild_id_mapping` lines 252-263 |
| Save world data | `src/world/persistence.rs` | `save_world` starts line 13 |
| Load world data | `src/world/persistence.rs` | `load_world` starts line 174 |
| Character system | `src/character/` | `system.rs` |
| Character movement state/constants | `src/character/movement.rs` | `MovementState`, `MOVE_SPEED`, `JUMP_IMPULSE`, `CharacterMovement` |
| Character runtime update | `src/character/system.rs` | `CharacterSystem::update` lines 47-107 |
| Character collision | `src/character/system.rs` | static collision 109-159; dynamic collision 160-214 |
| Character spawn | `src/character/spawning.rs` | `spawn_at_random_point` lines 4-65 |
| Character custom animation | `src/character_custom/animation.rs` | clips/skeleton/track evaluation |
| Character rig | `src/character_custom/rig.rs` | skeleton/pose types |
| Character appearance | `src/character_custom/appearance.rs` | material slots/customization |
| Character mesh | `src/character_custom/geometry.rs` | `generate_character_mesh` |
| Character animation blending | `src/character_custom/blend.rs` | `CharacterAnimationController` |
| Character input / WASD | `src/engine/app.rs` | Play input lines 666-690 |
| Character fixed-step execution | `src/engine/app.rs` | physics clock + character update lines 697-710 |
| Character jump input | `src/engine/app.rs` | window keyboard handling around lines 286-291 |
| Gameplay camera | `src/renderer/camera.rs` | `GameplayCamera` |
| Editor camera | `src/renderer/camera.rs` | `CameraController` |
| Physics world | `src/engine/physics.rs` | `PhysicsWorld` lines 149-160 |
| Physics initial world registration | `src/engine/physics.rs` | `register_from_world` lines 572-608 |
| Physics incremental synchronization | `src/engine/physics.rs` | `sync_with_world` lines 613+ |
| Physics bodies | `src/engine/physics.rs` | `PhysicsBody` |
| Main application coordinator | `src/engine/app.rs` | `App` lines 14-57 |
| Editor state | `src/editor/editor.rs` | `Editor` lines 47-86 |
| Editor Properties panel | `src/editor/editor.rs` | `render_properties_content` lines 406+ |
| Editor property mutations | `src/editor/editor.rs` | `PropertyChange` lines 369-381; `apply_property_changes` 383-405 |
| Editor tools | `src/editor/tools.rs` | toolbar starts line 124 |
| Grid generation | `src/editor/grid/grid.rs` | grid generation/plane selection |
| World picking/raycast | `src/editor/grid/picking.rs` | `raycast_world` starts line 60 |
| Editor navigation | `src/editor/navigation.rs` | `NavigationWindow` |
| Script editor | `src/editor/script_editor.rs` | `ScriptEditor` starts line 72 |
| AeoScript runtime | `src/scripting/runtime.rs` | `ScriptRuntime` lines 17-27; tick 171-205 |
| AeoScript scheduler/fibers | `src/scripting/execution.rs` | scheduler lines 79-276 |
| AeoScript VM/interpreter | `src/scripting/interpreter.rs` | `ScriptInstance` lines 12-22; interpreter is the large execution core |
| AeoScript host bridge | `src/scripting/host.rs` | concrete AeoEngine -> AeoScript EngineHost implementation |
| AeoScript values/handles | `src/scripting/value.rs` | `HandleKind` lines 4-23; `Value` lines 79-95 |
| AeoScript host API | `src/scripting/api.rs` | `EngineHost` lines 4-24 |
| Script bindings | `src/scripting/binding.rs` | `ScriptBinding` lines 8-13 |
| AeoScript standard library | `src/scripting/stdlib.rs` | dispatch starts line 7; `math.random`; `cell.new`; `cell.delete` |
| World authored grid | `src/world/world.rs` | `World.cells` map |
| World runtime storage | `src/world/world.rs` | `World.runtime_cells`, `runtime_state.is_deleted` and indices |
| Script scene/lifecycle integration | `src/scripting/scene.rs` | `ScriptScene` and scene/runtime orchestration |
| AeoScript syntax tree | `src/scripting/ast.rs` | `Program`, declarations, statements, expressions |
| AeoScript parser | `src/scripting/parser.rs` | parser implementation |
| AeoScript lexer | `src/scripting/lexer.rs` | tokenization |
| Script diagnostics | `src/scripting/diagnostic.rs` | diagnostic/error representation |
| Script source locations | `src/scripting/source_map.rs` | source-map/line mapping |
| Project management | `src/project/project.rs` | `ProjectManager` |
| Recent projects | `src/project/recent.rs` | recent-project data |
| Open/save/play coordination | `src/engine/app.rs` | load/save/play/runtime coordination |

---

# 2. Top-Level Source Layout

```text
src/
├── main.rs
├── character/
├── character_custom/
├── editor/
├── engine/
├── project/
├── renderer/
├── scripting/
└── world/
```

`src/main.rs` is the application entry point and declares the major engine modules.

Relevant module declarations are at the top of `src/main.rs`.

---

# 3. Application / Engine Coordinator

## `src/main.rs`

**Purpose:** OS/window/OpenGL/egui startup and top-level event loop.

Important areas:

- Module declarations: lines 0-7.
- OpenGL/Winit setup: lines 28-91.
- `App` construction: lines 83-83.
- Window events routed into `App`: lines 105-128.
- Main engine tick: line 138.
- Renderer + egui UI pass: lines 167-199.

The application itself is coordinated by `src/engine/app.rs`.

---

## `src/engine/app.rs`

**Purpose:** Central application coordinator. Connects Editor, World, Renderer, PhysicsWorld, CharacterSystem, ScriptScene, EntityManager, project management, and input.

### `App`

Lines **14-57**

Owns:

- `Renderer`
- `Editor`
- `World`
- `ProjectManager`
- `PhysicsClock`
- `PhysicsWorld`
- `CharacterSystem`
- `GameplayCamera`
- saved editor camera
- `ScriptScene`
- `EntityManager`
- input state

### Input/event handling

`App::on_window_event`

Starts around line **116**.

Important control areas:

- Keyboard state tracking: around lines 121-143.
- Editor shortcuts: around lines 160-294.
- Jump request: around lines 286-291.
- Mouse input: lines 300+.
- Cursor/editor interaction: lines 361+.

### Editor build/select range operations

`apply_tool_to_range`

Lines **488-546**.

Build behavior:

- line 520: Build tool
- line 521: clones build template
- lines 523-528: writes the cell into the World

Selection and erase are in the same function.

### Main frame update

`App::update`

Starts at line **548**.

Major sections:

- Play-mode entry: lines 555-576.
- Play-mode exit: lines 577-589.
- Script update: lines 592-658.
- Physics sync and gameplay input: lines 660-690.
- Fixed physics step: lines 697-710.
- Gameplay camera follow: lines 714-717.
- Editor camera movement: lines 724-754.
- Save/exit requests: lines 756-772.
- Script startup: lines 776-805.
- Script shutdown/runtime cleanup: lines 806-818.
- `PlayerSpawned` event dispatch: lines 819-839.
- Exit to home/reset: lines 841-858.

### Project loading

`load_project`

Starts at line **1010**.

World file:

```text
<project>/world.dat
```

### Project saving

`save_project`

Starts at line **1049**.

This is the application-level entry point that writes the current project/world to disk.

---

# 4. World System

## Directory

```text
src/world/
├── mod.rs
├── cell.rs
├── coordinate.rs
├── persistence.rs
├── world.rs
└── block.rs
```

`src/world/mod.rs` exports:

- `Cell`
- `CellType`
- `WorldCoord`
- `World`

---

## `src/world/cell.rs`

**AUTHORITATIVE CELL DATA MODEL.**

This is the first place to look when changing what a Cell fundamentally contains.

### `CellType`

Lines **2-18**

Current variants:

- `Empty`
- `Block`
- `FxBlock`
- `Player`
- `NPC`
- `Light`
- `SpawnPoint`

### `Cell`

Lines **19-37**

Authoritative Cell fields include:

- `id`
- `cell_type`
- `visible`
- `solid`
- `anchored`
- `texture`
- `color_rgb`
- authored light properties
- `entity_identity`

### Runtime-only Cell state

`RuntimeCellState`

Lines **38-51**

Runtime overrides are intentionally separate from authored Cell data.

### Cell defaults

`Cell::default`

Lines **52-70**

### Cell constructors

- `Cell::new_block`: lines 71-82
- `Cell::new_light`: lines 83-100
- `Cell::new_spawn_point`: lines 102-113

**For a new persistent Cell field, start here.**

---

## `src/world/world.rs`

**AUTHORITATIVE WORLD CONTAINER AND CELL LOOKUP/EDITING API.**

### `World`

Lines **28-50**

Contains:

- authored `cells`
- runtime state
- physics dirty-cell IDs
- `id_to_coord`
- gravity
- lighting
- script bindings

### Basic Cell lookup

- `get`: lines 64-66
- `get_mut`: lines 68-70

### ID lookup

`resolve_cell_id`

Lines **72-75**

This resolves a persistent Cell ID to its current coordinate.

### Physics dirty marking

`mark_physics_dirty`

Lines **77-80**

### Runtime state

The effective-property getters/setters occupy approximately lines 81-215.

### Runtime state cleanup

`clear_runtime_state`

Lines **216-222**

### Cell creation/removal

`set_cell`

Lines **223-250**

This is the authoritative World path for creating/removing Cells.

### Cell ID index maintenance

- `update_id_mapping`: lines 252-256
- `rebuild_id_mapping`: lines 257-263

### Cell ID generation

`generate_unique_id`

Lines **265-297**

### Active Cells

`active_blocks`

Lines **299-302**

Despite the historical name, this returns coordinates from the World Cell map.

---

## `src/world/persistence.rs`

**AUTHORITATIVE WORLD FILE FORMAT.**

The engine currently uses a simple text world format.

### Save

`save_world`

Starts at **line 13**.

The serializer writes:

- gravity
- lighting
- script bindings
- Cell records by CellType

### Load

`load_world`

Starts at **line 174**.

It resets the World and parses the project world file.

**Any change to persistent Cell data must account for this file.**

---

## `src/world/coordinate.rs`

`WorldCoord`

Small value type representing integer voxel/world coordinates.

---

## `src/world/block.rs`

Block-specific world module.

---

# 5. Character System

## Directory

```text
src/character/
├── mod.rs
├── animation.rs
├── character.rs
├── collision.rs
├── movement.rs
├── spawning.rs
├── system.rs
└── transform.rs
```

There is **no `src/character/controller.rs` on the current main branch**.

For character control, use the following map instead.

---

## `src/character/system.rs`

**MAIN CHARACTER GAMEPLAY/CONTROL SYSTEM.**

### `CharacterSystem`

Lines **8-12**

Owns runtime Characters and runtime IDs.

### Spawn

`spawn_player`

Lines **21-30**

### Active characters

`get_active_characters`

Lines **32-35**

### Play-mode update

`CharacterSystem::update`

Lines **47-107**

This is the main character simulation path.

Order:

1. Horizontal movement: lines 49-59
2. Jumping: lines 60-66
3. Gravity: lines 67-69
4. Position integration: lines 70-72
5. Orientation: lines 73-78
6. Static collision: lines 79-83
7. Dynamic body collision: lines 84-85
8. Animation state: lines 86-100
9. Animation controller update: lines 101-104
10. Pose evaluation: lines 105-106

### Character/static collision

Lines **109-159**

### Character/dynamic-body collision

Lines **160-214**

### Character tests

Lines **216+**

---

## `src/character/movement.rs`

**MOVEMENT DATA + CORE MOVEMENT CONSTANTS.**

Lines:

- `MovementState`: 2-6
- `MOVE_SPEED`: 8
- `JUMP_IMPULSE`: 9
- `CharacterMovement`: 10-15
- defaults: 17-25

So:

```text
Character speed/jump tuning
→ src/character/movement.rs
```

Actual input-to-movement wiring:

```text
Keyboard input
→ src/engine/app.rs:666-690
→ CharacterSystem::update
→ src/character/system.rs:47-107
```

---

## `src/character/spawning.rs`

`spawn_at_random_point`

Lines **4-65**

Spawn-point discovery and spawn-position search.

`has_character_clearance`

Lines **67-91**

Checks the world for enough space around the character.

---

## `src/character/character.rs`

Core runtime `Character` data structure.

---

## `src/character/collision.rs`

Character collision dimensions/data.

---

## `src/character/animation.rs`

Basic high-level animation state:

- `Idle`
- `Walk`

---

## `src/character/transform.rs`

Character transform data:

- position
- rotation
- scale

---

# 6. Custom Character System

## Directory

```text
src/character_custom/
├── mod.rs
├── animation.rs
├── appearance.rs
├── blend.rs
├── collision.rs
├── geometry.rs
└── rig.rs
```

This is separate from the higher-level runtime `src/character/` system.

### `animation.rs`

**477 lines**

Animation primitives:

- `Keyframe`
- `TransformTrack`
- `AnimationClip`
- default skeleton generation
- Idle clip generation
- Walk clip generation

Useful ranges:

- Track evaluation: lines 3-59
- `AnimationClip`: lines 60-75
- default skeleton: lines 76-171
- Idle clip: lines 173-317
- Walk clip: lines 319-476

### `appearance.rs`

**44 lines**

Appearance customization and material slots.

### `blend.rs`

**105 lines**

`CharacterAnimationController` and animation blending/target selection.

### `collision.rs`

**21 lines**

Custom character collision representation.

### `geometry.rs`

**89 lines**

Character mesh generation.

### `rig.rs`

**77 lines**

Skeleton/rig/pose structures.

---

# 7. Physics

## `src/engine/physics.rs`

**RUNTIME PHYSICS SYSTEM.**

### `PhysicsWorld`

Lines **149-160**

Owns:

- dynamic bodies
- static colliders
- physics ID generator
- step state

### Initial world registration

`register_from_world`

Lines **572-608**

Builds runtime physics representation from the authored World.

### Incremental synchronization

`sync_with_world`

Starts at **line 613**.

This is the important location for change-driven synchronization between World state and runtime physics.

Dirty Cell IDs are drained starting at line **618**.

### Other major physics operations

The main simulation methods include:

- gravity application
- position integration
- dynamic collision resolution
- static collision resolution
- support/sleeping updates
- body management

For physics behavior, start with `PhysicsWorld` and then trace into `App::update` where the fixed-step order is executed.

---

# 8. Renderer

## Directory

```text
src/renderer/
├── mod.rs
├── camera.rs
├── mesh.rs
└── shader.rs
```

## `src/renderer/mod.rs`

**1701 lines**

Main OpenGL `Renderer`.

The `Renderer` structure begins at lines **20-77**.

Owns OpenGL resources such as:

- grid VAOs/VBOs
- selection/highlight geometry
- block geometry
- billboard geometry
- shadow framebuffer/depth texture
- texture cache
- fallback texture

Renderer creation starts at line **81**.

---

## `src/renderer/camera.rs`

**376 lines**

Contains the camera implementations.

Use this file for:

- editor camera behavior
- gameplay camera behavior
- orbit/zoom
- follow behavior
- camera basis calculations
- camera collision

`GameplayCamera` is the runtime third-person camera.

---

## `src/renderer/mesh.rs`

**137 lines**

OpenGL vertex/mesh construction and upload helpers.

---

## `src/renderer/shader.rs`

**34 lines**

Shader/program creation helpers and shader source.

---

# 9. Editor

## Directory

```text
src/editor/
├── mod.rs
├── editor.rs
├── tools.rs
├── navigation.rs
├── script_editor.rs
└── grid/
    ├── mod.rs
    ├── grid.rs
    └── picking.rs
```

---

## `src/editor/editor.rs`

**1173 lines**

Main editor state and editor UI.

### `Editor`

Lines **47-86**

Contains:

- editor mode
- camera
- selection
- tool state
- properties/navigation windows
- script workspace
- save/exit flags
- build template
- undo/redo history
- terminal output

### Default editor behavior

`Editor::new`

Lines **87-120**

Current defaults include:

- `current_tool = Select`
- `plane_picking = false`

### Main UI

`show_ui`

Lines **127-150**

### Properties/navigation panel layout

`draw_right_panel`

Starts around **line 267**.

### Property change model

`PropertyChange`

Lines **369-381**

Current engine properties include:

- light color/intensity/range/shadows
- solid
- anchored
- visible
- color
- texture
- entity identity

### Property application

`apply_property_changes`

Lines **383-405**

### Properties panel contents

`render_properties_content`

Starts at **line 406**.

This is the primary editor location for the selected Cell Properties panel.

The current panel includes:

- Identity
- ID
- Cell type
- entity identity
- Script attachment/status
- other property sections further down the function

**Future Cell Attributes UI belongs in this same Properties system.**

---

## `src/editor/tools.rs`

**259 lines**

Toolbar and reusable editor property controls.

### Toolbar

`draw_tool_bar`

Lines **124-244**

Includes:

- Script workspace
- Editor/Play mode
- tool selection
- plane picking
- build template settings
- rendering controls
- physics controls

### Color editing

`draw_color_edit`

Lines **7-54**

### Texture editing/import

`draw_texture_edit`

Lines **57-123**

---

## `src/editor/navigation.rs`

Navigation window state and coordinate text buffers.

`NavigationWindow` begins at line **6**.

---

## `src/editor/grid/grid.rs`

Grid geometry and grid-plane selection.

- `GRID_RADIUS`: line 3
- `generate_grid_vertices`: line 5
- `select_grid_plane`: line 41

---

## `src/editor/grid/picking.rs`

World picking/raycasting.

### Plane hover

`update_hover`

Starts at line **5**.

### Voxel raycast

`raycast_world`

Starts at line **60**.

Uses voxel DDA traversal.

---

## `src/editor/script_editor.rs`

**939 lines**

AeoScript editor workspace.

### `ScriptDocument`

Lines **9-70**

Tracks:

- source
- original source
- dirty state
- diagnostics
- parsed program
- source map

### `ScriptEditor`

Lines **72-91**

Tracks open scripts, active document, search, dialogs, diagnostics, output.

### Script discovery

`refresh_scripts`

Lines **119-133**

Scripts are loaded from:

```text
<project>/scripts/*.aeo
```

### File opening

`open_file`

Lines **134-142**

### Saving

`save_active`: lines 156-164

`save_all`: lines 165-173

### Script creation

`create_new_script`

Lines **175-215**

---

# 10. AeoScript

## Directory

```text
src/scripting/
├── mod.rs
├── ast.rs
├── diagnostic.rs
├── execution.rs
├── interpreter.rs
├── lexer.rs
├── parser.rs
├── source.rs
├── source_map.rs
├── token.rs
├── value.rs
├── runtime.rs
├── scene.rs
├── api.rs
├── binding.rs
├── log.rs
└── stdlib.rs
```

---

## `src/scripting/runtime.rs`

**690 lines**

Runtime owner for live script execution.

### `ScriptRuntime`

Lines **17-27**

Owns:

- interpreter
- scheduler
- fibers
- event handlers

### Runtime tick

`tick`

Starts at **line 171**.

Each ready task receives an execution slice.

### Fiber storage

`spawn`: lines 145-164

`remove_fiber`: lines 214-223

---

## `src/scripting/execution.rs`

**548 lines**

Cooperative script scheduler.

### `ScriptTaskId`

Lines **2-17**

### `YieldReason`

Lines **19-42**

### `ScriptTaskState`

Lines **45-53**

### `FiberResult`

Lines **54-64**

### `ScriptScheduler`

Lines **79-276**

Key methods:

- `spawn`
- `state`
- `begin_running`
- `apply_result`
- `tick`
- `pop_ready`
- `cancel`

`wait()` scheduling behavior ultimately flows through this system.

---

## `src/scripting/interpreter.rs`

**3818 lines**

**Main AeoScript VM/interpreter implementation.**

### VM execution budget

Lines **10-11**

Current defaults:

- 100,000 operations
- max call depth 64

### `ScriptInstance`

Lines **12-23**

Script entity fields live here.

For interpreter/runtime work, this is the primary file.

---

## `src/scripting/value.rs`

**483 lines**

AeoScript runtime value model.

### Engine handle kinds

Lines **4-23**

Current handle kinds:

- `Entity`
- `Light`
- `Cell`

### Map keys

Lines **53-71**

### Basket runtime data

Lines **73-77**

### `Value`

Lines **79-95**

Current runtime values:

- Number
- Bool
- String
- Nil
- Basket/Array
- Map
- opaque engine Handle

Handles carry an identifier rather than a Rust object pointer.

---

## `src/scripting/api.rs`

**341 lines**

Engine-to-script host boundary.

### `EngineHost`

Lines **4-24**

This is the key API boundary for exposing engine functionality to AeoScript.

Important operations include:

- entity manager access
- position get/set
- light access
- `get_all_cells_of_class`
- `find_objects`
- children/parent
- Cell object lookup
- property get/set
- method calls

**For exposing a new engine object property/API to AeoScript, start here.**

---

## `src/scripting/scene.rs`

**2361 lines**

AeoScript scene/runtime integration.

This is the large orchestration layer connecting:

- authored script bindings
- script instances
- lifecycle functions
- runtime fibers
- world/entity handles
- scheduler results
- persistent lifecycle state

**For lifecycle behavior or engine-to-script object synchronization, start here.**

---

## `src/scripting/binding.rs`

**67 lines**

Persistent authored script binding.

`ScriptBinding` lines **8-13**:

- `target_identity: u64` — persistent Cell ID
- `script_path: String`

This is the authored binding model.

---

## `src/scripting/stdlib.rs`

**310 lines**

Standard-library dispatch.

`call_stdlib_function` starts at line **7**.

Current namespace dispatch includes math/basket/string functionality.

---

## `src/scripting/ast.rs`

**273 lines**

Parsed AeoScript syntax tree.

`Program` and `Declaration` begin at lines **2-13**.

Entity declarations and members begin at lines **25-36**.

Statements begin around line **100**.

---

## `src/scripting/parser.rs`

**1395 lines**

Converts lexer tokens into the AeoScript AST.

Use this for language grammar/parser changes.

---

## `src/scripting/lexer.rs`

**566 lines**

Converts script source text into tokens.

Use this for tokenization/lexical syntax changes.

---

## `src/scripting/diagnostic.rs`

**173 lines**

Structured script diagnostics.

---

## `src/scripting/source_map.rs`

**158 lines**

Maps script source offsets/lines for diagnostics/editor integration.

---

## `src/scripting/source.rs`

Source spans/locations.

---

## `src/scripting/token.rs`

AeoScript token definitions.

---

## `src/scripting/log.rs`

Script/runtime logging records and severity.

---

# 11. Entity / Handle Infrastructure

## `src/engine/entity.rs`

**201 lines**

Runtime Entity identity and entity-manager infrastructure.

AeoScript `Entity` handles eventually resolve through this runtime system.

Use this together with:

```text
src/scripting/value.rs
src/scripting/api.rs
src/scripting/scene.rs
```

when changing engine-object handle behavior.

---

# 12. Project System

## Directory

```text
src/project/
├── mod.rs
├── project.rs
└── recent.rs
```

## `src/project/project.rs`

**192 lines**

`ProjectManager`.

Responsible for project-level lifecycle, opening/creating project state, and current project path management.

## `src/project/recent.rs`

Recent-project tracking.

---

# 13. Character Change Paths

This is the canonical "where do I go?" map for character work.

### Change movement speed

```text
src/character/movement.rs
    MOVE_SPEED
```

### Change jump strength

```text
src/character/movement.rs
    JUMP_IMPULSE
```

### Change WASD input mapping

```text
src/engine/app.rs
    App::update
    lines 666-690
```

### Change what happens after movement input is received

```text
src/character/system.rs
    CharacterSystem::update
    lines 47-107
```

### Change grounded/floor/wall/ceiling collision

```text
src/character/system.rs
    resolve_static_voxel_collisions
    lines 109-159
```

### Change character vs dynamic physics bodies

```text
src/character/system.rs
    resolve_dynamic_body_collisions
    lines 160-214
```

### Change spawn selection/clearance

```text
src/character/spawning.rs
    spawn_at_random_point
    lines 4-65

src/character/spawning.rs
    has_character_clearance
    lines 67-91
```

### Change Idle/Walk state selection

```text
src/character/system.rs
    CharacterSystem::update
    lines 86-100
```

### Change animation clips

```text
src/character_custom/animation.rs
    create_idle_clip
    lines 173-317

src/character_custom/animation.rs
    create_walk_clip
    lines 319-476
```

### Change animation blending/controller

```text
src/character_custom/blend.rs
```

### Change rig/skeleton

```text
src/character_custom/rig.rs
```

### Change generated character geometry

```text
src/character_custom/geometry.rs
```

### Change appearance/material customization

```text
src/character_custom/appearance.rs
```

### Change third-person gameplay camera

```text
src/renderer/camera.rs
```

### Change the full Play-mode character pipeline

```text
src/engine/app.rs
    App::update
    lines 555-722

and

src/character/system.rs
    CharacterSystem::update
```

---

# 14. Cell / World Change Paths

### Add/change a fundamental Cell field

```text
src/world/cell.rs
    Cell
    lines 19-37
```

### Change default Cell behavior

```text
src/world/cell.rs
    Cell::default
    lines 52-70
```

### Change block defaults

```text
src/world/cell.rs
    Cell::new_block
    lines 71-82
```

### Change world Cell creation/deletion

```text
src/world/world.rs
    World::set_cell
    lines 223-250
```

### Change coordinate lookup

```text
src/world/world.rs
    World::get
    World::get_mut
    lines 64-70
```

### Change persistent Cell-ID lookup

```text
src/world/world.rs
    resolve_cell_id
    lines 72-75

id_to_coord maintenance:
    lines 252-263
```

### Change saved world format

```text
src/world/persistence.rs
    save_world
    line 13+

    load_world
    line 174+
```

### Change selected Cell Properties UI

```text
src/editor/editor.rs
    render_properties_content
    line 406+
```

### Change how Property edits are represented/applied

```text
src/editor/editor.rs
    PropertyChange
    lines 369-381

    apply_property_changes
    lines 383-405
```

---

# 15. Physics Change Paths

### Change a PhysicsBody field/behavior

```text
src/engine/physics.rs
    PhysicsBody
```

### Change initial conversion from authored World to runtime physics

```text
src/engine/physics.rs
    PhysicsWorld::register_from_world
    lines 572-608
```

### Change change-driven world/physics synchronization

```text
src/engine/physics.rs
    PhysicsWorld::sync_with_world
    lines 613+
```

### Change the order of the runtime physics simulation

```text
src/engine/app.rs
    App::update
    lines 697-710
```

Current fixed-step order there is:

```text
apply gravity
→ integrate positions
→ resolve dynamic collisions
→ resolve static collisions
→ refresh dynamic support
→ update sleeping
→ CharacterSystem::update
```

---

# 16. Editor Change Paths

### Add/change an editor state value

```text
src/editor/editor.rs
    Editor
    lines 47-86
```

### Change editor defaults

```text
src/editor/editor.rs
    Editor::new
    lines 87-120
```

### Change Properties panel layout

```text
src/editor/editor.rs
    draw_right_panel
    lines 267+
```

### Change selected Cell property UI

```text
src/editor/editor.rs
    render_properties_content
    lines 406+
```

### Change editor toolbar

```text
src/editor/tools.rs
    draw_tool_bar
    lines 124-244
```

### Change world/grid picking

```text
src/editor/grid/picking.rs
    update_hover
    lines 5-58

    raycast_world
    lines 60-140
```

### Change grid appearance/planes

```text
src/editor/grid/grid.rs
    generate_grid_vertices
    select_grid_plane
```

### Change Navigation window

```text
src/editor/navigation.rs
```

### Change script editor

```text
src/editor/script_editor.rs
    ScriptDocument
    lines 9-70

    ScriptEditor
    lines 72-91

    refresh_scripts
    lines 119-133
```

---

# 17. AeoScript Change Paths

### Add a new runtime value type

```text
src/scripting/value.rs
```

### Change engine handle kinds

```text
src/scripting/value.rs
    HandleKind
    lines 4-23
```

### Add/change an engine-exposed property

```text
src/scripting/api.rs
    EngineHost
    lines 4-24

Then trace the concrete implementation/bridge in the engine.
```

### Add/change object discovery

```text
src/scripting/api.rs
    get_all_cells_of_class
    find_objects

and

src/scripting/scene.rs
```

### Add/change script properties

```text
src/scripting/api.rs
    get_property
    set_property
```

### Add/change script methods

```text
src/scripting/api.rs
    call_method
```

### Change fibers/wait/scheduling

```text
src/scripting/execution.rs
src/scripting/runtime.rs
```

### Change script execution behavior

```text
src/scripting/interpreter.rs
```

### Change script lifecycle/scene behavior

```text
src/scripting/scene.rs
```

### Change script binding identity

```text
src/scripting/binding.rs
src/world/world.rs
src/world/persistence.rs
src/editor/editor.rs
```

### Add/change standard library functionality

```text
src/scripting/stdlib.rs
```

### Change parser grammar

```text
src/scripting/parser.rs
src/scripting/ast.rs
```

### Change lexical/token rules

```text
src/scripting/lexer.rs
src/scripting/token.rs
```

### Change diagnostics/source locations

```text
src/scripting/diagnostic.rs
src/scripting/source.rs
src/scripting/source_map.rs
```

---

# 18. Runtime Flow

The main runtime ownership flow is:

```text
src/main.rs
    ↓
src/engine/app.rs
    ↓
┌──────────────────────────────────────────┐
│ App                                      │
│                                          │
│ World                                    │
│ Editor                                   │
│ Renderer                                 │
│ PhysicsWorld                             │
│ CharacterSystem                          │
│ ScriptScene                               │
│ EntityManager                             │
└──────────────────────────────────────────┘
```

Play mode begins in:

```text
src/engine/app.rs
App::update
lines 555-576
```

and initializes:

```text
PhysicsWorld
CharacterSystem
ScriptScene
EntityManager
```

Each frame:

```text
ScriptScene update
    ↓
PhysicsWorld sync
    ↓
input → camera-relative movement
    ↓
fixed physics step
    ↓
CharacterSystem update
    ↓
GameplayCamera update
```

---

# 19. Authored vs Runtime Data

The important authority boundary is:

```text
AUTHORED
src/world/
    World
    Cell
    script_bindings
    world.dat

          ↓ runtime conversion

RUNTIME
src/engine/physics.rs
    PhysicsWorld

src/character/
    CharacterSystem

src/scripting/
    ScriptScene
    ScriptRuntime
    fibers
    ScriptInstance

src/engine/entity.rs
    EntityManager
```

When changing a persistent world concept, start in `src/world/`.

When changing a temporary Play-mode representation, start in the corresponding runtime subsystem.

---

# 20. Persistence Map

Project-level save/load coordination:

```text
src/engine/app.rs
    load_project
    save_project

        ↓

src/world/persistence.rs
    load_world
    save_world
```

World data lives in the project as:

```text
<project>/world.dat
```

Scripts are discovered under:

```text
<project>/scripts/*.aeo
```

Textures are imported into:

```text
.assets/textures/
```

---

# 21. Current Attribute-System Landing Points

For the Attribute system planned for the current development task, the authoritative existing locations are:

```text
DATA MODEL
src/world/cell.rs
    Cell
    lines 19-37

PERSISTENCE
src/world/persistence.rs
    save_world
    load_world

EDITOR
src/editor/editor.rs
    render_properties_content
    lines 406+

SCRIPT EXPOSURE (implemented)
src/scripting/value.rs
src/scripting/api.rs
src/scripting/scene.rs
src/scripting/host.rs (bridge implementation)
```

The intended eventual flow is:

```text
Cell.attributes
    ↓
world.dat persistence
    ↓
Properties panel
    ↓
Cell handle / EngineHost
    ↓
AeoScript
```

Do not create a second Cell data store for attributes.

---

# 22. Tests

The repository has a top-level:

```text
tests/
```

directory.

In addition, many systems keep focused unit tests directly inside their implementation files under `#[cfg(test)]`.

High-value current test locations include:

```text
src/character/mod.rs
src/character/system.rs
src/scripting/execution.rs
src/scripting/runtime.rs
src/scripting/binding.rs
```

When adding a focused regression test, prefer the existing test module for the subsystem being changed unless a repository-level integration test is specifically required.

---

# 23. Documentation

Current documentation is organized under:

```text
docs/
├── README.md
├── architecture/
├── development/
├── guides/
└── language/
```

High-level areas:

```text
docs/architecture/
    engine
    world
    runtime
    editor
    physics
    characters
    rendering
    assets
    persistence

docs/development/
    building
    testing
    debugging
    contributing
    releases

docs/guides/
    getting started
    building a game
    world building
    physics gameplay
    character gameplay
    scripting
    textures
    debugging

docs/language/
    AeoScript overview/API/editor/grammar/VM
    types
    standard library
    handles
    lifecycle
    examples
```

---

# 24. Working-Tree / Agent Rule

This file is a navigation map, not an architecture proposal.

When using an AI coding tool:

1. Use this map to give the tool the exact file and symbol to change.
2. Do not ask the tool to discover the architecture when the relevant location is already mapped here.
3. Keep unrelated changes out of the task.
4. Update `MAP.md` when a system is moved, split, renamed, or its important locations substantially change.

The goal is simple:

```text
"I want to change X."

→ Find X in MAP.md.

→ Open the exact file/symbol.

→ Give the AI the exact location.

→ Change only that system.
```
