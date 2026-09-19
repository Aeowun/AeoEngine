# AeoScript

AeoEngine's gameplay scripting language.

**File extension:** `.aeo`

**Current status:** First implementation (V1)

---

AeoScript is used for gameplay logic such as character behavior, physics interactions, and world events.

## Execution
AeoScript runs inside AeoEngine's VM. Every script has an instruction budget per frame to prevent a single script from hanging the engine.

## Types
Type annotations are optional. Values can be explicitly typed or inferred.

## Objects
Properties use `.`, while method calls use `:`.

## Runtime State
Changes made by scripts are kept separately from the authored project data.

---

# 1. Syntax Example

```aeoscript
entity Door {
    open: bool = false

    fn interact(player: Entity) {
        open = !open
        debug.log("Door toggled by: " + player.name)
    }
}
```

---

# 2. Values and Types

### Basic Types
* `number`: 64-bit floating point.
* `bool`: `true` or `false`.
* `string`: UTF-8 text. Supports concatenation using `+`.
* `nil`: The absence of a value.

### Engine Types
* `Entity`: A handle to a live engine object (Player, NPC).
* `Cell`: A handle to authored world content (Block, Light).
* `Basket`: A 0-indexed collection (Array).

### Optional Types
Types followed by `?` can contain a value or `nil`.
```aeoscript
target: Entity? = nil
```

---

# 3. Variables and Operators

Variables are mutable by default. Use `const` for values that cannot be reassigned.
```aeoscript
const MAX_SPEED = 10
current_speed = 5
current_speed += 2
```

### String Concatenation
The `+` operator performs string concatenation if either operand is a string. The non-string operand is converted to its string representation.
```aeoscript
debug.log("Lights found: " + lights.len())
```

---

# 4. Functions

Functions use `fn`:
```aeoscript
fn heal(amount: number) {
    health += amount
}

fn is_alive(): bool {
    return health > 0
}
```

---

# 5. Control Flow

Conditionals:
```aeoscript
if health <= 0 {
    die()
} else {
    keep_fighting()
}
```

Loops:
```aeoscript
while timer > 0 {
    timer -= 1
}

for item in collection {
    item:action()
}
```

---

# 6. Unique Cell ID

The engine assigns every cell (Block, Light, etc.) a unique 8-digit ID upon creation.

* **Stability**: The ID remains the same if the cell moves.
* **Safety**: Once an ID is issued, the engine never reuses it, even if the cell is deleted.
* **Scripting**: Script handles use this ID to find cells. If a cell is deleted, its handle becomes invalid and will not resolve to a new cell.

Exposed as: `cell.id` (read-only).

---

# 7. Runtime Changes

AeoScript cannot directly modify the saved world data. Changes made while the game is running are stored as temporary overrides:

1. The project contains the authored world.
2. Scripts create temporary changes (color, visibility, etc).
3. The engine uses those changes while the game is running.
4. The changes are discarded when the game stops.

Saving during Play mode does not save these runtime changes.

---

# 8. Built-in Functions

* `print(value)`: Prints to the terminal.
* `debug.log(value)`: Prints to the terminal with SCRIPT severity.
* `getAllCellsOfClass(class)`: Returns a Basket of Cell handles for a type (e.g. "Light").
* `find(query)`: Returns a Basket of handles matching a name or identity.
* `wait(seconds)`: Pauses script execution for a specified duration.

---

# 9. The Terminal (Print Output)

The **PRINT OUTPUT** panel displays all runtime diagnostics.

* **Colors**: Errors are Red, Warnings are Yellow.
* **Context**: Messages include the script path and the specific event or function name.
* **Selection**: Supports standard mouse selection, `Ctrl+A`, and `Ctrl+C`.

---

## Implementation Notes

AeoScript is parsed into an AST and executed by an interpreter in Rust. It uses persistent fibers to support cooperative multi-tasking (like `wait()` loops) without blocking the engine's main thread.
