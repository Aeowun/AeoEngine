# AeoScript VM

The AeoScript VM is the runtime that executes AeoScript.

Rust owns the runtime.

AeoScript runs inside it.

The VM is responsible for executing script code, managing script state, enforcing runtime limits, handling errors, and calling the engine APIs that AeoScript is allowed to use.

The VM should stay separate from the internal implementation of AeoEngine.

---

# 1. Basic Flow

The current runtime is built around the AeoScript lexer, parser, AST, interpreter, fibers, scheduler, and engine host API.

Conceptually:

```text
AeoScript source
        |
        v
Lexer
        |
        v
Parser
        |
        v
AST
        |
        v
AeoScript Runtime / VM
        |
        v
Engine Host API
        |
        v
AeoEngine
```

A bytecode compiler and bytecode execution layer may be introduced later.

That is not a requirement for the first usable runtime.

---

# 2. Why a VM?

A VM gives AeoEngine a controlled scripting boundary.

It gives us control over:

```text
execution
script state
runtime errors
execution limits
events
engine API access
debugging
hot reload
sandboxing
```

AeoScript should never become native Rust code.

---

# 3. Execution

Scripts are parsed into the existing AeoScript representation and executed by the runtime.

The runtime is responsible for:

```text
evaluating expressions
executing statements
calling functions
maintaining script state
handling fibers
scheduling script execution
calling engine APIs
dispatching events
```

Compile-time and runtime responsibilities should remain separate.

A syntax or semantic error should prevent invalid code from executing.

A runtime error should stop the affected script execution without crashing the engine.

---

# 4. Values

The VM needs a runtime representation for AeoScript values.

Initial value categories include:

```text
number
bool
string
nil
Cell
Entity
Basket
vec2
vec3
vec4
```

The exact Rust representation is an implementation detail.

The language-facing terminology should remain consistent:

```text
nil
Basket
Cell
Entity
```

---

# 5. Nil

The runtime representation for the AeoScript `nil` value is:

```text
Value::Nil
```

Example:

```aeoscript id="5pk2b9"
target: Entity? = nil
```

`nil` represents the absence of a value.

The old `null` terminology should not remain in the language-facing VM model.

---

# 6. Baskets

A Basket is AeoScript's collection value.

Baskets are:

```text
0-based
```

and support:

```aeoscript id="mvsqpn"
items[0]
items.len()
```

Baskets may contain supported AeoScript values, including:

```text
Cells
Entities
Baskets
numbers
strings
booleans
nil
```

The runtime is responsible for safe indexing, iteration, and mutation according to the operations supported by the language.

---

# 7. Cells

A Cell represents authored world content.

A Cell inside the VM is a safe engine-managed reference.

It should not be:

```text
raw pointer
Rust reference
memory address
```

A Cell reference must be validated when used.

If the referenced authored cell no longer exists or has been replaced, the runtime must handle the invalid reference safely.

A Cell is read-only from the scripting side unless the engine explicitly exposes a writable operation.

---

# 8. Entities

An Entity represents a live runtime object.

Examples include the runtime Player and other engine-managed runtime objects.

An Entity inside the VM is a safe engine-managed reference.

The VM must not store raw Rust pointers or memory addresses as script values.

A destroyed Entity must be detectable by the runtime.

Script code must never be able to turn an invalid Entity reference into an engine crash.

---

# 9. Object Access

The VM provides the runtime side of AeoScript's generic object model.

Properties use:

```aeoscript id="h6fsyh"
object.property
```

Methods use:

```aeoscript id="j33v1n"
object:method(...)
```

Examples:

```aeoscript id="ilox5e"
player.name
player.position

player:destroy()
character:jump()
```

The VM resolves these through the engine's registered object/API system.

It should not contain a growing list of special cases for every engine object.

---

# 10. Cell to Entity

A Cell may expose a live runtime object through:

```aeoscript id="upr5c5"
object = cell:getObject()
```

The result is:

```text
Entity
```

when a corresponding runtime object exists.

Otherwise:

```text
nil
```

This is a generic Cell → Entity operation.

There should not be separate methods for individual object types.

---

# 11. Discovery

The VM must be able to execute the generic discovery APIs exposed by the engine.

Examples:

```aeoscript id="gq7j53"
lights = getAllCellsOfClass("Light")
```

```aeoscript id="jtw2dr"
objects = find("Door")
```

and:

```aeoscript id="sl0szs"
children = object:get_children()
parent = object:get_parent()
```

The runtime receives and passes around the resulting Cell/Entity references as normal AeoScript values.

Discovery should not require an object-specific VM instruction.

---

# 12. Script Instances

An attached script gets its own script instance.

For example:

```text
Player
    |
    +-- Player.aeo
          |
          +-- health
          +-- speed
```

Another entity using the same script gets its own state.

Script source is shared.

Script instance state is not.

Unattached scripts are also valid and can act as game/server-style scripts that discover objects and respond to engine events.

