# Getting Started

This guide covers the basic AeoEngine workflow from opening the project to running an authored World in Play mode.

---

# 1. Requirements

AeoEngine currently targets:

* Windows 10 or Windows 11.
* Rust stable toolchain.

---

# 2. Build and Run

From the project root:

~~~text
cargo run
~~~

AeoEngine starts in its project/home workflow or opens the active project depending on the current application state.

---

# 3. Create or Open a Project

Use the project workflow to create a project or open an existing one.

A project contains the authored World and project-owned resources used by the engine.

Important project areas include:

~~~text
world data
scripts/
.assets/
~~~

---

# 4. Enter the Editor

The Editor is the primary World-authoring environment.

The main workspace contains the authored World viewport alongside the World hierarchy and Properties/navigation panels.

---

# 5. Build a World

Start by creating Blocks in the World.

The editor supports:

* Build.
* Erase.
* Drag operations.
* Line operations.
* Plane operations.
* Plane picking.
* Top-Block picking.
* Face-aware surface placement.
* Undo and Redo.

---

# 6. Add Scene Objects

Depending on the game, add the authored objects needed for runtime:

* Blocks.
* Lights.
* SpawnPoints.
* Other supported authored objects.

Give gameplay-relevant objects useful identity names where appropriate.

Remember that a Cell's identity/name is not its unique instance ID.

---

# 7. Configure Properties

Select a Cell and use the Properties panel to edit its authored properties.

Typical Block properties include:

* Color.
* Solidity.
* Anchored state.
* Texture where supported.

Light Cells provide their supported authored lighting properties.

---

# 8. Save

Save the project after making authored changes.

The authored World is the persistent source of truth.

Runtime simulation state should not be treated as saved World data.

---

# 9. Play the World

Enter Play mode to initialize the runtime systems.

AeoEngine can initialize:

* Runtime physics.
* Character state.
* Gameplay camera state.
* Script runtime state.
* Runtime rendering state.

---

# 10. Build a First Gameplay Loop

A useful first test is:

~~~text
Build a small room
      ↓
Add a SpawnPoint
      ↓
Add lighting
      ↓
Enter Play
      ↓
Move the character
      ↓
Test collisions
      ↓
Stop Play
      ↓
Return to the editor
~~~

This verifies the editor/runtime boundary before adding scripting.

---

# 11. Add Scripting

Create an `.aeo` script in the project's `scripts/` directory.

Attach the script to an authored Cell through the Properties panel.

The script binding targets the Cell's persistent numeric ID.

---

# 12. Run and Debug

Use the script output/diagnostic panel to inspect runtime messages and errors.

A typical development loop is:

~~~text
Edit
 ↓
Save
 ↓
Play
 ↓
Observe
 ↓
Stop
 ↓
Fix
 ↓
Repeat
~~~

---

# 13. Important Runtime Rule

Play mode is not another editing mode for the authored World.

Runtime changes such as physics motion, character motion, script fibers, and supported Cell overrides are temporary.

When Play mode stops, the authored World returns and all play mode changes are thrown away.

