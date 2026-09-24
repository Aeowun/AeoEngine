# AeoScript Grammar

---

# 1. Source Structure

An AeoScript source file is a sequence of top-level declarations.

```text
program:
    declaration*

declaration:
      entity_declaration
    | function_declaration
    | import_declaration
    | event_declaration
```

### Entity Declaration

```text
entity_declaration:
    "entity" identifier "{" entity_member* "}"
```

An entity contains authored fields and functions.

```aeoscript
entity Door {
    open: bool = false

    fn update(dt: number) {
        ...
    }
}
```

### Entity Members

```text
entity_member:
      field_declaration
    | function_declaration
```

### Field Declaration

```text
field_declaration:
    identifier type_annotation? initializer?
```

A field must provide either a type annotation, an initializer, or both.

Examples:

```aeoscript
health: number = 100
name: string
enabled = true
```

### Function Declaration

Functions may exist at the top level or inside an entity.

```text
function_declaration:
    "fn" identifier "(" parameter_list? ")" return_type? block
```

Examples:

```aeoscript
fn double(value: number): number {
    return value * 2
}
```

and:

```aeoscript
entity Player {
    fn move_to_origin() {
        ...
    }
}
```

### Import Declaration

```text
import_declaration:
    "import" identifier ("." identifier)*
```

Example:

```aeoscript
import game
import game.physics
```

### Event Declaration

```text
event_declaration:
    "on" identifier "(" parameter_list? ")" block
```

Example:

```aeoscript
on PlayerSpawned(player: Entity) {
    ...
}
```

---

# 2. Parameters and Return Types

### Parameter List

```text
parameter_list:
    parameter ("," parameter)*

parameter:
    identifier type_annotation?
```

Example:

```aeoscript
fn move(speed: number, grounded: bool) {
    ...
}
```

### Return Type

```text
return_type:
    ":" type
```

Example:

```aeoscript
fn add(a: number, b: number): number {
    return a + b
}
```

Functions may omit a return type.

---

# 3. Statements

```text
statement:
      variable_declaration
    | assignment_statement
    | if_statement
    | while_statement
    | for_statement
    | return_statement
    | expression_statement
```

### Variable Declaration

```text
variable_declaration:
      "const" identifier type_annotation? initializer
    | identifier type_annotation? initializer?
```

A variable must provide either a type annotation or an initializer.

Examples:

```aeoscript
const limit = 10
value: number = 5
name = "Ghost"
```

### Assignment

```text
assignment_statement:
    expression assignment_operator expression
```

Supported assignment operators:

```text
=
+=
-=
*=
/=
```

Examples:

```aeoscript
health = 100
health += 5
velocity *= 0.5
```

Assignment targets may be variables, members, or indexed collection values where supported by the runtime.

Examples:

```aeoscript
health = 100
cell.visible = false
items[0] = "Sword"
values["name"] = "Door"
```

### `if`

```text
if_statement:
    "if" expression block
    ("else" "if" expression block)*
    ("else" block)?
```

Example:

```aeoscript
if health <= 0 {
    dead = true
} else if health < 25 {
    warning = true
} else {
    warning = false
}
```

### `while`

```text
while_statement:
    "while" expression block
```

Example:

```aeoscript
while active {
    update_logic()
}
```

### `for`

```text
for_statement:
    "for" identifier "in" expression block
```

Example:

```aeoscript
for block in blocks {
    block.visible = true
}
```

The iterable expression is evaluated by the runtime. Baskets are the primary sequence collection used by the current language.

### `return`

```text
return_statement:
    "return" expression?
```

Examples:

```aeoscript
return
```

```aeoscript
return health
```

### Expression Statement

```text
expression_statement:
    expression
```

Expressions may be used as standalone statements for function calls, method calls, and other side effects.

---

# 4. Expressions

```text
expression:
    logical_or_expression
```

### Primary Expressions

```text
primary:
      number_literal
    | string_literal
    | boolean_literal
    | "nil"
    | identifier
    | basket_literal
    | map_literal
    | anonymous_function
    | "(" expression ")"

anonymous_function:
    "fn" "(" parameter_list? ")" return_type? block
```
```

### Basket Literals

```text
basket_literal:
    "[" (expression ("," expression)*)? "]"
```

Examples:

```aeoscript
[]
```

```aeoscript
[1, 2, 3]
```

```aeoscript
["red", "green", "blue"]
```

Baskets are zero-indexed collections.

### Map Literals

```text
map_literal:
    "{"
    (map_entry ("," map_entry)*)?
    "}"

map_entry:
    map_key ":" expression

map_key:
      string_literal
    | number_literal
    | identifier
```

Examples:

```aeoscript
{}
```

```aeoscript
{
    name: "Ghost",
    health: 100
}
```

```aeoscript
{
    1: "numeric key",
    "1": "string key"
}
```

Identifier map keys are treated as string keys.

---

# 5. Postfix Expressions

Postfix operations may be chained.

```text
postfix_expression:
    primary postfix*
