# AeoScript API

This document lists the properties, methods, and functions currently available to AeoScript.

---

# 1. Global Functions

* `print(value)`: Prints a value to the editor terminal.
* `debug.log(value)`: Prints a value to the editor terminal with `SCRIPT` severity.
* `getAllCellsOfClass(class_name)`: Finds all authored world cells of a specified type, such as `"Light"`, `"Block"`, or `"SpawnPoint"`. Returns a Basket of `Cell` handles.
* `find(query)`: Finds Entities or Cells matching an identity or tag. Returns a Basket of handles.
* `get_entity(name)`: **Legacy.** Finds a runtime entity by name. Returns an `Entity` handle or `nil`.
* `get_light(x, y, z)`: **Legacy.** Finds a light cell at the specified coordinates. Returns a `Cell` handle or `nil`.
* `wait(seconds)`: Pauses the current script for the specified duration.

---

# 2. Common Object Properties

Cells and Entities expose the following properties:

* `.id`: The object's ID. Read-only.
* `.name`: The name or identity string assigned in the editor. Read-only.
* `.position`: The object's current world position. Returns a Basket `[x, y, z]`. Read-only for Cells.

---

# 3. Cell Handles

A `Cell` handle refers to authored world content. Changes made to writable Cell properties are runtime changes and are discarded when the game stops.

### Properties

* `.cellType`: The cell type, such as `"Block"` or `"Light"`. Read-only.
* `.visible`: Whether the cell is rendered. Read/Write.
* `.enabled`: Whether a Light cell is emitting. Read/Write.
* `.solid`: Whether the cell participates in physics collisions. Read/Write.
* `.anchored`: Whether the cell is fixed in place or affected by gravity. Read/Write.
* `.color`: The current RGB color. Returns or accepts a Basket `[r, g, b]`. Read/Write.
* `.offset`: A visual displacement from the cell's authored position. Returns or accepts a Basket `[x, y, z]`. Read/Write.

### Methods

* `:getObject()`: Returns the runtime `Entity` associated with the cell, if one exists. Otherwise returns `nil`.

---

# 4. Entity Handles

An `Entity` handle refers to a live runtime object, such as a spawned Player or NPC.

### Properties

* `.id`: The runtime entity ID. Read-only.
* `.name`: The entity's name. Read-only.
* `.position`: The entity's current world position. Returns a Basket `[x, y, z]`. Read-only.

### Methods

* `:is_valid()`: Returns whether the entity still exists.
* `:destroy()`: Removes the entity from the world.
* `:set_position(x, y, z)`: Moves the entity to the specified position.
* `:translate(dx, dy, dz)`: Moves the entity relative to its current position.

---

# 5. Basket

A `Basket` is a 0-indexed list of values.

* `.len()`: Returns the number of items.
* `[index]`: Returns the item at the specified index.

Chained access is supported:

```aeoscript
obj.position[1]
```

---

# 6. Global Host Objects

### `time`

* `time.delta`: Duration of the previous frame in seconds.

---

# 7. Errors

The following operations produce runtime errors:

* Calling a method on `nil`.
* Passing the wrong number of arguments to an engine function.
* Passing an argument of the wrong type.
* Modifying a property on an invalid handle, such as a destroyed Entity.

Errors are reported in the **PRINT OUTPUT** panel with the script path and execution context.
