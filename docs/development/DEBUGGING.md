# Development Debugging

AeoEngine debugging should begin with the actual failure and work toward the owning subsystem rather than modifying code based only on the first plausible theory.

The debugging process should establish a reproducible failure, identify the state boundary involved, trace the relevant data or control flow, and verify the resulting fix with tests or runtime behavior.

---

# 1. Reproduce First

Before changing implementation, reproduce the issue consistently when possible.

Record:

* What action was performed.
* What was expected.
* What actually happened.
* Whether the issue occurs in Editor mode or Play mode.
* Whether the issue survives restarting the application.
* Whether the issue occurs consistently or intermittently.

A reproducible failure provides a concrete starting point for investigation.

---

# 2. Identify the State Boundary

AeoEngine contains several distinct categories of state.

```text
Authored state
Runtime state
Editor state
Renderer state
Physics state
Character state
Script state
```

The first debugging question should be:

> Which system owns the incorrect value or behavior?

For example:

* An incorrect saved Cell property may belong to authored World state.
* A temporary gameplay value may belong to runtime state.
* A selection problem may belong to editor state.
* A mesh or GPU problem may belong to renderer state.
* A collision problem may belong to physics state.
* An execution problem may belong to script state.

Do not change a subsystem until its ownership has been established.

---

# 3. Trace Before Patching

When a failure appears to involve a subsystem, trace the actual control and data flow.

For example, a scripting failure may involve:

```text
Source
 ↓
Parser
 ↓
Interpreter
 ↓
ScriptScene
 ↓
Fiber
 ↓
Scheduler
 ↓
Engine host
```

A rendering issue may involve:

```text
Authored World
 ↓
Effective World state
 ↓
Render representation
 ↓
Chunk mesh
 ↓
GPU resource
 ↓
Draw submission
```

A physics issue may involve:

```text
Authored World / Runtime state
 ↓
Physics synchronization
 ↓
PhysicsWorld
 ↓
Collision / integration
 ↓
Character or dynamic body
```

Do not assume the component closest to the visible symptom owns the bug.

---

# 4. Reduce the Reproduction

Reduce a large problem to the smallest scene, World, or script that still demonstrates the failure.

Examples:

```text
Physics problem
→ one floor + one body

Character problem
→ one SpawnPoint + one floor

Script problem
→ one entity + one function

Rendering problem
→ one Cell + one camera

Editor problem
→ one selected Cell + one operation
```

A minimal reproduction makes ownership and causality easier to inspect and reduces unrelated system activity.

---

# 5. Instrumentation

Use targeted diagnostics rather than large amounts of permanent logging.

For AeoScript:

```aeoscript
debug.log("Reached update")
debug.log("Target ID:", target.id)
```

For Rust, temporary diagnostics should identify the actual state transition or value being investigated.

Useful diagnostics generally answer questions such as:

* What value entered the subsystem?
* What value did the subsystem produce?
* Which object or Cell was affected?
* Which branch of execution was taken?
* When did the state change?

Diagnostic output should be removed, reduced, or converted into appropriate permanent diagnostics once the investigation is complete.

Avoid leaving large volumes of temporary logging in performance-sensitive paths.

---

# 6. Use the Appropriate Debugging Tool

AeoEngine can be investigated using both its own diagnostics and third-party development tools.

Common tools include:

* Rust compiler and Cargo diagnostics.
* IDE or debugger tools for stepping through Rust code.
* RenderDoc for inspecting OpenGL frames and rendering state.
* GPU or system profiling tools for performance investigations.
* Git for comparing working state and isolating changes.

These tools serve different purposes.

For example:

```text
Compiler / Cargo
→ Build and test failures

Debugger
→ Rust execution and state inspection

RenderDoc
→ OpenGL frame and rendering investigation

Profiler
→ CPU / GPU performance investigation

Git
→ Change and regression isolation
```

Detailed procedures for these tools belong in:

```text
docs/development/DEBUGGING.md
```

This document defines their role in the development process rather than documenting every tool command.

---

# 7. Use Tests to Confirm the Failure

When a failure can be expressed as a deterministic condition, add or use an automated test.

Examples include:

* Persistence behavior.
* Cell data serialization.
* Script execution.
* Parser behavior.
* Runtime state transitions.
* Collision calculations.
* Renderer data construction.

A useful debugging progression is:

```text
Reproduce failure
      ↓
Create focused test when practical
      ↓
Observe failure
      ↓
Fix implementation
      ↓
Run focused test
      ↓
Run broader test suite
```

A regression test is especially valuable when fixing a bug that is likely to return later.

