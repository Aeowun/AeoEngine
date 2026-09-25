# AeoScript Types

AeoScript uses a small runtime value model centered on gameplay values, collections, functions, and references to engine-managed objects.

---

# 1. Core Types

The core runtime values are:

```text
number
bool
string
nil
basket
map
function
engine handle
```

---

# 2. `number`

`number` is a 64-bit floating-point value.

```aeoscript
health: number = 100
speed = 4.5
```

Numbers are used for gameplay values, timers, coordinates, and mathematics.

---

# 3. `bool`

`bool` is:

```text
true
false
```

Example:

```aeoscript
enabled: bool = true
```

---

# 4. `string`

`string` represents Unicode text.

```aeoscript
name: string = "Ghost"
```

String functions operate on Unicode scalar values rather than individual UTF-8 bytes.

---

# 5. `nil`

`nil` represents the absence of a value.

```aeoscript
const missing = nil
```

Missing map keys return `nil`.

Assigning `nil` to a map key removes that entry.

---

# 6. `basket`

A basket is a zero-indexed reference-backed sequence.

```aeoscript
const items = ["Sword", "Shield", "Potion"]
```

Baskets are aliases when assigned:

```aeoscript
const a = [1, 2]
const b = a

b[0] = 99
```

Both variables observe the same collection.

Use:

```aeoscript
basket.clone(a)
```

for a separate shallow outer basket.

Use:

```aeoscript
basket.freeze(a)
```

for a read-only basket.

---

# 7. `map`

A map is a reference-backed key/value collection.

```aeoscript
const stats = {}

stats["health"] = 100
stats[1] = "numeric key"
```

Numeric and string keys are different keys.

Missing keys return `nil`.

Assigning `nil` removes a key.

---

# 8. Functions

Functions are first-class values.

```aeoscript
const double = fn(value) {
    return value * 2
}
```

Functions may be:

* Stored in variables.
* Passed to other functions.
* Returned from functions.
* Captured as closures.
* Assigned to supported engine callback properties.

---

# 9. `Cell`

A `Cell` is an engine-managed handle to an authored or runtime World Cell.

It does not contain the actual native Cell object.

The Cell has:

* Persistent ID.
* Human-readable identity/name.
* Cell type.
* Supported runtime properties.
* Attribute access.

The ID identifies the specific instance.

The name does not have to be unique.

---

# 10. `Entity`

An `Entity` is an engine-managed handle to a live runtime Entity.

Entities are runtime objects.

They are distinct from authored World Cells.

---

# 11. `Light`

Light handles reference engine-managed Light Cells.

A Light handle uses the same general engine-handle model as other World object handles while exposing Light-specific properties through the host API.

---

# 12. `Sound`

A `Sound` handle represents an audio runtime interface associated with an AudioEmitter Cell or runtime audio object.

Supported runtime state includes:

* Playing.
* Looping.
* Volume.
* Play.
* Pause.
* Stop.

---

# 13. `Ui`

A `Ui` handle references a runtime UI element.

Current UI element types include:

* Panel.
* Text.
* Button.

The handle exposes properties such as:

* Position.
* Size.
* Visibility.
* Enabled state.
* Color.
* Text.
* Button callback.

---

# 14. `Mouse`

The mouse handle represents the runtime mouse input interface.

Current controls include:

* Cursor visibility.
* Screen locking.

The underlying OS/input resource remains engine-owned.

---

# 15. Optional Types

A type can be marked optional:

```aeoscript
target: Entity?
```

An optional value may contain the specified type or `nil`.

---

# 16. `const`

`const` prevents reassignment of the variable binding.

It does not automatically freeze a referenced object.

```aeoscript
const items = [1, 2]

items[0] = 99
```

The basket remains mutable.

Use `basket.freeze()` to make a basket itself read-only.

---

# 17. Value and Reference Semantics

Primitive values behave as values.

Baskets and maps are reference-backed.

Engine handles are lightweight references to engine-managed objects.

Function values may also carry captured lexical state.

---

# 18. Runtime State

Handle values provide access to engine runtime state.

Changing a supported Cell property during Play changes runtime state.

It does not automatically modify authored World data.

The language therefore preserves the engine's broader state boundary:

```text
Authored World
      ↓
Runtime effective state
      ↓
AeoScript handles
```

---

# 19. Type Design

The language intentionally keeps its fundamental value model small.

The engine can add new handle categories or runtime values as concrete gameplay workflows require without turning every internal engine type into an AeoScript type.