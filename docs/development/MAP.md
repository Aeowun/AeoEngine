# AeoEngine Project Map

**Repository:** `https://github.com/Aeowun/AeoEngine`
**Primary branch:** `main`

This document is a developer navigation map for the AeoEngine source tree.

It answers:

> Where does this system live, what does it own, and what other systems does it connect to?

This file intentionally avoids line numbers. Line numbers become stale quickly as files grow or are reorganized. File paths, module names, and important symbols are the more durable navigation points.

Update this document when a subsystem is moved, split, renamed, or its ownership changes significantly.

---

# 1. Fast Lookup

Use this section first when locating a system.

| Area | Start Here | Main Files |
|---|---|---|
| Application coordinator | `App` | `src/engine/app.rs` |
| Project management | `ProjectManager` | `src/project/project.rs` |
| Authored Cell data | `Cell` | `src/world/cell.rs` |
| World container | `World` | `src/world/world.rs` |
| World persistence | `WorldStorage` / persistence | `src/world/storage.rs`, `src/world/persistence.rs` |
| World coordinates | `WorldCoord` | `src/world/coordinate.rs` |
| Cell / World tests | `#[cfg(test)]` modules | `src/world/` |
| Editor state | `Editor` | `src/editor/editor.rs` |
| Editor tools | toolbar/tool state | `src/editor/tools.rs` |
| World picking | raycast / grid picking | `src/editor/grid/picking.rs` |
| Grid | grid generation / plane selection | `src/editor/grid/grid.rs` |
| Navigation | `NavigationWindow` | `src/editor/navigation.rs` |
| Script editor | `ScriptEditor` | `src/editor/script_editor.rs` |
| Renderer | `Renderer` | `src/renderer/mod.rs` |
| Renderer camera | `CameraController`, `GameplayCamera` | `src/renderer/camera.rs` |
| Render meshes | mesh construction/upload | `src/renderer/mesh.rs` |
| Shaders | shader/program helpers | `src/renderer/shader.rs` |
| Renderer benchmarks | large-world performance tests | renderer test modules / benchmark tests |
| Physics | `PhysicsWorld` | `src/engine/physics.rs` |
| Character runtime | `CharacterSystem` | `src/character/system.rs` |
| Character movement | `CharacterMovement`, movement constants | `src/character/movement.rs` |
| Character spawning | spawn logic | `src/character/spawning.rs` |
| Character collision | character collision paths | `src/character/system.rs`, `src/character/collision.rs` |
| Character animation | animation state | `src/character/animation.rs`, `src/character_custom/` |
| Character geometry | generated character mesh | `src/character_custom/geometry.rs` |
| Character rig | skeleton / pose | `src/character_custom/rig.rs` |
| Character appearance | materials / customization | `src/character_custom/appearance.rs` |
| AeoScript runtime | `ScriptRuntime` | `src/scripting/runtime.rs` |
| Script scheduling | `ScriptScheduler` | `src/scripting/execution.rs` |
| Script execution | interpreter / VM | `src/scripting/interpreter.rs` |
| Script scene integration | `ScriptScene` | `src/scripting/scene.rs` |
| Engine-to-script API | `EngineHost` | `src/scripting/api.rs` |
| Concrete script bridge | AeoEngine host implementation | `src/scripting/host.rs` |
| Script values / handles | `Value`, `HandleKind` | `src/scripting/value.rs` |
| Script bindings | `ScriptBinding` | `src/scripting/binding.rs` |
| Standard library | stdlib dispatch | `src/scripting/stdlib.rs` |
| Parser | parser implementation | `src/scripting/parser.rs` |
| Lexer | tokenization | `src/scripting/lexer.rs` |
| AST | syntax tree | `src/scripting/ast.rs` |
| Script diagnostics | diagnostics | `src/scripting/diagnostic.rs` |
| Script source mapping | source locations | `src/scripting/source.rs`, `src/scripting/source_map.rs` |
| Runtime entities | `EntityManager` | `src/engine/entity.rs` |
| Tests | unit/integration tests | `src/**`, `tests/` |
| Build / test workflow | development commands | `docs/development/BUILDING.md` |
| Debugging workflow | debugging and external tools | `docs/development/DEBUGGING.md` |
| Architecture docs | subsystem architecture | `docs/architecture/` |

