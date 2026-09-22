# AeoScript API Reference

This document provides a reference for the built-in functions, namespaces, engine handles, properties, and runtime behavior currently available in AeoScript.

The API is intentionally integrated with AeoEngine's authored World and runtime systems. Runtime changes remain temporary and do not modify the authored World unless an engine API explicitly performs an authored edit.

---

# 1. Global Functions

### `debug.log(...)`

```aeoscript
debug.log("Hello")
debug.log("Health:", health)
```

Writes a structured script message to the AeoEngine output/diagnostic system.

---

### `find(name)`

```aeoscript
const matches = find("Ghost")
```

Finds authored Cells and runtime Entities whose authored/name identity matches `name`.

Returns a `basket` of handles.

Names are not required to be unique, so the result may contain multiple objects.

---

### `getAllCellsOfClass(class_name)`

```aeoscript
const blocks = getAllCellsOfClass("Block")
```

Returns a `basket` containing handles to authored Cells of the specified class.

Examples include:

```text
"Block"
"Light"
```

---

### `wait(seconds)`

```aeoscript
wait(0.5)
```

Suspends the current AeoScript fiber for the specified duration.

The fiber resumes after the wait while preserving its execution state, including:

* Current function
* Nested function calls
* Local variables
* Call-stack state
* Loop state
* Instruction position

A function called from another function may also yield:

```aeoscript
fn delayed_action() {
    wait(0.5)
    score += 10
}

fn update(dt) {
    delayed_action()
}
```

The suspended execution resumes inside `delayed_action()` rather than restarting the function.

---

# 2. `math` Namespace

Provides deterministic mathematical operations.

### `math.abs(x)`

Returns the absolute value of `x`.

### `math.min(...)`

Returns the smallest supplied numeric argument.

```aeoscript
const lowest = math.min(8, 3, 12, 1)
```

### `math.max(...)`

Returns the largest supplied numeric argument.

```aeoscript
const highest = math.max(8, 3, 12, 1)
```

### `math.floor(x)`

Returns the largest integer less than or equal to `x`.

### `math.ceil(x)`

Returns the smallest integer greater than or equal to `x`.

### `math.round(x)`

Rounds `x` to the nearest integer.

### `math.sqrt(x)`

Returns the square root of `x`.

### `math.pow(base, exponent)`

Returns `base` raised to `exponent`.

### `math.sin(x)`

Returns the sine of `x`, where `x` is in radians.

### `math.cos(x)`

Returns the cosine of `x`, where `x` is in radians.

### `math.tan(x)`

Returns the tangent of `x`, where `x` is in radians.

### `math.clamp(value, min, max)`

Restricts `value` to the inclusive range `[min, max]`.

Produces a runtime error when `min > max`.

### `math.lerp(a, b, t)`

Returns linear interpolation between `a` and `b`:

```text
a + (b - a) * t
```

### `math.deg_to_rad(degrees)`

Converts degrees to radians.

### `math.rad_to_deg(radians)`

Converts radians to degrees.

### `math.random([min, max])`

Generates a pseudo-random number.

*   When called without arguments, returns a float in the range `[0.0, 1.0)`.
*   When called with two integer arguments `min` and `max`, returns an integer in the inclusive range `[min, max]`.

```aeoscript
const chance = math.random()
const roll = math.random(1, 20)
```

---

# 3. `basket` Namespace

A `basket` is a zero-indexed, reference-backed sequence.

Namespace functions may be called directly:

```aeoscript
basket.insert(items, 0, "Sword")
```

Supported basket methods may also be called through the basket:

```aeoscript
items.insert(0, "Sword")
```

Aliases refer to the same underlying basket.

---

### `basket.create(count, [value])`

Creates a new basket containing `count` elements.

When `value` is omitted, elements are initialized to `nil`.

```aeoscript
const a = basket.create(3, 5)
const b = basket.create(3)
```

---

### `basket.insert(b, [position], value)`

Inserts `value` at the specified position.

When the position is omitted, the value is appended.

Basket indices are zero-based.

---

### `basket.remove(b, [position])`

Removes and returns an element.

When the position is omitted, the final element is removed.

---

### `basket.clear(b)`

Removes all elements from the basket.

Because baskets are reference-backed, all aliases observe the cleared state.

---

### `basket.find(b, value, [init])`

Returns the zero-based index of the first matching value.

`init` defaults to `0`.

Returns `nil` when the value cannot be found.

---

### `basket.move(src, a, b, t, [dst])`

Copies the inclusive source range `src[a..b]` into destination position `t`.

When `dst` is omitted, the source basket is used as the destination.

The operation is overlap-safe.

---

### `basket.concat(b, [separator], [i], [j])`

Returns a string by joining the specified basket elements.

`i` and `j` define an inclusive range.

