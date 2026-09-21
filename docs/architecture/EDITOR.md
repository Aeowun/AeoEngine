# Editor Architecture

The AeoEngine Editor is the authoring environment for persistent World and project data.

The editor is responsible for interaction, selection, tools, project assets, script bindings, and authored-state history.

---

# 1. Editor Workspace

The editor uses a docked workspace centered around the World viewport.

The major editor regions include:

* Home screen (Project management, Templates, Learning).
* World hierarchy.
* Main 3D editor viewport.
* Properties and navigation panels.
* Script workspace where applicable.

The viewport remains the primary spatial authoring surface.

---

# 2. World Hierarchy

The World hierarchy exposes authored Cells organized for editor navigation.

Selecting an item in the hierarchy selects the corresponding authored Cell and updates the Properties panel.

Hierarchy selection and viewport selection operate on the same authored World selection model.

---

# 3. Selection

The editor supports:

* Single-cell selection.
* Multi-selection.
* Viewport drag selection.
* Persistent selection outlines.
* Hierarchy selection.
* Camera focus on the active selection.

Selection is editor state and does not become runtime simulation state.

---

# 4. Picking

The editor supports different picking strategies.

### Plane Picking

Uses the active editor grid plane to determine the target position.

### Top-Block Picking

Uses the camera ray and visible authored World geometry to identify the top visible Block.

Top-Block Build can place a new Cell outward from the selected face.

Shift-based overwrite behavior is available for the targeted location.

---

# 5. Build and Erase

The editor supports:

* Single-cell Build.
* Drag Build.
* Line Build.
* Plane Build.
* Single-cell Erase.
* Drag Erase.
* Line Erase.
* Plane Erase.
* Ghost previews for drag operations.

Build operations use the configured Block or Light properties from the editor tool state.

---

# 6. Undo and Redo

Authored World editing supports Undo and Redo for supported editor mutations.

History applies to authored World changes rather than temporary runtime simulation.

---

# 7. Properties

The Properties panel edits authored Cell data and exposes script-binding information.

Depending on the selected Cell, the panel can expose:

* Identity/name.
* Cell type.
* Color.
* Visibility.
* Solidity.
* Anchored state.
* Offset-related authored data.
* Light properties.
* Texture information.
* Cell attributes (game-defined data).
* Script binding controls.

When editing a runtime value during Play, the runtime override system is responsible for preserving the authored World boundary.

---

# 8. Script Bindings

Scripts are attached to authored Cells through the Properties panel.

Bindings target the persistent Cell ID, not the human-readable identity/name.

The editor can display:

* The current bound script.
* Stale binding diagnostics.
* Binding removal controls.
* Other script binding information.

Removing a binding removes the relationship from the authored World. It does not delete the script file.

---

# 9. Assets

The editor manages project-owned textures through the project asset workflow.

Imported textures are copied into the project's `.assets/textures` directory and receive identifiers used by the runtime renderer.

The editor filters supported PNG texture assets during import.

---

# 10. Editor and Play Separation

Editor camera state and gameplay camera state are separate.

Entering Play preserves the editor camera and allows runtime gameplay to use the GameplayCamera.

Leaving Play restores the editor camera state.

Editor tools do not intentionally mutate authored World data from gameplay simulation.

---

# 11. Input Ownership

Editor World interaction is kept separate from egui interaction.

When the mouse is interacting with an egui control or panel, viewport tools such as Build, Erase, Select, and Navigate should not interpret that interaction as a World edit.

This prevents UI interaction from leaking into World authoring.

---

# 12. Editor Authority

The editor is authoritative for authored project changes.

Runtime systems are authoritative only for temporary Play-mode state.

~~~text
Editor
  ↓
Authored World
  ↓
Runtime
~~~

