# AeoScript VM Execution Model

AeoScript uses a single-threaded interpreter with resumable execution state integrated into AeoEngine's runtime loop.

It is not currently a native-code compiler or a conventional standalone bytecode virtual machine.

The runtime parses source into an AST and builds internal execution plans that allow instruction position and control-flow state to survive cooperative yields.

---

# 1. Execution Pipeline

The broad execution path is:

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
Execution Plan
      ↓
ScriptFiber
      ↓
ScriptScheduler
      ↓
ScriptScene
      ↓
AeoEngine
```

---

# 2. Interpreter

The interpreter is responsible for:

* Evaluating expressions.
* Executing statements.
* Resolving functions.
* Executing engine-host operations.
* Creating/resuming execution state.
* Enforcing execution limits.
* Producing runtime diagnostics.

The interpreter owns language semantics.

It does not own the World, renderer, physics, or character systems.

---

# 3. ScriptFiber

A `ScriptFiber` represents one resumable execution.

A fiber contains the state required to continue execution, including:

* Active script instance.
* Call stack.
* Instruction position.
* Local scopes.
* Loop state.
* Return state.
* Completion state.
* Yield state.

When a script calls `wait()`, the fiber remains alive.

---

# 4. Cooperative Yielding

AeoScript yielding is cooperative.

```text
Script execution
      ↓
wait(seconds)
      ↓
Fiber yields
      ↓
Scheduler stores wake state
      ↓
Engine continues
      ↓
Wake time reached
      ↓
Same fiber resumes
```

There is no scripting thread created for an individual `wait()`.

---

# 5. Single-Threaded Execution

The current scripting runtime executes on the engine's runtime thread.

Scripts do not execute concurrently on a separate scripting thread.

This makes runtime state access straightforward while requiring execution to respect the configured operation limits.

---

# 6. Fiber Results

A fiber can produce results such as:

```text
Continue
Yield
Complete
Failed
```

### Continue

The fiber remains runnable.

### Yield

The fiber waits for a future scheduler time.

### Complete

Execution reached its end.

### Failed

A runtime error terminated execution.

The ScriptScene and scheduler use the result to update runtime state.

---

# 7. Call Stack

Each active function call has execution state associated with it.

A frame retains information such as:

* Execution plan.
* Instruction position.
* Local scope.
* Loop state.
* Function context.

Nested calls therefore remain valid across yields.

Example:

```text
update()
  ↓
gameplay_function()
  ↓
helper_function()
  ↓
wait()
```

After the wait, the runtime resumes inside `helper_function()`.

---

# 8. Execution Budget

The interpreter uses an operation budget for each execution slice.

The current default is:

```text
100,000 operations
```

A custom budget may be supplied by the runtime.

If the budget is exhausted before execution yields or completes, the script fails with an execution-budget error.

Budget exhaustion is not equivalent to `wait()`.

The budget exists to prevent runaway execution from monopolizing the engine runtime.

---

# 9. Call Depth

The interpreter also limits nested user-function calls.

The current default maximum depth is:

```text
64
```

Exceeding the limit produces a runtime error.

This protects against unbounded recursion or pathological call chains.

---

# 10. ScriptInstance

A `ScriptInstance` represents persistent script state for a scripted object.

It contains information such as:

* Script/entity identity.
* Script path.
* Persistent entity fields.

Example:

```aeoscript
entity Counter {
    ticks: number = 0
}
```

`ticks` belongs to the script instance.

---

# 11. Fiber State vs Script State

The distinction is:

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

Entity fields survive lifecycle executions.

Local variables belong to active execution frames.

A local can survive `wait()` without becoming a persistent field.

---

# 12. Scheduler

`ScriptScheduler` determines which fibers are runnable.

Typical task states include:

```text
Ready
Running
Waiting
Complete
Failed
```

During a runtime tick the scheduler:

```text
1. Advances script time.
2. Wakes tasks whose wait time expired.
3. Takes ready tasks.
4. Executes each for one slice.
5. Applies the resulting FiberResult.
```

This prevents a runnable script from repeatedly consuming the entire same runtime tick.

---

# 13. Waiting

A waiting task stores a future wake time.

The fiber itself remains intact.

When the scheduler reaches the wake time:

```text
Waiting
   ↓
Ready
   ↓
Resume same fiber
```

The runtime does not start the lifecycle function over.

---

# 14. Closures

Function values can capture lexical scope.

The captured state becomes part of the function value and remains available while the closure exists.

This is separate from the persistent script-instance model.

---

# 15. Collections

Baskets and maps are reference-backed runtime values.

The scripting runtime stores references to their collection state.

`basket.clone()` creates a distinct shallow outer collection.

`basket.freeze()` prevents further mutation.

The current runtime is single-threaded, so these collections are not designed for concurrent access.

---

# 16. Engine Handles

Engine objects are represented as handles containing an engine-managed kind and identifier.

Examples include:

```text
Cell
Entity
Light
Sound
Ui
Mouse
```

The interpreter does not own the underlying engine object.

Property/method access is resolved through the host integration.

---

# 17. Host Integration

Engine interaction crosses a host boundary:

```text
Interpreter
      ↓
EngineHost
      ↓
AeoEngine host implementation
      ↓
World / Physics / Character / Entity / Renderer
```

This keeps the interpreter independent from concrete engine implementations.

---

# 18. Runtime Diagnostics

Runtime errors retain structured context where available.

Diagnostics can include:

* Script path.
* Entity name.
* Entity ID.
* Function context.
* Source location.
* Error severity.
* Runtime message.

The diagnostic system forwards this information to AeoEngine's script output facilities.

---

# 19. Lifecycle Execution

A typical scripted object follows:

```text
Script binding
      ↓
ScriptInstance
      ↓
on_spawn
      ↓
on_ready
      ↓
update
      ↓
yield / complete
      ↓
ScriptScheduler
```

Events such as `on_touch` and `on_overlap` enter the same runtime execution infrastructure.

---

# 20. Runtime Boundaries

The current runtime does not provide:

* Native-code compilation.
* Multithreaded script execution.
* A separate scripting thread.
* Arbitrary script-created OS threads.
* Automatic continuation after budget exhaustion.
* A general module/package ecosystem.

The current design is intentionally centered on:

* Parsed AeoScript.
* Internal execution plans.
* Resumable fibers.
* Cooperative yielding.
* Engine-host integration.
* Explicit runtime limits.

This provides the behavior needed by the current engine without requiring the complexity of a native compiler or multithreaded scripting VM.