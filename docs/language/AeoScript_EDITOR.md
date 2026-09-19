# AeoScript Editor

AeoEngine will use Monaco for editing `.aeo` files.

The editor is part of the scripting workflow, but it should not define the language.

The lexer, parser, runtime, and engine API define the language.

---

# 1. Basic Workflow

The editor should make the normal scripting workflow simple:

```text
Create / Open Script
      |
      v
Open in Monaco
      |
      v
Edit
      |
      v
Save
      |
      v
Validate / Compile
      |
      v
Run
```

When a script is attached to an object, the editor should also make it easy to open that object's script.

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

Scripts should remain normal `.aeo` project files.

---

# 3. New Script

The editor should be able to create a new `.aeo` file.

When creating a script for an object, the initial template should match the scripting model actually supported by AeoEngine.

For example:

```aeoscript
entity NewEntity {

    fn update(dt: number) {
    }
}
```

The final name should come from the selected object or from a user-entered name.

The editor should not invent special script behavior that the runtime does not support.

---

# 4. Syntax Highlighting

The editor should eventually understand the current AeoScript syntax, including:

```text
entity
fn
if
else
while
for
return
const
on
true
false
nil
```

It should also understand the object model syntax:

```text
.
:
```

Types should include supported types such as:

```text
number
bool
string
Entity
Cell
Basket
vec2
vec3
vec4
```

Strings, numbers, comments, operators, properties, methods, and function calls should also receive appropriate highlighting.

The editor should follow the actual language grammar rather than maintaining its own version of it.

---

# 5. Diagnostics

Compile and parse errors should appear directly in Monaco.

Example:

```text
Door.aeo:12:9

Unknown property:
transform.velcity

Did you mean:
transform.velocity
```

The editor should place a marker on the relevant source location.

Diagnostics should come from the actual AeoScript compiler/parser/runtime where possible.

---

# 6. Autocomplete

Typing:

```aeoscript
transform.
```

should eventually show the properties and methods actually exposed by the AeoEngine API.

For example:

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

Generic object access should work the same way.

For example:

```aeoscript
object.
```

should show the properties available for that object type.

These suggestions should come from the same registered API information used by the runtime.

---

# 7. Hover

Hovering over something such as:

```aeoscript
character.jump()
```

could display:

```text
character.jump()

Requests a jump from the character controller.
```

Hover information should eventually come from the same API metadata used by completion and other language-service features.

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

The formatter should produce predictable output without changing the meaning of the script.

---

# 11. Save and Validation

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
Compile / Prepare
  |
  v
Run
```

Whether compilation happens immediately or when Play begins depends on the final runtime/editor workflow.

The editor should not maintain a separate compiler.

---

# 12. Play Mode

AeoScript is especially important during Play Mode.

The user should be able to quickly:

```text
Play
    |
    v
Select object
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
Reload / Re-run
```

The goal is fast iteration.

---

# 13. Runtime Errors

When a script fails during Play Mode, the editor should show something like:

```text
AeoScript Runtime Error

Enemy.aeo
Line 41

Attempted to access a destroyed entity.
```

Clicking the error should open the script at the correct line when source information is available.

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
signature help
```

The language service should use the same parser and semantic information as the compiler/runtime wherever possible.

The editor should not become a second implementation of the language.

---

# 15. API Metadata

Engine APIs have metadata for editor features.

For example:
* `character.jump` — Requests a jump.
* `cell.color` — Sets the temporary runtime color.

---

# 16. Script Templates

The editor may eventually provide simple templates such as:

```text
Empty
Player
Enemy
Interactable
Trigger
```

These should just generate normal `.aeo` source files.

No important scripting behavior should be hidden inside the editor.

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

Warnings and errors should be clearly distinguished in the editor.

---

# 18. Print Output

AeoEngine provides a **PRINT OUTPUT** terminal for runtime diagnostics.

Features:
* **Selectable**: Standard mouse text selection.
* **Clipboard**: Support for `Ctrl+A` (Select All) and `Ctrl+C` (Copy to clipboard).
* **Severity Colors**: 
    * Normal output: Standard text color.
    * Warnings: Yellow.
    * Errors: Red.
* **Context**: Errors include the originating script path and event/function name.

---

# 19. File Handling

Scripts should remain normal files.

The editor should not hide source code inside engine-specific storage.

A user opening the project in another editor should still be able to read and edit the `.aeo` files.

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
load .aeo file
save .aeo file
AeoScript syntax highlighting
basic diagnostics
```

Everything else can come after that.

The editor should support the language that AeoEngine actually has. It should not get ahead of the language runtime or create its own scripting rules.
