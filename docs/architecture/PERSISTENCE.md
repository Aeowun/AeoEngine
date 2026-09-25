# Persistence Architecture

AeoEngine persistence stores authored project state separately from temporary runtime simulation state.

The persistence layer is responsible for restoring authored Worlds and project metadata without serializing transient gameplay simulation.

---

# 1. Persistence Boundary

Persistent data includes authored scene and project information.

Temporary runtime information is not part of the persistent World representation.

~~~text
Authored state
      ↓
Persistence
      ↓
Project files

Runtime state
      ↓
Play mode only
      ↓
Discarded when Play stops
~~~

---

# 2. World Persistence

`world.dat` (format 2) stores global world metadata. Authored cells live in
individual `chunks/<x>_<y>_<z>.chunk` files, which lets the runtime load and
evict bounded areas without reconstructing the entire world.

World persistence stores authored World data including Cells and their relevant authored properties.

Depending on the Cell type, persisted information can include:

* Cell ID.
* Cell type.
* World coordinate.
* Identity/name.
* Visibility.
* Solidity.
* Anchored state.
* Color.
* Light properties.
* Cell attributes (game-defined data).
* Other authored properties supported by that Cell type.

---

# 3. Stable Cell IDs

Cell IDs are persisted as part of authored World data.

When a World is saved and loaded, the same Cell ID identifies the same authored instance.

This is important for script bindings and any future system that requires persistent instance identity.

---

# 4. Script Binding Persistence

Script bindings are persisted as authored World metadata.

The current binding model is:

~~~text
Cell ID → script path
~~~

During loading, bindings are resolved against the current authored World.

If the target Cell ID no longer exists, the binding can be reported as stale.

Legacy name-based bindings may be migrated when the name resolves to exactly one authored Cell.

If multiple Cells share the old name, migration must not guess which instance was intended.

---

# 5. Project Assets

Project-owned assets such as imported textures live in the project asset directories.

The authored World stores references to those project assets rather than embedding the runtime GPU objects themselves.

---

# 6. Runtime State Is Not Authored Persistence

The following examples are runtime state and are not intended to overwrite authored World data when Play stops:

* Dynamic PhysicsBody position.
* Physics velocity.
* Sleeping state.
* Runtime character position.
* Runtime character state.
* Script fiber instruction position.
* Script local variables.
* Runtime Cell overrides.
* Gameplay camera state.

Persistent script fields are a special case within the running ScriptScene: they are script runtime state that survives across lifecycle executions while the runtime scene is alive. They are not automatically written into the authored World unless an explicit authored API performs such a change.

---

# 7. Load Flow

A simplified World/script load flow is:

~~~text
Project files
    ↓
World metadata loader
    ↓
WorldStreamer loads nearby chunks on demand
    ↓
Cell IDs restored
    ↓
Script bindings resolved
    ↓
Runtime systems initialized
~~~

---

# 8. Save Flow

A simplified authored save flow is:

~~~text
Editor-authored state
       ↓
Persistence layer
       ↓
World/project files
~~~

Runtime simulation state should not silently enter this path.

---

# 9. Legacy Data

Persistence code may retain compatibility paths for older World formats.

Examples include legacy Cell type names and older script-binding representations.

Compatibility logic should migrate old data deliberately rather than silently inventing ambiguous object associations.

---

# 10. Authoritative Data Rule

Persistence reinforces the same architectural rule used throughout AeoEngine:

~~~text
Authored World
      ↓
authoritative persistent data

Runtime state
      ↓
temporary derived/simulated data
~~~

Saving should preserve that boundary.