---

# 2. Source Tree

The major source areas are:

~~~text
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
~~~

The source tree is organized primarily by subsystem ownership.

---

# 3. Application Layer

## `src/main.rs`

The application entry point.

Responsible for:

* Window creation.
* Winit event handling.
* OpenGL context setup.
* egui setup.
* Creating the application coordinator.
* Routing events and frames into `App`.

The main engine/application state is owned by:

~~~text
src/engine/app.rs
~~~

---

## `src/engine/app.rs`

`App` is the central application coordinator.

It connects the major subsystems:

~~~text
App
├── ProjectManager
├── World
├── Editor
├── Renderer
├── PhysicsClock
├── PhysicsWorld
├── CharacterSystem
├── GameplayCamera
├── ScriptScene
└── EntityManager
~~~

Use `App` when investigating:

* Editor ↔ runtime transitions.
* Home ↔ Editor transitions.
* Play-mode entry and exit.
* Per-frame coordination.
* Input routing.
* Project open/save coordination.
* The order in which major runtime systems execute.

`App` coordinates systems but should not become the implementation owner of unrelated subsystem behavior.

---

# 4. World System

The World system is the authoritative owner of persistent authored scene data.

## Directory

~~~text
src/world/
├── mod.rs
├── cell.rs
├── coordinate.rs
├── persistence.rs
├── storage.rs
├── world.rs
└── block.rs
~~~

The exact file set may grow as World responsibilities are separated further.

---

## `src/world/cell.rs`

The authoritative Cell data model.

### `CellType`

Defines the kinds of Cells represented by the World.

Current Cell types include:

* `Empty`
* `Block`
* `FxBlock`
* `Player`
* `NPC`
* `Light`
* `SpawnPoint`

### `Cell`

The persistent authored representation of an individual World Cell.

This is the first place to inspect when changing:

* Cell fields.
* Cell defaults.
* Authored visibility.
* Solidity.
* Anchoring.
* Texture references.
* Color.
* Light properties.
* Entity identity.
* Cell attributes.
* Other properties that are fundamentally part of saved World data.

### Runtime Cell state

Runtime overrides belong in the runtime state layer rather than becoming alternate authored Cell fields.

The important distinction is:

~~~text
Cell
→ authored persistent values

RuntimeCellState
→ temporary Play-mode state
~~~

When adding a new persistent Cell concept, start here and then trace its persistence, editor, runtime, and scripting consumers.

---

## `src/world/world.rs`

The main World container and World editing API.

`World` owns the authored Cell collection and associated World-level state.

Use this file for:

* Cell lookup.
* Cell mutation.
* Cell creation and removal.
* Persistent Cell-ID resolution.
* ID index maintenance.
* Runtime Cell state associated with the World.
* Physics-dirty tracking.
* World-level configuration.
* Effective authored/runtime Cell access.

Important conceptual paths include:

~~~text
coordinate
    ↓
World::get / World::get_mut
    ↓
Cell
~~~

and:

~~~text
persistent Cell ID
    ↓
World::resolve_cell_id
    ↓
current World coordinate
~~~

The World remains the source of truth for persistent scene data.

---

## `src/world/coordinate.rs`

Defines World coordinate types and coordinate-related behavior.

Use this file when changing the representation or semantics of voxel/world positions.

---

## `src/world/storage.rs`

The persistence abstraction for World storage.

`WorldStorage` is responsible for persistence operations such as:

* Loading stored World data.
* Saving World data.
* Checking whether stored World data exists.
* Removing stored World data when explicitly required.

The storage layer should not own:

* Rendering.
* Physics.
* Character simulation.
* Script execution.
* Editor interaction.

The architectural relationship is:

~~~text
World
  ↕
WorldStorage
  ↕
Persistent project data
~~~

---

## `src/world/persistence.rs`

Contains the current World file serialization/deserialization implementation.

Use this file when changing:

* The on-disk World format.
* World save/load parsing.
* Backward compatibility with existing World files.
* Serialization of authored Cell data.
* Persistence tests.

The persistence format is currently text-based.

The World persistence path is:

