# AeoScript Language Overview

AeoScript is AeoEngine's gameplay scripting language, designed for world interaction, gameplay logic, runtime object control, physical interactions, and world events.

AeoScript is integrated directly into AeoEngine and can operate on authored World objects through engine handles while keeping runtime changes separate from authored scene data.

---

# 1. Values and Types

### Basic Types

* `number`: 64-bit floating-point number.
* `bool`: `true` or `false`.
* `string`: UTF-8 Unicode text.
* `nil`: Represents the absence of a value.

### Collections

* `basket`: A zero-indexed, reference-backed sequence of values.
* `map`: A reference-backed key/value collection.

### Engine Handles

Handles are opaque references to engine-managed objects.

* `Cell`: A reference to an authored World Cell such as a Block or Light.
* `Entity`: A reference to a live runtime object such as a Player or NPC.

A stale or invalid engine handle does not silently modify unrelated objects. Using an invalid handle produces a runtime error.

---

# 2. Baskets

A `basket` is a zero-indexed collection of values.

Baskets use **reference semantics**:

* Assigning a basket to another variable creates an alias to the same collection.
* Mutating one alias is visible through every other alias.
* `basket.clone(b)` creates a distinct shallow copy.
* `basket.freeze(b)` prevents further mutation.

```aeoscript
const ids = [1, 2]

ids[0] = 99       // VALID
```

The basket now contains:

```text
[99, 2]
```

Because baskets are zero-indexed, the first element is at index `0`.

---

# 3. Maps

A `map` stores values using keys.

Maps are not positional collections and do not use zero-based indexing semantics.

Numeric and string keys are distinct:

```aeoscript
const values = {}

values[1] = "numeric"
values["1"] = "string"

debug.log(values[1])
debug.log(values["1"])
```

Reading a missing key returns `nil`.

Assigning `nil` to an existing key removes that key:

```aeoscript
values["name"] = nil
```

Maps use reference semantics, so aliases refer to the same underlying collection.

Nested maps and baskets can therefore be mutated through aliases and retain those changes.

---

# 4. Variables and Constants

Variables are mutable by default.

A variable can be explicitly typed:

```aeoscript
health: number = 100
```

or inferred:

```aeoscript
const name = "Ghost"
```

### `const`

`const` prevents reassignment of the variable.

For reference-backed collections, `const` does not make the collection immutable.

```aeoscript
const ids = [1, 2]

ids[0] = 99      // VALID

ids = [3, 4]     // ERROR
```

Use `basket.freeze()` when the basket itself should no longer be mutated.

---

# 5. Unique Cell ID vs Identity

AeoEngine distinguishes between the **unique Cell ID** of an authored object and its **human-readable name**.

### Unique Cell ID (`id`)

Every authored Cell has a persistent numeric ID.

The ID is the unique identity of that specific Cell instance.

* Script bindings target the Cell ID.
* Different Cells can have different scripts even when they share the same name.
* Moving or renaming a Cell does not change its ID.
* The ID allows runtime systems and script bindings to reference a specific authored instance without depending on its name.

Example:

```text
Block
  ID: 18374291
  Identity: Wall

Block
  ID: 62918403
  Identity: Wall
```

These are two different Cells even though they have the same name.

### Entity Identity / Name

The `entity_identity` value is the human-readable game-logic name assigned in the editor.

Names do not have to be unique.

```aeoscript
find("Wall")
```

can therefore return multiple matching objects.

Use the unique `id` when a specific authored Cell must be targeted.

---

# 6. Functions

Functions are declared with `fn`.

Functions may accept parameters and return values.

```aeoscript
entity Player {
    health: number = 100

    fn take_damage(amount: number) {
        health = math.max(0, health - amount)
    }

    fn get_health() {
        return health
    }
}
```

Functions can call other functions.

Function locals remain part of the active execution state and are preserved when the current script fiber yields.

---

# 7. Events and Lifecycle

AeoScript supports engine lifecycle functions and event handlers.

Common lifecycle functions include:

