# AeoScript Lifecycle

AeoScript lifecycle execution connects authored script bindings to runtime script instances and fibers.

The lifecycle is designed so that persistent script fields survive across frames while temporary local execution state remains inside the active fiber.

---

# 1. Script Binding

A scripted authored Cell has a persistent binding:

~~~text
Cell ID → .aeo script path
~~~

The binding identifies the exact authored Cell instance.

The human-readable Cell identity/name does not have to be unique.

---

# 2. Script Instance

When the script is loaded for a bound object, AeoEngine creates a runtime `ScriptInstance` representing that scripted object.

Persistent entity fields belong to the ScriptInstance.

Example:

~~~aeoscript
entity Door {
    open: bool = false
    uses: number = 0
}
~~~

`open` and `uses` are persistent script fields for that running script instance.

---

# 3. Lifecycle Execution

The common lifecycle functions are:

~~~text
on_spawn
on_ready
update(dt)
on_touch(cell)
on_destroy
~~~

Not every script must implement every lifecycle function.

---

# 4. `on_ready`

`on_ready()` runs during initial script setup when the scripted object is ready for runtime logic.

Fields modified here persist into later lifecycle executions.

~~~aeoscript
entity Door {
    ready: bool = false

    fn on_ready() {
        ready = true
    }
}
~~~

Later `update()` execution observes the persistent field value.

---

# 5. `on_touch(cell)`

`on_touch(cell)` runs when a character (the player) contacts the scripted object.

The `cell` argument is a handle to the contacted Cell.

~~~aeoscript
entity Trap {
    fn on_touch(c) {
        debug.log("Player hit trap!")
        c.visible = false
    }
}
~~~

This event only dispatches if `collision_events_enabled` is true for that Cell (default: true).

Global event handlers can also react to any contact:

~~~aeoscript
on on_touch(c) {
    debug.log("Something was touched:", c.name)
}
~~~

---

# 6. `update(dt)`

`update(dt)` is the normal per-frame script lifecycle entry point.

`dt` represents the elapsed frame time supplied to the script.

Scripts can use it for frame-based gameplay logic.

~~~aeoscript
entity Counter {
    ticks: number = 0

    fn update(dt) {
        ticks += 1
    }
}
~~~

The entity field remains available to subsequent update executions.

---

# 6. Yielding from `update`

An update function can call `wait()`.

~~~aeoscript
entity Blink {
    visible: bool = true

    fn update(dt) {
        visible = false
        wait(0.25)
        visible = true
    }
}
~~~

The update execution becomes a waiting fiber rather than disappearing.

The same fiber resumes after the wait.

---

# 7. Nested Yielding

A function called by a lifecycle function can also yield.

~~~aeoscript
entity Example {
    value: number = 0

    fn delayed_change() {
        wait(0.1)
        value = 10
    }

    fn update(dt) {
        delayed_change()
    }
}
~~~

The call stack is preserved across the wait.

The runtime resumes inside `delayed_change()` and eventually returns to the caller.

---

# 8. Local State Across Wait

Function locals belong to the active fiber's call frame.

~~~aeoscript
fn delayed_increment() {
    progress: number = 10

    wait(0.05)

    progress += 5
}
~~~

`progress` remains available after the wait because the fiber preserves its local scopes.

---

# 9. Loop State Across Wait

Loop state is also part of the fiber execution state.

~~~aeoscript
fn count_steps() {
    total: number = 0

    for value in [1, 2, 3] {
        total += value
        wait(0.01)
    }

    return total
}
~~~

The fiber resumes the loop at the correct iteration rather than restarting the loop from the beginning.

---

# 10. Persistent Fields vs Locals

There are two important categories of script state:

~~~text
ScriptInstance
└── Persistent entity fields

ScriptFiber
├── Call stack
├── Local variables
├── Loop state
├── Instruction position
└── Wait state
~~~

Entity fields survive between lifecycle tasks.

Function locals survive only while their fiber call frame remains active.

A local can therefore survive a `wait()` but does not become a persistent entity field simply because it survives the yield.

---

# 11. Fiber State Synchronization

The active lifecycle fiber contains the mutable execution copy of the ScriptInstance.

As the lifecycle task advances, the mutated instance state is synchronized back to the persistent `ScriptEntity`.

This prevents field changes from being lost when a lifecycle task completes or yields.

This distinction fixed an important runtime class of bugs where `update()` appeared to execute but persistent fields were reverting to their previous values between frames.

---

# 12. Waiting Tasks

A waiting lifecycle task remains alive in the runtime scheduler.

A new update task should not replace the suspended update task simply because another frame has elapsed.

The scheduler waits until the fiber's wake time and then resumes the same continuation.

---

# 13. Destruction

When a scripted runtime object is destroyed, its lifecycle and runtime state are cleaned up according to the ScriptScene ownership rules.

Script fibers must not outlive the runtime object they belong to.

---

# 14. Lifecycle Design Principle

The lifecycle model is based on two complementary rules:

~~~text
Persistent fields belong to the script instance.

Execution state belongs to the fiber.
~~~

Together they allow AeoScript to support both ordinary frame updates and long-running gameplay routines that yield with `wait()`.

