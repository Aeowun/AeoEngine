# Editor Architecture

The AeoEngine Editor is the authoring environment for persistent World and project data.

The editor is responsible for interaction, selection, tools, project assets, script bindings, and authored-state history.

The editor operates on **authored project state**. Runtime systems may derive temporary state from that authored data during Play mode, but runtime simulation does not become editor-authored state.

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

Editor panels provide supporting access to the same authored project state rather than maintaining separate copies of World data.

---

# 2. World Hierarchy

The World hierarchy exposes authored Cells for editor navigation.

Selecting an item in the hierarchy selects the corresponding authored Cell and updates the Properties panel.

Hierarchy selection and viewport selection operate on the same authored World selection model.

Hierarchy state is editor state and does not become runtime simulation state.

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

Selection outlines are editor visualization and do not modify the authored Cell itself.

---

# 4. Picking

The editor supports different picking strategies depending on the active tool.

### Plane Picking

Uses the active editor grid plane to determine the target position.

Plane-based tools use the editor grid and current tool configuration to determine where an authored Cell operation should occur.

### Top-Block Picking

Uses the camera ray and visible authored World geometry to identify the top visible Block.

Top-Block Build can place a new Cell outward from the selected face.

Shift-based overwrite behavior is available for the targeted location.

Picking is an editor interaction mechanism. It determines an authored edit target but does not create runtime simulation state by itself.

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

Erase and Build operations modify authored World data and therefore participate in editor history where supported.

---

# 6. Undo and Redo

Authored World editing supports Undo and Redo for supported editor mutations.

History applies to authored World changes rather than temporary runtime simulation.

Undo and Redo do not restore or modify temporary Play-mode state.

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
* Collision events enablement (controls whether `on_touch` triggers).
* Offset-related authored data.
* Light properties.
* Texture information.
* Cell attributes (game-defined data).
* Script binding controls.

Properties shown by the editor describe authored data unless a specific control explicitly represents runtime or diagnostic state.

When editing a runtime value during Play, the runtime override system is responsible for preserving the authored World boundary.

Runtime changes do not implicitly become authored Properties data.

---

# 8. Script Bindings

Scripts are attached to authored Cells through the Properties panel.

Bindings target the persistent Cell ID, not the human-readable identity/name.

The editor can display:

* The current bound script.
* Script enabled/disabled state (controls whether lifecycle and top-level code runs).
* Stale binding diagnostics.
* Binding removal controls.
* Other script binding information.

Removing a binding removes the relationship from the authored World. It does not delete the script file.

Because bindings target persistent Cell IDs, changing a Cell's human-facing identity/name does not by itself change the binding target.

---

# 9. Assets

The editor manages project-owned textures through the project asset workflow.

Imported textures are copied into the project's `.assets/textures` directory and receive identifiers used by the runtime renderer.

The editor filters supported PNG texture assets during import.

Project-owned assets remain part of persistent project data and are separate from temporary Play-mode runtime resources.

---

# 10. Editor and Play Separation

Editor camera state and gameplay camera state are separate.

Entering Play preserves the editor camera and allows runtime gameplay to use the GameplayCamera.

Leaving Play restores the editor camera state.

Editor tools do not intentionally mutate authored World data from gameplay simulation.

Play mode creates temporary runtime state from the authored World.

Runtime changes such as physics movement, runtime Cell changes, runtime property overrides, script execution state, character state, camera state, and runtime UI state are discarded when Play mode stops unless a separate authored workflow explicitly records a change.

The authored World remains the source of truth for persistent scene data.

---

# 11. Input Ownership

Editor World interaction is kept separate from egui interaction.

When the mouse is interacting with an egui control or panel, viewport tools such as Build, Erase, Select, and Navigate should not interpret that interaction as a World edit.

This prevents UI interaction from leaking into World authoring.

Input ownership therefore follows the active interaction context:

```text
egui interaction
      ↓
UI owns the interaction

viewport interaction
      ↓
Editor tool owns the interaction
```

The separation applies to pointer interaction and prevents editor tools from responding to input that belongs to an editor UI control.

---

# 12. Editor Authority

The editor is authoritative for authored project changes.

Runtime systems are authoritative only for temporary Play-mode state.

```text
Editor
  ↓
Authored World
  ↓
Runtime
```

More specifically:

```text
Authored World
      |
      +--------------------+
      |                    |
      v                    v
Editor Representation   Runtime Conversion
      |                    |
      v                    v
Editor Interaction     Runtime Systems
                       Physics
                       Characters
                       Scripts
                       Camera
                       UI
```

The editor may maintain its own temporary interaction state such as selection, tool state, previews, camera state, and UI state.

These editor interaction states are not part of the runtime simulation and are not automatically persisted as World content.

---

# 13. Authored State Boundary

The primary architectural boundary of the editor is the separation between **authored state** and **temporary interaction or runtime state**.

Authored state includes persistent project information such as:

* World Cells.
* Cell properties.
* Light properties.
* Cell attributes.
* Script bindings.
* Project asset references.
* Other persistent World configuration.

Editor interaction state may include:

* Current selection.
* Active tool.
* Build or Erase mode.
* Ghost previews.
* Editor camera state.
* Panel state.
* Navigation state.
* Other temporary authoring interaction state.

Runtime state may include:

* Physics bodies.
* Character movement state.
* Runtime Cell state.
* Runtime property overrides.
* Script execution state.
* Gameplay camera state.
* Runtime UI.
* Other simulation-derived state.

The editor is responsible for changing the authored layer. Runtime systems operate on derived or temporary state.

This separation allows Play mode to be started and stopped repeatedly without replacing the authored World with temporary simulation state.

---

# 14. Persistence Boundary

The editor's authored World changes are the changes intended to become persistent project data.

Editor systems may use project persistence mechanisms to load and save authored state, but persistence is separate from rendering, physics, and gameplay simulation.

The architectural relationship is:

```text
Editor Interaction
        ↓
Authored World State
        ↓
Persistence
```

and independently:

```text
Authored World State
        ↓
Runtime Conversion
        ↓
Physics / Characters / Scripts / Rendering
```

The rendering and runtime systems therefore consume authored data without becoming the authority for what is saved.

---

# Change Log

*Added: Explicit authored-state boundary between editor interaction, persistent World data, and runtime state.
*Added: Clarification that hierarchy selection and selection outlines are editor state.
*Added: Clarification that editor Properties represent authored data unless a control explicitly represents runtime or diagnostic state.
*Added: Clarification that runtime overrides do not implicitly become authored Properties data.
*Added: Cell-ID binding clarification: script bindings target persistent `Cell.id`, while `entity_identity` remains a human-facing name.
*Added: Explicit input-ownership model separating egui interaction from viewport tools.
*Added: Explicit persistence boundary separating authored World persistence from rendering, physics, and gameplay systems.
*Added: Distinction between authored state, editor interaction state, and runtime state.
*Changed: World hierarchy wording to describe it as navigation over authored Cells rather than an independent state model.
*Changed: Picking documentation to clarify that picking determines an editor edit target and does not itself create runtime state.
*Changed: Build and Erase documentation to explicitly identify these operations as authored World mutations.
*Changed: Undo/Redo documentation to clarify that editor history does not operate on temporary Play-mode state.
*Changed: Editor and Play separation to explicitly document discarded runtime changes and preservation of authored World data.
*Changed: Editor Authority section to show the complete editor → authored World → runtime relationship.