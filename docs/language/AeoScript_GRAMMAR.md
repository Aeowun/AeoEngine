# AeoScript Grammar

This document describes the syntax currently accepted by the AeoScript parser.

Grammar acceptance does not automatically imply that the referenced runtime API exists. Runtime names, properties, methods, and engine handles are resolved separately.

---

# 1. Source Structure

An AeoScript source file contains top-level declarations and executable statements.

```text
program:
    top_level_item*

top_level_item:
      declaration
    | statement
```

Declarations include:

```text
entity
fn
on
import
```

---

# 2. Entity Declarations

```text
entity_declaration:
    "entity" identifier "{" entity_member* "}"
```

Example:

```aeoscript
entity Door {
    open: bool = false

    fn update(dt) {
        ...
    }
}
```

---

# 3. Entity Members

```text
entity_member:
      field_declaration
    | function_declaration
```

---

# 4. Fields

Fields may include a type annotation, initializer, or both.

```text
field_declaration:
    identifier type_annotation? initializer?
```

Examples:

```aeoscript
health: number = 100
name: string
enabled = true
```

---

# 5. Functions

```text
function_declaration:
    "fn" identifier "(" parameter_list? ")" return_type? block
```

Functions may exist at the top level or inside an entity.

---

# 6. Events

```text
event_declaration:
    "on" identifier "(" parameter_list? ")" block
```

Example:

```aeoscript
on PlayerSpawned(player) {
    debug.log(player)
}
```

---

# 7. Imports

The parser supports import declarations.

```text
import_declaration:
    "import" identifier ("." identifier)*
```

Import/runtime availability depends on the current language/runtime implementation.

---

# 8. Parameters

```text
parameter_list:
    parameter ("," parameter)*

parameter:
    identifier type_annotation?
```

Example:

```aeoscript
fn move(speed: number, grounded: bool) {
}
```

---

# 9. Return Types

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

---

# 10. Statements

Supported statement forms include:

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

---

# 11. Variables

```text
variable_declaration:
      "const" identifier type_annotation? initializer
    | identifier type_annotation? initializer?
```

Examples:

```aeoscript
const limit = 10
value: number = 5
name = "Ghost"
```

---

# 12. Assignment

Supported assignment operators include:

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

Assignment targets may include variables, properties, and indexed collection values where supported.

---

# 13. `if`

```text
if_statement:
    "if" expression block
    ("else" "if" expression block)*
    ("else" block)?
```

---

# 14. `while`

```text
while_statement:
    "while" expression block
```

---

# 15. `for`

```text
for_statement:
    "for" identifier "in" expression block
```

Baskets are the primary sequence collection used by the current runtime.

---

# 16. `return`

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

---

# 17. Expressions

```text
expression:
    logical_or_expression
```

Expressions support:

* Literals.
* Variables.
* Function calls.
* Member access.
* Method calls.
* Indexing.
* Unary operators.
* Binary operators.
* Anonymous functions.
* Parenthesized expressions.

---

# 18. Primary Expressions

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
```

---

# 19. Anonymous Functions

```text
anonymous_function:
    "fn" "(" parameter_list? ")" return_type? block
```

Example:

```aeoscript
const callback = fn(value) {
    return value * 2
}
```

Anonymous functions can become closures by capturing surrounding lexical state.

---

# 20. Basket Literals

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

---

# 21. Map Literals

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

Identifier map keys are treated as string keys.

---

# 22. Postfix Expressions

```text
postfix_expression:
    primary postfix*

postfix:
      call
    | member_access
    | method_call
    | index
```

---

# 23. Function Calls

```text
call:
    "(" argument_list? ")"
```

Example:

```aeoscript
move_player(10)
```

---

# 24. Member Access

```text
member_access:
    "." identifier
```

Example:

```aeoscript
cell.visible
```

---

# 25. Method Calls

```text
method_call:
    ":" identifier "(" argument_list? ")"
```

Example:

```aeoscript
items.insert(0, value)
```

Method-style calls may be routed through engine-supported runtime methods.

---

# 26. Indexing

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

---

# 27. Arguments

```text
argument_list:
    expression ("," expression)*
```

---

# 28. Unary Operators

```text
unary_expression:
      "!" unary_expression
    | "-" unary_expression
    | postfix_expression
```

Supported unary operators:

```text
!
-
```

---

# 29. Binary Operators

Multiplicative:

```text
*
/
%
```

Additive:

```text
+
-
```

Relational:

```text
<
>
<=
>=
```

Equality:

```text
==
!=
```

Logical:

```text
&&
||
```

---

# 30. Operator Precedence

From highest to lowest:

```text
1. Call / Member / Method / Index
2. Unary
3. Multiplicative
4. Additive
5. Relational
6. Equality
7. Logical AND
8. Logical OR
```

Example:

```aeoscript
const result = a + b * c
```

is equivalent to:

```text
a + (b * c)
```

---

# 31. Types

```text
type_annotation:
    ":" type

type:
    identifier
    | identifier "?"
```

Examples:

```aeoscript
health: number
target: Entity
value: number?
```

The parser accepts named types as syntax.

The runtime determines whether the referenced type is actually supported.

---

# 32. Literals

Number:

```aeoscript
42
3.14
```

String:

```aeoscript
"Hello"
```

Boolean:

```aeoscript
true
false
```

Nil:

```aeoscript
nil
```

Basket:

```aeoscript
[1, 2, 3]
```

Map:

```aeoscript
{
    name: "Ghost"
}
```

---

# 33. Statement Termination

AeoScript uses newline and block structure for statement termination.

Semicolons are not required by the current syntax.

---

# 34. Comments

Line comments use:

```aeoscript
// comment
```

Comments are ignored by the parser.

---

# 35. Scope and Execution

Function locals belong to the current execution scope.

Entity fields belong to the script instance.

Fiber execution state preserves:

* Call frames.
* Local scopes.
* Loop state.
* Instruction position.
* Wait state.

This is what allows nested functions and loops to continue after `wait()`.

---

# 36. Syntax vs Runtime

A parsed construct still has to resolve at runtime.

The overall path is:

```text
Source
 ↓
Lexer
 ↓
Parser
 ↓
AST
 ↓
Runtime resolution
```

Therefore:

* A syntactically valid property may not be a supported engine property.
* A syntactically valid method may not exist.
* A named type may not be backed by a runtime value.
* A parsed function call may still produce a runtime error.

The grammar defines syntax. The API and runtime define behavior.