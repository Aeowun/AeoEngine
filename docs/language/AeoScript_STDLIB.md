# AeoScript Standard Library

The AeoScript standard library provides common gameplay utilities through built-in namespaces.

The current standard library contains:

~~~text
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
~~~

These functions execute inside the normal AeoScript runtime.

---

# 1. `math`

The `math` namespace provides numeric, trigonometric, interpolation, conversion, and random-value utilities.

### `math.abs(x)`

Returns the absolute value.

### `math.min(a, b)`

Returns the smaller of exactly two numeric arguments.

~~~aeoscript
const lowest = math.min(8, 3)
~~~

### `math.max(a, b)`

Returns the larger of exactly two numeric arguments.

### `math.floor(x)`

Returns the largest integer less than or equal to `x`.

### `math.ceil(x)`

Returns the smallest integer greater than or equal to `x`.

### `math.round(x)`

Rounds to the nearest integer.

### `math.sqrt(x)`

Returns the square root.

### `math.pow(base, exponent)`

Raises `base` to `exponent`.

### `math.sin(x)`

Returns sine of an angle in radians.

### `math.cos(x)`

Returns cosine of an angle in radians.

### `math.tan(x)`

Returns tangent of an angle in radians.

### `math.clamp(value, min, max)`

Constrains `value` to the inclusive range `[min, max]`.

An invalid range where `min > max` produces a runtime error.

### `math.lerp(a, b, t)`

Returns:

~~~text
a + (b - a) * t
~~~

### `math.deg_to_rad(degrees)`

Converts degrees to radians.

### `math.rad_to_deg(radians)`

Converts radians to degrees.

---

# 2. `basket`

The `basket` namespace provides operations for reference-backed baskets.

Basket functions can be called through the namespace or through the basket method syntax where supported.

~~~aeoscript
basket.create(3, 5)
items.insert(0, "Sword")
items.clear()
~~~

### `basket.create(count, [value])`

Creates a basket containing `count` elements.

When `value` is omitted, the elements are initialized to `nil`.

### `basket.insert(b, [position], value)`

Inserts a value at a position. Without a position, appends to the basket.

### `basket.remove(b, [position])`

Removes and returns an element. Without a position, removes the last element.

### `basket.clear(b)`

Removes all elements.

### `basket.find(b, value, [init])`

Returns the first matching zero-based index, starting at `init` or `0`.

Returns `nil` when no match exists.

### `basket.move(src, a, b, t, [dst])`

Copies the inclusive source range `src[a..b]` to the destination starting at `t`.

The operation is overlap-safe.

### `basket.concat(b, [separator], [i], [j])`

Joins basket elements into a string over the inclusive range `i..j`.

### `basket.clone(b)`

Creates a distinct shallow copy of the outer basket.

### `basket.freeze(b)`

Marks the basket read-only. Later mutation attempts produce runtime errors.

### `basket.sort(b)`

Sorts numeric or string elements in ascending order.

Custom comparator functions are not currently supported.

---

# 3. `string`

The `string` namespace provides Unicode-aware text operations.

### `string.len(s)`

Returns the number of Unicode scalar values in `s`.

~~~aeoscript
string.len("é")
~~~

returns `1`.

### `string.lower(s)`

Returns a lowercase string.

### `string.upper(s)`

Returns an uppercase string.

### `string.reverse(s)`

Reverses Unicode scalar values.

### `string.split(s, [separator])`

Returns a basket of strings.

If a separator is supplied, the string is split using that separator.

If the separator is omitted or empty, the string is split into individual Unicode scalar values.

---

# 4. Method Routing

Baskets support method-style calls that route into the basket namespace.

~~~aeoscript
const items = [3, 1, 2]

items.len()
items.sort()
items.insert(0, 10)
items.remove(0)
~~~

The object is supplied to the underlying native namespace operation as the appropriate first argument.

---

# 5. Engine-Integrated Namespaces

AeoScript 0.7.3 expands the standard library with engine-integrated namespaces:

* `get`: `get.mouse()`
* `input`: `get_move_vector()`, `is_jump_pressed()`, `get_orbit_delta()`
* `camera`: `get_horizontal_basis()`, `set_position(...)`, `set_target(...)`, `set_orientation(...)`
* `physics`: `resolve_camera_collision(...)`
* `player`: `set_horizontal_velocity(...)`, `set_facing_direction(...)`, `select_animation(...)`, `is_grounded()`, `apply_vertical_impulse(...)`, `position`
* `ui`: `new(...)`, `delete(...)`, `get_viewport_size()`
* `cell`: `new(...)`, `delete(...)`, `get(...)`, `exists(...)`
* `entity`: `get(...)`, `exists(...)`
* `script`: `enable(...)`, `disable(...)`, `is_enabled(...)`
* `event`: `fire(...)`
* `test`: `complete(...)`, `is_completed(...)`, `passed(...)`, `summary()`

---

# 6. Standard Library Design

The library focuses on deterministic, engine-owned gameplay interactions.

Features planned for future passes include:

* Custom comparator functions for basket sorting.
* Advanced pattern-based string matching and regex.
* Arbitrary runtime mesh instantiation beyond currently supported `cell.new()` and `ui.new()` elements.
* Expanded spatial query APIs beyond `physics.resolve_camera_collision()`.


