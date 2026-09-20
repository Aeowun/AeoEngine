# AeoScript VM Execution Model

The AeoScript runtime is a single-threaded interpreter integrated into AeoEngine's main simulation loop.

Source code is parsed into an AST. Functions and events are then converted into compact internal execution plans so their instruction position and control-flow state can be preserved across cooperative yields.

AeoScript is therefore an **interpreter with resumable execution state**, rather than a separate native-code or bytecode virtual machine.

---

# 1. Runtime Architecture

The runtime is divided into several cooperating parts:

```text
AeoScript Source
      ↓
Lexer
      ↓
Parser
      ↓
AST / Program
      ↓
Interpreter
      ↓
Compiled Function / Event Execution Plan
      ↓
ScriptFiber
      ↓
ScriptScheduler
      ↓
ScriptScene / Engine
```

### Interpreter

The interpreter:

* Evaluates AeoScript expressions.
* Executes statements.
* Resolves user functions.
* Executes engine-host functions and properties.
* Creates and resumes script fibers.
* Enforces execution limits.
* Produces structured runtime diagnostics.

### ScriptFiber

A `ScriptFiber` represents one resumable script execution.

A fiber owns:

* The active `ScriptInstance`.
* The current function/event call stack.
* Instruction position for each active call frame.
* Local scopes.
* `for` loop state.
* Completion state.
* Return state.

This state remains alive while a fiber is waiting or scheduled to resume.

### ScriptRuntime

`ScriptRuntime` owns the active fibers and connects them to the scheduler.

It:

* Creates fibers.
* Stores fibers by task ID.
* Advances scheduler time.
* Resumes ready fibers.
* Applies fiber results to the scheduler.
* Removes completed or failed fibers.

Each ready task receives at most one execution slice during a runtime tick.

---

# 2. Threading and Execution Model

AeoScript execution is currently **single-threaded**.

Script execution runs synchronously with the engine's runtime update rather than on a separate scripting thread.

The runtime does not block the engine while a script is waiting.

Instead:

```text
Script execution
      ↓
wait(seconds)
      ↓
Fiber yields
      ↓
Scheduler stores wake time
      ↓
Engine continues
      ↓
Wake time reached
      ↓
Fiber resumes
```

This makes `wait()` cooperative rather than thread-based.

---

# 3. Fibers and Cooperative Yielding

AeoScript lifecycle functions such as `update()` execute inside persistent fibers managed by `ScriptRuntime`.

A fiber can produce several execution outcomes:

```text
Continue
Yield
Complete
Failed
```

### Yield

A `wait(seconds)` operation produces a wait yield.

The scheduler records the time at which the fiber may resume.

The fiber itself is not destroyed.

### Complete

The fiber has reached the end of its execution.

The owning `ScriptScene` can then remove the completed task and transition the scripted entity to its next lifecycle state.

### Failed

A runtime error terminates the fiber.

The runtime records structured diagnostic information and the owning script entity is stopped according to the lifecycle rules.

---

# 4. `wait()` and Resumption

`wait(seconds)` suspends the current fiber.

The suspended fiber retains the entire active execution state.

Example:

```aeoscript id="qz55w6"
fn delayed_action() {
    local_value: number = 10

    wait(0.5)

    local_value += 5
}
```

The `local_value` variable remains available after the wait because its scope is stored inside the fiber.

The same applies to nested function calls:

```text
update()
  ↓
gameplay_function()
  ↓
helper_function()
  ↓
wait()
```

The entire call chain remains represented by the fiber's call stack.

When the scheduler wakes the fiber, execution resumes after the yielding instruction rather than starting the function again.

---

# 5. Fiber Call Stack

Each resumable fiber maintains its own stack of function call frames.

A call frame contains the execution state required to resume that function, including:

* Compiled function instructions.
* Program counter / instruction position.
* Local scopes.
* `for` loop state.
* Function name used for diagnostics.

When a user function calls another user function, a new call frame is pushed.

When the nested function returns, its frame is removed and execution continues in the caller.

This allows a nested function to yield without losing the caller's execution state.

---

# 6. Execution Budget

Each interpreter resume slice has a maximum operation budget.

