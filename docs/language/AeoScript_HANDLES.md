# AeoScript Engine Handles

AeoScript interacts with AeoEngine through opaque engine handles.

Handles allow scripts to reference engine-managed objects without directly owning native engine memory.

---

# 1. Why Handles Exist

A script should be able to hold a reference to a World Cell or runtime Entity without storing a raw pointer into engine memory.

Instead, the script value identifies an engine-managed object.

~~~text
AeoScript handle
      ↓
Engine lookup
      ↓
Authoritative engine object
~~~

This keeps object ownership inside AeoEngine.

---

# 2. Cell Handles

A `Cell` handle refers to an authored World Cell.

Examples include Blocks and Lights.

Cell handles expose supported runtime properties through the scripting host.

---

# 3. Cell ID

Every authored Cell has a persistent numeric `id`.

This identifies the specific Cell instance.

~~~aeoscript
const id = cell.id
~~~

The ID is read-only from script.

The ID is also the target used by persistent script bindings.

---

# 4. Cell Name

A Cell also exposes its human-readable name/identity.

~~~aeoscript
const name = cell.name
~~~

Names are not required to be unique.

Two different Cells can therefore have the same `name` while having different IDs.

---

# 5. Cell Runtime Properties

Supported Cell runtime properties include:

* `visible`
* `solid`
* `anchored`
* `color`
* `offset`
* `position` as effective runtime position where supported

Example:

~~~aeoscript
cell.visible = false
cell.solid = false
cell.color = [1, 0, 0]
cell.offset = [0, 2, 0]
~~~

These are runtime changes and do not silently modify authored World values.

---

# 6. Dynamic Properties and Methods

Engine handles (both Cells and Entities) support dynamic property assignment. This allows scripts to attach custom data or logic to specific engine objects without needing to modify the engine itself.

~~~aeoscript
const door = cell.get(10552341)

// Store custom data
door.isOpen = false

// Assign a custom method
door.toggle = fn() {
    self.isOpen = !self.isOpen
    self.visible = !self.isOpen
    self.solid = !self.isOpen
}

// Call the custom method
door:toggle()
~~~

### The `self` Keyword

Inside a function called as a method using the colon syntax (`handle:method()`), the `self` variable automatically refers to the handle the method was called on.

---

# 7. Explicit Handle Lookup

While `find()` and `getAllCellsOfClass()` are useful for discovery, scripts can also target specific instances directly by their persistent ID.

~~~aeoscript
const trap = cell.get(882210)
const player = entity.get(1)
~~~

These functions return `nil` if the specified object does not exist.

---

# 8. Finding Cells

Scripts can discover Cells by name:

~~~aeoscript
const walls = find("Wall")
~~~

The result is a basket of handles.

Because names can be duplicated, `find()` may return multiple Cells.

Scripts can also query by class:

~~~aeoscript
const blocks = getAllCellsOfClass("Block")
~~~

---

# 7. Entity Handles

An `Entity` handle refers to a live runtime Entity such as a Player or NPC.

Entity handles represent runtime objects rather than authored World Cells.

Supported properties include `.name` and `.position`. Supported methods include `is_valid()`, `name()`, `set_position(x, y, z)`, and `translate(dx, dy, dz)`.

---

# 8. Sound Handles

A `Sound` handle refers to an audio channel bound to an `AudioEmitter` cell or runtime sound object (`cell.sound`).

Supported properties and methods:

* `sound.playing`: `bool` (read/write)
* `sound.looped`: `bool` (read/write)
* `sound.volume`: `number` (read/write)
* `sound.play()`: method
* `sound.stop()`: method
* `sound.pause()`: method

---

# 9. UI Handles

A `Ui` handle refers to a runtime UI element created with `ui.new("Panel" | "Text" | "Button")`.

Supported properties:

* `position`: `[x, y]` pixel position
* `size`: `[width, height]` pixel dimensions
* `visible`: `bool`
* `enabled`: `bool`
* `color`: `[r, g, b, a]` color vector
* `text`: `string` (Text and Button)
* `on_click`: callback function assigned to Button handles (`btn.on_click = fn() { ... }`)

---

# 10. Handle Lifetime

A handle can outlive the engine object it originally referenced.

The script runtime must resolve the handle against current engine state rather than assuming the underlying object still exists.

Operations against invalid or stale handles produce runtime errors rather than silently affecting another object.

---

# 11. Handles and Runtime State

Handles provide access to runtime state.

They do not transfer ownership of the underlying engine object into the script.

~~~text
Script
  │
  └── Handle
          ↓
     Engine-owned object
~~~

The engine remains responsible for the object's lifetime.

---

# 12. Handles and Script Bindings

A handle's Cell ID and a script binding's target ID describe the same authored instance identity.

This is why bindings do not use the human-readable name as their unique target.

~~~text
Authored Cell
├── id: 18374291
└── name: Ghost

Script Binding
└── target ID: 18374291
~~~

The name can change without changing which Cell the binding targets.


