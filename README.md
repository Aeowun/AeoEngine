# AEOENGINE

A voxel engine and editor built for authored world creation, runtime physics simulation, lighting, and interactive character systems.

## What is it?

AeoEngine is a voxel building engine and editor designed for creating authored worlds and simulating them at runtime.

Build and edit worlds with a grid-based editor, configure authored cell properties, select and manipulate world objects, then transition into Play mode to simulate the authored world using runtime physics and character systems.

The engine maintains a strict boundary between authored data and runtime state. The authored `World` remains the source of truth, while runtime `PhysicsBodies` and characters are created and managed separately during Play mode.

The editor supports single-cell building, drag-based line and plane building, multi-selection, multiple picking modes, live ghost previews, world hierarchy navigation, and configurable build properties.

---

## How to use it

### Requirements

* Windows 10 or 11.
* Latest Rust stable toolchain.

### Build and Run

1. Open a terminal in the project root.
2. Run the application:

```text
cargo run
```

3. Or build a release version:

```text
cargo build --release
```

---

# Editor Usage

## Projects and Navigation

### Home

From the Home screen you can:

* Create a new project.
* Open an existing project.
* Open a recent project.

Projects store the authored world and editor camera state.

### Toggle Home / Editor

Press:

```text
G
```

to toggle between the Home screen and the Editor.

### Escape

Escape is context-sensitive.

In Play mode:

```text
Esc → Stop Play Mode
```

In Editor mode:

```text
Active drag → Cancel drag
Selection → Clear selection
Nothing active → Return to Home
```

---

# Camera Controls

The normal editor camera is used while editing.

* **W / A / S / D** — move the camera target.
* **Middle Mouse + Drag** — orbit around the focus point.
* **Mouse Wheel** — zoom.

Press:

```text
F
```

to focus the camera on the current selection.

With multiple selected cells, the camera focuses on the center of their combined bounds.

---

# Editor Tools

The editor has four primary tools.

## Navigate

Navigate moves the editor anchor/focus to a world location.

Its picking behavior depends on the current Picking mode.

## Select

Select is used to inspect and edit authored cells.

Supports:

* Single click selection.
* Drag multi-selection.
* World hierarchy selection.

The selected cells remain highlighted in the viewport.

The active/primary selection is shown in the Properties panel.

## Build

Build creates authored cells using the current build template.

Supports:

* Single placement.
* Drag-based line/plane placement.
* Plane picking.
* Top-Block picking.
* Surface stacking.
* Shift-based overwrite.

## Erase

Erase removes authored cells.

Supports:

* Single target removal.
* Drag-based removal.
* Plane picking.
* Top-Block picking.

---

# Picking Modes

The editor has a Picking toggle:

```text
Picking: Plane [ ON ]
```

## Plane ON

The mouse is projected onto the active editor grid plane.

The active plane can be:

* XZ
* YZ
* XY

All four editor tools use the grid-plane result.

This mode is useful for precise coordinate-based editing.

## Plane OFF

The editor uses camera-ray / Top-Block picking for authored world objects.

The first eligible visible authored cell hit by the ray becomes the target.

Example:

```text
Camera
   |
   v
[ Block A ]  ← target
[ Block B ]  ← blocked by A
```

If Block A visually covers Block B, Block A is targeted.

This mode respects depth instead of ignoring it.

### Empty Space

When Top-Block picking does not hit an authored cell, tools that require open-space editing can fall back to the existing grid-plane position where appropriate.

---

# Building

## Plane Mode

With Plane picking enabled, Build uses the active grid plane.

You can:

* Build one cell.
* Drag to build a line.
* Drag across two axes to build a plane.

The existing grid workflow remains available for precise editing.

## Top-Block Mode

With Plane picking disabled, Build becomes surface-aware.

When the camera ray hits an authored block, the actual face hit by the ray determines where the new cell is placed.

Normal Build:

```text
[ New Block      ]
[ Existing Block ]
```

The new block is placed one grid cell outward from the visible surface.

This makes stacking natural.

### Shift + Build

Hold **Shift** while building to intentionally overwrite the hovered cell.