~~~text
App / Project
      ↓
World
      ↓
WorldStorage / persistence implementation
      ↓
world.dat
~~~

A persistence change should normally include a test for both writing and reading the affected data.

---

## `src/world/block.rs`

Block-specific World definitions and behavior.

Use this when the change is specifically about Block data rather than the general Cell model.

---

# 5. Editor

## Directory

~~~text
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
~~~

The editor owns authoring interaction and editor-only state.

It does not become the owner of runtime simulation state.

---

## `src/editor/editor.rs`

Main editor state and editor UI.

`Editor` owns state such as:

* Editor mode.
* Selection.
* Editor camera state.
* Active tool.
* Build template.
* Undo/redo history.
* Properties state.
* Navigation state.
* Script workspace state.
* Editor UI state.
* Terminal/output state.

Use this file for:

* Properties panel behavior.
* Selection behavior.
* Editor history.
* Editor-specific state.
* World-editing UI.
* Script binding UI.
* Editor/runtime mode presentation.

---

## `src/editor/tools.rs`

Editor toolbar and reusable editor controls.

Use this for:

* Tool selection.
* Build/Erase controls.
* Plane-picking controls.
* Build template configuration.
* Rendering-related editor controls.
* Physics-related editor controls.
* Shared property widgets.

---

## `src/editor/grid/grid.rs`

Editor grid representation and grid-plane selection.

Use this for:

* Grid geometry.
* Grid appearance.
* Grid plane selection.
* Grid-related editor state.

---

## `src/editor/grid/picking.rs`

World picking and viewport raycasting.

Use this for:

* Plane picking.
* World raycasts.
* Top-Block picking.
* Voxel traversal.
* Determining which authored Cell or face the viewport is targeting.

The result of picking is an editor target. Picking itself does not own World persistence or runtime simulation.

---

## `src/editor/navigation.rs`

Navigation UI and navigation-specific state.

Use this for editor navigation panels and coordinate navigation behavior.

---

## `src/editor/script_editor.rs`

Integrated AeoScript editor.

Use this for:

* Script discovery.
* Open script documents.
* Editing.
* Saving.
* Diagnostics presentation.
* Script search.
* Script editor UI.

The Script Editor works with AeoScript source files but does not own the AeoScript runtime.

---

# 6. Renderer

## Directory

~~~text
src/renderer/
├── mod.rs
├── camera.rs
├── mesh.rs
└── shader.rs
~~~

The renderer owns graphics representation and GPU resources.

It does not own authored World state.

The architectural flow is:

~~~text
World / Runtime State
        ↓
Renderer
        ↓
OpenGL
        ↓
Screen
~~~

---

## `src/renderer/mod.rs`

Main `Renderer` implementation.

Use this file for:

* OpenGL rendering operations.
* Static World rendering.
* Runtime object rendering.
* Lighting.
* Sky environment.
* Editor overlays.
* Render chunk management.
* GPU resource ownership.
* Render-state preparation.
* Chunk rebuild behavior.

The renderer maintains derived graphics state such as:

* Render chunks.
* Chunk meshes.
* GPU buffers.
* Texture resources.
* Shadow geometry.
* Other graphics resources.

These are runtime resources, not persistent World data.

---

## Spatial chunk rendering

The current voxel renderer uses spatial chunks.

Conceptually:

~~~text
World
  ↓
Effective Cell state
  ↓
Spatial chunks
  ↓
Chunk meshes
  ↓
GPU resources
  ↓
Camera visibility
  ↓
Draw submission
~~~

Important renderer responsibilities include:

* Chunk construction.
* Exposed-face evaluation.
* Cross-chunk neighbor checks.
* Texture batching.
* Shadow geometry generation.
* Selective rebuilding of affected chunks.
* Camera frustum culling.
* Persistent reuse of unchanged GPU resources.

A localized World edit should not require rebuilding unrelated render chunks.

---

## `src/renderer/camera.rs`

Contains camera implementations.

Current responsibilities include:

* Editor camera.
* Gameplay camera.
* Orbit behavior.
* Follow behavior.
* Camera movement.
* Camera basis calculations.
* Camera collision-related behavior.

When changing gameplay camera behavior, start with `GameplayCamera`.

