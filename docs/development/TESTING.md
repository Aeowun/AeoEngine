# Testing AeoEngine

AeoEngine uses several levels of testing because no single test layer is sufficient for an interactive engine.

---

# 1. Unit Tests

Unit tests verify focused engine behavior.

Examples include:

* Physics calculations.
* Character behavior.
* Parser behavior.
* Interpreter behavior.
* Scheduler behavior.
* Persistence behavior.
* World operations.

Run the complete Rust suite with:

~~~text
cargo test
~~~

---

# 2. Focused Tests

During implementation, run the smallest relevant test group first.

Examples:

~~~text
cargo test physics
cargo test scripting
cargo test scene
~~~

The exact filter can vary with the test names in the current codebase.

Focused tests reduce iteration time while a bug is being investigated. And can allow multiple developers and AI assisted agents to work in parallel

---

# 3. Integration Tests

AeoScript uses integration-style coverage to exercise multiple engine systems together.

The integration harness can validate:

* Language semantics.
* Maps and baskets.
* Standard library behavior.
* Unicode strings.
* World object discovery.
* Cell IDs.
* Runtime Cell overrides.
* Script lifecycle behavior.
* Fiber yielding.
* Persistent script fields.

Integration tests are valuable because a subsystem can pass its own unit tests while still failing when connected to another subsystem.

---

# 4. Manual Runtime Verification

Editor and Play-mode features also require manual verification.

A typical manual test is:

~~~text
Open AeoEngine
 ↓
Load or create test World
 ↓
Reproduce behavior
 ↓
Observe editor/runtime state
 ↓
Enter/exit Play
 ↓
Save/reload where relevant
 ↓
Verify expected result
~~~

Manual testing is the final authority for visual and interaction behavior that automated tests do not cover completely.

---

# 5. Regression Tests

When a real bug is fixed, add a focused regression test when practical.

A useful regression test should:

* Reproduce the old failure.
* Fail before the fix.
* Pass after the fix.
* Cover the smallest meaningful behavior.

The goal is to prevent the same architectural failure from returning later.

---

# 6. Integration Harnesses

A larger integration harness is appropriate when several features must work together.

For example, an AeoScript integration script can exercise:

~~~text
Language
 ↓
Collections
 ↓
Standard library
 ↓
World lookup
 ↓
Runtime properties
 ↓
Lifecycle
 ↓
Fibers
 ↓
wait/resume
~~~

A single integration harness should still report individual feature results so the failing layer is identifiable.

---

# 7. Test the State Boundary

Many AeoEngine bugs involve confusion between authored and runtime state.

Tests should explicitly distinguish:

~~~text
Authored World state
Runtime state
Effective state
~~~

For example, a script changing `cell.visible` during Play should change the effective runtime result without modifying the authored value.

---

# 8. Testing Persistent Script State

AeoScript lifecycle tests should distinguish persistent fields from fiber-local state.

Test cases should cover:

* Field changes surviving update tasks.
* Local variables surviving `wait()`.
* Loop state surviving `wait()`.
* Nested function calls surviving `wait()`.
* Waiting tasks not being duplicated every frame.

---

# 9. Full Verification Before Commit

For substantial runtime changes, the preferred final sequence is:

~~~text
cargo check
 ↓
cargo test
 ↓
Manual engine verification
 ↓
Git diff review
 ↓
Commit
~~~

A green test suite should not be treated as proof that a visual/editor/runtime workflow is correct until the relevant manual behavior has also been exercised.

---

# 10. Test Quality Principle

Tests should document behavior that matters to the engine rather than merely increasing test count.

A small test that captures a durable architectural rule is more valuable than many tests that only repeat implementation details.

