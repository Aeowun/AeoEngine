# Contributing to AeoEngine

AeoEngine development benefits from small, explicit changes and a strong emphasis on preserving architectural boundaries.

---

# 1. Start with the Desired Behavior

Before changing code, define what the engine should do.

A useful change description should state:

* Desired behavior.
* Current behavior.
* Relevant subsystem.
* Important constraints.
* Verification required.

This is especially important for changes that cross Editor, World, runtime, physics, and scripting boundaries.

---

# 2. Preserve Existing Work

AeoEngine development may involve multiple active local changes.

Do not overwrite unrelated work.

Prefer:

~~~text
Inspect
 ↓
Isolate
 ↓
Change only the relevant subsystem
 ↓
Verify
~~~

Avoid destructive Git commands when their effect would include unrelated work.

---

# 3. Architecture Before Implementation

For non-trivial changes, decide which system should own the new behavior before implementing it.

Examples:

* Authored scene data belongs to World.
* Editing behavior belongs to Editor.
* Physical simulation belongs to PhysicsWorld.
* Character gameplay belongs to CharacterSystem.
* Script execution belongs to ScriptScene/interpreter.
* GPU resources belong to Renderer.

Do not duplicate ownership merely because a second system could technically perform the same operation.

---

# 4. Prefer Focused Changes

A feature may touch multiple files when the architecture requires it, but each changed subsystem should have a clear reason for being included.

Avoid opportunistic cleanup mixed into a feature unless it is required for correctness.

---

# 5. Regression Coverage

A real bug fix should usually add regression coverage when practical.

The test should encode the behavior that was previously broken.

For runtime bugs, include both focused Rust tests and manual verification where appropriate.

---

# 6. AeoScript Development

When changing AeoScript, consider the full pipeline:

~~~text
Source
 ↓
Lexer
 ↓
Parser
 ↓
AST / execution plan
 ↓
Interpreter
 ↓
Fiber
 ↓
ScriptScene
 ↓
Engine host
~~~

A syntax feature is not complete until the runtime semantics and relevant diagnostics are defined.

Similarly, a runtime API is not complete simply because an internal Rust function exists; it should be represented correctly in AeoScript and documented.

---

# 7. AI Assistance

AI tools may be used as part of the development workflow, particularly for small or repetitive work that has already been fully defined.

Appropriate examples include:

Writing repetitive tests from an established pattern.
Updating several documentation files to an established format.
Mechanical refactors.
Implementing a clearly specified API across known files.
Generating repetitive boilerplate.

For AI-assisted work, provide explicit implementation direction. Do not only describe the desired outcome. State the expected behavior, implementation approach, constraints, relevant files or subsystems, and required verification.

For larger work, establish the architecture and constraints first. Define the expected files to be changed and the exact implementation direction before allowing the AI tool to proceed.

Always actively monitor the AI tool's changes and reasoning while it works.

Things to watch for include:

Reasoning about work outside the current task.
Incorrect or poorly supported math.
Overly confident assertions with little or no observed evidence.
Introducing architecture or behavior that was not requested.
Expanding the scope of the task without justification.

Stop the AI and correct it when it begins moving in the wrong direction.

For small, trivial tasks, the tool may be allowed to finish, but its actions must still be reviewed and corrected as necessary. Keep track of every file changed during that task.

# 8. Human Review of AI Work

AI-generated changes must still be reviewed like ordinary code, and in most cases should be reviewed more carefully.

Verify:

The implementation matches the intended architecture.
Existing behavior is preserved unless a change was explicitly requested.
Tests actually exercise the changed behavior.
No unrelated files were modified.
No local user edits were overwritten.
Manual behavior is correct where applicable.
The reported result is supported by actual command output, test results, or observed behavior.
The final diff contains only the changes that were intended.

---

# 9. Documentation

A durable architectural rule should be documented.

Use the appropriate documentation layer:

~~~text
User workflow
→ guides/

Language/API behavior
→ language/

Engine architecture
→ architecture/

Development process
→ development/
~~~

Do not allow important architectural knowledge to exist only in implementation comments or chat history.

---

# 10. Commit Discipline

Before committing a substantial change:

~~~text
git status
 ↓
git diff
 ↓
cargo check
 ↓
cargo test
 ↓
Manual verification
 ↓
Review staged diff
 ↓
Commit
~~~

Stage only the files intended for the change.

Generated artifacts, temporary files, screenshots, and unrelated local project state should not be swept into a commit accidentally.

---

# 11. Commit Messages

Commit messages are going to be boring, they usually come after very long sessions. Don't expect a Horah. Check the Changelog. 

---

# 12. Development Principle

The goal is not simply to make the code compile.

The goal is to make the engine easier to understand, safer to change, and more useful for building actual games.