When changing editor navigation camera behavior, start with `CameraController`.

---

## `src/renderer/mesh.rs`

Mesh construction and GPU upload helpers.

Use this when changing:

* Vertex formats.
* Mesh upload.
* Runtime mesh construction.
* OpenGL mesh resource creation.

---

## `src/renderer/shader.rs`

Shader/program creation and shader source.

Use this for shader-related changes.

---

# 7. Physics

## `src/engine/physics.rs`

Main runtime physics implementation.

`PhysicsWorld` owns:

* Dynamic physics bodies.
* Static collision representation.
* Physics IDs.
* Physics stepping state.
* Runtime collision relationships.

Important paths include:

### Initial World registration

~~~text
Authored World
      ↓
PhysicsWorld registration
      ↓
Static collision / runtime physics state
~~~

### Incremental synchronization

~~~text
World change
      ↓
Physics dirty state
      ↓
PhysicsWorld synchronization
      ↓
Affected runtime collision state
~~~

Use `PhysicsWorld::register_from_world` when changing initial World-to-physics conversion.

Use `PhysicsWorld::sync_with_world` when changing ongoing World/physics synchronization.

Use the physics step implementation when changing:

* Gravity.
* Integration.
* Dynamic collision.
* Static collision.
* Support/grounding.
* Sleeping.
* Body lifecycle.

The physics system owns runtime physical state, not authored World data.

---

# 8. Character System

## Directory

~~~text
src/character/
├── mod.rs
├── animation.rs
├── character.rs
├── collision.rs
├── movement.rs
├── spawning.rs
├── system.rs
└── transform.rs
~~~

Character runtime simulation is separate from the custom character asset/animation implementation under `src/character_custom/`.

---

## `src/character/system.rs`

Main runtime character simulation.

`CharacterSystem` owns live runtime character state.

Use this file for:

* Character update order.
* Movement application.
* Jump handling.
* Gravity.
* Character collision.
* Character/dynamic-body interaction.
* Animation-state selection.
* Runtime character updates.

The general flow is:

~~~text
Input
  ↓
CharacterSystem
  ↓
Movement / Jump / Gravity
  ↓
Collision
  ↓
Animation state
  ↓
Pose evaluation
~~~

---

## `src/character/movement.rs`

Movement state and movement constants.

Use this file for:

* Movement speed.
* Jump impulse.
* Movement state.
* Character movement parameters.

Input wiring itself is handled by the application layer.

The normal path is:

~~~text
Keyboard / input state
      ↓
src/engine/app.rs
      ↓
CharacterSystem
      ↓
src/character/movement.rs
~~~

---

## `src/character/spawning.rs`

Character spawn logic.

Use this for:

* Spawn-point discovery.
* Spawn position selection.
* Clearance checks.
* Spawn-related validation.

---

## `src/character/character.rs`

Core runtime Character data.

---

## `src/character/collision.rs`

Character collision representation and collision-specific data.

---

## `src/character/animation.rs`

High-level runtime character animation state.

---

## `src/character/transform.rs`

Character position, rotation, and scale data.

---

# 9. Custom Character System

## Directory

~~~text
src/character_custom/
├── mod.rs
├── animation.rs
├── appearance.rs
├── blend.rs
├── collision.rs
├── geometry.rs
└── rig.rs
~~~

This subsystem contains lower-level character representation and animation infrastructure.

---

## `src/character_custom/animation.rs`

Animation primitives and clip data.

Use this for:

* Keyframes.
* Transform tracks.
* Animation clips.
* Procedural/default animation clips.
* Animation evaluation.

---

## `src/character_custom/blend.rs`

Animation controller and blending behavior.

---

## `src/character_custom/rig.rs`

Skeleton, rig, and pose structures.

---

## `src/character_custom/geometry.rs`

Generated character geometry.

Use this for character mesh generation.

---

## `src/character_custom/appearance.rs`

Character material slots and appearance/customization data.

---

## `src/character_custom/collision.rs`

Custom character collision representation.

---

# 10. AeoScript

## Directory

~~~text
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
├── host.rs
├── log.rs
└── stdlib.rs
~~~

AeoScript is split into language implementation, runtime execution, engine integration, and editor-facing diagnostics.

