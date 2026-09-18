# AeoScript VM

The AeoScript VM is the runtime that executes compiled AeoScript.

The VM is intentionally separate from the rest of AeoEngine.

Rust owns the VM.

AeoScript runs inside it.

---

# 1. Basic Flow

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
Compiler
        |
        v
Bytecode
        |
        v
AeoScript VM
        |
        v
AeoEngine API
```

---

# 2. Why a VM?

A VM gives AeoEngine a clean scripting boundary.

It also gives us control over:

```text
execution
errors
limits
debugging
hot reload
serialization
sandboxing
```

We do not need to compile user scripts into native Rust.

---

# 3. Source Compilation

Scripts should be compiled before execution.

Possible flow:

```text
Door.aeo
    |
    v
tokens
    |
    v
AST
    |
    v
semantic checks
    |
    v
bytecode
```

Compile errors stop execution of that script.

---

# 4. Bytecode

The bytecode format is not final.

It will likely contain instructions for:

```text
loading values
storing values
arithmetic
comparisons
jumps
function calls
returns
property reads
property writes
indexing
native calls
```

Possible instructions:

```text
LOAD
STORE
PUSH
POP
ADD
SUB
MUL
DIV
MOD
NEG
NOT
COMPARE
JUMP
JUMP_IF_FALSE
CALL
RETURN
GET_PROPERTY
SET_PROPERTY
GET_INDEX
SET_INDEX
CALL_NATIVE
```

This is a starting point, not the final instruction set.

---

# 5. Values

The VM needs a runtime value representation.

Initial value categories:

```text
number
bool
string
null
vec2
vec3
vec4
Entity
collection
```

The exact internal Rust representation is an implementation detail.

---

# 6. Entity Handles

An `Entity` inside the VM should be a safe engine-managed reference.

It should not be:

```text
raw pointer
Rust reference
memory address
```

The VM should be able to detect an entity that has been destroyed.

---

# 7. Script Instances

Each entity using a script receives a script instance.

Example:

```text
Player
    |
    +-- Player.aeo
          |
          +-- health = 100
          +-- speed = 6
```

Another player gets another set of values.

Script source is shared.

Script state is not.

---

# 8. Calling update

The engine controls lifecycle calls.

For example:

```text
engine frame
    |
    v
script instance
    |
    v
update(dt)
```

The script does not control when `update` is called.

---

# 9. Events

The same applies to events.

Example:

```text
engine
   |
   v
on_trigger_enter(other)
   |
   v
VM
```

The VM invokes the matching script function.

If the script does not define the event, nothing happens.

---

# 10. Instruction Limits

A broken script must not be able to lock the entire engine forever.

Potential limits:

```text
maximum instructions per update
maximum call depth
maximum recursion depth
maximum collection sizes where appropriate
```

The actual limits should be configurable for development.

---

# 11. Runtime Errors

Runtime errors should stop the affected script execution without taking down the engine.

Example:

```text
AeoScript Runtime Error

Enemy.aeo:41:12

Attempted to access a destroyed entity.
```

The VM should return structured error information to the engine.

---

# 12. Stack

The VM will need a call stack.

A frame will probably contain:

```text
function
instruction position
local values
```

The final structure depends on the compiler and bytecode design.

---

# 13. Locals

Functions need local variables.

Example:

```aeoscript
fn attack(target: Entity) {

    damage = 10

    target.damage(damage)
}
```

`damage` should exist only for the appropriate scope.

---

# 14. Fields

Entity fields live longer than a single function call.

Example:

```aeoscript
entity Enemy {

    health: number = 100
}
```

`health` belongs to the script instance.

---

# 15. Native Calls

Native calls cross from the VM into Rust.

Example:

```aeoscript
character.jump()
```

Conceptually:

```text
AeoScript
    |
    v
CALL_NATIVE
    |
    v
Rust binding
    |
    v
Character system
```

Bindings must be registered explicitly.

---

# 16. No Arbitrary Native Calls

The VM must not have an instruction that means:

```text
call_any_rust_function(...)
```

Every native function must be known by the runtime.

---

# 17. Garbage Collection / Memory

The memory model is not finalized.

Possible directions include:

```text
reference counting
arena allocation
garbage collection
hybrid allocation
```

The choice should be driven by actual runtime behavior.

Do not add a garbage collector just because scripting languages usually have one.

---

# 18. Strings

Strings will need VM-managed storage.

String lifetime must not depend on temporary Rust references.

---

# 19. Collections

Collections are still being designed.

The VM must eventually support:

```text
creation
indexing
iteration
mutation
```

The first collection implementation should be simple.

---

# 20. Hot Reload

The VM should eventually support replacing compiled script code while the engine is running.

Possible flow:

```text
old bytecode
     |
     v
compile changed source
     |
     v
new bytecode
     |
     v
replace script code
```

Existing state may be retained where possible.

The exact state migration rules are not final.

---

# 21. Serialization

Script state may eventually be serialized.

The VM needs a clear difference between:

```text
serializable data
runtime-only values
engine references
temporary locals
```

A destroyed entity should not become a saved raw handle.

---

# 22. Debugging

Eventually the VM should expose:

```text
current script
line
column
function
call stack
locals
fields
instruction count
```

That information can feed Monaco.

---

# 23. Determinism

The VM should avoid introducing unnecessary nondeterminism.

Execution order should be controlled by the engine.

The VM should not create unmanaged threads for normal script execution.

---

# 24. VM and Physics

The VM should not run physics itself.

The engine performs physics.

The script requests gameplay actions.

Example:

```aeoscript
physics.apply_force(force)
```

The actual physics solver remains Rust code.

---

# 25. VM and Rendering

The VM should not directly access OpenGL.

Scripts can tell the engine what should happen.

The renderer remains outside the VM.

---

# 26. Development Mode

Development builds may expose more VM information:

```text
instruction count
slow scripts
runtime errors
stack traces
script reloads
```

Release builds can remove or reduce some diagnostics.

---

# 27. Initial VM Goal

The first VM does not need to be incredibly fast.

It needs to be:

```text
correct
predictable
safe
easy to debug
easy to extend
```

Performance work comes after the basic language actually works.

---

# 28. First VM Milestone

The smallest useful VM should be able to execute:

```aeoscript
entity Test {

    value: number = 10

    fn update(dt: number) {
        value += dt
    }
}
```

Once that works reliably, start wiring it into real engine systems.

