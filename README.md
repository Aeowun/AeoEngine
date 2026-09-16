# AEOENGINE

A voxel engine and editor built for authored world creation and runtime physics simulation.

## What is it?

AeoEngine is a voxel building engine and editor built for world creation and runtime physics simulation. tool. You can build worlds using a grid based editor and then transition to a Play mode to simulate those worlds using a fixed timestep physics system. It maintains a strict boundary between authored data and runtime state so you can iterate on your designs without losing your original world layout.

## How to use it

### Requirements
*   Windows 10 or 11.
*   Latest Rust stable toolchain.

### Build and Run
1.  Open a terminal in the project root.
2.  Run the application: `cargo run`.
3.  Or build a release version: `cargo build --release`.

### Usage

#### Menu and Navigation
*   **Projects**: Create or open projects from the Home screen.
*   **Toggle Editor**: Press `G` to switch between the Home screen and the Editor.
*   **Exit to Home**: Press `H` or `Escape` while in the editor to save and return to the menu.

#### Camera Controls
*   **Move Focus**: Use `W`, `A`, `S`, and `D` to move the camera target along the current grid plane.
*   **Orbit**: Hold **Middle Mouse** and drag to rotate the camera around the focus point.
*   **Zoom**: Use the **Mouse Wheel** to move the camera closer to or further from the focus.

#### Editor Tools
*   **Navigate**: Click a grid cell to set it as the new camera anchor and target.
*   **Build**: Place new blocks into the world.
*   **Erase**: Remove blocks from the world.
*   **Select**: Inspect a specific cell to view and edit its properties in the Side Panel.

#### Simulation
*   **Play Mode**: Use the Mode toggle in the tool bar to switch to Play mode.
*   **Editor Mode**: Switch back to Editor mode to stop the simulation and restore the authored world state.
