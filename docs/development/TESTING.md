# Testing AeoEngine

AeoEngine uses multiple levels of testing because an interactive engine contains systems that can be verified independently as well as behavior that only appears when those systems operate together.

Testing is intended to verify:

* Individual engine systems.
* System-to-system integration.
* Persistent data behavior.
* Runtime lifecycle behavior.
* Rendering and performance behavior.
* Editor and Play-mode workflows.

No single test layer is sufficient for the entire engine.

---

# 1. Unit Tests

Unit tests verify focused behavior within an individual subsystem.

Examples include:

* World operations.
* Persistence serialization and loading.
* Physics calculations.
* Character behavior.
* Parser behavior.
* Lexer behavior.
* Interpreter behavior.
* Script scheduler behavior.
* Runtime state transitions.

Many unit tests live directly beside the implementation under `#[cfg(test)]`.

Run the complete Rust test suite with:

```text
cargo test
```

Unit tests should remain focused on deterministic behavior that can be verified without requiring the complete running application.

---

# 2. Focused Tests

During development, run the smallest relevant test group first.

Examples:

```text
cargo test physics
cargo test scripting
cargo test scene
cargo test persistence
```

The exact filter depends on the current test names.

Focused tests reduce iteration time while investigating a specific failure.

They are especially useful when several engine changes are being developed in parallel because each change can be verified against its affected subsystem before the complete suite is run.

Focused tests are an optimization for development speed, not a replacement for the full test suite.

---

# 3. Integration Tests

Integration tests verify behavior that crosses subsystem boundaries.

AeoScript provides an important integration layer because script behavior depends on the language runtime, engine host, World state, runtime state, and other engine systems operating together.

Integration coverage can include:

* Language semantics.
* Maps and baskets.
* Standard library behavior.
* Unicode strings.
* World object discovery.
* Persistent Cell IDs.
* Runtime Cell overrides.
* Script lifecycle behavior.
* Fiber scheduling.
* `wait()` and resume behavior.
* Persistent script fields.
* Engine object handles.
* Runtime callbacks.
* World interaction.

Integration tests are valuable because a subsystem can pass its own unit tests while still failing when connected to another subsystem.

---

# 4. In-Game AeoScript Integration Suite

AeoEngine contains an in-game AeoScript integration test suite.

The suite runs through the actual engine runtime and verifies language and engine integration together.

The master test script covers areas such as:

* Core language behavior.
* Control flow.
* Collections.
* Strings.
* Functions.
* Closures.
* Waiting and resuming.
* Cells.
* Attributes.
* Entities.
* Lights.
* World behavior.
* Handles.
* Callbacks.
* Math.

The important distinction is:

```text
Rust unit tests
      ↓
Test individual engine components

AeoScript integration tests
      ↓
Test the language interacting with the engine
```

The integration suite is not intended to replace ordinary Rust tests.

---

# 5. Persistence Tests

Persistent World data requires both serialization and compatibility testing.

Persistence tests should verify:

* Authored data is written correctly.
* Saved data can be loaded correctly.
* Multiple values survive save/load.
* Empty optional data does not create unnecessary records.
* Existing World files without newer records continue to load when backward compatibility is required.
* Malformed optional records are handled according to the persistence system's loading rules.

A persistence test should normally exercise the complete path:

```text
World state
    ↓
Save
    ↓
world.dat
    ↓
Load
    ↓
World state
```

Do not test only the serializer in isolation when the behavior being changed depends on the complete save/load path.

---

# 6. Runtime State Boundary Tests

Many AeoEngine bugs involve confusion between authored state and temporary runtime state.

Tests should distinguish:

```text
Authored World state
Runtime state
Effective state
```

For example:

```text
Authored Cell
      ↓
Runtime override
      ↓
Effective runtime value
```

A runtime change should affect the effective value used by gameplay or rendering without silently replacing the authored value.

Tests for runtime properties should therefore verify both sides of the boundary:

```text
Before Play
    authored value

During Play
    runtime/effective value changes

After Play
    authored value remains unchanged
```

This distinction is one of the most important regression areas in the engine.

---

# 7. Script Lifecycle and Fiber Tests

AeoScript lifecycle tests should verify the difference between persistent script state and temporary execution state.

Important cases include:

* Field changes surviving update tasks.
* Local variables surviving `wait()`.
* Loop state surviving `wait()`.
* Nested function calls surviving `wait()`.
* Yielded tasks resuming correctly.
* Waiting tasks not being duplicated every frame.
* Script instances shutting down correctly.
* Runtime script state being cleared when Play mode ends.

The important execution relationship is:

```text
Script
  ↓
Fiber
  ↓
Scheduler
  ↓
Resume
```

Tests should verify behavior across suspension and resumption rather than only testing the uninterrupted execution path.

---

# 8. Physics and Character Tests

Physics and character systems should be tested at both focused and integration levels.

Examples include:

* Gravity.
* Position integration.
* Static collision.
* Dynamic collision.
* Grounding.
* Jumping.
* Character movement.
* Spawn-point selection.
* Character clearance.
* Runtime physics synchronization.

