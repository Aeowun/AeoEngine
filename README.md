# AEOENGINE

A voxel engine and editor built for authored world creation, runtime physics simulation, and interactive character systems.

## What is it?

AeoEngine is a voxel building engine and editor designed for creating worlds and simulating them at runtime. Build and edit worlds with a grid based editor, configure block properties, then transition into Play mode to simulate the authored world using a fixed timestep physics system.

The engine maintains a strict boundary between authored data and runtime state. The authored World remains the source of truth, while runtime PhysicsBodies receive their required properties when simulation begins.

The editor supports single cell building as well as drag based line and plane building, with live ghost previews and configurable build properties.

## How to use it

### Requirements
* Windows 10 or 11.
* Latest Rust stable toolchain.

### Build and Run
1. Open a terminal in the project root.
2. Run the application: `cargo run`.
3. Or build a release version: `cargo build --release`.

### Usage

#### Menu and Navigation
* **Projects**: Create or open projects from the Home screen.
* **Toggle Editor**: Press `G` to switch between the Home screen and the Editor.
* **Exit to Home**: Press `H` or `Escape` while in the editor to save and return to the menu.

#### Camera Controls
* **Move Focus**: Use `W`, `A`, `S`, and `D` to move the camera target.
* **Orbit**: Hold **Middle Mouse** and drag to rotate the camera around the focus point.
* **Zoom**: Use the **Mouse Wheel** to move the camera closer to or further from the focus.

#### Editor Tools
* **Navigate**: Click a grid cell to move the camera anchor and focus.
* **Build**: Create Blocks with the current build properties.
* **Erase**: Remove Blocks from the world.
* **Select**: Inspect and edit the properties of an existing cell in the Properties panel.

#### Building
* **Single Click**: Build or erase one cell.
* **Drag in one direction**: Build or erase a line.
* **Drag across two axes of the active grid plane**: Build or erase a plane.
* **Ghost Preview**: Shows the cells affected by the current drag before the action is committed.
* **Build Properties**: Configure properties such as color, visibility, texture, solid, and anchored state before building.

#### Block Properties
Blocks support editable authored properties including:
* **ColorRGB**
* **Visible**
* **Solid**
* **Anchored**
* **Texture**

These properties are preserved when Blocks become runtime PhysicsBodies.

#### Simulation
* **Play Mode**: Use the Mode toggle in the tool bar to enter runtime simulation.
* **Physics**: Dynamic bodies retain their authored runtime properties while simulating.
* **Editor Mode**: Switch back to Editor mode to stop the simulation and return to the authored world state.

#### Runtime Lighting
* Global ambient and directional lighting are supported.
* Point light cells can illuminate the world.
* Directional shadows are supported through shadow mapping.

## Current Architecture

The engine keeps major responsibilities separated:

* **World**: Authoritative authored scene data.
* **Editor**: World editing, selection, camera, tools, and UI state.
* **PhysicsWorld**: Runtime simulation and dynamic PhysicsBodies.
* **Renderer**: Rendering of authored world geometry, runtime bodies, lighting, shadows, and editor helpers.
* **Character System**: Dedicated runtime character structure for future player movement, collision, and animation systems.