# AeoScript Language Overview

AeoScript is AeoEngine's gameplay scripting language.

It is designed for gameplay logic, World interaction, runtime object control, input, events, UI, audio, and other engine-integrated behavior.

AeoScript operates through engine-owned handles and APIs. Runtime script changes are temporary unless an explicit engine operation modifies authored project data.
Learn more at [AEOWUN](https://aeowun.com).
---

# 1. Core Values

AeoScript's core value model includes:

```text
number
bool
string
nil
basket
map
engine handles
function values
```

---

# 2. Numbers

`number` is a 64-bit floating-point value.

```aeoscript
health: number = 100
speed = 4.5
```

Numbers are used for gameplay values, timers, coordinates, mathematical calculations, and other numeric state.

---

# 3. Booleans

`bool` represents:

```text
true
false
```

Example:

```aeoscript
enabled: bool = true
```

---

# 4. Strings

`string` represents Unicode text.

```aeoscript
name = "Ghost"
```

String operations are Unicode-aware rather than treating individual UTF-8 bytes as characters.

---

# 5. Nil

`nil` represents the absence of a value.

```aeoscript
const missing = nil
```

Missing map keys return `nil`.

Assigning `nil` to a map key removes the key.

---

# 6. Baskets

A `basket` is a zero-indexed, reference-backed sequence.

```aeoscript
const items = [1, 2, 3]
```

Baskets use reference semantics.

```aeoscript
const a = [1, 2]
const b = a

b[0] = 99
```

Both variables now refer to a basket whose first element is `99`.

Use `basket.clone()` for a distinct shallow copy.

Use `basket.freeze()` to make a basket read-only.

---

# 7. Maps

A `map` is a reference-backed key/value collection.

```aeoscript
const values = {}

values["name"] = "Ghost"
values[1] = "numeric key"
```

Numeric and string keys are distinct.

Missing keys return `nil`.

Assigning `nil` removes a key.

There is no separate map namespace. Maps use indexing and assignment directly.

---

# 8. Functions

Functions are declared with `fn`.

```aeoscript
fn add(a: number, b: number): number {
    return a + b
}
```

Functions can:

* Accept parameters.
* Return values.
* Call other functions.
* Be assigned to variables.
* Be passed as values.
* Capture lexical state when created as closures.

---

# 9. Closures

AeoScript supports function values and lexical closures.

```aeoscript
fn make_counter() {
    count: number = 0

    return fn() {
        count += 1
        return count
    }
}
```

The returned function can retain access to the captured scope.

Closures are ordinary runtime values and can be stored, passed, returned, or assigned to supported engine properties such as UI callbacks.

---

# 10. Entities and Fields

An `entity` declaration defines script state and functions associated with a scripted object.

```aeoscript
entity Counter {
    count: number = 0

    fn update(dt) {
        count += 1
    }
}
```

Entity fields belong to the running script instance.

Function locals belong to the active fiber.

---

# 11. Control Flow

AeoScript supports:

* `if`.
* `else if`.
* `else`.
* `while`.
* `for`.
* `return`.

Example:

```aeoscript
if health <= 0 {
    dead = true
}

for item in items {
    debug.log(item)
}
```

---

# 12. Top-Level Execution

AeoScript source files may contain executable top-level statements.

These execute once when the file is initialized.

```aeoscript
debug.log("Game script loaded")

const version = 1.0
```

Top-level code executes separately from function and event declarations.

Top-level execution can also yield using `wait()`.

---

# 13. Lifecycle and Events

AeoScript supports engine lifecycle functions and event handlers.

Common lifecycle functions include:

* `on_spawn()`
* `on_ready()`
* `update(dt)`
* `on_destroy()`

Collision events include:

* `on_touch(cell)`
* `on_overlap(overlapping, cell)`

Other engine events may be dispatched through the event system.

Lifecycle and event execution occurs through the script runtime and scheduler.

---

# 14. Fibers and `wait()`

AeoScript uses cooperative fibers for resumable execution.

```aeoscript
fn delayed_action() {
    wait(0.5)
    score += 10
}
```

The fiber preserves:

* Current function.
* Call stack.
* Local variables.
* Loop state.
* Instruction position.
* Pending wait state.

Execution resumes after the `wait()` rather than starting the function over.

---

# 15. Engine Handles

AeoScript uses opaque handles for engine-managed objects.

Current handle categories include:

* `Cell`.
* `Entity`.
* `Light`.
* `Sound`.
* `Ui`.
* `Mouse`.

Handles identify engine-owned objects. The script runtime does not directly own the underlying native object.

Invalid handles are resolved against current engine state rather than becoming raw native pointers.

---

# 16. Authored World Interaction

AeoScript can discover and manipulate supported World objects.

Examples:

```aeoscript
const blocks = find("Wall")
const lights = getAllCellsOfClass("Light")
```

A specific Cell can be accessed through its persistent ID:

```aeoscript
const door = cell.get(12345)
```

Cell IDs identify a specific authored instance.

Cell names are human-readable and are not required to be unique.

---

# 17. Runtime World State

AeoScript distinguishes between authored World data and runtime state.

During Play:

```aeoscript
cell.visible = false
cell.solid = false
cell.color = [1, 0, 0]
```

these changes affect runtime behavior.

They do not silently rewrite authored World data.

Runtime-created Cells are also temporary.

---

# 18. Attributes

Cells can expose authored custom attributes.

Current attribute value types are:

* Number.
* Bool.
* String.

Example:

```aeoscript
const health = cell.attributes["health"]

cell.attributes["locked"] = true
```

Runtime attribute writes create temporary overrides.

Assigning `nil` to a runtime attribute removes the override and restores the authored value.

---

# 19. Input

AeoScript can access gameplay input.

Current input functionality includes:

* Movement vector.
* Jump state.
* Mouse orbit delta.
* Mouse cursor visibility.
* Mouse screen locking.

These APIs provide runtime input state rather than authoring editor input.

---

# 20. Camera

Scripts can interact with the gameplay camera.

Current operations include:

* Horizontal camera basis.
* Camera position.
* Camera target.
* Camera orientation.
* Camera collision resolution.

The gameplay camera remains a runtime system owned by AeoEngine.

---

# 21. Player APIs

AeoScript can control supported player movement behavior.

Current APIs include:

* Horizontal velocity.
* Facing direction.
* Animation selection.
* Grounded state.
* Vertical impulse.
* Player position.

These APIs express gameplay intent while CharacterSystem remains responsible for character simulation.

---

# 22. Runtime UI

Scripts can create runtime UI elements:

```aeoscript
const panel = ui.new("Panel")
const text = ui.new("Text")
const button = ui.new("Button")
```

Current UI types include:

* `Panel`.
* `Text`.
* `Button`.

UI elements expose runtime properties and Button handles can receive closure callbacks through `on_click`.

---

# 23. Audio

AudioEmitter Cells expose runtime sound control.

```aeoscript
const speaker = find("DoorSound")[0]

speaker.sound.play()
```

Supported sound operations include:

* Play.
* Stop.
* Pause.
* Playing state.
* Looping.
* Volume.

---

# 24. Standard Library

Current built-in namespaces include:

```text
math
basket
string
get
input
camera
physics
player
ui
cell
entity
script
event
test
```

Global functions and properties also include operations such as:

```text
debug.log(...)
find(...)
getAllCellsOfClass(...)
wait(...)
time.delta
```

See `AeoScript_API.md` and `AeoScript_STDLIB.md` for reference details.

---

# 25. Script Bindings

Scripts are attached to authored Cells through persistent Cell-ID bindings.

```text
Cell ID → .aeo script
```

The binding identifies the specific authored instance.

The human-readable Cell name is not the persistent binding key.

---

# 26. Errors and Diagnostics

AeoScript reports structured runtime diagnostics for invalid operations such as:

* Invalid handles.
* Invalid arguments.
* Invalid collection access.
* Unsupported properties or methods.
* Runtime execution failures.
* Execution-budget failures.

Diagnostics can include:

* Script path.
* Entity context.
* Function context.
* Source location.
* Runtime error information.

---

# 27. Execution Model

The language runtime is a single-threaded interpreter with resumable execution state.

The broad execution path is:

```text
Source
 ↓
Lexer
 ↓
Parser
 ↓
AST
 ↓
Interpreter
 ↓
Fiber
 ↓
Scheduler
 ↓
ScriptScene
 ↓
Engine
```

The runtime is designed around cooperative yielding rather than native-code compilation or a multithreaded scripting VM.

---

# 28. Language Design

AeoScript is intended to remain:

* Small enough to learn.
* General enough for gameplay.
* Closely integrated with AeoEngine.
* Explicit about engine-owned objects.
* Safe around engine-managed references.
* Suitable for resumable gameplay logic.
* Consistent with the authored/runtime boundary.

The language should grow in response to real engine workflows rather than accumulating specialized APIs without a demonstrated need.