---

# 13. Fields

Fields belong to a script instance.

Example:

```aeoscript id="8ejip8"
entity Enemy {

    health: number = 100
}
```

`health` belongs to the individual script instance.

Two entities using the same script do not share the same field value.

---

# 14. Locals

Functions have local variables.

Example:

```aeoscript id="62v8d5"
fn attack(target: Entity) {

    damage = 10

    target:damage(damage)
}
```

`damage` belongs to the appropriate function execution/scope.

Local values should not unexpectedly become persistent script state.

---

# 15. Functions

The VM executes AeoScript functions and manages their parameters, locals, return values, and call stack.

Example:

```aeoscript id="gx5k9v"
fn is_alive(): bool {
    return health > 0
}
```

Function calls remain within the controlled AeoScript runtime unless the function explicitly crosses the native API boundary.

---

# 16. Call Stack

The runtime needs a call stack for nested function execution.

A frame needs enough information to track things such as:

```text
function
instruction/expression position
locals
return state
source location
```

The exact Rust structure may change as the runtime evolves.

Source locations should be retained where practical so runtime errors can point back to the script.

---

# 17. Fibers

AeoScript already supports cooperative execution through fibers.

Fibers allow a script operation to suspend and resume without creating an unmanaged OS thread.

This is important for operations such as:

```aeoscript id="u9veb5"
wait(1)
```

The runtime controls when a suspended fiber resumes.

---

# 18. Scheduler

The scheduler controls runnable script execution.

Conceptually:

```text
Engine update
      |
      v
Script scheduler
      |
      +-- Fiber A
      +-- Fiber B
      +-- Fiber C
      |
      v
AeoScript execution
```

The runtime should remain single-threaded unless the engine explicitly introduces another execution model later.

Normal gameplay scripts should not create their own unmanaged threads.

---

# 19. Update

The engine controls when `update` runs.

Example:

```aeoscript id="b68ekd"
fn update(dt: number) {
    timer -= dt
}
```

The script does not create its own game loop.

The engine supplies `dt` and controls execution.

---

# 20. Lifecycle

Lifecycle functions are engine-controlled entry points.

Examples:

```aeoscript id="nbl9dg"
fn on_spawn() {
}

fn on_ready() {
}

fn update(dt: number) {
}

fn on_destroy() {
}
```

The VM invokes these through the existing script lifecycle mechanism.

---

# 21. Events

Events use the AeoScript event syntax:

```aeoscript id="g74q7f"
on EventName(args) {
    ...
}
```

Example:

```aeoscript id="ph8v1m"
on PlayerSpawned(player) {
    print("Player spawned")
}
```

The runtime stores registered event handlers and dispatches them when the engine emits a matching event.

Events are generic.

`PlayerSpawned` is not a special language construct.

---

# 22. Event Arguments

Event arguments are normal AeoScript values.

For example:

```aeoscript id="xq5g6s"
on PlayerSpawned(player) {
    print(player.name)
}
```

The `player` value is the live `Entity` reference supplied by the engine.

Future events can pass other supported values:

```aeoscript id="b2z5wl"
on DoorOpened(door) {
}

on EntityDestroyed(entity) {
}

on ButtonPressed(button) {
}
```

---

# 23. Player Spawn

The first concrete engine event is `PlayerSpawned`.

The event must originate from the existing engine spawn flow:

```text
SpawnPoint
    |
    v
CharacterSystem
    |
    v
runtime Player Entity
    |
    v
PlayerSpawned(player)
    |
    v
AeoScript Runtime
```

The VM does not create the Player.

The VM receives the actual runtime Entity created by the engine.

---

# 24. Native API Boundary

AeoScript can only interact with native engine functionality that has been explicitly exposed.

Conceptually:

```text
AeoScript
    |
    v
VM
    |
    v
Registered engine API
    |
    v
AeoEngine
```

There is no generic:

```text
call_any_rust_function(...)
```

operation.

---

# 25. Native Calls

A native API call may be reached through object methods or other registered engine functions.

Example:

```aeoscript id="v5m9zw"
character:jump()
```

Conceptually:

```text
AeoScript
    |
    v
VM
    |
    v
registered binding
    |
    v
CharacterSystem
```

The VM is responsible for validating arguments and handling the result returned by the engine API.

---

# 26. API Safety

A native API must define what AeoScript is allowed to do.

The VM should not expose:

```text
raw Rust references
raw pointers
memory addresses
arbitrary filesystem access
shell commands
arbitrary process creation
native DLL loading
direct OpenGL access
arbitrary OS APIs
```

AeoScript interacts with the engine through explicit APIs.

---

# 27. Instruction / Execution Limits

A broken script must not be able to lock the engine indefinitely.

The runtime should enforce appropriate limits such as:

```text
maximum work per update
maximum call depth
maximum recursion depth
```

Existing operation-budget and cooperative-fiber mechanisms should remain part of this protection.

Limits should be configurable where appropriate for development.