Where possible, deterministic calculations should be covered by automated tests.

Behavior that depends on the complete engine runtime should additionally receive integration or manual verification.

---

# 9. Renderer Tests and Benchmarks

Rendering requires both correctness verification and performance verification.

Renderer tests should cover the behavior of the render representation where practical.

Performance benchmarks should be used when changes affect:

* Chunk construction.
* Mesh generation.
* Render invalidation.
* Selective rebuilding.
* GPU resource management.
* Visibility/culling.
* Other performance-sensitive rendering paths.

The renderer benchmark should be treated as regression coverage for large World workloads.

A successful benchmark currently reports:

```text
test_benchmark_large_world_performance_cliff ... ok
1 passed, 0 failed
```

Performance results and benchmark workloads are documented in:

```text
docs/architecture/RENDERING.md
```

Performance verification should use representative workloads rather than relying only on a small test World.

---

# 10. Manual Runtime Verification

Some behavior cannot be completely verified through automated tests.

Manual verification is important for:

* Editor interaction.
* Viewport input.
* Selection.
* Build and Erase tools.
* Camera behavior.
* Play-mode transitions.
* Rendering.
* Lighting.
* Character movement.
* Runtime UI.
* Scripted gameplay.
* End-to-end project loading and saving.

A typical manual verification is:

```text
Open AeoEngine
      ↓
Load or create test World
      ↓
Reproduce the behavior
      ↓
Observe the relevant state
      ↓
Enter Play
      ↓
Exercise the runtime behavior
      ↓
Exit Play
      ↓
Save/reload when relevant
      ↓
Verify the expected result
```

Manual verification is particularly important for changes involving graphics, input, editor interaction, or multiple runtime systems.

---

# 11. Regression Tests

When a real bug is fixed, add focused regression coverage when practical.

A useful regression test should:

* Represent the behavior that actually failed.
* Fail before the fix when possible.
* Pass after the fix.
* Test the smallest meaningful behavior.
* Remain useful if the internal implementation changes later.

For example:

```text
Bug:
Authored Cell attributes disappear after save/load.

Regression:
Save/load test verifies the attributes survive.
```

Or:

```text
Bug:
A localized World edit rebuilds unrelated render geometry.

Regression:
Renderer performance test verifies selective rebuild behavior.
```

Regression tests should capture the behavior rather than merely checking the exact implementation used to fix it.

---

# 12. Test the System Boundary

When several systems interact, test the boundary between them.

Examples:

```text
World
  ↓
Persistence
```

```text
World
  ↓
PhysicsWorld
```

```text
World
  ↓
Renderer
```

```text
Script
  ↓
ScriptScene
  ↓
EngineHost
  ↓
World / Runtime systems
```

```text
Editor
  ↓
Authored World
  ↓
Runtime
```

A failure at a boundary may not be reproducible in either subsystem when tested independently.

Boundary tests and integration tests exist to catch these failures.

---

# 13. Testing Failure Reproduction

When investigating a bug, the preferred process is:

```text
Reproduce failure
      ↓
Identify affected subsystem
      ↓
Create focused test when practical
      ↓
Confirm failure
      ↓
Implement fix
      ↓
Run focused test
      ↓
Run full suite
      ↓
Run integration/manual verification
```

The test should be added before or alongside the fix when practical so that the original failure is explicitly captured.

---

# 14. Test Organization

Tests should remain close to the behavior they verify.

Use:

```text
src/<subsystem>/...
```

with `#[cfg(test)]` for focused subsystem tests.

Use:

```text
tests/
```

when repository-level integration coverage is more appropriate.

Use the in-game AeoScript integration suite for behavior that requires the actual AeoScript runtime and engine host.

Use renderer benchmarks for performance-sensitive rendering behavior.

Use manual runtime verification for editor, graphics, interaction, and end-to-end behavior that automated tests cannot fully represent.

---

# 15. Full Verification Before Commit

For a substantial engine change, the final verification should normally include:

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
      ↓
Git diff review
```

Not every change requires every step.

For example:

```text
Persistence change
→ focused persistence tests
→ cargo test
```

```text
AeoScript change
→ focused scripting tests
→ AeoScript integration suite
→ manual Play verification
```

```text
Renderer change
→ cargo test
→ renderer benchmark
→ manual editor / Play verification
```

```text
Editor change
→ focused tests where practical
→ cargo test
→ manual editor verification
```

The verification level should match the systems affected by the change.

---

# 16. Test Quality Principle

Tests should document behavior that matters to the engine rather than merely increasing test count.

A useful test should answer one of these questions:

```text
Does this subsystem produce the correct result?

Does this state transition preserve the correct boundary?

Do these systems integrate correctly?

Does this regression stay fixed?

Does this performance-sensitive path remain within expected behavior?
```

A small test that captures a durable engine rule is more valuable than many tests that only repeat implementation details.

The goal of the test suite is not maximum test volume.

The goal is confidence that the engine's important behavior continues to work as the architecture evolves.