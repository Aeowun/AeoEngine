# AeoScript Types

AeoScript has a small runtime type system designed around gameplay values, collections, and references to AeoEngine objects.

---

# 1. `number`

`number` represents a 64-bit floating-point value.

Examples:

~~~aeoscript
health: number = 100
speed = 4.5
angle = math.deg_to_rad(90)
~~~

Numbers are used for gameplay values, mathematical operations, timers, coordinates represented as components, and other numeric state.

---

# 2. `bool`

`bool` represents either `true` or `false`.

~~~aeoscript
enabled: bool = true
alive = false
~~~

Boolean values are used by conditionals and logical operators.

---

# 3. `string`

`string` represents UTF-8 Unicode text.

~~~aeoscript
name: string = "Ghost"
message = "Hello, world!"
~~~

String operations are Unicode-aware. String length and character splitting operate on Unicode scalar values rather than UTF-8 bytes.

---

# 4. `nil`

`nil` represents the absence of a value.

~~~aeoscript
const missing = nil
~~~

Missing map keys evaluate to `nil`.

Assigning `nil` to a map key removes that key.

~~~aeoscript
values["name"] = nil
~~~

---

# 5. `basket`

A `basket` is a zero-indexed reference-backed sequence.

~~~aeoscript
items: basket = ["Sword", "Shield", "Potion"]
~~~

The first element is index `0`.

Baskets use reference semantics. Assigning a basket to another variable creates an alias to the same underlying collection.

~~~aeoscript
const a = [1, 2, 3]
const b = a

b[0] = 99
~~~

After this operation both `a[0]` and `b[0]` are `99`.

Use `basket.clone()` to create a distinct shallow copy.

Use `basket.freeze()` to make a basket read-only.

---

# 6. `map`

A `map` is a reference-backed key/value collection.

Maps are not positional sequences.

~~~aeoscript
const stats = {}
stats["health"] = 100
stats[1] = "numeric key"
~~~

Numeric and string keys are distinct.

~~~aeoscript
stats[1] = "number"
stats["1"] = "string"
~~~

Reading a missing key returns `nil`.

Assigning `nil` deletes a key.

Maps can contain baskets and other maps, and those referenced values retain reference semantics.

---

# 7. `Cell`

`Cell` is an engine-managed handle representing an authored World Cell.

A Cell handle does not contain the actual World object. It identifies the engine-managed Cell instance.

A Cell has a persistent numeric `id` and a human-readable `name`/identity.

The ID uniquely identifies the instance. The name does not have to be unique.

---

# 8. `Entity`

`Entity` is an engine-managed handle representing a live runtime Entity such as a Player or NPC.

Entity state is runtime state rather than authored Cell state.

---

# 9. Optional Types

Types can be marked optional with `?`.

~~~aeoscript
target: Entity?
~~~

An optional value can contain the specified type or `nil`.

---

# 10. `const`

`const` applies to the variable binding rather than automatically making a referenced object immutable.

~~~aeoscript
const items = [1, 2]
items[0] = 99
~~~

The basket can still be mutated through the binding because the basket itself remains mutable.

Reassigning the `const` variable is not allowed.

~~~aeoscript
items = [3, 4]
~~~

Use `basket.freeze()` when the collection itself must become immutable.

---

# 11. Value vs Reference Semantics

Primitive values are copied as values.

Baskets and maps are reference-backed collections.

Engine handles are lightweight references to engine-managed objects.

The distinction matters when passing values to functions or assigning them to new variables.

---

# 12. Type Design Principle

AeoScript keeps its core value model intentionally small:

~~~text
number
bool
string
nil
basket
map
Cell
Entity
~~~

The engine can add new runtime values as gameplay needs develop without changing the basic authored/runtime state model.

