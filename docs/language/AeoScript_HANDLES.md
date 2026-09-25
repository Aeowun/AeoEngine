# AeoScript Engine Handles

AeoScript interacts with AeoEngine through opaque engine handles.

Handles identify engine-managed objects without transferring native object ownership to the script runtime.

---

# 1. Handle Model

The basic relationship is:

```text
AeoScript Handle
      ↓
Engine lookup
      ↓
Engine-owned object
```

The script stores an identifier and handle kind.

The engine remains responsible for object lifetime.

---

# 2. Cell Handles

A `Cell` handle can refer to an authored World Cell or a runtime-created Cell.

Cell handles expose supported properties through the script host.

Examples include:

```text
id
name
cellType
visible
solid
anchored
color
offset
position
attributes
```

---

# 3. Cell IDs

Every authored Cell has a persistent numeric ID.

```aeoscript
const id = cell.id
```

The ID is read-only from AeoScript.

The same ID is used for persistent script bindings.

---

# 4. Cell Names

A Cell's human-readable identity/name is not required to be unique.

```aeoscript
const name = cell.name
```

Two different Cells may therefore have:

```text
ID: 1001   Name: Door
ID: 1002   Name: Door
```

Use the ID for instance-specific identity.

Use the name for discovery and gameplay-oriented queries.

---

# 5. Cell Runtime Properties

Supported runtime Cell properties include:

* `visible`
* `solid`
* `anchored`
* `color`
* `offset`
* `position` where supported
* `attributes`

Example:

```aeoscript
cell.visible = false
cell.solid = false
cell.color = [1, 0, 0]
cell.offset = [0, 2, 0]
```

During Play these are runtime changes.

---

# 6. Cell Attributes

Cell handles expose authored/runtime attributes.

```aeoscript
cell.attributes["health"] = 100
```

Authored attributes form the persistent baseline.

Runtime attribute changes are temporary overrides.

---

# 7. Dynamic Handle Properties

Supported engine handles can expose dynamic script properties.

```aeoscript
const door = cell.get(1001)

door.is_open = false

door.toggle = fn() {
    self.is_open = !self.is_open
    self.visible = !self.is_open
}
```

The `self` value refers to the handle receiving the method-style call.

Dynamic properties are script/runtime state and are not automatically written into the authored World.

---

# 8. Explicit Cell Lookup

A specific Cell can be looked up by persistent ID:

```aeoscript
const door = cell.get(1001)
```

The lookup returns `nil` when the ID is not present.

---

# 9. Cell Discovery

Name-based discovery:

```aeoscript
const doors = find("Door")
```

Class-based discovery:

```aeoscript
const blocks = getAllCellsOfClass("Block")
```

Name-based results can contain multiple handles.

---

# 10. Entity Handles

An `Entity` handle refers to a live runtime Entity such as a Player or NPC.

Typical properties include:

* `name`
* `position`

Entity handles represent runtime state rather than authored Cell data.

---

# 11. Entity Lookup

A specific runtime Entity can be resolved with:

```aeoscript
const player = entity.get(1)
```

and tested with:

```aeoscript
entity.exists(1)
```

A missing Entity returns `nil` from `entity.get()`.

---

# 12. Sound Handles

A `Sound` handle represents runtime audio control.

Supported state includes:

* `playing`
* `looped`
* `volume`

Supported operations include:

* `play()`
* `stop()`
* `pause()`

Example:

```aeoscript
const speaker = find("DoorSound")[0]
speaker.sound.play()
```

---

# 13. UI Handles

A `Ui` handle represents a runtime UI element.

Supported UI properties include:

* `position`
* `size`
* `visible`
* `enabled`
* `color`
* `text`
* `on_click`

Button callbacks use function values and can capture lexical state.

---

# 14. Mouse Handles

A mouse handle is retrieved through:

```aeoscript
const mouse = get.mouse()
```

Supported controls include:

```text
setCursorVisible
setScreenLocked
```

---

# 15. Handle Lifetime

A handle can remain in script state after its underlying object has been destroyed.

The engine must therefore resolve the handle against current runtime state.

Operations against stale or invalid handles produce runtime errors or absence results according to the API being used.

A stale handle must never silently redirect to another engine object.

---

# 16. Ownership

Handles do not transfer ownership.

```text
Script
  │
  └── Handle
        ↓
   Engine-owned object
```

The engine owns:

* Allocation.
* Lifetime.
* Destruction.
* Native resources.
* Runtime object storage.

The scripting runtime owns only the script-side value representing the handle.

---

# 17. Handles and Script Bindings

Script bindings use persistent authored Cell IDs.

For example:

```text
Authored Cell
├── id: 18374291
└── name: Ghost

Script binding
└── target ID: 18374291
```

Changing the human-readable name does not change the binding target.

---

# 18. Handles and Runtime State

Handles expose effective runtime state, not a second authoritative World representation.

The architectural relationship is:

```text
Authored World
      ↓
Runtime effective state
      ↓
Handle resolution
      ↓
AeoScript
```

The script runtime therefore interacts with engine-owned state without becoming the owner of that state.