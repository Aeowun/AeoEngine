# AeoScript Gameplay Scripting

AeoScript is AeoEngine's gameplay scripting language.

The scripting workflow connects `.aeo` source files, authored Cell bindings, persistent script fields, engine handles, and runtime fibers.

---

# 1. Create a Script

Create an `.aeo` file in the project's `scripts/` directory.

A simple script can look like:

~~~aeoscript
entity Controller {
    count: number = 0

    fn update(dt) {
        count += 1
        debug.log("Update:", count)
    }
}
~~~

---

# 2. Name the Scripted Object

Give the authored Cell a useful identity/name in the Properties panel.

The name is human-readable and may be shared by other Cells.

The actual script binding targets the Cell's persistent numeric ID.

---

# 3. Attach and Manage Scripts

Select the Cell and use the script controls in the Properties panel.

The engine creates a persistent binding between:

~~~text
Cell ID → script path
~~~

### Script Enable / Disable

You can temporarily disable a script from the **INSPECTOR** panel in the Script Editor. When disabled, the script will not execute top-level code, lifecycle hooks, or event handlers.

Removing a binding does not delete the source script.

---

# 4. Lifecycle Events

In addition to `update(dt)`, AeoScript supports contextual events.

### Contact Events

The `on_touch(cell)` function runs when the player character makes contact with a scripted object.

~~~aeoscript
entity Spike {
    fn on_touch(c) {
        debug.log("Player took damage!")
    }
}
~~~

Use the **Collision Events** toggle in the Cell Properties panel to enable or disable these callbacks for specific objects.

---

# 5. Use `update(dt)`

`update(dt)` is the normal per-frame lifecycle function.

Use the supplied delta time for time-scaled logic.

~~~aeoscript
entity Mover {
    position_x: number = 0
    speed: number = 2

    fn update(dt) {
        position_x += speed * dt
    }
}
~~~

---

# 5. Find World Objects

Use `find()` when you want name-based discovery.

~~~aeoscript
const doors = find("Door")
~~~

Use `getAllCellsOfClass()` when you want a class-based query.

~~~aeoscript
const blocks = getAllCellsOfClass("Block")
~~~

Both return baskets of handles.

---

# 6. Change Runtime Properties

Supported Cell runtime properties can be modified directly.

~~~aeoscript
for block in blocks {
    block.visible = false
    block.solid = false
    block.attributes["health"] = 100
}
~~~

These changes are temporary runtime overrides. Assign `nil` to an attribute to restore its authored value.

---

# 7. Dynamic Object Management

Use the `cell` namespace to create or remove objects during Play.

~~~aeoscript
const coin = cell.new("Block")
coin.position = [10, 5, 2]
coin.color = [1, 0.8, 0]

// Later...
cell.delete(coin)
~~~

---

# 8. Use Persistent Fields

Script fields persist across lifecycle executions.

~~~aeoscript
entity Counter {
    hits: number = 0

    fn update(dt) {
        hits += 1
    }
}
~~~

This is different from a local variable inside a function.

---

# 8. Use Functions

Break gameplay behavior into functions.

~~~aeoscript
entity Door {
    open: bool = false

    fn set_open(value: bool) {
        open = value
    }
}
~~~

Functions can call other functions and can yield with `wait()`.

---

# 9. Use `wait()`

Use `wait()` for delayed gameplay behavior.

~~~aeoscript
entity Blink {
    visible: bool = true

    fn update(dt) {
        visible = false
        wait(0.5)
        visible = true
    }
}
~~~

The same fiber resumes after the wait.

---

# 10. Nested Functions Can Yield

A nested function call can contain a wait.

~~~aeoscript
entity Example {
    value: number = 0

    fn delayed_change() {
        wait(0.25)
        value = 10
    }

    fn update(dt) {
        delayed_change()
    }
}
~~~

Local variables, call frames, and loop state are preserved by the fiber.

---

# 11. Collections

Baskets are zero-indexed and reference-backed.

~~~aeoscript
const items = ["A", "B", "C"]
items.insert(0, "Start")
~~~

Maps are key/value collections.

~~~aeoscript
const stats = {}
stats["health"] = 100
~~~

Missing map keys return `nil`.

---

# 12. Standard Library

The current standard-library namespaces are:

~~~text
math
basket
string
~~~

Examples:

~~~aeoscript
const speed = math.clamp(velocity, 0, 10)
items.sort()
const words = string.split("one,two,three", ",")
~~~

---

# 13. Debugging Scripts

Use structured output during gameplay development.

~~~aeoscript
debug.log("Started")
debug.log("Health:", health)
debug.log("Target ID:", target.id)
~~~

Use the PRINT OUTPUT panel to inspect runtime messages and errors.

---

# 14. Script State Model

A useful mental model is:

~~~text
Authored Cell
    ↓
Script Binding
    ↓
ScriptInstance
    ↓
ScriptFiber
    ├── Call stack
    ├── Locals
    ├── Loop state
    └── wait state
~~~

Persistent fields belong to the ScriptInstance.

Execution state belongs to the Fiber.

---

# 15. Recommended Workflow

Use this development loop:

~~~text
Write script
 ↓
Save
 ↓
Run Play
 ↓
Observe output
 ↓
Test gameplay
 ↓
Stop Play
 ↓
Fix script
 ↓
Repeat
~~~

Build the smallest working behavior first, then add complexity.