The default interpreter budget is currently:

```text
100,000 operations
```

A custom budget can also be supplied by the runtime.

Every interpreter operation consumes budget.

If the budget reaches zero before the fiber yields or completes, execution fails with an AeoScript execution-budget error.

Budget exhaustion is **not** treated as a normal cooperative yield.

The purpose of the budget is to prevent an accidentally runaway script from monopolizing the engine thread.

Infinite or extremely expensive loops should therefore either:

* perform useful work within a bounded number of operations, or
* explicitly yield using `wait()`.

---

# 7. Call Depth Protection

The interpreter also limits nested user-function call depth.

The default maximum call depth is currently:

```text
64
```

Exceeding the limit produces a runtime error.

This protects the runtime from unbounded recursive or deeply nested function calls.

---

# 8. Script Instance State

A `ScriptInstance` represents the persistent state of one scripted entity.

It contains:

* Script/entity ID.
* Entity declaration name.
* Script path.
* Persistent entity fields.

For example:

```aeoscript id="xbrl7w"
entity Counter {
    ticks: number = 0
}
```

`ticks` belongs to the persistent `ScriptInstance`.

Local variables declared inside functions do not belong to the persistent instance. They belong to the active fiber's call-frame scopes.

This distinction is important:

```text
ScriptInstance
├── Persistent entity fields
│
└── Fiber
    ├── Call stack
    ├── Local scopes
    ├── Loop state
    └── Instruction position
```

---

# 9. Lifecycle State Persistence

A lifecycle fiber starts from the entity's persistent `ScriptInstance`.

During execution, the active fiber owns the mutable execution copy of that instance.

Before a lifecycle result is discarded or processed by `ScriptScene`, the mutated instance state is synchronized back to the persistent `ScriptEntity`.

This ensures that fields modified during:

* `on_spawn`
* `on_ready`
* `update`

survive across lifecycle executions.

Example:

```aeoscript id="z84xyu"
entity Counter {
    ticks: number = 0

    fn update(dt) {
        ticks += 1
    }
}
```

The value of `ticks` persists between update tasks.

The fiber's local execution state is discarded only when the fiber itself is finished; persistent entity fields have already been synchronized back to the owning entity.

---

# 10. Scheduler

The `ScriptScheduler` controls when fibers are eligible to execute.

A task can be in states such as:

```text
Ready
Running
Waiting
Complete
Failed
Cancelled
```

The scheduler maintains the current script time and moves waiting tasks back into the ready state once their wake time is reached.

During a runtime tick:

```text
1. Advance script time.
2. Wake tasks whose wait time has expired.
3. Pop ready tasks.
4. Resume each task for one execution slice.
5. Apply its FiberResult to the scheduler.
```

A task returning `Continue` is not immediately executed repeatedly within the same runtime tick. This prevents one script from consuming the entire tick simply because it remains runnable.

---

# 11. Standard Library Dispatch

AeoScript standard-library functionality is dispatched through the interpreter's native standard-library module.

Current namespaces include:

```text
math
basket
string
```

Namespace-style calls are resolved internally:

```aeoscript id="h5jq7n"
math.sqrt(25)
basket.create(3)
string.upper("hello")
```

Basket methods are also routed through the basket namespace:

```aeoscript id="76ct8v"
items.len()
items.insert(0, value)
items.clear()
```

The runtime passes the basket object as the appropriate first argument to the namespace implementation.

The standard library executes inside the interpreter and does not create a separate scripting runtime.

---

# 12. Value and Collection Memory Model

AeoScript values are owned by the scripting runtime.

The current value model includes:

```text
number
bool
string
nil
basket
map
engine handle
```

### Baskets

Baskets are reference-backed collections.

Assigning or passing a basket copies the reference to the shared collection rather than copying all elements.

Therefore:

```aeoscript id="9ed7c4"
const a = [1, 2, 3]
const b = a

b[0] = 99
```

also changes `a[0]`.

`basket.clone()` creates a separate shallow outer basket.

Frozen baskets carry read-only state and reject later mutation attempts.

### Maps

Maps are also reference-backed.

Numeric and string map keys are distinct.

