# AeoScript Grammar

This document describes the grammar for AeoScript.

It is a working document while the language is being built.

The parser is the final authority.

If this document and the parser disagree, fix whichever one is wrong. Do not let them drift apart.

The goal is a small, predictable language that is easy to use and easy to extend without adding special syntax for every new engine feature.

---

# 1. Source

A source file contains zero or more declarations.

```text
source
    = declaration*
```

---

# 2. Declarations

Current declaration types:

```text
declaration
    = entity_declaration
    | function_declaration
    | const_declaration
    | event_declaration
```

Imports and modules are planned for later.

---

# 3. Entity Declaration

```text
entity_declaration
    = "entity" identifier "{" entity_member* "}"
```

Example:

```aeoscript
entity Door {

    open: bool = false

    fn update(dt: number) {
    }
}
```

---

# 4. Entity Members

An entity can contain fields and functions.

```text
entity_member
    = field_declaration
    | function_declaration
```

---

# 5. Fields

A field can have an explicit type or use type inference.

```text
field_declaration
    = identifier type_annotation? "=" expression
```

Example:

```aeoscript
health: number = 100
```

Type inference:

```aeoscript
health = 100
```

---

# 6. Constants

```text
const_declaration
    = "const"
      identifier
      type_annotation?
      "="
      expression
```

Example:

```aeoscript
const MAX_HEALTH = 100
```

Constants are immutable after initialization.

---

# 7. Functions

```text
function_declaration
    = "fn"
      identifier
      "(" parameter_list? ")"
      return_type?
      block
```

Example:

```aeoscript
fn heal(amount: number) {
    health += amount
}
```

With a return type:

```aeoscript
fn is_alive(): bool {
    return health > 0
}
```

---

# 8. Parameters

```text
parameter_list
    = parameter ("," parameter)*

parameter
    = identifier type_annotation?
```

Example:

```aeoscript
fn damage(target: Entity, amount: number) {
}
```

---

# 9. Events

Events use:

```text
event_declaration
    = "on"
      identifier
      "(" parameter_list? ")"
      block
```

Example:

```aeoscript
on PlayerSpawned(player) {
    print("Player spawned")
}
```

Event names are identifiers.

The grammar does not contain special syntax for individual events.

`PlayerSpawned` is simply one event name.

Future events can use the same syntax:

```aeoscript
on DoorOpened(door) {
}

on EntityDestroyed(entity) {
}

on ButtonPressed(button) {
}
```

---

# 10. Types

Initial built-in types:

```text
type
    = "number"
    | "bool"
    | "string"
    | "Entity"
    | "Cell"
    | "Basket"
    | "vec2"
    | "vec3"
    | "vec4"
```

Optional types:

```text
optional_type
    = type "?"
```

Example:

```aeoscript
target: Entity?
```

A value of an optional type may contain `nil`.

Generic type syntax such as:

```text
Basket<Entity>
```

is not part of the language yet.

---

# 11. Nil

The language uses:

```aeoscript
nil
```

There is no `null` literal.

Grammar:

```text
nil
    = "nil"
```

Example:

```aeoscript
target = nil
```

`nil` represents the absence of a value.

---

# 12. Blocks

```text
block
    = "{" statement* "}"
```

Example:

```aeoscript
fn update(dt: number) {

    health -= dt

    if health <= 0 {
        die()
    }
}
```

---

# 13. Statements

Current statement forms:

```text
statement
    = variable_or_assignment
    | const_declaration
    | if_statement
    | while_statement
    | for_statement
    | return_statement
    | expression_statement
```

---

# 14. Variables and Assignment

A variable may be initialized with or without an explicit type.

```aeoscript
health = 100
```

```aeoscript
health: number = 100
```

The same surface form is used when assigning to an existing variable:

```aeoscript
health = 50
```

Compound assignment is also supported:

```aeoscript
health += 10
health -= 10
health *= 2
health /= 2
```

The assignment target may be a variable or a writable property.

Examples:

```aeoscript
health = 100
transform.position = new_position
entity.name = "Guard"
```

---

# 15. Assignment Operators

```text
assignment_operator
    = "="
    | "+="
    | "-="
    | "*="
    | "/="
```

---

# 16. If

```text
if_statement
    = "if"
      expression
      block
      ("else" "if" expression block)*
      ("else" block)?
```

