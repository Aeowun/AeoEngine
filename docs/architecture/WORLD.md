# World Architecture

The `World` is AeoEngine's authoritative representation of authored scene data.

It stores authored Cells on a 3D integer grid and also owns the temporary runtime overrides required while Play mode is running.

---

# 1. World Storage

The authored World stores Cells by `WorldCoord`.

~~~text
World
└── cells
    └── WorldCoord → Cell
~~~

The World is sparse. Empty coordinates do not need to be stored.

---

# 2. World Coordinates

A `WorldCoord` identifies an authored grid position using integer coordinates:

~~~text
(x, y, z)
~~~

The coordinate is the authored placement of a Cell. Runtime physics can use continuous positions independently of this authored coordinate.

---

# 3. Cells

A Cell is an authored object in the World.

Current authored Cell types include Blocks, Lights, SpawnPoints, Players, and NPCs.

Depending on the Cell type, authored data may include:

* Cell type.
* Persistent ID.
* Identity/name.
* Visibility.
* Solidity.
* Anchored state.
* Color.
* Texture.
* Type-specific properties.
* Custom attributes (Number, Bool, String).

---

# 4. Persistent Cell IDs

Every authored Cell has a persistent numeric `id`.

The Cell ID identifies the specific authored instance.

~~~text
Cell
├── id              → unique instance identity
└── entity_identity → human-readable name
~~~

The two concepts must not be conflated.

A name may be shared by multiple Cells. A Cell ID is used when one specific authored instance must be targeted, including script bindings.

---

# 5. Identity / Name

`entity_identity` is the authored human-readable name of a Cell.

For example:

~~~text
Identity: Door
~~~

Multiple Cells can share that identity.

Name-based lookup is therefore a search operation, not a unique-object lookup.

~~~aeoscript
const doors = find("Door")
~~~

The result can contain multiple handles.

---

# 6. Runtime Cell State

The World maintains runtime state separately from authored Cell data.

~~~text
World
├── Authored Cells
└── Runtime Cell State
        └── Cell ID → overrides
~~~

Runtime overrides can affect supported properties such as:

* Visibility.
* Color.
* Solidity.
* Anchored state.
* Visual offset.
* Runtime light state.

When an override exists, runtime access uses the override. Otherwise the authored value remains the source.

---

# 7. Runtime State Is Temporary

Runtime overrides are cleared when Play mode stops.

~~~text
Runtime overrides
      ↓
Play mode stops
      ↓
Runtime state discarded
      ↓
Authored values remain
~~~

This allows scripts and gameplay systems to modify the running game without silently modifying the saved scene.

---

# 8. World-Wide Settings

The World also stores scene-wide authored settings.

### Gravity

The World stores a gravity vector used by runtime physics.

The default gravity is:

~~~text
(0, -9.81, 0)
~~~

### Lighting

The World stores authored lighting configuration including global lighting, direction, color, intensity, ambient intensity, and shadow-related state.

---

# 9. Script Bindings

The World stores authored script bindings.

Each binding associates a persistent Cell ID with an `.aeo` script path.

~~~text
Cell ID → script path
~~~

The binding does not use the human-readable identity as its persistent target.

---

# 10. World and Runtime Systems

The authored World feeds multiple runtime systems:

~~~text
World
 ├── Renderer
 ├── PhysicsWorld
 ├── CharacterSystem
 └── ScriptScene
~~~

Those systems may create their own runtime representations.

For example:

* PhysicsWorld creates runtime PhysicsBodies.
* CharacterSystem creates runtime Character state.
* ScriptScene creates ScriptInstances and fibers.
* Renderer creates GPU resources.

Those derived objects do not replace the authored World.

---

# 11. World Editing

Editor operations such as Build, Erase, Delete, property editing, Undo, and Redo operate on authored World data.

Runtime simulation should not silently become an authored edit.

---

# 12. World Authority

The fundamental rule is:

~~~text
Authored World = persistent scene truth

Runtime systems = temporary simulation state
~~~

Future architecture changes should preserve that distinction.

