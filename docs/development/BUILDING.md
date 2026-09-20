# Building AeoEngine

This document describes the development build workflow for AeoEngine itself.

---

# 1. Platform

AeoEngine currently targets:

* Windows 10.
* Windows 11.

The engine is developed and tested using the Rust stable toolchain.

---

# 2. Repository Root

Run development commands from the AeoEngine repository root.

~~~text
C:\Dev\AeoEngine
~~~

The exact local path may differ.

---

# 3. Check the Project

Use Cargo to verify that the project compiles:

~~~text
cargo check
~~~

`cargo check` is the fast compile-oriented verification step and does not produce the final executable.

---

# 4. Build and Run

Run the engine with:

~~~text
cargo run
~~~

This is the normal development path for manually exercising the engine and editor.

---

# 5. Tests

Run the full Rust test suite with:

~~~text
cargo test
~~~

The full suite should be run before committing substantial runtime changes.

---

# 6. Development Loop

A practical engine development loop is:

~~~text
Change
  ↓
cargo check
  ↓
Focused tests
  ↓
Full cargo test
  ↓
Manual AeoEngine run
  ↓
Inspect behavior
  ↓
Repeat
~~~

Manual Play-mode behavior is especially important for editor, rendering, physics, character, and scripting integration.

---

# 7. Generated and Local Files

Development may produce generated artifacts, imported assets, temporary project state, and local configuration files.

Before committing, inspect:

~~~text
git status
~~~

Do not assume every untracked file belongs in the commit.

Generated artifacts and unrelated local files should remain outside feature commits unless they are intentionally part of the change.

---

# 8. Preserving the Working Tree

AeoEngine development frequently contains multiple related changes at once.

Do not use destructive commands to clean up unrelated work.

Avoid commands such as:

~~~text
git reset --hard
git restore .
~~~

unless there is an explicit decision to discard the affected work.

When an unrelated change is present, preserve it and stage only the intended files.

---

# 9. Inspecting Changes

Use Git to understand what actually changed.

~~~text
git status --short
git --no-pager diff
git --no-pager diff --cached
~~~

`git --no-pager diff` is useful in environments where the default Git pager makes the command appear to stop responding.

---

# 10. Build Verification Principle

A successful build is necessary but not sufficient.

For runtime work, the expected verification sequence is:

~~~text
Compiles
  +
Tests pass
  +
Manual behavior works
  =
Verified change
~~~

The last step is obviously important for editor and gameplay behavior that is difficult to express entirely through unit tests.

# 11. AI Prompt example:

~~~

Implement the **Cell Attribute data model and persistence only**.

Do not implement the editor Attributes panel, AeoScript attribute access, runtime attribute mutation, attribute schemas, or gameplay behavior.

### `src/world/cell.rs`

At `Cell` (`src/world/cell.rs:19-37`), add an authored attribute map directly to the `Cell`:

```rust
pub attributes: BTreeMap<String, AttributeValue>,
```

Add:

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum AttributeValue {
    Number(f64),
    Bool(bool),
    String(String),
}
```

Add the required `BTreeMap` and `serde` imports at the top of the file.

`AttributeValue` belongs in `src/world/cell.rs`.

The attribute map is authored Cell data. It is not runtime state and must not be stored in `RuntimeCellState`.

### `src/world/cell.rs` — constructors

At `Cell::default` (`src/world/cell.rs:52-69`), initialize:

```rust
attributes: BTreeMap::new(),
```

At `Cell::new_light` (`src/world/cell.rs:84-100`), initialize:

```rust
attributes: BTreeMap::new(),
```

Do not add redundant initialization to `new_block` or `new_spawn_point`; they already use `..Default::default()`.

Do not change any other Cell defaults.

### `src/world/persistence.rs`

Use the existing text world format in `src/world/persistence.rs:13-160`.

After each Cell's existing record is written, write its authored attributes as one additional line **only when the attribute map is non-empty**:

```text
ATTRIBUTES <cell_id> <json>
```

The `<json>` value is the compact `serde_json` serialization of:

```rust
BTreeMap<String, AttributeValue>
```

Example:

```text
ATTRIBUTES 12345 {"coins":{"type":"Number","value":1000.0},"difficulty":{"type":"String","value":"easy"},"locked":{"type":"Bool","value":true}}
```

Place this write immediately after the existing `match cell.cell_type` block in `save_world`, while `cell` is still in scope.

Use the existing `serde_json` dependency. Do not add a new dependency.

Map serialization errors into the existing `std::io::Result` return type. Do not use `unwrap()` for persistence serialization.

### `src/world/persistence.rs` — loading

At `load_world` (`src/world/persistence.rs:174-347`), add handling for an `ATTRIBUTES` record immediately after the existing `SCRIPT_BINDING` handling and before the generic Cell-record parsing.

Parse:

```text
ATTRIBUTES <cell_id> <json>
```

Resolve `<cell_id>` with the existing:

```rust
world.resolve_cell_id(...)
```

at `src/world/world.rs:72-75`.

Assign the decoded `BTreeMap<String, AttributeValue>` directly to that Cell's `attributes`.

Use the existing tolerant loading style: malformed attribute records must not panic or abort the entire world load.

Old world files contain no `ATTRIBUTES` records. They must continue loading normally with each Cell's attribute map empty.

Do not create another persistence system.

Do not modify the existing Cell record format.

### `src/world/persistence.rs` — tests

Add the new tests inside the existing test module beginning at `src/world/persistence.rs:349`.

Add:

1. A default Cell has an empty attribute map.
2. A Cell clone preserves its attributes.
3. Number, Bool, and String attributes survive save/load.
4. Multiple attributes on the same Cell survive save/load.
5. A world using the old Cell text format with no `ATTRIBUTES` line loads successfully and produces an empty attribute map.
6. An empty attribute map does not produce an `ATTRIBUTES` line in the saved world.

Use the existing temporary-file pattern already used by the persistence tests and remove every temporary file created by the new tests.

Do not add a separate test framework or new persistence helpers unless the existing file absolutely requires one.

### Scope

Only modify:

```text
src/world/cell.rs
src/world/persistence.rs
```

and the relevant documentation file below.

Do not modify:

```text
src/editor/
src/scripting/
src/character/
src/engine/
src/renderer/
```

Do not reorganize the world system or perform unrelated cleanup.

### Final Verification

Run:

```powershell
cargo check
cargo test
```

Then update:

```text
docs/architecture/WORLD.md
```

Only edit the relevant existing lines in **Section 3 — Cells (`docs/architecture/WORLD.md:32-49`)** to document that authored Cells now contain custom Attributes supporting Number, Bool, and String values.

Do **not** overwrite the documentation file. Do not rewrite unrelated sections.