When the range is omitted, the entire basket is used.

---

### `basket.clone(b)`

Returns a new shallow copy of `b`.

The outer basket is independent from the original, but referenced nested values remain shared.

---

### `basket.freeze(b)`

Marks the basket as read-only.

Any later mutation attempt produces a runtime error.

---

### `basket.sort(b)`

Sorts numeric or string elements in ascending order.

Custom comparator functions are not currently supported.

---

# 4. `string` Namespace

String operations operate on Unicode scalar values rather than UTF-8 bytes.

### `string.len(s)`

Returns the number of Unicode scalar values in `s`.

```aeoscript
string.len("é")
```

returns `1`.

---

### `string.lower(s)`

Returns a lowercase version of `s`.

---

### `string.upper(s)`

Returns an uppercase version of `s`.

---

### `string.reverse(s)`

Returns `s` with its Unicode scalar values in reverse order.

---

### `string.split(s, [separator])`

Returns a `basket` of strings.

When a separator is supplied, the string is split using that separator.

When the separator is omitted or empty, the string is split into individual Unicode scalar values.

---

# 5. Maps

Maps are reference-backed key/value collections.

Maps are not positional sequences.

Numeric and string keys are distinct:

```aeoscript
const values = {}

values[1] = "numeric"
values["1"] = "string"
```

Reading a missing key returns `nil`:

```aeoscript
const missing = values["does_not_exist"]
```

Assigning `nil` removes a key:

```aeoscript
values["name"] = nil
```

Maps can contain baskets and nested maps, and those values retain reference semantics.

There is currently no separate `map` namespace; map operations use indexing and assignment directly.

---

# 6. Cell Handles

A `Cell` handle refers to an authored World Cell such as a Block or Light.

Cell property changes made during Play mode are runtime overrides.

They do not modify the authored World.

## Properties

### `.id`

Persistent numeric ID of the specific authored Cell.

Read-only.

The ID identifies the unique Cell instance and is used by script bindings.

Do not use the human-readable name as the unique object identifier.

---

### `.name`

Human-readable authored identity/name.

Read-only.

Names do not have to be unique.

Multiple Cells may therefore return from:

```aeoscript
find("Wall")
```

---

### `.cellType`

String representation of the Cell type.

Read-only.

Examples include:

```text
"Block"
"Light"
```

---

### `.visible`

Controls whether the Cell is rendered.

Read/write.

Runtime override.

---

### `.solid`

Controls whether the Cell participates in collision.

Read/write.

Runtime override.

---

### `.anchored`

Controls whether the Cell remains static or behaves as a dynamic physical object.

Read/write.

Runtime override.

---

### `.color`

RGB color represented by a three-element basket:

```aeoscript
cell.color = [1, 0, 0]
```

Read/write.

Runtime override.

---

### `.offset`

Visual offset from the Cell's authored position:

```aeoscript
cell.offset = [0, 2, 0]
```

Read/write.

Runtime override.

---

### `.position`

Effective runtime world position.

For authored Cells, this is typically read-only. For runtime-created Cells, this is read/write and allows the Cell to be moved between grid coordinates.

---

### `.collisionEventsEnabled`

Controls whether this Cell dispatches the `on_touch` event.

Read/write.

---

### `.attributes`

A map-like interface to the Cell's authored and runtime attributes.

```aeoscript
const coins = cell.attributes["coins"]
cell.attributes["active"] = true
```

Assigning `nil` to a runtime attribute removes the override and restores the authored value.

---

### Dynamic Properties

Cells support dynamic property assignment. You can store arbitrary data or functions directly on a handle.

```aeoscript
const c = cell.get(12345)
c.myValue = 10
c.on_touch = fn(other) { debug.log("Touched!") }
```

All mutations made to Cell handles during Play mode are treated as **runtime overrides** or **temporary script state**. They never modify the authored World file.

---

# 7. `cell` Namespace

Provides utilities for dynamic Cell management during Play mode.

### `cell.new(type_name)`

Creates a new runtime-only Cell of the specified type (e.g., `"Block"`, `"Light"`).

Returns a `Cell` handle.

Runtime Cells exist only during the current Play session and are not persisted to the World file.

### `cell.delete(handle)`

Removes a Cell from the active world.

*   If the target is a runtime-created Cell, it is destroyed.
*   If the target is an authored Cell, it is temporarily removed from the Play session but remains unchanged in the authored World.

### `cell.get(id)`

Returns a `Cell` handle for the authored or runtime Cell with the specified persistent ID.

Returns `nil` if no Cell with that ID exists.

### `cell.exists(id)`

Returns `true` if a Cell with the specified ID exists in the current session.

---

# 8. Entity Handles

An `Entity` handle refers to a live runtime object such as a Player or NPC.

Entity handles are runtime references rather than authored World Cells.