```text
Normal:
hit cell → build outward

Shift:
hit cell → build directly in the hit cell
```

Shift is useful when replacing or modifying an existing cell.

---

# Build Types

The Build tool currently supports the authored types that have meaningful editor construction behavior, including:

* Cube / Block
* Spawn Point
* Light

Build templates are replaced with each type's proper defaults when switching types.

## Lights

Lights are authored world objects rather than solid block geometry.

They have:

* Light color.
* Intensity.
* Range.
* Shadow setting.

Lights use an editor marker/ghost instead of appearing as normal solid blocks.

---

# Build Properties

The Build tool provides settings for the current build template.

Available authored settings include:

* **Color**
* **Visible**
* **Texture**
* **Solid**
* **Anchored**

The current build template is copied into the authored World when a cell is placed.

---

# Selection

Selection is persistent and supports multiple authored cells.

## Single Selection

Click an authored cell with Select.

The cell becomes:

* selected
* the active/primary selection
* the target shown in Properties

## Multi-Selection

Drag across a region with Select.

Only authored/non-empty cells inside the selected region become selected.

Selected cells remain visibly outlined.

The active selection is still used by the Properties panel.

## World Hierarchy Selection

The WORLD hierarchy provides another way to select exact authored cells by coordinate.

Selecting an object through the hierarchy uses the same editor selection state as viewport selection.

---

# Delete

With authored cells selected:

```text
Delete
```

removes the selected cells from the authored World.

Multiple selected cells can be deleted together.

Delete is an authored-world edit and participates in Undo/Redo.

---

# Undo and Redo

The editor supports authored-world Undo/Redo.

```text
Ctrl + Z → Undo
Ctrl + Y → Redo
```

Build, Erase, and Delete operations are treated as editor history operations.

A multi-cell drag is intended to behave as one operation rather than creating one history step per cell.

Undo restores the previous authored cell state, including customized properties.

---

# Save

Save the authored map with:

```text
Ctrl + S
```

This uses the same project save path as the editor's Save command.

The authored world and editor camera state are persisted through the existing project system.

---

# World Hierarchy

The WORLD panel provides a tree-style view of authored world contents.

It groups authored cells into categories such as:

```text
BLOCKS
LIGHTS
SPAWN POINTS
FX BLOCKS
PLAYERS
NPCS
```

Each category shows a count and the coordinates of its contents.

Example:

```text
▼ BLOCKS (120)
    (0, 0, 0)
    (1, 0, 0)

▼ LIGHTS (3)
    (4, 5, 2)

▼ SPAWN POINTS (1)
    (0, 2, 0)
```

Clicking an entry selects that authored cell and updates the Properties panel.

The hierarchy is derived from the existing authored World and does not maintain a second object registry.

---

# Properties

The Properties panel displays the active selection.

Sections currently include:

* **Identity**
* **Transform**
* **Light** where applicable
* **Physics**
* **Rendering**

Block and other authored-cell properties remain part of the authored World.

---

# Work Area

The editor work-area grid provides the visual editing reference for Plane picking.

### Plane ON

The work-area grid is visible.

### Plane OFF

The work-area grid is hidden to make Top-Block mode visually cleaner.

The editor's center/anchor marker remains visible even when the grid is hidden.

---

# Runtime / Play Mode

Use the Editor / Play control in the toolbar to enter runtime simulation.

During Play mode:

* Runtime physics is active.
* Runtime characters can be spawned.
* Runtime physics bodies are created from the authored World.
* The gameplay camera follows an active runtime character.

When no active runtime character exists, the normal editor camera is used instead.

This allows a world to be inspected in Play mode even when no spawn point is present.

Press:

```text
Esc
```

to stop Play mode.

Returning to Editor mode restores the saved editor camera.

---

# Runtime Lighting

AeoEngine supports:

* Global ambient lighting.
* Global directional lighting.
* Point light cells.
* Configurable point-light color, intensity, and range.
* Directional shadow mapping.

Lights are authored in the World and become part of the rendered scene.

---

# Keyboard Shortcuts

Current editor shortcuts:

| Shortcut     | Action                                      |
| ------------ | ------------------------------------------- |
| **Ctrl + S** | Save Map                                    |
| **Delete**   | Delete selected cell(s)                     |
| **Ctrl + Z** | Undo                                        |
| **Ctrl + Y** | Redo                                        |
| **F**        | Focus camera on selection                   |
| **Escape**   | Cancel drag / clear selection / return Home |
| **1**        | Navigate                                    |
| **2**        | Select                                      |
| **3**        | Build                                       |
| **4**        | Erase                                       |

Existing application controls also include:

| Shortcut  | Action               |
| --------- | -------------------- |
| **G**     | Toggle Home / Editor |
| **Space** | Jump in Play mode    |

Keyboard shortcuts respect egui keyboard focus so they do not interfere with active text or numeric fields.

---

# Editor Workflow

A typical workflow is:

```text
Create / Open Project
        ↓
Editor
        ↓
Choose Build
        ↓
Choose build type
        ↓
Build world
        ↓
Select / inspect objects
        ↓
Use WORLD hierarchy for exact object access
        ↓
Ctrl + S
        ↓
Play
        ↓
Test the world
        ↓
Esc
        ↓
Continue editing
```

For precise grid work:

```text
Picking: Plane ON
```

For visual/depth-based editing:

```text
Picking: Plane OFF
```

For stacking:

```text
Plane OFF
+ normal Build
```

For intentional replacement:

```text
Plane OFF
+ Shift + Build
```

---

# Current Architecture

The engine keeps major responsibilities separated:

* **World** — authoritative authored scene data.
* **Editor** — world editing, hierarchy, selection, camera, tools, and UI state.
* **PhysicsWorld** — runtime simulation and dynamic PhysicsBodies.
* **Renderer** — authored geometry, runtime bodies, characters, lighting, shadows, and editor helpers.
* **Character System** — runtime character state, movement, animation, and spawning.
* **Project System** — project creation, opening, recent projects, and authored-world persistence.

The central design boundary is:

**Authored world data is not the same thing as runtime state.**

A Player or NPC runtime character should not automatically be treated as a simple voxel just because a corresponding `CellType` exists.

---

# Planned Performance & Scalability

The current engine architecture is intentionally prototype-oriented, but several systems have known scalability limits.

These are **planned engineering improvements**, not claims that the current implementation is incapable of handling large scenes. Actual optimization decisions should be guided by profiling and representative workloads.

## World Rendering Scalability

The current authored World stores cells individually, and the active-block collection currently requires the renderer to iterate through the stored authored cells.

This is appropriate for small and medium authored worlds, but very large worlds will eventually require spatial partitioning and more efficient rendering.

Current conceptual path:

```text
World
  ↓
Stored authored cells
  ↓
Active block collection
  ↓
Renderer
  ↓
Individual block rendering
```

A future scalable path may use:

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

### Planned Options

#### Option A — Chunked World Storage

Partition the authored world into fixed-size spatial chunks.

```text
World
├── Chunk (0,0,0)
├── Chunk (1,0,0)
├── Chunk (0,1,0)
└── Chunk (1,1,0)
```

The renderer can then determine which chunks are relevant to the current camera instead of traversing the entire world.

Benefits:

* Better large-world scalability.
* Natural foundation for streaming.
* Faster spatial queries.
* Provides a foundation for future chunk meshing.

#### Option B — Chunked Storage + Mesh Generation

Combine chunking with generated render meshes.

Instead of submitting every block independently:

```text
Blocks
 ↓
Chunk
 ↓
Generated mesh
 ↓
GPU
```

The mesh can be rebuilt only when the affected authored cells change.

This provides a path toward:

* Reduced draw calls.
* Reduced per-block rendering overhead.
* Greedy meshing or face culling.
* Efficient large authored environments.

The exact meshing strategy remains an implementation decision.

---

## Dynamic Physics Scalability

The current dynamic collision system performs pairwise collision checks between dynamic bodies.

Conceptually:

```text
Body A ↔ Body B
Body A ↔ Body C
Body A ↔ Body D
...
Body B ↔ Body C
...
```

This is effectively an **O(n²)** broad-phase approach.

It is suitable for a small number of runtime physics bodies, but the cost grows rapidly as the number of dynamic bodies increases.