---

# 8. Integration Testing

Some failures only appear when multiple engine systems interact.

Examples include:

* AeoScript interacting with World state.
* Scripts interacting with physics.
* Character systems interacting with collision.
* Editor state interacting with renderer state.
* Play mode entering or leaving runtime systems.
* Persistence interacting with authored World data.

For these cases, isolated unit tests may not be sufficient.

Use the appropriate integration coverage, including the in-game AeoScript integration test suite and manual runtime testing.

The goal is to determine whether the failure exists:

```text
Inside one subsystem
```

or only when:

```text
Subsystem A
     ↓
Subsystem B
     ↓
Subsystem C
```

interact.

---

# 9. Performance Debugging

Performance issues should be investigated using measurements rather than visual assumptions alone.

Start by determining whether the cost is primarily associated with:

* World processing.
* Render chunk construction.
* GPU submission.
* Physics synchronization.
* Physics simulation.
* Character simulation.
* Script execution.
* Asset loading.
* Editor interaction.

For renderer problems, use the renderer benchmarks and profiling tools to compare representative workloads.

For example:

```text
Full render construction
        vs
Selective chunk rebuild
```

A performance fix should be evaluated against the same workload that demonstrated the problem.

Do not replace a measured bottleneck with a more complex subsystem without evidence that the additional complexity addresses the actual cost.

---

# 10. Root Cause First

The preferred debugging progression is:

```text
Symptom
 ↓
Reproduce
 ↓
Reduce
 ↓
Trace
 ↓
Identify ownership
 ↓
Identify root cause
 ↓
Make focused fix
 ↓
Add regression coverage
 ↓
Re-run verification
```

Avoid patching multiple unrelated systems simply because they appear along the same execution path.

A fix should address the cause of the failure rather than only changing the visible symptom.

---

# 11. Working-Tree Safety

Engine development often occurs with several unrelated local edits present.

Never use destructive cleanup commands to make the working tree easier to reason about unless the affected work is intentionally disposable.

Avoid commands such as:

```text
git reset --hard
git restore .
```

unless there is an explicit decision to discard the affected changes.

Preserve unrelated edits and isolate the intended fix instead.

Git can be used to inspect the affected changes safely:

```text
git status --short
git --no-pager diff
```

---

# 12. Verification After a Debugging Fix

A debugging fix is not complete when the symptom disappears once.

Verify the change at the appropriate levels.

A typical sequence is:

```text
Focused test
    ↓
Full cargo test
    ↓
Relevant integration test
    ↓
Relevant benchmark
    ↓
Manual runtime verification
```

Not every issue requires every step.

Examples:

```text
Persistence bug
→ focused persistence test
→ cargo test
```

```text
AeoScript bug
→ focused runtime test
→ AeoScript integration test
→ manual Play verification
```

```text
Renderer performance bug
→ renderer benchmark
→ manual editor / Play verification
```

```text
Editor interaction bug
→ targeted test where practical
→ cargo test
→ manual editor verification
```

The verification should match the system and failure being investigated.

---

# 13. Regression Coverage

When a bug is fixed, add regression coverage when practical.

The regression test should capture the behavior that originally failed rather than merely testing the implementation detail used to fix it.

For example:

```text
Bug:
Cell attributes disappear after save/load.

Regression:
Save/load test verifies the authored attributes survive.
```

or:

```text
Bug:
A localized World edit rebuilds the entire renderer.

Regression:
Renderer benchmark or targeted test verifies selective rebuild behavior.
```

The purpose is to make the original failure detectable if a future change reintroduces it.

---

# 14. Debugging Principle

The debugging process should preserve the engine's architectural ownership boundaries.

```text
Authored World
      ↓
Runtime Conversion
      ↓
Runtime Systems
      ↓
Renderer / Gameplay
```

and:

```text
Editor
 ↓
Authored World
```

A bug should be investigated within the system that owns the affected state.

Do not move state between subsystems merely to make a symptom disappear.

---

# 15. Debugging Checklist

For a difficult issue, verify:

```text
[ ] Failure reproduced
[ ] Expected behavior recorded
[ ] Owning subsystem identified
[ ] State boundary identified
[ ] Control/data flow traced
[ ] Reproduction reduced where practical
[ ] Targeted diagnostics used
[ ] Appropriate tests run
[ ] Appropriate third-party tool used if necessary
[ ] Root cause identified
[ ] Focused fix implemented
[ ] Regression coverage added where practical
[ ] Full verification completed
```

The repository, tests, benchmarks, debugger output, and actual runtime behavior are the authority when determining whether a debugging change is correct.