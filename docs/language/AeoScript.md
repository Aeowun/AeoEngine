# AeoScript Language Overview

AeoScript is AeoEngine's high-performance gameplay scripting language, designed for character logic, physical interactions, and world events.

---

# 1. Values and Types

### Basic Types
* `number`: 64-bit floating point.
* `bool`: `true` or `false`.
* `string`: UTF-8 Unicode text. Supports `+` for concatenation.
* `nil`: Represents the absence of a value.

### Engine Handles
Handles are opaque references to engine-managed objects. They are safe; using a stale handle (e.g., to a deleted block) will not crash but will produce a runtime error.
* `Cell`: A reference to authored world data (e.g., a Block or Light).
* `Entity`: A reference to a live runtime object (e.g., a Player or NPC).

### Basket (Arrays)
A `Basket` is a zero-indexed collection of values. Baskets use **reference semantics**:
* Assigning a basket to a new variable creates an **alias** to the same shared collection.
* Mutation through one alias is visible to all others.
* Use `basket.clone(b)` to create a distinct shallow copy.
* Use `basket.freeze(b)` to prevent further mutation.

---

# 2. Variables and Constants

* **Mutation**: Variables are mutable by default.
* **`const`**: Prevents **reassignment** of a variable.
  * For Baskets, `const` prevents assigning a *different* basket to the variable, but the *elements* of the basket can still be modified unless it is frozen.

```aeoscript
const ids = [1, 2]
ids[0] = 99      // VALID (mutating elements)
ids = [3, 4]     // ERROR (reassigning a const)
```

---

# 3. Unique Cell ID vs Identity

AeoEngine distinguishes between the **Unique Identity** and the **Game Logic Name** of authored objects.

### Unique Cell ID (`id`)
Every Cell (Block, Light, etc.) is assigned a permanent, unique 8-digit numeric ID upon creation.
* **Binding**: Script bindings target this unique ID. If you have five blocks named "Wall", you can attach a different script to each one because they have unique IDs.
* **Stability**: The ID never changes, even if the cell is moved or renamed.

### Entity Identity (`name`)
This is the human-readable string (e.g., "Ghost", "Door") assigned in the editor.
* **Not Unique**: Multiple Cells can share the same Identity.
* **Search**: `find("Ghost")` returns a Basket containing all Cells or Entities sharing that Identity.

---

# 4. Functions and Events

* **Functions**: Declared with `fn` inside an `entity` block.
* **Event Handlers**: Declared with `on` at the top level to respond to global engine events.

```aeoscript
entity Player {
    health: number = 100

    fn take_damage(amount: number) {
        health = math.max(0, health - amount)
    }
}

on PlayerSpawned(player: Entity) {
    debug.log("A new player has arrived!")
}
```

---

# 5. Built-in Namespaces

Standard functions are organized into namespaces:
* **`math`**: Deterministic math (sin, cos, sqrt, clamp, lerp, etc.).
* **`basket`**: Collection management (insert, remove, sort, move, etc.).
* **`string`**: Unicode-aware text manipulation (len, reverse, lower, upper, split).

---

# 6. Runtime Overrides

AeoScript operates on a "Runtime Override" model. When a script modifies a property (like `cell.color`), it creates a temporary change that exists only until the game stops. 