Example:

```aeoscript
if health <= 0 {
    die()
} else if health < 25 {
    retreat()
} else {
    attack()
}
```

---

# 17. While

```text
while_statement
    = "while"
      expression
      block
```

Example:

```aeoscript
while timer > 0 {
    timer -= 1
}
```

The runtime may impose execution limits.

---

# 18. For

The initial `for` form iterates over a value.

```text
for_statement
    = "for"
      identifier
      "in"
      expression
      block
```

Example:

```aeoscript
for enemy in enemies {
    enemy:highlight()
}
```

Baskets are the primary collection used for this kind of iteration.

Numeric/range iteration can be added later if needed.

---

# 19. Return

Without a value:

```aeoscript
return
```

With a value:

```aeoscript
return health > 0
```

Grammar:

```text
return_statement
    = "return" expression?
```

---

# 20. Expressions

Expressions include:

```text
literals
identifiers
function calls
property access
method calls
index access
unary operators
binary operators
parenthesized expressions
```

The object model uses the same expression system for Cells, Entities, Baskets, and other supported values.

---

# 21. Primary Expressions

```text
primary
    = number_literal
    | string_literal
    | boolean_literal
    | nil
    | identifier
    | "(" expression ")"
```

---

# 22. Numbers

Examples:

```text
100
3.14
0.5
```

Negative numbers are handled through the unary `-` operator rather than being a separate literal.

Example:

```aeoscript
speed = -5
```

---

# 23. Strings

```aeoscript
"hello"
"door opened"
```

---

# 24. Booleans

```aeoscript
true
false
```

---

# 25. Identifiers

```text
identifier
    = letter (letter | digit | "_")*
```

Identifiers cannot begin with a digit.

Examples:

```text
health
player
move_speed
target2
```

Keywords cannot be used as identifiers.

---

# 26. Postfix Expressions

AeoScript uses one common postfix expression model for calls, properties, methods, and indexing.

```text
postfix_expression
    = primary postfix*
```

Possible postfix forms:

```text
postfix
    = "." identifier
    | ":" identifier "(" argument_list? ")"
    | "(" argument_list? ")"
    | "[" expression "]"
```

This allows chained expressions such as:

```aeoscript
player.position
```

```aeoscript
player:destroy()
```

```aeoscript
player:get_children()
```

```aeoscript
children[0]
```

```aeoscript
world:find("Door")[0]
```

The same model is used for engine objects and normal AeoScript values where supported.

---

# 27. Property Access

Properties use dot syntax.

```aeoscript
transform.position
player.name
light.enabled
```

Grammar:

```text
property_access
    = expression "." identifier
```

Properties are resolved by the object being accessed.

The language does not create separate syntax for different engine object types.

---

# 28. Method Calls

Methods use colon syntax.

```aeoscript
object:method()
```

With arguments:

```aeoscript
object:method(argument)
```

Examples:

```aeoscript
character:jump()
entity:destroy()
light:set_enabled(false)
```

Grammar:

```text
method_call
    = expression ":" identifier "(" argument_list? ")"
```

The colon form is the standard AeoScript method syntax.

---

# 29. Function Calls

Normal function calls use parentheses.

```aeoscript
print("Hello")
```

```aeoscript
move(direction, speed)
```

Grammar:

```text
call
    = expression "(" argument_list? ")"

argument_list
    = expression ("," expression)*
```

Methods and normal functions are separate syntactic forms.

---

# 30. Index Access

Baskets use 0-based indexing.

Example:

```aeoscript
items[0]
items[1]
```

Grammar:

```text
index_access
    = expression "[" expression "]"
```

The index expression may be any valid expression that resolves to an appropriate index value.

---

# 31. Baskets

Baskets are AeoScript's collection type.

Baskets are 0-based.

Example:

```aeoscript
children = object:get_children()

first = children[0]
count = children.len()
```

Baskets may contain supported AeoScript values including:

```text
Cells
Entities
Baskets
numbers
strings
booleans
nil
```

A Basket is not a separate scripting syntax. It is a runtime value.

---

# 32. Cells

A `Cell` represents authored world content.

Examples:

```text
Block
Light
SpawnPoint
future authored objects
```

Cells are safe script-facing references to authored content.

They are not live runtime Entities.

