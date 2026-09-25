# AeoScript Standard Library

The AeoScript standard library provides common language and engine-integrated gameplay operations.

The current library is intentionally divided between general-purpose collection/math/string utilities and APIs that explicitly interact with AeoEngine runtime systems.

---

# 1. Core Namespaces

The current built-in namespaces include:

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

Additional global/property APIs include:

```text
debug.log(...)
find(...)
getAllCellsOfClass(...)
wait(...)
time.delta
```

---

# 2. `math`

Numeric and mathematical utilities.

Available operations include:

```text
abs
min
max
floor
ceil
round
sqrt
pow
sin
cos
tan
clamp
lerp
deg_to_rad
rad_to_deg
random
```

Examples:

```aeoscript
const x = math.clamp(value, 0, 10)
const angle = math.deg_to_rad(90)
const roll = math.random(1, 6)
```

---

# 3. `basket`

Reference-backed sequence operations.

Available operations include:

```text
create
insert
remove
clear
find
move
concat
clone
freeze
sort
```

Example:

```aeoscript
const items = ["Sword", "Potion"]

items.insert(0, "Map")
items.sort()
```

Baskets are zero-indexed.

Custom comparator sorting is not currently supported.

---

# 4. `string`

Unicode-aware string operations include:

```text
len
lower
upper
reverse
split
```

Example:

```aeoscript
const parts = string.split("one,two", ",")
```

The runtime operates on Unicode scalar values rather than raw UTF-8 bytes.

---

# 5. `get`

General engine access.

Current operation:

```text
get.mouse()
```

Example:

```aeoscript
const mouse = get.mouse()
```

---

# 6. `input`

Gameplay input access.

Current operations:

```text
input.get_move_vector()
input.is_jump_pressed()
input.get_orbit_delta()
```

These return runtime input state.

---

# 7. `camera`

Gameplay camera control.

Current operations include:

```text
camera.get_horizontal_basis()
camera.set_position(...)
camera.set_target(...)
camera.set_orientation(...)
```

The APIs control the active gameplay camera.

---

# 8. `physics`

Runtime physics queries.

Current operation:

```text
physics.resolve_camera_collision(...)
```

This resolves a desired gameplay-camera position against collision geometry.

---

# 9. `player`

Runtime player-character controls.

Current operations include:

```text
set_horizontal_velocity
set_facing_direction
select_animation
is_grounded
apply_vertical_impulse
position
```

The APIs express gameplay intent while CharacterSystem remains responsible for character simulation.

---

# 10. `ui`

Runtime UI management.

Current operations include:

```text
ui.new(...)
ui.delete(...)
ui.get_viewport_size()
```

Supported UI element types are:

```text
Panel
Text
Button
```

Button callbacks use AeoScript function values.

---

# 11. `cell`

Runtime Cell management and lookup.

Current operations include:

```text
cell.new(...)
cell.delete(...)
cell.get(...)
cell.exists(...)
```

`cell.new()` creates runtime-only Cells.

`cell.delete()` can remove runtime-created Cells or temporarily remove authored Cells during Play.

---

# 12. `entity`

Runtime Entity lookup.

Current operations:

```text
entity.get(...)
entity.exists(...)
```

---

# 13. `script`

Runtime script execution control.

Current operations:

```text
script.enable(path)
script.disable(path)
script.is_enabled(path)
```

Disabling a script affects runtime execution.

It does not delete the `.aeo` source file.

---

# 14. `event`

Runtime event dispatch.

Current operation:

```text
event.fire(name, ...)
```

This is used to dispatch engine/script events.

---

# 15. `test`

In-game integration testing.

Current operations include:

```text
test.complete(...)
test.is_completed(...)
test.passed(...)
test.summary()
```

These are used by AeoEngine's in-game scripting test harness.

---

# 16. `debug`

`debug.log(...)` writes structured script diagnostics/output.

Example:

```aeoscript
debug.log("Player ID:", player.id)
```

It is intended for development diagnostics rather than persistent game state.

---

# 17. Global Discovery

## `find(name)`

Searches active/effective World Cells and runtime Entities by their supported names.

Because names are not unique, multiple results are possible.

## `getAllCellsOfClass(class_name)`

Returns active/effective World Cells matching the requested class.

Example:

```aeoscript
const lights = getAllCellsOfClass("Light")
```

---

# 18. `wait`

`wait(seconds)` is part of the runtime rather than a general collection/math namespace.

It yields the current fiber cooperatively.

```aeoscript
wait(1.0)
```

The scheduler resumes the same execution state after the wait.

---

# 19. Standard Library Design

The standard library should remain centered around small, composable APIs.

The preferred model is:

```text
Small language core
      ↓
Small standard library
      ↓
Engine host APIs
      ↓
Owning runtime system
```

The library should express gameplay intent rather than expose internal engine architecture.

New namespaces and specialized APIs should be added when a concrete gameplay workflow requires them.