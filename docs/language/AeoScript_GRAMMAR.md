# AeoScript Grammar

This document describes the planned grammar for AeoScript.

It is intentionally incomplete while the language is being built.

The grammar should follow the actual parser as the implementation develops.
If the parser and this document disagree, fix the document or the parser rather
than letting them drift apart.

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
    | import_declaration
```

Imports are planned but may not exist in the first implementation.

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

An entity can currently contain fields and functions.

```text
entity_member
    = field_declaration
    | function_declaration
```

---

# 5. Fields

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
    = "const" identifier type_annotation? "=" expression
```

Example:

```aeoscript
const MAX_HEALTH = 100
```

Whether constants are allowed inside entities is still open.

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

# 9. Types

Initial built-in types:

```text
type
    = "number"
    | "bool"
    | "string"
    | "Entity"
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

---

# 10. Blocks

```text
block
    = "{" statement* "}"
```

---

# 11. Statements

Current statement direction:

```text
statement
    = variable_declaration
    | assignment
    | if_statement
    | while_statement
    | for_statement
    | return_statement
    | expression_statement
```

---

# 12. Variable Declaration

Possible forms:

```aeoscript
health = 100
```

```aeoscript
health: number = 100
```

Grammar:

```text
variable_declaration
    = identifier type_annotation? "=" expression
```

---

# 13. Assignment

Simple assignment:

```aeoscript
health = 100
```

Compound assignment:

```aeoscript
health += 10
health -= 10
health *= 2
health /= 2
```

Planned operators:

```text
=
+=
-=
*=
/=
```

---

# 14. If

```text
if_statement
    = "if" expression block
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

# 15. While

```text
while_statement
    = "while" expression block
```

Example:

```aeoscript
while timer > 0 {
    timer -= 1
}
```

The runtime may impose execution limits.

---

# 16. For

Initial direction:

```text
for_statement
    = "for" identifier "in" expression block
```

Example:

```aeoscript
for enemy in enemies {
    enemy.highlight()
}
```

---

# 17. Return

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

# 18. Expressions

Expressions include:

```text
literals
identifiers
property access
function calls
index access
unary operators
binary operators
parenthesized expressions
```

---

# 19. Literals

Numbers:

```text
100
3.14
-5
```

Strings:

```aeoscript
"hello"
```

Booleans:

```text
true
false
```

Null:

```text
null
```

---

# 20. Identifiers

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

---

# 21. Property Access

Example:

```aeoscript
transform.position
```

Grammar direction:

```text
member_access
    = expression "." identifier
```

Chained access:

```aeoscript
player.character.velocity
```

Whether arbitrary chaining remains legal everywhere is still being worked out.

---

# 22. Function Calls

Example:

```aeoscript
print("Hello")
```

Arguments:

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

---

# 23. Index Access

Example:

```aeoscript
items[0]
```

Grammar direction:

```text
index_access
    = expression "[" expression "]"
```

---

# 24. Arithmetic

Operators:

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

# 25. Comparison

Operators:

```text
==
!=
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

# 26. Boolean Operators

Operators:

```text
&&
||
!
```

Example:

```aeoscript
if alive && health > 0 {
    attack()
}
```

---

# 27. Operator Precedence

Initial intended order, highest first:

```text
1. function calls / member access / index access
2. unary operators
3. multiplication / division / modulo
4. addition / subtraction
5. comparisons
6. equality
7. &&
8. ||
```

This may change once parser implementation begins.

---

# 28. Parentheses

Parentheses explicitly group expressions.

```aeoscript
damage = (base + bonus) * multiplier
```

---

# 29. Comments

Single line:

```aeoscript
// comment
```

Multi-line:

```aeoscript
/*
    comment
*/
```

Comments are discarded before semantic analysis.

---

# 30. Whitespace

Whitespace is not significant.

These should mean the same thing:

```aeoscript
health = 100
```

and:

```aeoscript
health     =     100
```

Indentation is for readability only.

---

# 31. Semicolons

Semicolons are not required.

Preferred:

```aeoscript
health = 100
```

Not:

```aeoscript
health = 100;
```

Whether semicolons are accepted as optional syntax is still open.

---

# 32. Formal Grammar Status

This grammar is a working design.

The parser should become the authority once implementation starts.

The goal is not to produce a giant academic grammar.

The goal is to make the language predictable and easy to implement.

---

# 33. Next Grammar Work

Before adding advanced syntax, settle:

```text
expression precedence
assignment rules
field initialization
function declarations
entity declarations
optional values
collections
```

After that:

```text
imports
modules
closures
maps
advanced types
```

Do not add language features just because another language has them.