---

## `src/scripting/lexer.rs`

Converts source text into tokens.

Use this for lexical/tokenization changes.

---

## `src/scripting/parser.rs`

Converts tokens into the AeoScript AST.

Use this for grammar and parsing behavior.

---

## `src/scripting/ast.rs`

Defines the syntax tree representation.

Use this when adding or changing language-level syntax structures.

---

## `src/scripting/value.rs`

Defines runtime AeoScript values and engine handles.

Important concepts include:

* Numbers.
* Booleans.
* Strings.
* Nil.
* Baskets/arrays.
* Maps.
* Engine handles.

`HandleKind` defines the supported engine object handle categories.

Use this when changing the runtime value model or adding a new handle type.

---

## `src/scripting/interpreter.rs`

Main AeoScript VM/interpreter implementation.

Use this for:

* Expression execution.
* Statement execution.
* Function calls.
* Runtime evaluation.
* Call-stack behavior.
* Execution semantics.

This is the main execution core and should be changed carefully.

---

## `src/scripting/execution.rs`

Cooperative scheduling and fibers.

Use this for:

* Script task state.
* Yielding.
* `wait()`.
* Scheduling.
* Fiber lifecycle.
* Task cancellation.

---

## `src/scripting/runtime.rs`

Owns live script execution at the runtime level.

Use this for:

* Runtime script instances.
* Runtime ticking.
* Fiber ownership.
* Runtime event execution.

---

## `src/scripting/scene.rs`

The main scripting-to-engine orchestration layer.

Use this for:

* Script bindings.
* Script lifecycle.
* Script instance creation.
* Scene/runtime synchronization.
* Engine object handles.
* Runtime script state.
* Integration between the script runtime and World/entities.

---

## `src/scripting/api.rs`

Defines the engine-facing AeoScript API boundary.

`EngineHost` is the key abstraction between the language runtime and AeoEngine.

Use this when exposing engine functionality such as:

* Cell properties.
* Cell discovery.
* Entity access.
* Position access.
* Engine object lookup.
* Runtime property mutation.
* Engine methods.

---

## `src/scripting/host.rs`

Concrete AeoEngine implementation of the script host boundary.

When adding an API, the normal path may cross:

~~~text
AeoScript syntax / stdlib
      ↓
EngineHost abstraction
      ↓
AeoEngine host implementation
      ↓
World / Entity / Runtime subsystem
~~~

---

## `src/scripting/binding.rs`

Persistent script binding representation.

Script bindings reference persistent Cell identities rather than relying on temporary runtime object identities.

Use this when changing the authored script-binding model.

---

## `src/scripting/stdlib.rs`

AeoScript standard-library dispatch.

Use this when adding or changing:

* `math`.
* `basket`.
* `string`.
* Other built-in library functions.

---

## `src/scripting/diagnostic.rs`

Structured script diagnostics and errors.

---

## `src/scripting/source.rs`

Source spans and source locations.

---

## `src/scripting/source_map.rs`

Maps runtime/parser locations back to source text locations.

---

## `src/scripting/token.rs`

Token definitions.

---

## `src/scripting/log.rs`

Runtime/script diagnostic and logging records.

---

# 11. Runtime Entity Infrastructure

## `src/engine/entity.rs`

Runtime Entity identity and entity manager infrastructure.

Use this when changing:

* Runtime entity lifetime.
* Entity IDs.
* Entity lookup.
* Live entity storage.
* Entity handle resolution.

When an AeoScript handle is involved, trace through:

~~~text
src/scripting/value.rs
        ↓
src/scripting/api.rs
        ↓
src/scripting/scene.rs
        ↓
src/engine/entity.rs
~~~

---

# 12. Project System

## Directory

~~~text
src/project/
├── mod.rs
├── project.rs
└── recent.rs
~~~

---

## `src/project/project.rs`

`ProjectManager` owns project-level lifecycle.

Use this for:

* Creating projects.
* Opening projects.
* Current project path.
* Project lifecycle.
* Project-level filesystem coordination.

---

## `src/project/recent.rs`

Recent-project tracking and recently opened project data.

---

# 13. Common Change Paths

These are intended as practical navigation examples.