Missing map keys return `nil`, and assigning `nil` removes an existing key.

The scripting runtime is currently single-threaded, so these reference-backed collections are designed for controlled interpreter-side mutation rather than concurrent access.

---

# 13. Engine Handles

Engine objects are represented inside AeoScript as opaque handles.

A handle contains an engine-managed object kind and numeric identifier rather than a raw native pointer.

Examples include:

```text
Cell
Entity
Light
```

The scripting runtime does not own the underlying engine object.

When a handle property or method is accessed, the host integration resolves the identifier against authoritative engine state.

This allows script values to remain lightweight while preventing the scripting VM from directly owning engine objects.

---

# 14. Runtime Property Access

Engine properties are resolved through the host integration layer.

For example:

```aeoscript id="stj5pi"
cell.visible = false
cell.color = [1, 0, 0]
```

The interpreter evaluates the expression and delegates the actual engine property access to the host API.

This keeps the interpreter independent from concrete rendering, physics, World, and entity implementations.

Runtime Cell modifications are handled as runtime overrides rather than silently rewriting authored World data.

---

# 15. Script Bindings

Script bindings connect authored World Cells to `.aeo` scripts.

The World stores the binding target as the Cell's persistent numeric ID.

During loading:

```text
World binding
      ↓
Cell ID lookup
      ↓
Authored Cell
      ↓
Script entity instantiation
      ↓
Script fiber lifecycle
```

The binding identifies the specific Cell instance.

The Cell's human-readable `entity_identity` remains a name and does not have to be unique.

This allows multiple Cells with the same name to have independent bindings.

Legacy name-based bindings may be migrated when they resolve unambiguously to exactly one authored Cell.

---

# 16. Native Engine Integration

The interpreter communicates with AeoEngine through the host API abstraction.

The host layer provides access to engine-owned functionality without embedding renderer, physics, or World implementation details inside the language interpreter.

Host integration can provide:

* Cell property access.
* Entity property access.
* Object discovery.
* Runtime object manipulation.
* Script diagnostics.
* Time information.
* Engine-defined functions and methods.

The interpreter therefore evaluates language semantics while the engine remains responsible for the actual game-world state.

Conceptually:

```text
AeoScript
    ↓
Interpreter
    ↓
Host API
    ↓
AeoEngine
├── World
├── Physics
├── Characters
├── Entities
└── Renderer
```

---

# 17. Runtime Diagnostics

The interpreter maintains structured diagnostic context for the currently executing script.

Diagnostic records can include:

* Severity.
* Script path.
* Entity name.
* Entity ID.
* Function/context name.
* Runtime error or warning message.

This information is forwarded to AeoEngine's script output system.

The goal is to make runtime failures actionable rather than exposing only generic interpreter errors.

---

# 18. Execution Lifecycle

A typical scripted entity follows this lifecycle:

```text
Authored Cell
      ↓
Script binding resolved
      ↓
ScriptInstance created
      ↓
on_spawn / on_ready
      ↓
update(dt)
      ↓
Fiber yields or completes
      ↓
Entity state synchronized
      ↓
Next lifecycle task
```

When `update()` calls `wait()`:

```text
update()
   ↓
user function
   ↓
wait()
   ↓
FiberResult::Yield
   ↓
Scheduler Waiting
   ↓
wake time reached
   ↓
same fiber resumes
   ↓
execution continues
```

When the lifecycle task completes, its persistent entity fields have already been synchronized back to the owning `ScriptEntity`.

---

# 19. Current Runtime Boundaries

The current VM/runtime intentionally has several boundaries.

It does not yet provide:

* A bytecode compiler.
* Native machine-code compilation.
* Multithreaded script execution.
* Automatic budget-based continuation yielding.
* User-defined first-class function values.
* Arbitrary coroutine creation from script.
* A general-purpose module/package system.

The current execution model is intentionally centered on:

* Parsed AeoScript.
* Resumable compiled execution plans.
* Persistent fibers.
* Cooperative yielding.
* Engine-host integration.
* Explicit runtime limits.

The runtime can evolve toward a more formal bytecode VM later without requiring AeoScript's language semantics or engine integration model to change.