A Cell may be used with the normal object model:

```aeoscript
cell.name
cell.cellType
```

and:

```aeoscript
cell:getObject()
```

The latter may return an Entity or `nil`.

---

# 33. Entities

An `Entity` represents a live runtime object.

Example:

```aeoscript
on PlayerSpawned(player) {
    player.name
    player.position
}
```

Entities may expose properties and methods through the normal object model.

Entity references may also be stored in Baskets and passed to functions or event handlers.

---

# 34. Discovery

Discovery uses normal function calls and object methods.

Examples:

```aeoscript
lights = getAllCellsOfClass("Light")
```

```aeoscript
objects = find("Door")
```

```aeoscript
children = object:get_children()
```

```aeoscript
parent = object:get_parent()
```

Discovery does not introduce object-specific grammar.

New engine object types should use the existing object model.

---

# 35. Cell to Entity

A Cell may provide a live runtime object through:

```aeoscript
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

---

# 36. Unary Operators

Initial unary operators:

```text
!
-
+
```

Examples:

```aeoscript
!alive
-health
+speed
```

---

# 37. Arithmetic Operators

```text
+
-
*
/
%
```

Example:

```aeoscript
damage = base_damage * multiplier
```

---

# 38. Comparison Operators

```text
<
>
<=
>=
```

Example:

```aeoscript
if health <= 0 {
    die()
}
```

---

# 39. Equality Operators

```text
==
!=
```

Example:

```aeoscript
if target == nil {
    return
}
```

---

# 40. Boolean Operators

```text
&&
||
```

Example:

```aeoscript
if alive && health > 0 {
    attack()
}
```

---

# 41. Operator Precedence

Initial intended order, highest first:

```text
1. function calls / method calls / property access / index access
2. unary operators
3. multiplication / division / modulo
4. addition / subtraction
5. comparisons
6. equality
7. &&
8. ||
```

Parentheses override normal precedence.

---

# 42. Parentheses

Parentheses explicitly group expressions.

```aeoscript
damage = (base + bonus) * multiplier
```

```aeoscript
if (health > 0 && alive) {
    attack()
}
```

---

# 43. Comments

Single-line comments:

```aeoscript
// comment
```

Multi-line comments:

```aeoscript
/*
    comment
*/
```

Comments do not become part of the AST.

---

# 44. Whitespace

Whitespace is not intended to change the meaning of an expression.

These should mean the same thing:

```aeoscript
health = 100
```

and:

```aeoscript
health     =     100
```

Indentation is for readability.

---

# 45. Semicolons

Semicolons are not required by normal AeoScript style.

Preferred:

```aeoscript
health = 100
```

not:

```aeoscript
health = 100;
```

Whether the parser accepts semicolons as compatibility syntax is an implementation detail and should not be relied upon.

---

# 46. Event Example

A complete event example:

```aeoscript
on PlayerSpawned(player) {

    print("Player spawned")

    print(player.name)
}
```

The event handler receives the live `Entity` reference supplied by the engine.

---

# 47. Attached and Unattached Scripts

AeoScript supports both attached and unattached scripts.

An attached script can work with its parent object through the normal object model.

An unattached script can act as a game/server-style script by discovering objects and responding to events.

The grammar does not need separate syntax for these two cases.

---

# 48. Language Growth

Adding a new engine object should normally not require a new language construct.

For example, adding:

```text
Door
NPC
Vehicle
Trigger
Button
Spawner
Camera
```

should normally mean exposing that object through the existing:

```text
Cell
Entity
property
method
discovery
event
```

model.

The lexer and parser should remain stable.

---

# 49. Formal Grammar Status

This grammar is a working specification.

It is intentionally smaller than a full academic grammar.

The important rules are:

```text
small language
generic object model
predictable syntax
no object-specific language features
```

The parser is the implementation authority.

The documentation should be updated whenever the implemented language changes.

---

# 50. Next Grammar Work

Before adding advanced language features, the core syntax should remain stable around:

```text
entities
fields
functions
constants
variables
optional values
nil
Cells
Entities
Baskets
property access
method calls
index access
events
control flow
expressions
```

Later features can include:

```text
imports
modules
closures
maps
advanced types
```

These should only be added when they solve an actual problem in AeoEngine.

Do not add language features just because another language has them.