## Properties

### `.id`

Unique runtime Entity ID.

Read-only.

---

### `.name`

Entity name.

Read-only.

---

### `.position`

Current runtime world position.

Read-only.

---

### Dynamic Properties

Entities support dynamic property assignment, similar to Cells.

```aeoscript
const e = entity.get(1)
e.state = "idle"
```

---

# 9. `entity` Namespace

### `entity.get(id)`

Returns an `Entity` handle for the live runtime Entity with the specified ID.

Returns `nil` if not found.

### `entity.exists(id)`

Returns `true` if an Entity with the specified ID exists.

---

# 10. Anonymous Functions

AeoScript supports first-class anonymous functions. These can be assigned to variables, passed as arguments, or stored as dynamic properties on engine handles.

```aeoscript
const callback = fn(x) {
    return x * 2
}

const result = callback(10) // 20
```

### The `self` Keyword

Inside a function called as a method (e.g., `obj:method()`), the `self` keyword refers to the object it was called on.

```aeoscript
const c = cell.get(101)
c.toggle = fn() {
    self.visible = !self.visible
}

c:toggle()
```

---

# 11. Object References and Validity

Engine handles refer to engine-managed objects.

A handle may become invalid when the referenced runtime object is destroyed or otherwise removed.

Operations performed on invalid references produce runtime errors rather than silently affecting another object.

Scripts should therefore treat long-lived runtime references as potentially stale after destruction or lifecycle changes.

---

# 9. Runtime Overrides

Cell runtime properties are temporary.

For example:

```aeoscript
cell.visible = false
cell.solid = false
cell.anchored = true
cell.color = [1, 0, 0]
cell.offset = [0, 2, 0]
```

These modify the running game state.

They do not rewrite the authored World.

When Play mode ends, the authored values remain authoritative.

Conceptually:

```text
Authored World
      ↓
Runtime
      ↓
Runtime overrides
      ↓
Play stops
      ↓
Authored World remains unchanged
```

---

# 10. Script Lifecycle

AeoScript scripts attached to authored objects can participate in lifecycle execution.

Common lifecycle functions include:

```aeoscript
fn on_ready() {
    ...
}

fn update(dt) {
    ...
}

fn on_destroy() {
    ...
}
```

`update(dt)` receives the elapsed frame time.

Fields modified by lifecycle code persist between lifecycle executions.

For example:

```aeoscript
entity Counter {
    ticks: number = 0

    fn update(dt) {
        ticks += 1
    }
}
```

`ticks` remains part of the persistent script instance across update executions.

---

# 11. Script Bindings

Script bindings attach an `.aeo` script to a specific authored Cell.

Bindings target the Cell's persistent numeric ID.

They do not target the human-readable `.name`.

This permits multiple Cells with the same name to have independent bindings:

```text
Block ID 1001 → scripts/door.aeo
Block ID 1002 → scripts/button.aeo
Block ID 1003 → scripts/door.aeo
```

Legacy name-based bindings may be migrated when the name resolves to exactly one authored Cell.

Ambiguous legacy bindings must not arbitrarily select one matching Cell.

---

# 12. Time

### `time.delta`

Returns the elapsed simulation time since the previous frame in seconds.

This is useful for frame-based gameplay logic:

```aeoscript
position_x += speed * time.delta
```

---

# 13. Lifecycle and Fiber State

AeoScript uses fibers to preserve execution state across yielded operations.

When a lifecycle function or nested function executes `wait()`, the active fiber retains:

* Current instruction position
* Function call stack
* Local variables
* Loop state
* Entity script fields
* Pending wait state

The fiber resumes from the suspension point rather than starting the function again.

Persistent entity fields are synchronized from the active lifecycle fiber so mutations survive across frames.

---

# 14. Errors

AeoScript reports descriptive runtime errors for invalid operations including:

* Basket indexing out of bounds
* Mutating a frozen basket
* Invalid argument counts or types
* Accessing properties on `nil`
* Accessing invalid engine handles
* Invalid map/basket operations
* Invalid `math` arguments
* Unsupported basket comparator sorting

Runtime errors include available script context such as the script path, function, and source location.

Errors are reported through the AeoEngine script diagnostics/output system.

---

# 15. Current API Boundaries

The API is intentionally growing with the engine.

The current standard library does not yet provide every utility that may eventually be useful for gameplay scripting.

Examples of functionality that remain future API work include:

* Custom comparator functions for basket sorting
* Advanced string pattern operations
* Random-number generation with defined runtime seeding semantics
* Runtime object spawning and destruction APIs beyond currently supported engine objects
* Audio APIs
* Animation control APIs
* Expanded physics query APIs

New APIs should preserve AeoScript's existing type system, reference semantics, runtime-state model, and engine-owned object model.
