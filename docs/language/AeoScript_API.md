# AeoScript API Reference

This document describes the currently exposed AeoScript functions, namespaces, engine handles, properties, and runtime-facing behavior.

The API is implemented through the AeoEngine script host. Runtime changes are temporary unless an explicit authored-data operation changes persistent project state.

---

# 1. Global Functions and Properties

## `debug.log(...)`

Writes a structured message to the AeoEngine script output/diagnostic system.

```aeoscript
debug.log("Health:", health)
```

---

## `find(name)`

Searches active/effective World Cells by `entity_identity` and runtime Entities by their runtime name.

Returns a `basket` of handles.

Names are not required to be unique, so multiple results are possible.

```aeoscript
const doors = find("Door")
```

---

## `getAllCellsOfClass(class_name)`

Returns active/effective World Cells of the requested class.

```aeoscript
const blocks = getAllCellsOfClass("Block")
```

---

## `wait(seconds)`

Suspends the current fiber for a finite positive duration of scheduler time.

The fiber preserves its execution state and resumes at the suspension point.

```aeoscript
wait(0.5)
```

---

## `time.delta`

Returns the current frame delta time supplied to the script host.

```aeoscript
speed += acceleration * time.delta
```

---

# 2. `math`

### `math.abs(x)`

Absolute value.

### `math.min(a, b)`

Smaller of two numeric values.

### `math.max(a, b)`

Larger of two numeric values.

### `math.floor(x)`

Largest integer less than or equal to `x`.

### `math.ceil(x)`

Smallest integer greater than or equal to `x`.

### `math.round(x)`

Rounds to the nearest integer.

### `math.sqrt(x)`

Square root.

### `math.pow(base, exponent)`

Raises `base` to `exponent`.

### `math.sin(x)`

Sine of radians.

### `math.cos(x)`

Cosine of radians.

### `math.tan(x)`

Tangent of radians.

### `math.clamp(value, min, max)`

Clamps `value` to `[min, max]`.

### `math.lerp(a, b, t)`

Linear interpolation:

```text
a + (b - a) * t
```

### `math.deg_to_rad(degrees)`

Degrees to radians.

### `math.rad_to_deg(radians)`

Radians to degrees.

### `math.random()`

Returns a floating-point value in:

```text
[0.0, 1.0)
```

### `math.random(min, max)`

With integer arguments, returns an integer in the inclusive range:

```text
[min, max]
```

---

# 3. `basket`

Baskets are zero-indexed reference-backed sequences.

Namespace calls:

```aeoscript
basket.insert(items, 0, "Sword")
```

Method-style calls are also supported where implemented:

```aeoscript
items.insert(0, "Sword")
```

### `basket.create(count, [value])`

Creates a basket with `count` elements.

### `basket.insert(b, [position], value)`

Inserts a value.

### `basket.remove(b, [position])`

Removes and returns an element.

### `basket.clear(b)`

Removes all elements.

### `basket.find(b, value, [init])`

Returns the first matching index or `nil`.

### `basket.move(src, a, b, t, [dst])`

Moves/copies an inclusive source range.

The operation is overlap-safe.

### `basket.concat(b, [separator], [i], [j])`

Joins basket elements into a string.

### `basket.clone(b)`

Creates a shallow outer copy.

### `basket.freeze(b)`

Marks the basket read-only.

### `basket.sort(b)`

Sorts numeric or string values in ascending order.

Custom comparator functions are not currently supported.

---

# 4. `string`

String operations use Unicode scalar values.

### `string.len(s)`

Returns Unicode scalar-value length.

### `string.lower(s)`

Lowercase conversion.

### `string.upper(s)`

Uppercase conversion.

### `string.reverse(s)`

Reverses Unicode scalar values.

### `string.split(s, [separator])`

Returns a basket of strings.

Without a separator, the string is split into individual Unicode scalar values.

---

# 5. Maps

Maps use direct indexing and assignment.

```aeoscript
const values = {}

values["name"] = "Ghost"
values[1] = "numeric"
```

Numeric and string keys are distinct.

Missing keys return `nil`.

Assigning `nil` removes a key.

There is no separate map namespace.

---

# 6. Cell Handles

A `Cell` handle refers to an engine-managed authored or runtime Cell.

## `.id`

Persistent numeric ID.

Read-only.

## `.name`

Human-readable identity/name.

Read-only for authored Cells.

Names are not required to be unique.

## `.cellType`

Cell type as a string.

Read-only.

Examples:

```text
Block
Light
SpawnPoint
AudioEmitter
```

## `.visible`

Runtime visibility override.

## `.solid`

Runtime collision override.

## `.anchored`

Runtime static/dynamic override.

## `.color`

RGB color:

```aeoscript
cell.color = [1, 0, 0]
```

## `.offset`

Runtime visual offset.

```aeoscript
cell.offset = [0, 2, 0]
```

## `.position`

Effective runtime position.

Runtime-created Cells can move using this property.

## `.attributes`

Cell attribute access.

```aeoscript
cell.attributes["coins"] = 100
```

Runtime attribute writes create temporary overrides.

## Dynamic Properties

Supported engine handles can expose dynamic script properties and functions.

```aeoscript
cell.myValue = 10

cell.toggle = fn() {
    self.visible = !self.visible
}
```

---

# 7. `cell`

## `cell.new(type_name)`

Creates a runtime-only Cell.

Supported types include:

```text
Block
FxBlock
Player
NPC
Light
SpawnPoint
```

Returns a Cell or Light handle as appropriate.

## `cell.delete(handle)`

Removes the target from the active runtime world.

Authored Cells are temporarily removed for the Play session.

Runtime-created Cells are destroyed.

## `cell.get(id)`

Returns the Cell with the specified ID or `nil`.

## `cell.exists(id)`

Returns whether the specified Cell exists in the current runtime world.

---

# 8. Entity Handles

An `Entity` handle refers to a live runtime Entity.

## `.name`

Runtime Entity name.

## `.position`

Current runtime world position.

## Dynamic Properties

Runtime Entities can expose dynamic script properties.

---

# 9. `entity`

## `entity.get(id)`

Returns a live Entity handle or `nil`.

## `entity.exists(id)`

Returns whether a runtime Entity exists.

---

# 10. Function Values and `self`

Functions are first-class values.

```aeoscript
const callback = fn(x) {
    return x * 2
}
```

A function may be called:

```aeoscript
callback(10)
```

When a function is called using method syntax:

```aeoscript
object:method()
```

`self` refers to the handle on which the method was invoked.

---

# 11. Runtime State

Cell changes made during Play affect runtime state.

```aeoscript
cell.visible = false
cell.solid = false
cell.color = [1, 0, 0]
```

These do not rewrite authored World values.

When Play stops, runtime overrides are discarded.

---

# 12. Script Lifecycle

Common lifecycle functions include:

```aeoscript
fn on_spawn() { }
fn on_ready() { }
fn update(dt) { }
fn on_destroy() { }
```

Collision lifecycle events include:

```aeoscript
fn on_touch(cell) { }
fn on_overlap(overlapping, cell) { }
```

Persistent entity fields survive between lifecycle executions.

---

# 13. Script Bindings

Bindings identify a specific authored Cell by persistent Cell ID.

```text
Cell ID → script path
```

The Cell's human-readable name is not the binding key.

---

# 14. Mouse and Input

## `get.mouse()`

Returns the runtime mouse handle.

### Mouse properties

```text
setCursorVisible
setScreenLocked
```

Example:

```aeoscript
const mouse = get.mouse()

mouse.setCursorVisible = false
mouse.setScreenLocked = true
```

## `input.get_move_vector()`

Returns the current movement vector.

## `input.is_jump_pressed()`

Returns whether jump input is currently pressed.

## `input.get_orbit_delta()`

Returns the mouse-orbit delta.

---

# 15. Camera

## `camera.get_horizontal_basis()`

Returns `[forward, right]` horizontal basis vectors.

## `camera.set_position(...)`

Sets gameplay camera position.

## `camera.set_target(...)`

Sets gameplay camera target.

## `camera.set_orientation(yaw, pitch)`

Sets gameplay camera orientation.

---

# 16. Physics Query

## `physics.resolve_camera_collision(target, desired)`

Resolves a camera position against collision geometry.

Returns the resolved position.

```aeoscript
const actual = physics.resolve_camera_collision(target, desired)
```

---

# 17. Player

## `player.set_horizontal_velocity(vx, vz)`

Sets runtime player horizontal velocity.

## `player.set_facing_direction(dx, dz)`

Sets runtime player facing direction.

## `player.select_animation(name)`

Selects a runtime character animation.

## `player.is_grounded()`

Returns whether the active player is grounded.

## `player.apply_vertical_impulse(impulse)`

Applies a vertical impulse.

## `player.position`

Returns the active player's runtime position.

---

# 18. Runtime UI

## `ui.new(element_type)`

Creates a runtime UI element.

Supported types:

```text
Panel
Text
Button
```

## `ui.delete(handle)`

Removes a runtime UI element.

## `ui.get_viewport_size()`

Returns:

```text
[width, height]
```

## UI properties

Supported properties include:

* `position`
* `size`
* `visible`
* `enabled`
* `color`
* `text`
* `on_click`

Button `on_click` accepts a function value.

---

# 19. Audio

AudioEmitter Cells expose a sound handle.

```aeoscript
const speaker = find("DoorSound")[0]

speaker.sound.play()
```

Supported sound properties:

* `playing`
* `looped`
* `volume`

Supported methods:

* `play()`
* `stop()`
* `pause()`

---

# 20. Script Control

## `script.enable(path)`

Enables script execution for a script path.

## `script.disable(path)`

Disables script execution for a script path and cancels active fibers associated with that script.

## `script.is_enabled(path)`

Returns the current enabled state.

This is runtime script execution control, not deletion of the underlying source file or authored script binding.

---

# 21. Events

## `event.fire(name, ...)`

Dispatches an engine/script event with the supplied arguments.

Event handlers can respond through `on` declarations.

---

# 22. Automated Test API

## `test.complete(name, passed)`

Completes a named in-game test.

## `test.is_completed(name)`

Returns whether the named test has completed.

## `test.passed(name)`

Returns whether the named test passed.

## `test.summary()`

Returns:

```text
[passed_count, failed_count, total_count]
```

---

# 23. Current API Boundary

The API intentionally does not expose arbitrary access to internal Rust state.

Not every engine subsystem is exposed directly to AeoScript.

The preferred direction is:

```text
AeoScript intent
      ↓
Small host API
      ↓
Owning engine subsystem
```

New APIs should be added when a concrete gameplay workflow requires them.

The language should avoid exposing implementation-only architecture through script syntax.