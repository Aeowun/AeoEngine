# Building AeoEngine

This document describes how AeoEngine is built, tested, and verified during engine development.

The goal is not only to produce a working executable, but to verify that changes remain compatible with the engine's automated tests, integration tests, performance-sensitive systems, and manual runtime behavior.

---

# 1. Platform

AeoEngine currently targets:

* Windows 10.
* Windows 11.

The engine is developed and tested using the Rust stable toolchain.

---

# 2. Repository Root

Run development commands from the AeoEngine repository root.

Example:

```text
C:\Dev\AeoEngine
```

The exact local path may differ. It is dependent on wherever you put the repository.

---

# 3. Rust and Cargo

AeoEngine uses Cargo for building, testing, dependency management, and development verification.

The primary commands are:

```text
cargo check
cargo test
cargo run
```

Additional verification commands may be used during development:

```text
cargo fmt --check
cargo clippy
```

These commands serve different purposes and should not be treated as interchangeable.

---

# 4. Compile Verification

Use:

```text
cargo check
```

`cargo check` verifies that the Rust project compiles without performing the complete final build.

It is the fast first verification step after a source change.

A typical development cycle should use `cargo check` before spending time on the complete test suite or manual runtime verification.

---

# 5. Development Build

Use:

```text
cargo run
```

This builds and launches the engine for manual development testing.

It is used to exercise systems that depend on the actual application, graphics context, project loading, editor interaction, Play mode, runtime simulation, and other behavior that may not be fully represented by automated tests.

---

# 6. Automated Tests

Run the Rust test suite with:

```text
cargo test
```

The test suite should be run after changes that affect engine behavior, persistence, scripting, physics, rendering, or other shared systems.

Automated tests are especially important for changes where regressions may not be immediately visible during manual testing.

Tests should verify behavior at the smallest practical scope before broader integration testing is performed.

---

# 7. AeoScript Integration Tests

AeoEngine contains an in-game AeoScript integration test suite.

The integration suite exercises the scripting system through the running engine and covers areas such as:

* Language features.
* Control flow.
* Arrays and maps.
* Strings.
* Functions.
* Closures.
* Wait behavior.
* Cells.
* Attributes.
* Entities.
* Lights.
* World behavior.
* Handles.
* Callbacks.
* Math.

The integration suite is intended to verify the complete relationship between AeoScript and the engine runtime.

It complements Rust unit and integration tests rather than replacing them.

A change to scripting behavior should therefore be evaluated at the appropriate levels:

```text
Rust tests
    ↓
AeoScript integration tests
    ↓
Manual runtime verification
```

---

# 8. Renderer Performance Benchmarks

The renderer contains dedicated performance benchmarks for large World workloads.

These benchmarks are used to detect rendering performance regressions that ordinary correctness tests may not reveal.

Current benchmark workloads include:

* Large 3D cubes.
* Large 2D planes.
* Large 1D line distributions.

The benchmark distinguishes full render construction from selective rebuild behavior.

A successful benchmark run currently reports:

```text
test_benchmark_large_world_performance_cliff ... ok
1 passed, 0 failed
```

Representative results are documented in:

```text
docs/architecture/RENDERING.md
```

Performance changes should be evaluated against representative workloads rather than a single small test World.

The 1D line workload contains a known pathological case at larger sizes and should not be treated as representative of normal voxel World construction.

---

# 9. Formatting and Static Checks

Before committing substantial Rust changes, formatting and lint checks should also be considered.

Run:

```text
cargo fmt --check
```

to verify Rust formatting.

Run:

```text
cargo clippy
```

to identify common Rust correctness and maintainability issues.

These checks complement compilation and tests.

A clean development verification sequence is:

```text
cargo fmt --check
      ↓
cargo check
      ↓
cargo clippy
      ↓
cargo test
```

Not every small edit requires every command immediately, but substantial changes should receive the full verification pass.

---

# 10. Development Verification Loop

A typical engine development workflow is:

```text
Change
  ↓
cargo check
  ↓
Focused tests
  ↓
cargo test
  ↓
Performance benchmark when relevant
  ↓
cargo run
  ↓
Manual runtime verification
```

The appropriate steps depend on the system being changed.

For example:

```text
Persistence change
  → persistence tests
  → cargo test
```

```text
AeoScript change
  → scripting tests
  → AeoScript integration tests
  → manual runtime verification
```

```text
Renderer change
  → cargo test
  → renderer benchmark
  → manual editor / Play verification
```