---

# 28. Runtime Errors

Runtime errors should stop the affected script execution without taking down the engine.

Example:

```text
AeoScript Runtime Error

Enemy.aeo:41:12

Attempted to access a destroyed entity.
```

The runtime should return structured error information containing source information where available.

The editor can then display that information in Monaco.

---

# 29. Invalid Object References

Invalid engine references must be handled safely.

For example:

```aeoscript id="29lwt9"
target: Entity? = nil
```

If `target` later refers to a destroyed Entity, the VM must not dereference invalid native state.

The engine/API decides whether a particular operation:

```text
returns nil
reports an error
reports that the object is unavailable
```

but the VM must never crash because a script retained an invalid reference.

---

# 30. Collections and References

Cells and Entities can be stored in Baskets.

Example:

```aeoscript id="a8g5cc"
children = object:get_children()

first = children[0]
```

The Basket contains script references, not copies of the underlying native object.

The lifetime of the actual engine object remains owned by AeoEngine.

---

# 31. Garbage Collection / Memory

The final memory strategy is not fixed yet.

Possible approaches include:

```text
reference counting
arena allocation
garbage collection
hybrid allocation
```

Do not add a garbage collector simply because scripting languages commonly have one.

Choose the memory model based on the actual AeoScript runtime.

Engine-owned references such as Cells and Entities must remain separately controlled by the engine.

---

# 32. Strings

Strings must have runtime-managed storage.

A script string must not depend on the lifetime of a temporary Rust reference.

The exact storage implementation is internal to the VM.

---

# 33. Hot Reload

Hot reload is an eventual runtime feature.

The intended flow is:

```text
script source changes
        |
        v
parse / validate
        |
        v
new script representation
        |
        v
replace running script code
```

Whether existing script state survives a reload is a separate runtime decision.

The architecture should leave room for preserving compatible state.

---

# 34. Serialization

Script state may eventually be serialized.

The VM needs to distinguish between:

```text
serializable script data
runtime-only values
Cell references
Entity references
temporary locals
active fibers
```

A raw runtime reference must never become arbitrary saved memory.

Serialization rules will be defined as the save/load system matures.

---

# 35. Debugging

The runtime should eventually expose useful debugging information:

```text
current script
line
column
function
call stack
locals
fields
event
execution count
```

That information can feed the Monaco editor and future debugger.

---

# 36. Determinism

The VM should avoid introducing unnecessary nondeterminism.

Normal script execution is controlled by the engine.

The VM should not create unmanaged threads for normal gameplay scripting.

Execution order for scripts and events should be defined by the runtime/engine rather than depending on accidental thread timing.

---

# 37. Physics

The VM does not perform physics.

AeoEngine performs physics.

A script requests gameplay behavior through the engine API.

Example:

```aeoscript id="yv0wkl"
physics:apply_force(force)
```

The actual physics solver remains Rust code.

---

# 38. Rendering

The VM does not directly access OpenGL.

Scripts interact with rendering-related behavior through explicit AeoEngine APIs where such APIs are provided.

The renderer remains outside the VM.

---

# 39. Development Mode

Development builds may expose additional runtime information:

```text
script execution time
instruction / operation count
slow scripts
runtime errors
stack traces
fiber state
event counts
reloads
```

This information is primarily for development and debugging.

---

# 40. Future Bytecode

A bytecode compiler may eventually be introduced.

A possible future flow is:

```text
AeoScript
    |
    v
Lexer
    |
    v
Parser
    |
    v
AST
    |
    v
Compiler
    |
    v
Bytecode
    |
    v
AeoScript VM
```

Possible bytecode operations could include:

```text
load value
store value
arithmetic
comparison
jump
call
return
property read
property write
index
native call
```

This is future architecture, not a requirement that the current runtime already use bytecode.

---

# 41. Performance

The first VM does not need to be extremely fast.

It needs to be:

```text
correct
predictable
safe
easy to debug
easy to extend
```

Performance optimization should come after the scripting model and runtime behavior are stable.

The engine should continue to own expensive systems such as:

```text
rendering
physics
animation
world processing
resource loading
```

Scripts should primarily coordinate those systems.

---

# 42. VM and Engine Boundary

The fundamental boundary is:

```text
AeoEngine
    |
    | Rust
    v
Script Host / API
    |
    v
AeoScript VM
    |
    v
Game Scripts
```

The VM does not own the engine.

The engine does not execute arbitrary script internals.

The VM uses the APIs explicitly provided by the engine.

---

# 43. Initial VM Goal

The first useful runtime should reliably execute scripts containing:

```text
variables
fields
functions
expressions
conditions
loops
property access
method calls
Baskets
Cells
Entities
nil
events
engine API calls
wait / cooperative execution
```

It should also provide:

```text
runtime limits
safe engine references
useful runtime errors
script state isolation
```

---



Once that works reliably, the runtime has crossed from language infrastructure into a (hopefully) genuinely useful system.