## Add a persistent Cell field

Start:

~~~text
src/world/cell.rs
~~~

Then inspect:

~~~text
src/world/persistence.rs
src/world/storage.rs
src/editor/editor.rs
~~~

and any runtime/script consumers that need the field.

The normal direction is:

~~~text
Cell data model
      ↓
Persistence
      ↓
Editor
      ↓
Runtime consumers
      ↓
Script API if required
~~~

---

## Add a runtime-only Cell property

Start with the runtime state representation:

~~~text
src/world/world.rs
~~~

Then trace:

~~~text
runtime state
      ↓
EngineHost / scripting
      ↓
Renderer / gameplay / other consumer
~~~

Do not add temporary Play-mode state directly to the persistent storage format unless it is intentionally authored data.

---

## Add a new AeoScript API

Typical path:

~~~text
AeoScript syntax / stdlib
      ↓
src/scripting/api.rs
      ↓
src/scripting/host.rs
      ↓
Owning engine subsystem
~~~

Also update:

~~~text
docs/language/
~~~

and relevant integration tests.

---

## Change renderer performance

Start with:

~~~text
src/renderer/mod.rs
~~~

Then inspect:

~~~text
chunk construction
mesh construction
GPU resource management
visibility/culling
renderer benchmark tests
~~~

Verify with:

~~~text
cargo test
~~~

and the relevant renderer performance benchmark.

---

## Change World persistence

Start with:

~~~text
src/world/storage.rs
src/world/persistence.rs
~~~

Then inspect:

~~~text
src/world/world.rs
src/world/cell.rs
~~~

Add or update persistence tests.

Verify old World files continue to load when backward compatibility is required.

---

## Change editor selection or picking

Start with:

~~~text
src/editor/editor.rs
src/editor/grid/picking.rs
src/editor/grid/grid.rs
~~~

Then inspect input routing in:

~~~text
src/engine/app.rs
~~~

Verify both viewport behavior and egui interaction boundaries.

---

# 14. Runtime Flow

The high-level runtime relationship is:

~~~text
src/main.rs
      ↓
src/engine/app.rs
      ↓
Application coordination
      ↓
┌─────────────────────────────────────┐
│ World                               │
│ Editor                              │
│ Renderer                            │
│ PhysicsWorld                        │
│ CharacterSystem                     │
│ ScriptScene                         │
│ GameplayCamera                      │
│ EntityManager                       │
└─────────────────────────────────────┘
~~~

Play mode derives runtime state from the authored World:

~~~text
Authored World
      ↓
Runtime conversion
      ├── PhysicsWorld
      ├── CharacterSystem
      ├── ScriptScene
      ├── GameplayCamera
      └── EntityManager
~~~

The renderer consumes both authored and runtime-effective state:

~~~text
World / Runtime State
      ↓
Renderer
      ↓
GPU
~~~

---

# 15. Authority Boundaries

The most important ownership boundary in the engine is:

~~~text
AUTHORED
    World
    Cell
    World settings
    Script bindings
    Persistent project assets
        ↓
    Persistence

RUNTIME
    PhysicsWorld
    CharacterSystem
    ScriptScene
    EntityManager
    GameplayCamera
    Runtime Cell state

DERIVED RENDER STATE
    Render chunks
    Chunk meshes
    GPU buffers
    Texture resources
    Shadow resources

EDITOR STATE
    Selection
    Tools
    Editor camera
    Undo/redo
    Panels
    Previews
~~~

The same conceptual data may appear in several representations, but ownership must remain explicit.

For example:

~~~text
Authored Cell
      ↓
Runtime effective Cell state
      ↓
Renderer chunk representation
~~~

The renderer does not become the owner of the Cell.

Likewise:

~~~text
Authored World
      ↓
PhysicsWorld
~~~

does not mean `PhysicsWorld` becomes the persistent World representation.

---

# 16. Tests

AeoEngine uses several layers of verification.

## Rust unit tests

Many subsystems keep focused tests close to the implementation:

~~~text
src/**/#[cfg(test)]
~~~

Use the existing test module for the subsystem whenever practical.

---

## Repository integration tests

The repository may also contain:

~~~text
tests/
~~~

