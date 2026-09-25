# Persistence Architecture

AeoEngine persistence stores authored project data separately from temporary runtime simulation state.

The persistence layer restores authored World state and project metadata. It does not serialize transient gameplay simulation.

---

# 1. Persistence Boundary

The fundamental boundary is:

```text
Authored state
      ↓
Persistence
      ↓
Project files
```

Runtime simulation follows a separate path:

```text
Runtime state
      ↓
Play mode
      ↓
Discarded when runtime ends
```

Persistence must not silently serialize temporary physics, character, script, camera, or UI state as authored World data.

---

# 2. World Format

The current World format contains:

```text
world.dat
chunks/
```

`world.dat` stores World-wide metadata and project-level World configuration.

Authored Cell data is stored in individual chunk files under:

```text
chunks/
```

Chunk files use the coordinate form:

```text
<x>_<y>_<z>.chunk
```

---

# 3. World Metadata

`world.dat` currently stores data such as:

* World format version.
* Next authored Cell ID.
* Gravity.
* Global lighting settings.
* Sky settings.
* Mouse settings.
* Script bindings.
* Disabled scripts.
* Selected character.
* Selected controller.
* Selected camera.

The exact serialized format is implementation detail and may evolve through explicit format versions.

---

# 4. Chunk Storage

Authored Cells are grouped by 16×16×16 `ChunkCoord`.

Each stored chunk contains:

* Storage version.
* Chunk coordinate.
* Stored Cell records.

Each stored Cell contains the persistent authored information needed to reconstruct the runtime `Cell`.

This includes data such as:

* Cell ID.
* Cell type.
* Local chunk coordinate.
* Visibility.
* Solidity.
* Anchoring.
* Texture.
* Color.
* Audio properties.
* Collision event state.
* Light properties.
* Entity identity.
* Attributes.

Runtime-only state is not stored in the chunk representation.

---

# 5. WorldStorage

`WorldStorage` owns the filesystem relationship between a `ChunkCoord` and its chunk file.

Its responsibilities include:

* Listing stored chunks.
* Checking whether a chunk exists.
* Loading a chunk.
* Saving a chunk.
* Deleting a chunk.
* Constructing a stored chunk from authored Cells.

It does not own:

* Rendering.
* Physics.
* Characters.
* Scripting.
* Camera behavior.
* Editor interaction.

---

# 6. Cell IDs

Persistent Cell IDs are saved as authored data.

Loading restores the same ID for the same authored Cell.

The World rebuilds its ID lookup indexes after loading.

The persistent ID is therefore stable across:

```text
Editor session
      ↓
Save
      ↓
Load
```

provided the authored Cell remains the same instance.

---

# 7. Script Binding Persistence

Script bindings are authored World metadata.

The current binding model is:

```text
Cell ID → script path + enabled state
```

The Cell's human-readable identity/name is not the persistent binding key.

Legacy name-based bindings may be migrated when they resolve to exactly one authored Cell.

Ambiguous legacy bindings must not guess which Cell was intended.

---

# 8. Attributes

Cell attributes are authored Cell data and are stored with the Cell.

Current attribute types are:

* Number.
* Bool.
* String.

Attributes therefore survive:

```text
Save
 ↓
Chunk storage
 ↓
Load
```

Runtime attribute overrides are not persisted.

---

# 9. Runtime State

The following are examples of runtime state and are not written back into authored World persistence automatically:

* Dynamic PhysicsBody position.
* Velocity.
* Sleeping state.
* Runtime character position.
* Runtime Character state.
* Script fiber position.
* Script local scopes.
* Runtime Cell overrides.
* Runtime-created Cells.
* Gameplay camera state.
* Runtime UI state.
* Audio playback state.

Persistent script fields belong to the running ScriptScene and are not automatically authored World data.

---

# 10. Save Flow

The authored save path is conceptually:

```text
World
  ↓
Collect authored metadata
  ↓
world.dat

World
  ↓
Group authored Cells by ChunkCoord
  ↓
WorldStorage
  ↓
chunks/*.chunk
```

Empty obsolete chunk files are removed during a complete World save.

---

# 11. Load Flow

The load path is conceptually:

```text
world.dat
    ↓
World metadata
    ↓
WorldStorage
    ↓
Stored chunks
    ↓
Authored Cells
    ↓
World indexes
```

Older World formats can be handled through explicit compatibility and migration logic.

---

# 12. Safe Chunk Writes

Chunk saves use a temporary file before replacing the destination chunk.

This reduces the risk of leaving a partially written chunk after a failed serialization or filesystem write.

---

# 13. Compatibility

Persistence changes should be versioned deliberately.

The loader must not silently reinterpret older data as a different meaning.

When ambiguity exists, the loader should prefer:

* Explicit migration.
* Tolerant loading where safe.
* Diagnostics for stale or ambiguous data.
* Preserving existing authored data.

---

# 14. Authority

Persistence is responsible for storing authored data.

It is not the authority for runtime simulation.

```text
Editor
  ↓
Authored World
  ↓
Persistence
```

and separately:

```text
Authored World
  ↓
Runtime conversion
  ↓
Temporary simulation
```

This preserves the authored/runtime boundary across save and load operations.