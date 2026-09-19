# AeoScript Grammar

---

# 1. Declarations

A source file is a sequence of declarations.

* `declaration`: `entity_declaration` | `function_declaration` | `const_declaration` | `event_declaration`
* `entity_declaration`: `"entity" identifier "{" entity_member* "}"`
* `entity_member`: `field_declaration` | `function_declaration`
* `event_declaration`: `"on" identifier "(" parameter_list? ")" block`

---

# 2. Statements

* `statement`: `variable_or_assignment` | `const_declaration` | `if_statement` | `while_statement` | `for_statement` | `return_statement` | `expression_statement`
* `if_statement`: `"if" expression block ("else" "if" expression block)* ("else" block)?`
* `while_statement`: `"while" expression block`
* `for_statement`: `"for" identifier "in" expression block`
* `return_statement`: `"return" expression?`

---

# 3. Expressions

* `primary`: `number_literal` | `string_literal` | `boolean_literal` | `"nil"` | `identifier` | `basket_literal` | `"(" expression ")"`
* `basket_literal`: `"[" (expression ("," expression)*)? "]"`
* `postfix_expression`: `primary (postfix)*`
* `postfix`: `"." identifier` (Property) | `":" identifier "(" argument_list? ")"` (Method) | `"(" argument_list? ")"` (Call) | `"[" expression "]"` (Index)

---

# 4. Operators

### Arithmetic and Concatenation

* `+`: Numeric addition or string concatenation.
* `-`, `*`, `/`, `%`: Numeric operations.

### Comparison and Equality

* `==`, `!=`: Equality checks.
* `<`, `>`, `<=`, `>=`: Numeric comparisons.

### Logical

* `&&`: Logical AND.
* `||`: Logical OR.
* `!`: Logical NOT.

---

# 5. Types

Optional types are supported using `?`.

* `type_annotation`: `":" type`
* `type`: `"number"` | `"bool"` | `"string"` | `"Entity"` | `"Cell"` | `"Basket"` | `type "?"`

---

# 6. Precedence Rules

1. Call, Property, Method, Index (`()`, `.`, `:`, `[]`)
2. Unary (`!`, `-`)
3. Multiplicative (`*`, `/`, `%`)
4. Additive (`+`, `-`)
5. Relational (`<`, `>`, `<=`, `>=`)
6. Equality (`==`, `!=`)
7. Logical AND (`&&`)
8. Logical OR (`||`)