### Planned Options

#### Option A — Spatial Hash / Uniform Grid

Divide the physics world into spatial regions and only test bodies occupying nearby regions.

```text
Physics bodies
      ↓
Spatial grid
      ↓
Nearby candidates
      ↓
Collision tests
```

This is relatively straightforward and fits well with a voxel-oriented engine.

#### Option B — Dedicated Broad-Phase Structure

Introduce a more general broad-phase system such as:

* Spatial hash.
* Sweep-and-prune.
* BVH.
* Another spatial acceleration structure appropriate to the final physics requirements.

The broad phase would identify possible collision pairs before the existing narrow-phase collision tests run.

The narrow-phase collision implementation can therefore remain separate from the broad-phase optimization.

---

## Continuous Collision Detection

The current runtime collision system uses discrete collision testing after position integration.

A sufficiently fast body could theoretically move through a thin collider between physics samples.

This is a known limitation of discrete collision detection.

### Planned Options

#### Option A — Swept Collision

Test the volume swept between the previous and new body positions.

```text
Previous position
        ↓
   swept volume
        ↓
New position
```

This can prevent fast-moving bodies from passing through thin colliders.

#### Option B — Selective Continuous Collision Detection

Rather than enabling continuous collision detection for every body, use it only for bodies where tunneling is important.

Potential candidates include:

* Fast projectiles.
* Small high-speed objects.
* Characters moving at high velocity.
* Other explicitly configured physics bodies.

This avoids applying the additional cost to every runtime object.

---

## Runtime Mesh Rendering

Some runtime rendering paths currently create, upload, draw, and destroy OpenGL vertex objects as part of the draw operation.

Conceptually:

```text
GenVertexArray
GenBuffer
BufferData
Draw
DeleteBuffer
DeleteVertexArray
```

Doing this repeatedly is functional but creates unnecessary GPU resource-management overhead.

### Planned Options

#### Option A — Persistent Mesh Buffers

Create the VAO/VBO once and retain them for the lifetime of the renderable object.

```text
Create
  ↓
Upload
  ↓
Draw
  ↓
Draw
  ↓
Draw
  ↓
Destroy on shutdown
```

If geometry changes, the existing buffer can be updated rather than recreated.

This is the preferred direction for stable runtime meshes.

#### Option B — Shared Mesh Resources

Introduce reusable renderer mesh resources so multiple objects can reference the same GPU geometry.

```text
Mesh Resource
      ↑
 ┌────┼────┐
 │    │    │
Body Body Body
```

This becomes particularly useful when many runtime objects share the same mesh.

It also provides a foundation for future instanced rendering and batching.

---

## Rendering Optimization Roadmap

The current renderer is intentionally simple enough to support rapid engine development.

Future optimization can proceed incrementally:

```text
Current
  ↓
Persistent GPU buffers
  ↓
Visibility / frustum culling
  ↓
Spatial chunks
  ↓
Chunk mesh generation
  ↓
Batching / instancing
  ↓
Further profiling-driven optimization
```

Not every stage is required for every project size.

The engine should retain the simpler implementation while workloads remain small, and introduce more complex systems when profiling demonstrates a meaningful benefit.

---

## Performance Engineering Principle

Performance improvements should preserve the existing authored/runtime boundary.

The intended architecture remains:

```text
Authored World
      ↓
Spatial / render representation
      ↓
Renderer

Authored World
      ↓
Runtime conversion
      ↓
PhysicsWorld / Characters
```

Optimization systems should accelerate these paths without turning runtime state into the authoritative authored representation.

Performance work should be validated using measured profiling data rather than assumptions based solely on theoretical complexity.

---


# Future Asset Workflow

A project asset library is planned using:

```text
Project/
├── world.dat
├── camera.dat
└── .assets/
    ├── textures/
    └── audio/
```

The intended workflow is:

```text
Import supported asset
        ↓
copy into the appropriate .assets category
        ↓
reference the project-owned asset
        ↓
use it from the editor
```

Only explicitly supported file formats should be accepted.

The immediate target is project-owned texture support for the existing Block Texture field.

This asset workflow is planned separately from the authored World hierarchy.