```

Supported postfix operations:

```text
postfix:
      call
    | member_access
    | method_call
    | index
```

### Function Call

```text
call:
    "(" argument_list? ")"
```

Example:

```aeoscript
move_player(10)
```

### Member Access

```text
member_access:
    "." identifier
```

Example:

```aeoscript
cell.visible
```

### Method Call

```text
method_call:
    ":" identifier "(" argument_list? ")"
```

Example:

```aeoscript
entity:set_position(0, 1, 0)
```

### Indexing

```text
index:
    "[" expression "]"
```

Examples:

```aeoscript
items[0]
values["name"]
values[id]
```

### Chaining

Postfix operations can be combined:

```aeoscript
items[0].name
```

```aeoscript
blocks.find(target)
```

```aeoscript
objects[0]:translate(1, 0, 0)
```

---

# 6. Arguments

```text
argument_list:
    expression ("," expression)*
```

Examples:

```aeoscript
math.max(a, b)
```

```aeoscript
basket.insert(items, 0, value)
```

---

# 7. Unary Operators

```text
unary_expression:
      "!" unary_expression
    | "-" unary_expression
    | postfix_expression
```

### Logical NOT

```aeoscript
!enabled
```

### Numeric Negation

```aeoscript
-speed
```

---

# 8. Binary Operators

### Multiplicative

```text
*
/
%
```

### Additive

```text
+
-
```

`+` supports numeric addition and string concatenation where the runtime types are compatible.

### Relational

```text
<
>
<=
>=
```

### Equality

```text
==
!=
```

### Logical

```text
&&
||
```

---

# 9. Operator Precedence

Operators are evaluated from highest precedence to lowest precedence:

```text
1. Call, Member, Method, Index
2. Unary (!, -)
3. Multiplicative (*, /, %)
4. Additive (+, -)
5. Relational (<, >, <=, >=)
6. Equality (==, !=)
7. Logical AND (&&)
8. Logical OR (||)
```

Example:

```aeoscript
const result = a + b * c
```

is evaluated as:

```text
a + (b * c)
```

Parentheses can be used to explicitly control grouping:

```aeoscript
const result = (a + b) * c
```

---

# 10. Types

Type annotations use a colon:

```text
type_annotation:
    ":" type
```

The parser accepts named types and optional forms:

```text
type:
    identifier
    | identifier "?"
```

Examples:

```aeoscript
health: number
name: string
target: Entity
cell: Cell
value: number?
```

The language's current runtime vocabulary includes:

```text
number
bool
string
nil
basket
map
Cell
Entity
```

Engine and runtime types may expand as AeoScript gains additional capabilities.

---

# 11. Optional Types

A type may be marked optional using `?`.

```aeoscript
target: Entity?
```

This expresses that the value may be absent (`nil`) as well as containing the specified type.

---

# 12. Literals

### Number

Numbers are represented as 64-bit floating-point values.

Examples:

```aeoscript
0
42
3.14
-10
```

### String

Strings are UTF-8 Unicode text.

Examples:

```aeoscript
"Hello"
"Ghost"
"é"
```

### Boolean

```aeoscript
true
false
```

### Nil

```aeoscript
nil
```

### Basket

```aeoscript
[1, 2, 3]
```

### Map

```aeoscript
{
    name: "Ghost",
    health: 100
}
```

---

# 13. Line Termination

AeoScript uses newlines to terminate statements.

Example:

```aeoscript
health = 100
visible = true
```

Statements inside blocks are separated by line termination and block boundaries.

The current grammar does not require semicolons.

---

# 14. Comments

Comments are ignored by the parser and may be placed alongside source statements where supported by the lexer.

Example:

```aeoscript
// Update the player's health
health -= damage
```

---

# 15. Scope and Execution

Variables declared inside functions belong to the current function execution.

Entity fields belong to the persistent script instance.

AeoScript fibers preserve active execution state across `wait()` operations, including:

* Function call stack
* Local variables
* Loop state
* Instruction position
* Pending wait state

This allows code such as:

```aeoscript
fn delayed_damage(amount: number) {
    const delay = 0.5

    wait(delay)

    health -= amount
}
```

to resume inside the same function with its local state intact.

---

# 16. Current Grammar Boundaries

The grammar describes syntax accepted by the parser. Runtime availability of a type, namespace member, property, or method is determined separately by the interpreter and engine API.

In particular:

* Syntax for a named type does not automatically create a runtime type.
* A parsed function call must still resolve to a valid runtime function.
* A parsed member access must still resolve to a valid runtime property.
* A parsed method call must still resolve to a valid runtime method.
* Collection indexing must obey the runtime's basket and map semantics.

The language therefore has a distinction between:

```text
Syntax
  ↓
Parser
  ↓
AST
  ↓
Runtime resolution
```

A construct being grammatically valid does not by itself guarantee that the referenced runtime API exists.
