# AeoScript Editor

The AeoScript editor is the integrated source-authoring environment for `.aeo` files.

Script source editing and script binding are separate concepts:

```text
Script Editor
→ edits .aeo source

World Properties
→ attaches a script to an authored Cell
```

---

# 1. Script Files

Project scripts are stored under:

```text
<project>/scripts/
```

The Script Editor discovers `.aeo` files from the project script directory.

---

# 2. Script Documents

The editor tracks the state of an open script document.

Relevant document state includes:

* Source text.
* Original source.
* Dirty state.
* Parse/diagnostic state.
* Parsed program.
* Source mapping information.

The editor uses this state to determine whether a file has unsaved changes and to display source diagnostics.

---

# 3. Script Workspace

The integrated script workspace provides:

* Script file discovery.
* Opening scripts.
* Editing.
* Saving.
* Creating new scripts.
* Search.
* Diagnostics.
* Script output.

AeoScript source is edited through the integrated editor interface.

---

# 4. Syntax and Diagnostics

The script editor can present syntax and parsing diagnostics associated with source locations.

The language pipeline is:

```text
Source
 ↓
Lexer
 ↓
Parser
 ↓
AST
 ↓
Diagnostics
```

Runtime errors use the runtime diagnostic system instead and may appear in the script output/terminal area.

---

# 5. Saving Scripts

Saving a script writes its `.aeo` source file to the project.

Script source persistence and World persistence are separate:

```text
AeoScript source
→ scripts/*.aeo

World authored data
→ world.dat / chunks/
```

Saving a script does not automatically modify the authored World.

---

# 6. Script Bindings

A script is attached to an authored Cell through the Properties panel.

The workflow is:

```text
Select authored Cell
      ↓
Properties
      ↓
Script binding
      ↓
Persistent Cell ID
      ↓
.aeo script path
```

The binding targets the Cell's persistent numeric ID.

The Cell's human-readable identity/name is not the persistent binding key.

---

# 7. Entity Declarations

A bound script can contain an `entity` declaration.

Example:

```aeoscript
entity Door {
    open: bool = false

    fn update(dt) {
        ...
    }
}
```

The entity declaration defines the script-side object state and functions.

The binding determines which authored Cell instance receives that script.

---

# 8. Binding Identity

The editor and runtime distinguish:

```text
Cell ID
→ specific authored instance

entity_identity
→ human-readable name
```

Multiple Cells can have the same identity/name.

The script binding remains independent because it targets the Cell ID.

---

# 9. Removing a Binding

Removing a script binding:

* Removes the Cell-to-script relationship.
* Marks authored World state as modified.
* Can be saved with the World.
* Does not delete the `.aeo` source file.

Other Cells using the same source script remain unaffected.

---

# 10. Binding Diagnostics

The editor can report binding problems such as stale targets.

A stale binding means the stored target Cell ID is no longer present in the current authored World.

The editor must not silently substitute another Cell with the same human-readable name.

Legacy name-based binding migration may occur only when the old name resolves unambiguously to one Cell.

---

# 11. Runtime Output

The editor/runtime script output can contain:

* `debug.log(...)` messages.
* Script runtime errors.
* Script warnings.
* Binding diagnostics.
* Other structured script diagnostics.

Runtime diagnostics should be treated separately from source-editing diagnostics.

---

# 12. Play Mode

The Script Editor and runtime operate on different states.

Editing and saving `.aeo` files changes source data.

Play mode executes the loaded script runtime.

Runtime state may include:

* Script fibers.
* Persistent script-instance fields.
* Runtime callbacks.
* Runtime Cell overrides.
* Other gameplay state.

Stopping Play clears temporary runtime state according to ScriptScene ownership.

---

# 13. Recommended Editor Workflow

A normal script-authoring workflow is:

```text
Open project
      ↓
Open Script Editor
      ↓
Create or edit .aeo
      ↓
Save script
      ↓
Select authored Cell
      ↓
Attach script binding
      ↓
Save World
      ↓
Enter Play
      ↓
Inspect runtime output / behavior
```

The Script Editor is a source-authoring tool.

It does not own World persistence, runtime simulation, or renderer state.

---

# 14. Separation of Responsibilities

The editor architecture is:

```text
Script Editor
→ source files and source diagnostics

Properties
→ authored script bindings

ScriptScene
→ runtime script instances and lifecycle

ScriptRuntime
→ fibers and scheduler

Interpreter
→ language execution

EngineHost
→ engine integration
```

Keeping these responsibilities separate prevents source-editing state from being confused with runtime script state.