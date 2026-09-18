# AeoScript Editor

AeoEngine will use Monaco for editing `.aeo` files.

The editor is part of the scripting workflow, but it should not define the language.

The compiler and parser define the language.

---

# 1. Basic Workflow

```text
Select Entity
      |
      v
Add Script
      |
      v
Create .aeo
      |
      v
Open Monaco
      |
      v
Edit
      |
      v
Save
      |
      v
Compile
      |
      v
Run
```

---

# 2. Script List

The editor should have a simple script/file view.

Example:

```text
Scripts

Player.aeo
Door.aeo
Enemy.aeo
Guard.aeo
```

Selecting a file opens it in Monaco.

---

# 3. New Script

Selecting an entity and pressing:

```text
Add Script
```

should create a new `.aeo` file.

The initial template could be:

```aeoscript
entity NewEntity {

    fn update(dt: number) {
    }
}
```

The final name should come from the selected entity or from a user-entered name.

---

# 4. Syntax Highlighting

The editor should eventually understand:

```text
entity
fn
if
else
while
for
return
const
true
false
null
```

Types:

```text
number
bool
string
Entity
vec2
vec3
vec4
```

Strings, numbers, comments, operators, and function calls should also receive appropriate highlighting.

---

# 5. Diagnostics

Compile errors should appear directly in Monaco.

Example:

```text
Door.aeo:12:9

Unknown property:
transform.velcity

Did you mean:
transform.velocity
```

The editor should place a marker on the relevant source location.

---

# 6. Autocomplete

Typing:

```aeoscript
transform.
```

should eventually show:

```text
position
rotation
scale
move
move_forward
rotate
look_at
```

Typing:

```aeoscript
character.
```

could show:

```text
move
jump
look
is_grounded
```

These suggestions should come from actual registered AeoEngine APIs.

---

# 7. Hover

Hovering over:

```aeoscript
character.jump()
```

could display:

```text
character.jump()

Requests a jump from the character controller.
```

Hover information should eventually come from the same API metadata used by the language service.

---

# 8. Go To Definition

For AeoScript functions and eventually modules:

```text
Go to Definition
```

should open the correct source file and line.

---

# 9. Search

The editor should support:

```text
search current script
search all scripts
```

The all-script search should operate on normal project files.

---

# 10. Formatting

AeoScript should eventually have one standard formatter.

Example:

```aeoscript
entity Door {

    open: bool = false

    fn update(dt: number) {

        if open {
            timer += dt
        }
    }
}
```

The formatter should produce predictable output.

---

# 11. Save and Compile

Saving a script should eventually trigger validation.

Possible development flow:

```text
Save
  |
  v
Parse
  |
  v
Check
  |
  +---- error --> editor diagnostic
  |
  v
Compile
  |
  v
Bytecode
```

Whether compilation happens immediately or on Play depends on the final editor workflow.

---

# 12. Play Mode

AeoScript is especially important in Play Mode.

The user should be able to:

```text
Play
    |
    v
Select entity
    |
    v
Open script
    |
    v
Edit
    |
    v
Save
    |
    v
Reload
```

The goal is to make iteration fast.

---

# 13. Runtime Errors

When a script fails during Play Mode, the editor should show something like:

```text
AeoScript Runtime Error

Enemy.aeo
Line 41

Attempted to access a destroyed entity.
```

Clicking the error should open the script at the correct line.

---

# 14. Monaco Language Integration

Monaco should eventually communicate with an AeoScript language service.

Possible features:

```text
completion
hover
diagnostics
definition
symbols
formatting
```

The language service should use the same parser and semantic information as the compiler whenever possible.

---

# 15. API Metadata

Engine APIs should have enough metadata for editor features.

For example:

```text
character.jump

Parameters:
    none

Returns:
    bool

Description:
    Requests a jump.
```

That information can drive:

```text
completion
hover
signature help
documentation
```

---

# 16. Script Templates

The editor may eventually provide templates:

```text
Empty
Player
Enemy
Interactable
Trigger
```

These should be simple files.

No special scripting behavior should be hidden inside the editor.

---

# 17. Errors vs Warnings

Errors should prevent execution.

Warnings may still allow the script to run.

Example warning:

```text
Variable 'speed' is never used.
```

Example error:

```text
Expected number.
Got string.
```

---

# 18. Development Console

AeoEngine may eventually show script output in the editor.

For example:

```aeoscript
debug.log("Hello")
```

could appear in a script/runtime panel.

Normal game output should remain separate from compiler diagnostics.

---

# 19. File Handling

Scripts should remain normal files.

The editor should not hide the source inside engine-specific storage.

If a user opens the project in another editor, the `.aeo` source should still be readable.

---

# 20. Future Debugger

A real debugger is a later feature.

Possible features:

```text
breakpoints
step over
step into
continue
inspect fields
inspect locals
call stack
```

Do not build this before the language and VM are stable.

---

# 21. First Editor Goal

The first useful editor version only needs:

```text
Monaco
syntax highlighting
load .aeo file
save .aeo file
basic diagnostics
```

Everything else can come after that.