* `on_ready()`
* `update(dt)`
* `on_destroy()`

Event handlers can respond to engine events:

```aeoscript
on PlayerSpawned(player: Entity) {
    debug.log("A new player has arrived!")
}
```

`update(dt)` receives the current frame delta time.

Lifecycle state persists between executions, allowing script fields to be modified in `on_ready()` or `update()` and observed by later executions.

---

# 8. Fibers and `wait()`

AeoScript uses cooperative script fibers for yielding execution.

The `wait(seconds)` operation suspends the current script execution and resumes it after the requested amount of time.

```aeoscript
fn flash() {
    visible = false

    wait(0.25)

    visible = true
}
```

The fiber preserves the execution state required to continue correctly after the wait.

This includes:

* Current function
* Nested function calls
* Local variables
* Call stack state
* Loop state
* Instruction position
* Script execution context

A function may therefore yield from inside another function:

```aeoscript
fn delayed_action() {
    wait(0.5)
    score += 10
}

fn update(dt) {
    delayed_action()
}
```

The suspended function resumes after the `wait()` rather than restarting from the beginning.

---

# 9. Built-in Namespaces

AeoScript provides standard-library functionality through namespaces.

### `cell`

Engine object creation:

* `new(type)`
* `delete(handle)`

`cell.new(type)` creates a new **Play-mode/runtime-only Cell** of the specified type. Runtime cells are not part of the authored World data, are not persisted to disk, and are discarded when Play mode stops or runtime state is cleared.

`cell.delete(handle)` removes a Cell from the current Play-time world. If the target is an authored World Cell, it is marked as deleted for the duration of the Play session but its authored data is preserved. If the target is a runtime-created Cell, it is destroyed. In both cases, the Cell becomes effective again or is removed entirely when Play stops.

Supported types:

* `"Block"`
* `"FxBlock"`
* `"Player"`
* `"NPC"`
* `"Light"`
* `"SpawnPoint"`

Returns a `Cell` or `Light` handle.

Example:

```aeoscript
const block = cell.new("Block")
block.position = [10, 5, 10]
block.color = [0, 1, 0]
```

### `math`

Deterministic mathematical operations including:

* `abs`
* `min`
* `max`
* `floor`
* `ceil`
* `round`
* `sqrt`
* `pow`
* `sin`
* `cos`
* `tan`
* `random`
* `clamp`
* `lerp`
* `deg_to_rad`
* `rad_to_deg`

`math.random()` returns a number in the range `[0.0, 1.0)`.

`math.random(min, max)` returns an integer in the inclusive range `[min, max]`.

Example:

```aeoscript
const speed = math.clamp(velocity, 0, 10)
const roll = math.random(1, 6)
```

### `basket`

Collection operations including:

* `create`
* `insert`
* `remove`
* `sort`
* `find`
* `move`
* `concat`
* `clone`
* `clear`
* `freeze`

Example:

```aeoscript
items.insert(0, "Sword")
```

### `string`

Unicode-aware text operations including:

* `len`
* `lower`
* `upper`
* `reverse`
* `split`

String operations work with Unicode scalar values rather than treating UTF-8 bytes as individual characters.

---

# 10. Runtime Overrides

AeoScript operates on a **runtime override** model.

AeoEngine separates **Authored State** (persisted project data) from **Runtime State** (temporary data used during Play mode).

* **EDITOR MODE**: Operations edit authored World state directly.
* **PLAY MODE**: Operations edit runtime state only.

When a script modifies a runtime property or attribute of a Cell, the change affects the running game without modifying the authored World data.

### Properties

```aeoscript
cell.visible = false
cell.solid = false
cell.anchored = true
cell.color = [1, 0, 0]
cell.offset = [0, 2, 0]
```

### Attributes

```aeoscript
cell.attributes["testBool"] = false
cell.attributes["testNum"] = 42
cell.attributes["Test"] = "runtime"
```

AeoScript attribute writes create temporary overrides. Reads return the effective value (the override if it exists, otherwise the authored baseline).

