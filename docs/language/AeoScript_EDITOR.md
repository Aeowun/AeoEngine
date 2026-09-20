# AeoScript Editor Workflow

The AeoScript workflow is split between the integrated script editor and the object-level script binding controls in the AeoEngine editor.

Scripts are authored as `.aeo` files and attached to specific authored World Cells through persistent Cell-ID bindings.

---

# 1. Attaching Scripts

Scripts are attached to authored World Cells through the **PROPERTIES** panel.

### 1. Select a Cell

Select a Cell from the 3D World or the World hierarchy.

The selected Cell's authored properties and script binding information appear in the Properties panel.

### 2. Set the Cell Identity

Assign a human-readable authored identity such as:

```text
Door
Ghost
FencePost
```

The identity is a **name**, not the unique identity of the Cell.

Multiple Cells may share the same identity.

### 3. Attach a Script

Use the script controls in the Properties panel to attach an `.aeo` script to the selected Cell.

The binding is stored against the Cell's persistent numeric `id`.

This means the binding belongs to that specific authored Cell instance even if:

* The Cell is renamed.
* Another Cell is given the same name.
* Multiple Cells share the same identity.

### 4. Entity Declaration Matching

The script contains an `entity` declaration describing the scripted object.

For example:

```aeoscript
entity Door {
    open: bool = false

    fn update(dt) {
        ...
    }
}
```

The binding identifies **which Cell instance** receives the script.

The Cell's authored identity/name is used for the script's object-facing identity, while the persistent Cell ID is used to identify the specific bound instance.

---

# 2. Removing a Script

A script binding can be removed from the selected Cell through the Properties panel.

Removing a binding:

* Removes the relationship between the Cell and the script.
* Does not delete the `.aeo` script file.
* Marks the authored World as modified so the binding change can be saved.

Removing a binding from one Cell does not affect other Cells that use the same script or share the same name.

---

# 3. Script Workspace

The AeoScript workspace provides an integrated environment for authoring `.aeo` files.

### File Explorer

The script workspace provides access to project script files stored in the project's `scripts/` directory.

### Monaco Editor

AeoScript is edited through the integrated Monaco-based editor.

The editor provides:

* Syntax highlighting
* Source editing
* Script file management
* Syntax and parsing diagnostics where available

### Saving

Saving a script writes the `.aeo` source to the project.

Runtime behavior should be understood in terms of the currently loaded script state; saving the source does not change the authored World or script bindings themselves.

---

# 4. Diagnostics and Output

AeoScript diagnostics are presented through the editor and AeoEngine's script output system.

### Editor Diagnostics

Syntax and parsing problems can be reported directly by the script editor.

These diagnostics are associated with the relevant source location when available.

### PRINT OUTPUT

The **PRINT OUTPUT** panel displays script and runtime diagnostics.

It can contain:

* `print()` output where supported.
* `debug.log(...)` messages.
* Script runtime errors.
* Script warnings.
* Binding and engine diagnostics.

Runtime diagnostic messages include available context such as the script path, entity, function, and execution location.

---

# 5. Identity and Binding

AeoScript uses two different concepts for authored objects:

### Cell ID

The Cell's persistent numeric `id` identifies the specific authored instance.

It is used for script bindings.

### Identity / Name

The Cell's authored `entity_identity` is the human-readable game-logic name.

Names are not required to be unique.

For example:

```text
Block
  ID: 12048371
  Identity: FencePost

Block
  ID: 90421763
  Identity: FencePost
```

These are two distinct Cells.

Their script bindings remain independent because each binding targets a different Cell ID.

---

# 6. Finding Objects by Identity

Because identities are names rather than unique IDs, `find()` can return multiple objects.

```aeoscript
const posts = find("FencePost")
```

The result is a `basket` containing all matching handles.

This makes identity useful for gameplay queries while Cell IDs remain responsible for stable instance-specific bindings.

---

# 7. Binding Diagnostics

The editor can display the current script binding for the selected Cell.

The binding workflow distinguishes between:

* A valid binding whose Cell ID exists.
* A stale binding whose target Cell ID no longer exists.
* The friendly authored identity/name displayed to the user.
* The persistent numeric ID used internally for binding.

A stale binding does not cause another Cell with the same name to be selected automatically.

Legacy name-based bindings may be migrated when the name resolves to exactly one authored Cell.

Ambiguous legacy bindings must not guess which Cell was intended.

---

# 8. Recommended Workflow

A typical AeoScript workflow is:

```text
Create or select a World Cell
        ↓
Assign an authored Identity/Name
        ↓
Create or open an .aeo script
        ↓
Declare the matching entity
        ↓
Attach the script in Properties
        ↓
Save the World and script
        ↓
Run in Play mode
        ↓
Use PRINT OUTPUT for runtime diagnostics
```

For instance-specific script behavior, rely on the Cell's persistent ID rather than assuming that the authored name is unique.

---

# 9. Important Separation

The editor manages several different kinds of data:

```text
Authored World
    ├── Cell type
    ├── Cell position
    ├── Cell ID
    ├── Cell identity/name
    └── Script binding

AeoScript Source
    └── .aeo files

Runtime
    ├── Script fibers
    ├── Script fields
    ├── Runtime Cell overrides
    ├── Physics state
    └── Character state
```

Changing a script's runtime state does not rewrite the authored World.

Changing a script binding changes authored World metadata and must therefore be saved with the World.

The persistent Cell ID is the key link between the authored Cell and its script binding.