```text
Editor interaction change
  → cargo test where applicable
  → cargo run
  → manual editor verification
```

The purpose of the workflow is to match the verification method to the type of system being changed.

---

# 11. Manual Runtime Verification

Some engine behavior cannot be adequately verified through compile checks and isolated tests alone.

Manual runtime verification is important for systems such as:

* Editor interaction.
* Viewport input.
* World building and erasing.
* Selection.
* Camera behavior.
* Play mode transitions.
* Physics behavior.
* Character movement.
* Rendering.
* Lighting.
* Runtime UI.
* Scripted gameplay.

Manual testing should use representative project Worlds rather than relying only on minimal test scenes.

When a change affects both editor and runtime behavior, verify both modes separately.

---

# 12. Third-Party Development and Debugging Tools

AeoEngine development may use external tools when Rust tests and normal runtime inspection are not sufficient.

Examples include:

* Git for source and change tracking.
* RenderDoc for graphics debugging and frame inspection.
* Visual Studio or another native debugger for low-level debugging when required.
* GPU or system profiling tools when investigating performance problems.

These tools are development aids. They do not replace AeoEngine's automated tests.

Detailed debugging procedures are documented separately in:

```text
docs/development/DEBUGGING.md
```

This document only establishes their role in the development workflow.

---

# 13. Testing Before a Commit

Before committing a substantial engine change, verify the affected system and then run the broader project checks.

A typical pre-commit sequence is:

```text
cargo fmt --check
cargo check
cargo clippy
cargo test
```

Then run the relevant runtime verification.

For changes affecting rendering, also run the renderer benchmark.

For changes affecting AeoScript, also run the in-game AeoScript integration suite.

The exact verification set should be appropriate to the systems touched by the change.

---

# 14. Generated and Local Files

Development may produce:

* Build artifacts.
* Generated assets.
* Temporary project state.
* Imported assets.
* Local configuration files.
* Debug output.
* Other development-only files.

Before committing, inspect:

```text
git status
```

Do not assume every untracked file belongs in the change.

Generated artifacts and unrelated local files should remain outside feature commits unless they are intentionally part of the project change.

---

# 15. Inspecting Changes

Use Git to verify what actually changed.

Useful commands include:

```text
git status --short
git --no-pager diff
git --no-pager diff --cached
```

These help verify:

* Which files changed.
* Whether generated files were introduced.
* Whether unrelated edits are present.
* What will actually be committed.

`git --no-pager diff` is useful in environments where Git's default pager makes output appear to stop responding.

---

# 16. Preserving the Working Tree

AeoEngine development may involve multiple related changes at the same time.

Do not use destructive Git commands to remove unrelated work.

Avoid commands such as:

```text
git reset --hard
git restore .
```

unless there is an explicit decision to discard the affected changes.

When unrelated work is present, preserve it and stage only the files belonging to the intended change.

---

# 17. Testing Scope

Tests should remain close to the system they verify.

Examples include:

```text
Cell / World data
        ↓
Unit and persistence tests
```

```text
AeoScript runtime
        ↓
Runtime tests
        ↓
In-game integration tests
```

```text
Renderer
        ↓
Rendering correctness tests
        ↓
Performance benchmarks
```

```text
Editor
        ↓
System tests where practical
        ↓
Manual application verification
```

The purpose of this structure is to catch regressions as early as possible while retaining integration coverage for systems that depend on the complete running engine.

---

# 18. Regression Verification

When fixing a demonstrated bug, the verification should include a test or benchmark that would fail if the same regression returned.

The preferred progression is:

```text
Reproduce problem
      ↓
Identify affected system
      ↓
Add or update automated verification
      ↓
Implement fix
      ↓
Run focused verification
      ↓
Run broader test suite
      ↓
Manually verify affected runtime behavior
```

Performance regressions should similarly have a representative benchmark rather than relying only on subjective frame-rate observations.

---

# 19. Build Verification Principle

A successful build is necessary but not sufficient.

For engine development, verification should combine the evidence appropriate to the change:

```text
Compiles
   +
Automated tests pass
   +
Relevant integration tests pass
   +
Relevant benchmarks pass
   +
Manual runtime behavior works
   =
Verified change
```

Not every change requires every category, but changes should be verified at the highest level appropriate to the systems they affect.

The objective is to leave the engine in a state where the source compiles, automated coverage remains green, performance-sensitive systems remain within expected behavior, and the actual application still works as intended.