Use this area when a test needs repository-level integration rather than a tightly scoped subsystem test.

---

## AeoScript integration tests

AeoEngine includes an in-engine AeoScript integration suite covering language/runtime behavior through the running engine.

This is particularly important for behavior that crosses:

~~~text
Script
  ↓
ScriptScene
  ↓
Engine host
  ↓
World / Runtime systems
~~~

---

## Renderer benchmarks

Renderer performance tests cover large World workloads and selective rebuild behavior.

Use them when modifying:

* Chunk construction.
* Mesh generation.
* Render invalidation.
* GPU resource management.
* Culling.
* Other renderer performance-sensitive paths.

Performance results and architecture are documented in:

~~~text
docs/architecture/RENDERING.md
~~~

---

# 17. Build and Verification

Development verification is documented in:

~~~text
docs/development/BUILDING.md
~~~

The normal Rust verification layers are:

~~~text
cargo fmt --check
cargo check
cargo clippy
cargo test
~~~

Then use the appropriate:

* AeoScript integration tests.
* Renderer benchmarks.
* Manual editor verification.
* Manual Play-mode verification.

The debugging process and third-party debugging tools are documented separately in:

~~~text
docs/development/DEBUGGING.md
~~~

---

# 18. Documentation Map

The documentation tree is organized by purpose.

~~~text
docs/
├── architecture/
├── development/
├── guides/
└── language/
~~~

## Architecture

~~~text
docs/architecture/
├── ENGINE.md
├── WORLD.md
├── RUNTIME.md
├── EDITOR.md
├── PHYSICS.md
├── CHARACTERS.md
├── RENDERING.md
├── PERSISTENCE.md
└── ASSETS.md
~~~

These documents describe ownership and system architecture rather than step-by-step beginner workflows.

---

## Development

~~~text
docs/development/
├── BUILDING.md
├── TESTING.md
├── DEBUGGING.md
├── MAP.md
└── CONTRIBUTING.md
~~~

Use these for:

* Building the engine.
* Running tests.
* Debugging and profiling.
* Navigating the repository.
* Development contribution workflow.

---

## Guides

~~~text
docs/guides/
├── GETTING_STARTED.md
├── WORLD_BUILDING.md
└── BUILDING_A_GAME.md
~~~

These documents describe using the engine to build projects.

---

## Language

~~~text
docs/language/
├── AeoScript.md
├── AeoScript_API.md
├── AeoScript_GRAMMAR.md
├── AeoScript_TYPES.md
├── AeoScript_HANDLES.md
├── AeoScript_STDLIB.md
├── AeoScript_LIFECYCLE.md
├── AeoScript_EXAMPLES.md
├── AeoScript_VM.md
└── AeoScript_EDITOR.md
~~~

These documents describe AeoScript as a language and runtime system.

---

# 19. Source Navigation Rules

When locating a change:

1. Start with the subsystem that owns the behavior.
2. Open the primary file listed in the Fast Lookup table.
3. Find the relevant symbol rather than relying on historical line numbers.
4. Trace into neighboring systems only when the data/control flow requires it.
5. Update tests with the implementation when behavior changes.
6. Update the relevant architecture or language documentation when ownership or public behavior changes.

Prefer:

~~~text
World → Cell → persistence
~~~

over searching the entire repository for every occurrence of a field.

Prefer:

~~~text
AeoScript API → host bridge → owning subsystem
~~~

when adding script functionality.

Prefer:

~~~text
Renderer → chunk construction → mesh/GPU path
~~~

for rendering work.

---

# 20. Maintaining MAP.md

`MAP.md` is a navigation document, not an exhaustive list of every function in the repository.

Keep entries focused on:

* Major subsystem ownership.
* Important source files.
* Important public/internal symbols.
* Data-flow boundaries.
* Common change paths.
* Test locations.
* Documentation locations.

Do not add every helper function or temporary implementation detail.

Do not rely on exact line numbers.

Update this file when:

* A subsystem moves.
* A file is split or merged.
* A major type changes ownership.
* A new architectural subsystem is introduced.
* Persistence responsibilities move.
* Renderer architecture changes.
* A major test/integration boundary changes.

The goal is to make the repository understandable without turning the map into a second copy of the source tree.