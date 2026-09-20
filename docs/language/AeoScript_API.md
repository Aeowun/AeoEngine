# AeoScript API Reference

This document provides a comprehensive reference for the built-in functions, namespaces, and object properties available in AeoScript.

---

# 1. Global Functions

* `print(value)`: Prints a value to the terminal.
* `debug.log(value)`: Prints a value to the terminal with `SCRIPT` severity.
* `getAllCellsOfClass(class_name)`: Finds all authored world cells of a specified type (e.g., `"Light"`, `"Block"`). Returns a **Basket** of handles.
* `find(name)`: Finds all Cells or Entities whose authored **Identity** matches the given name. Returns a **Basket** of handles.
* `get_parent()`: Returns the handle of the parent object for the current script.
* `wait(seconds)`: Pauses execution of the current script for the specified duration.

---

# 2. `math` Namespace

Provides deterministic mathematical operations. All arguments and return values are `number`.

* `math.abs(x)`: Absolute value.
* `math.min(a, b)`: Smaller of two values.
* `math.max(a, b)`: Larger of two values.
* `math.floor(x)`: Largest integer less than or equal to x.
* `math.ceil(x)`: Smallest integer greater than or equal to x.
* `math.round(x)`: Nearest integer.
* `math.sqrt(x)`: Square root.
* `math.pow(base, exp)`: base raised to the power of exp.
* `math.sin(x)`, `math.cos(x)`, `math.tan(x)`: Trigonometric functions (radians).
* `math.clamp(val, min, max)`: Restricts val to the range `[min, max]`. Produces an error if `min > max`.
* `math.lerp(a, b, t)`: Linear interpolation: `a + (b - a) * t`.
* `math.deg_to_rad(deg)`: Converts degrees to radians.
* `math.rad_to_deg(rad)`: Converts radians to degrees.

---

# 3. `basket` Namespace (Arrays)

A `Basket` is a reference-backed collection. Methods can be called using `basket.fn(b, ...)` or `b.fn(...)`.

* `basket.create(count, [value])`: Creates a new basket of size `count`, optionally filled with `value` (default is `nil`).
* `basket.insert(b, [pos], value)`: Inserts `value`. If `pos` is omitted, appends to the end.
* `basket.remove(b, [pos])`: Removes and returns the element at `pos`. If `pos` is omitted, removes the last element.
* `basket.clear(b)`: Removes all elements from the basket.
* `basket.find(b, value, [init])`: Returns the index of the first occurrence of `value` starting from `init` (default 0), or `nil` if not found.
* `basket.move(src, a, b, t, [dst])`: Copies elements from `src[a..b]` into `dst` starting at index `t`. Overlap-safe.
* `basket.concat(b, [sep], [i], [j])`: Returns a string by joining elements from index `i` to `j` using `sep`.
* `basket.clone(b)`: Returns a new, shallow copy of the basket.
* `basket.freeze(b)`: Makes the basket read-only. Any future mutation attempt produces a runtime error.
* `basket.sort(b)`: Sorts numbers or strings in ascending order. (Custom comparators not supported yet).

---

# 4. `string` Namespace

String operations treat text as a sequence of Unicode characters (scalar values).

* `string.len(s)`: Returns the number of Unicode characters.
* `string.lower(s)`: Returns a lowercase version of the string.
* `string.upper(s)`: Returns an uppercase version of the string.
* `string.reverse(s)`: Returns the string with characters in reverse order.
* `string.split(s, [sep])`: Returns a Basket of strings split by `sep`. If `sep` is empty or omitted, splits into individual characters.

---

# 5. Cell Handles

A `Cell` refers to authored world content (Blocks, Lights). Changes to properties are temporary runtime overrides.

### Properties
* `.id`: Unique persistent 8-digit ID (u64). **Read-only**.
* `.name`: Authored **Identity** string. **Read-only**.
* `.cellType`: String representation of the type (e.g., `"Block"`). **Read-only**.
* `.visible`: Whether the cell is rendered. **Read/Write**.
* `.solid`: Whether the cell has collision. **Read/Write**.
* `.anchored`: Whether the cell is static (`true`) or falls (`false`). **Read/Write**.
* `.color`: RGB color as a Basket `[r, g, b]`. **Read/Write**.
* `.offset`: Visual offset from authored position as `[x, y, z]`. **Read/Write**.
* `.position`: Effective world position `[x, y, z]`. **Read-only**.

---

# 6. Entity Handles

An `Entity` refers to a live runtime object (Player, NPC).

### Properties
* `.id`: Unique runtime ID. **Read-only**.
* `.name`: Entity name. **Read-only**.
* `.position`: Current world position as `[x, y, z]`. **Read-only**.

### Methods
* `:is_valid()`: Returns `true` if the entity still exists.
* `:destroy()`: Removes the entity from the world.
* `:set_position(x, y, z)`: Teleports the entity.
* `:translate(dx, dy, dz)`: Moves the entity relative to its current position.

---

# 7. Global Host Objects

### `time`
* `time.delta`: Time elapsed since the previous frame in seconds.

---

# 8. Errors

AeoScript produces descriptive runtime errors for:
*   Indexing a Basket out of bounds.
*   Mutating a **frozen** Basket.
*   Calling a function with the wrong number or type of arguments.
*   Accessing a property on `nil` or an invalid handle.

Errors appear in the **PRINT OUTPUT** panel with full context (script path, line, and function name).