Assigning `nil` to a runtime attribute removes the override and restores the authored value.

### Mutable Runtime Properties

Runtime-created Cells (`cell.new()`) support the following mutable properties:

* `position`: `[x, y, z]` (Relocates the cell in the grid)
* `visible`: `bool`
* `enabled`: `bool` (For Lights)
* `solid`: `bool`
* `anchored`: `bool`
* `color`: `[r, g, b]`
* `offset`: `[x, y, z]`
* `name`: `string`

All changes to these properties are temporary runtime state.

When Play mode ends:

```text
Authored World
      ↓
Runtime
      ↓
Temporary overrides
      ↓
Play mode stops
      ↓
Authored World remains unchanged
```

A script does not need to restore runtime overrides manually for the purpose of preserving authored scene data.

The authored World remains authoritative.

---

# 11. Object Discovery

AeoScript can discover engine objects through built-in World queries.

### `find()`

Searches by authored identity/name and returns a basket of matching objects.

```aeoscript
const walls = find("Wall")
```

Multiple objects may have the same identity.

### `getAllCellsOfClass()`

Returns authored Cells of a requested class:

```aeoscript
const blocks = getAllCellsOfClass("Block")
```

These APIs return handles that can then be inspected or modified through the supported runtime properties.

---

# 12. Script Bindings

Scripts can be attached to authored World Cells.

Bindings target the Cell's unique persistent numeric ID rather than its human-readable name.

This allows:

```text
Block ID 1001 → scripts/door.aeo
Block ID 1002 → scripts/button.aeo
Block ID 1003 → scripts/door.aeo
```

even when the Cells share the same human-readable identity.

Legacy name-based bindings can be migrated when they resolve unambiguously to a single authored Cell.

---

# 13. Authored State and Runtime State

AeoScript interacts with two related categories of state.

### Authored State

Persistent World data created through the editor.

Examples:

* Cell identity/name
* Cell type
* Authored position
* Authored Block color
* Authored solidity
* Authored anchoring
* Script bindings

### Runtime State

Temporary state used while the game is running.

Examples:

* PhysicsBody position
* Runtime Cell property overrides
* Character state
* Script fiber execution state
* Script lifecycle fields
* Temporary gameplay changes

The runtime may derive objects and state from the authored World, but it must not silently rewrite authored scene data.

---

# 14. Program Structure and Execution

An AeoScript program consists of declarations and top-level executable statements.

### Top-Level Statements

Executable statements at the top level of a script execute exactly once, in source order, when the world loads.

```aeoscript
debug.log("System initializing...")

const version = 1.0

if version > 0 {
    debug.log("Startup successful.")
}
```

Top-level statements run within a "Global" script instance for that file. They support yielding using `wait()`.

### Declarations

Declarations define items that can be used later but do not execute immediately.

* **Functions (`fn`)**: Declarations only. The body executes only when called.
* **Events (`on`)**: Registrations only. The body executes only when the event is dispatched.
* **Entities (`entity`)**: Declarations only. The entity's fields and lifecycle functions are used only when the script is bound to a World object.

### Unattached Lifecycle Warnings

If an `entity` declaration contains lifecycle hooks (`on_spawn`, `on_ready`, `update`, `on_destroy`) but no script binding exists for that entity in that script, the engine emits a script warning. This helps identify scripts that are incorrectly bound or entity names that do not match the world data.

---

# 15. Current Language Direction

AeoScript is intended to remain:

* General-purpose enough for gameplay logic
* Closely integrated with AeoEngine
* Simple to read and write
* Strongly connected to World and runtime objects
* Safe around engine-managed references
* Suitable for yielding gameplay logic
* Distinct in its own syntax and vocabulary

The language and standard library continue to expand as real games expose additional gameplay requirements.

See:

* [AeoScript API](AeoScript_API.md)
* [AeoScript Grammar](AeoScript_GRAMMAR.md)
* [AeoScript VM](AeoScript_VM.md)
* [AeoScript Editor](AeoScript_EDITOR.md)
