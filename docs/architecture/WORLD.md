# World Architecture

The `World` is the authoritative representation of authored scene data used by the editor and runtime.

It stores authored Cells on a 3D integer grid and maintains the temporary runtime state required while Play mode is active.

The World owns scene data. It does not own rendering, physics simulation, or script execution.

---

# 1. World Representation

The World contains authored Cells keyed by `WorldCoord`.

The resident in-memory representation is sparse:

~~~text
World
├── authored Cells
│   └── WorldCoord → Cell
├── runtime Cell state
├── Cell-ID indexes
├── spatial indexes
└── World settings
~~~

Empty coordinates do not need to be stored as Cells.

Persistent storage divides authored Cells into 16×16×16 spatial chunks.

```text
WorldCoord
    ↓
ChunkCoord
    ↓
StoredChunk
```

The chunk representation is a persistence structure. It is separate from the renderer's chunk mesh representation.

---

# 2. World Coordinates

`WorldCoord` identifies an authored grid position using integer coordinates.

```text
(x, y, z)
```

The coordinate represents the authored placement of a Cell.

Runtime systems may use continuous positions independently of the authored grid coordinate.

For example, a dynamic PhysicsBody can move continuously while the authored Cell remains at its original World coordinate.

---

# 3. Chunk Coordinates

A `ChunkCoord` identifies a 16×16×16 region of the World.

```text
CHUNK_SIZE = 16
```

Chunk coordinates are used by:

* World spatial indexing.
* Persistent chunk storage.
* Render chunk construction.
* Dirty-chunk tracking.
* World queries that operate on spatial regions.

The storage system owns the on-disk chunk representation.

The renderer owns its own derived GPU chunk representation.

---

# 4. Cells

A Cell is an authored object in the World.

Current Cell types include:

* Block.
* FxBlock.
* Player.
* NPC.
* Light.
* SpawnPoint.
* AudioEmitter.

Depending on the Cell type, authored data may include:

* Cell type.
* Persistent ID.
* Identity/name.
* Visibility.
* Solidity.
* Anchored state.
* Color.
* Texture.
* Light properties.
* Audio properties.
* Collision event state.
* Custom attributes.
* Other type-specific properties.

---

# 5. Persistent Cell IDs

Every authored Cell has a persistent numeric `id`.

The ID identifies the specific authored instance.

```text
Cell
├── id
└── entity_identity
```

The two concepts are different.

`id` is the stable instance identity.

`entity_identity` is the human-facing name.

Multiple Cells may share the same `entity_identity`.

Persistent Cell IDs are used by:

* Script bindings.
* Cell lookup.
* World ID indexes.
* Runtime references to authored Cells.
* Persistence.

The ID is not regenerated simply because a Cell moves.

---

# 6. Cell Identity and Names

`entity_identity` is a human-readable authored name.

For example:

```text
Door
Ghost
Wall
SpawnPoint
```

Names are not required to be unique.

A query such as:

```aeoscript
find("Door")
```

may therefore return multiple Cells.

Name-based lookup is a search operation.

Cell-ID lookup identifies one specific authored instance.

---

# 7. Cell Attributes

Authored Cells may contain custom attributes.

The current attribute value types are:

* Number.
* Bool.
* String.

Example:

```aeoscript
cell.attributes["health"] = 100
cell.attributes["locked"] = true
cell.attributes["difficulty"] = "hard"
```

Authored attributes are part of persistent Cell data.

During Play, AeoScript can create runtime attribute overrides without modifying the authored value.

---

# 8. Runtime Cell State

Runtime state is maintained separately from authored Cell data.

Conceptually:

```text
World
├── Authored Cells
└── Runtime State
      └── Cell ID → temporary overrides
```

Runtime state may contain temporary values for properties such as:

* Visibility.
* Solidity.
* Anchored state.
* Color.
* Offset.
* Light state.
* Audio playback state.
* Runtime attributes.
* Runtime deletion state.

When an effective runtime value exists, runtime systems use it instead of the authored baseline.

---

# 9. Runtime-Created Cells

AeoScript can create runtime-only Cells with:

```aeoscript
cell.new("Block")
```

Runtime-created Cells are stored separately from authored Cells.

They have their own runtime IDs and coordinate indexes.

They are temporary.

They are not written into authored World chunk storage and are discarded when Play mode ends or runtime state is cleared.

---

# 10. World Settings

The World contains authored scene-wide settings.

Current settings include:

* Gravity.
* Global lighting.
* Ambient lighting.
* Sky environment.
* Mouse settings.
* Selected character.
* Selected controller.
* Selected camera.
* Script bindings.
* Disabled scripts.

These values are persistent project data.

---

# 11. World Indexes

The World maintains indexes needed for efficient engine queries.

Important indexes include:

* Cell ID → World coordinate.
* Runtime Cell ID → runtime coordinate.
* World coordinate → runtime Cell ID.
* Authored spatial chunk index.
* Point-light IDs.
* Audio-emitter IDs.
* Editor-marker IDs.
* Dirty authored chunks.
* Physics-dirty Cell IDs.

Indexes are derived from World state and exist to make queries and synchronization efficient.

The indexes do not become separate authorities for Cell data.

---

# 12. World Editing

Editor operations modify authored World data.

Examples include:

* Build.
* Erase.
* Delete.
* Move.
* Paste.
* Property editing.
* Attribute editing.
* Lighting edits.
* Script binding changes.
* Undo.
* Redo.

These operations update the World and its relevant indexes.

Runtime simulation does not automatically become an authored edit.

---

# 13. World and Runtime Systems

The authored World feeds multiple runtime systems:

```text
World
 ├── Renderer
 ├── PhysicsWorld
 ├── CharacterSystem
 ├── ScriptScene
 └── EntityManager
```

Each system creates the runtime representation it owns.

For example:

* Renderer creates render chunks and GPU resources.
* PhysicsWorld creates runtime collision state and PhysicsBodies.
* CharacterSystem creates runtime character state.
* ScriptScene creates runtime script instances and fibers.
* EntityManager owns live runtime Entities.

These representations do not replace the authored World.

---

# 14. Authority

The World is authoritative for persistent scene data.

The architectural boundary is:

```text
Authored World
      ↓
Runtime conversion
      ↓
Temporary runtime state
```

Runtime systems may derive, override, or transform World data for simulation and rendering, but they must not silently become the persistent source of truth.

---

# 15. World Persistence

World persistence is handled by the World storage/persistence layer.

The persistent project consists of:

```text
world.dat
chunks/
```

`world.dat` stores World-wide metadata and project-level World configuration.

Individual authored chunks store the Cells belonging to each `ChunkCoord`.

The persistence architecture is documented separately in:

```text
docs/architecture/PERSISTENCE.md
```

The World architecture is concerned with what the data means; the persistence layer is concerned with how that data is stored and restored.