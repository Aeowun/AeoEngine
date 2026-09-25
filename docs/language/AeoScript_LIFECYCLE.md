# AeoScript Lifecycle

AeoScript lifecycle execution connects authored script bindings to runtime script instances and resumable fibers.

The lifecycle separates persistent script-instance fields from temporary fiber execution state.

---

# 1. Script Binding

A bound script is associated with an authored Cell through its persistent Cell ID.

```text
Cell ID → .aeo script path
```

The Cell's human-readable name is not the persistent binding key.

---

# 2. Script Instance

When a bound script is instantiated, the runtime creates a script instance representing that scripted object.

Example:

```aeoscript
entity Door {
    open: bool = false
    uses: number = 0
}
```

The fields belong to the running script instance.

---

# 3. Lifecycle Functions

Common lifecycle functions include:

```text
on_spawn()
on_ready()
update(dt)
on_touch(cell)
on_overlap(overlapping, cell)
on_destroy()
```

Not every script must implement every lifecycle function.

---

# 4. `on_spawn()`

`on_spawn()` runs when the runtime creates the scripted object instance.

It is useful for initial runtime setup.

Changes to persistent script fields remain available to later lifecycle calls.

---

# 5. `on_ready()`

`on_ready()` runs after runtime setup has reached the ready stage for the scripted object.

It is useful for initialization that depends on the object being fully connected to the runtime.

---

# 6. `update(dt)`

`update(dt)` is the normal per-frame lifecycle function.

`dt` is the elapsed frame time supplied to the script.

Example:

```aeoscript
entity Counter {
    ticks: number = 0

    fn update(dt) {
        ticks += 1
    }
}
```

The `ticks` field persists between update executions.

---

# 7. `on_touch(cell)`

`on_touch(cell)` can be used for contact gameplay.

The event provides a Cell handle associated with the contact.

Example:

```aeoscript
fn on_touch(other) {
    debug.log("Touched:", other.name)
}
```

Collision event dispatch is controlled by the Cell's collision-event setting.

---

# 8. `on_overlap(overlapping, cell)`

`on_overlap(overlapping, cell)` reports entry/exit overlap behavior for supported collision events.

`overlapping` is a boolean describing whether the overlap is beginning or ending.

---

# 9. `on_destroy()`

`on_destroy()` provides lifecycle notification before a scripted runtime object is removed.

The ScriptScene owns lifecycle cleanup.

---

# 10. Fibers

Lifecycle execution occurs inside runtime fibers.

A fiber preserves:

* Function call stack.
* Local variables.
* Loop state.
* Instruction position.
* Wait state.
* Active execution context.

A lifecycle function can therefore yield with:

```aeoscript
wait(0.5)
```

without losing its execution state.

---

# 11. Nested Yielding

A nested function can yield:

```aeoscript
fn delayed_action() {
    wait(0.1)
    value += 1
}

fn update(dt) {
    delayed_action()
}
```

The runtime preserves the call stack and resumes inside `delayed_action()`.

---

# 12. Locals Across `wait()`

Function locals remain available while their call frame remains active.

```aeoscript
fn delayed_increment() {
    amount: number = 10

    wait(0.05)

    amount += 5
}
```

The local `amount` survives the wait.

It does not become a persistent entity field.

---

# 13. Persistent Fields vs Fiber State

The runtime separates two forms of script state.

```text
ScriptInstance
└── Persistent entity fields

ScriptFiber
├── Call stack
├── Local scopes
├── Loop state
├── Instruction position
└── Wait state
```

Persistent fields survive between lifecycle tasks.

Fiber state exists while execution remains active.

---

# 14. Loop State

Loops can suspend and resume.

```aeoscript
fn count_steps() {
    total: number = 0

    for value in [1, 2, 3] {
        total += value
        wait(0.01)
    }

    return total
}
```

The fiber resumes the correct loop iteration after each wait.

---

# 15. Scheduler Interaction

The scheduler manages lifecycle fibers.

A waiting fiber remains alive until its wake time.

A later frame does not automatically create a replacement execution of the same suspended fiber.

Conceptually:

```text
Lifecycle task
      ↓
wait()
      ↓
Fiber waiting
      ↓
Scheduler
      ↓
Wake time
      ↓
Same fiber resumes
```

---

# 16. Script State Synchronization

Persistent script fields must remain synchronized with the owning script instance as lifecycle execution advances.

This allows:

```aeoscript
ticks += 1
```

in `update()` to remain visible in later updates.

Local execution scopes remain separate from persistent entity fields.

---

# 17. Script Disable

Scripts can be disabled at runtime with:

```aeoscript
script.disable("scripts/example.aeo")
```

Disabling a script affects runtime execution and cancels active fibers associated with the disabled script.

Re-enabling controls subsequent runtime execution.

The source file remains unchanged.

---

# 18. Destruction

When a scripted runtime object is destroyed:

* Its lifecycle execution is stopped.
* Its active fibers are cleaned up.
* Its runtime state is removed according to ScriptScene ownership.

Fibers must not outlive the runtime object they belong to.

---

# 19. Lifecycle Principle

The lifecycle model is based on:

```text
Persistent fields
→ script instance

Execution state
→ fiber

Scheduling
→ ScriptScheduler

Object/lifecycle ownership
→ ScriptScene
```

This separation allows AeoScript to support both ordinary frame-based logic and long-running gameplay routines that yield with `wait()`.