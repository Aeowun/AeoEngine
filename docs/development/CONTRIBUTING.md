# Contributing to AeoEngine

AeoEngine development emphasizes clear subsystem ownership, focused changes, regression coverage, and verification against the actual engine.

Contributions should preserve the separation between authored World data, editor state, runtime simulation, and derived rendering state.

---

# 1. Understand the Owning System

Before changing code, identify which subsystem owns the behavior.

The major ownership boundaries are:

```text
World
→ Authored scene data

WorldStorage / Persistence
→ Persistent project data

Editor
→ Authoring interaction and editor state

PhysicsWorld
→ Runtime physics

CharacterSystem
→ Runtime character gameplay

ScriptScene / scripting runtime
→ AeoScript execution

EntityManager
→ Runtime entities

Renderer
→ Graphics representation and GPU resources
```

Do not duplicate ownership simply because a second subsystem could technically perform the same operation.

---

# 2. Keep Changes Focused

A feature may require changes in multiple subsystems.

Each changed subsystem should have a clear architectural reason for being involved.

Avoid unrelated cleanup, formatting changes, renames, and refactors inside a feature change unless they are required for correctness.

Small diffs are easier to test and review.

---

# 3. Protect the Working Tree

Development sessions may contain several unrelated changes.

Do not overwrite unrelated work.

Avoid destructive commands such as:

```text
git reset --hard
git restore .
```

unless the affected changes are intentionally disposable.

Use Git to inspect the current state:

```text
git status --short
git --no-pager diff
```

Stage only the files belonging to the intended change.

---

# 4. Tests

Changes should include appropriate verification.

Use:

```text
cargo check
cargo test
```

and additional verification when relevant:

```text
cargo fmt --check
cargo clippy
```

Subsystem-specific testing may include:

* AeoScript integration tests.
* Persistence tests.
* Renderer benchmarks.
* Manual editor testing.
* Manual Play-mode testing.

See:

```text
docs/development/TESTING.md
docs/development/BUILDING.md
```

---

# 5. Regression Coverage

When fixing a real bug, add regression coverage when practical.

A useful regression test should capture the behavior that previously failed.

Prefer:

```text
Bug
 ↓
Reproduction
 ↓
Regression test
 ↓
Fix
 ↓
Test passes
```

over testing only the internal implementation detail used by the fix.

---

# 6. Architecture Changes

Large architectural changes should establish ownership before implementation.

A useful design sequence is:

```text
Desired behavior
      ↓
Owning subsystem
      ↓
State boundary
      ↓
Data flow
      ↓
Implementation
      ↓
Tests
      ↓
Documentation
```

Architectural documentation should describe the resulting system rather than become a design proposal disconnected from the code.

---

# 7. AeoScript Changes

AeoScript changes may cross several layers.

Typical language flow:

```text
Source
 ↓
Lexer
 ↓
Parser
 ↓
AST
 ↓
Interpreter
 ↓
Fiber / Scheduler
 ↓
ScriptScene
 ↓
EngineHost
 ↓
Engine subsystem
```

A language feature is complete only when its runtime behavior and relevant diagnostics are defined.

An engine API exposed to AeoScript should also be represented consistently in:

* The host API.
* The runtime implementation.
* Tests.
* Language documentation.

---

# 8. Documentation

Documentation should live at the appropriate layer.

```text
Engine architecture
→ docs/architecture/

Development workflow
→ docs/development/

Language and API reference
→ docs/language/

Project-building guidance
→ project documentation outside the architecture/reference layer
```

Document durable architectural rules and public behavior.

Do not preserve stale implementation details merely because they once existed.

---

# 9. AI-Assisted Development

AI tools may be used as development tools.

They are particularly useful for:

* Repetitive implementation.
* Focused test additions.
* Documentation updates.
* Mechanical refactors.
* Clearly bounded changes.

The developer remains responsible for:

* Architecture.
* Scope.
* Correctness.
* Verification.
* Reviewing generated changes.

AI output should be verified through actual source changes, tests, command output, benchmarks, and runtime behavior.

Do not treat an AI summary such as "fixed" or "tests passed" as evidence unless the repository output confirms it.

---

# 10. Manual Review of Generated Changes

Review the actual diff after any substantial automated or AI-assisted change.

Verify:

* Only intended files changed.
* The implementation matches the architecture.
* Existing behavior is preserved unless intentionally changed.
* Tests exercise the changed behavior.
* No unrelated cleanup was introduced.
* No local work was overwritten.
* Documentation matches the current implementation.

---

# 11. Development Verification

For a substantial change:

```text
cargo fmt --check
      ↓
cargo check
      ↓
cargo clippy
      ↓
cargo test
      ↓
Relevant integration tests
      ↓
Relevant benchmarks
      ↓
Manual verification
```

Not every change needs every stage, but verification should match the systems affected.

---

# 12. Commit Discipline

Before committing:

```text
git status
      ↓
git diff
      ↓
Run appropriate tests
      ↓
Review staged diff
      ↓
Commit
```

Generated artifacts, temporary files, screenshots, and unrelated local state should not be included accidentally.

---

# 13. Commit Messages and Changelog

Commit messages are primarily repository history.

The user-facing development history belongs in:

```text
CHANGELOG.md
```

When a change materially affects engine behavior, release status, or public functionality, update the changelog as part of the normal development process.

Commit messages do not need to reproduce the entire session or implementation history.

---

# 14. Contribution Principle

The goal of a contribution is not merely to make the code compile.

A good change should make the engine:

* Correct.
* Testable.
* Understandable.
* Architecturally consistent.
* Easier to maintain.
* Useful for building actual games.

The source tree, tests, documentation, and runtime behavior should all agree about what the engine actually does.