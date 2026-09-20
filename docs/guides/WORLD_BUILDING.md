# World Building

This guide covers the authored World workflow in the AeoEngine editor.

---

# 1. Authoring Model

The editor creates persistent World data.

The World is a sparse grid of authored Cells.

~~~text
WorldCoord → Cell
~~~

Empty locations do not require stored Cells.

---

# 2. Blocks

Blocks are the primary voxel building Cell.

Block authoring can include:

* Color.
* Solidity.
* Anchored state.
* Texture where supported.
* Identity/name.

---

# 3. Lighting

Light Cells can be authored directly in the World.

World lighting also contains global lighting configuration.

Use authored Lights for local scene lighting and the World lighting settings for scene-wide illumination behavior.

---

# 4. Plane Building

Plane picking uses the editor's grid-plane model.

It is useful when you want predictable grid placement without targeting visible authored geometry.

---

# 5. Top-Block Building

Top-Block picking uses the camera ray to target visible authored Blocks.

Build operations can place a new Cell outward from the targeted face.

This makes stacked construction easier because the editor can determine the surface being built against.

Shift-based overwrite behavior can be used when the intended operation is to replace the targeted Cell location instead.

---

# 6. Drag Building

The editor supports drag-based Build and Erase operations.

The editor can preview the affected area before committing the operation.

Line and plane operations extend the same authoring concept to larger shapes.

---

# 7. Selection

Select Cells from the viewport or World hierarchy.

Selection supports:

* Single selection.
* Multi-selection.
* Selection outlines.
* Camera focus.

---

# 8. Properties

Select a Cell and edit its authored properties through the Properties panel.

Remember that authored properties describe what should be saved in the World.

Runtime overrides created during Play are a separate system.

---

# 9. Undo and Redo

Use Undo and Redo while authoring to revert or restore supported World edits.

History applies to authored editor operations, not temporary Play-mode simulation.

---

# 10. Naming Objects

Give gameplay-relevant objects readable identity names.

For example:

~~~text
Door
Button
EnemySpawn
Goal
MovingPlatform
~~~

Names are not unique IDs.

Two Cells can have the same name.

When a specific instance matters, use its persistent Cell ID through the scripting/runtime systems.

---

# 11. SpawnPoints

Use authored SpawnPoint Cells to define candidate character spawn locations.

The runtime validates the location and can search nearby clear space if necessary.

---

# 12. Save Frequently

Save after meaningful authored changes.

The saved World represents persistent scene state, while Play mode can create temporary runtime state.

---

# 13. Authoring Principle

Build the World as the player should experience it.

Use runtime systems for things that change during play.

~~~text
Editor → persistent scene
Runtime → temporary simulation
~~~

