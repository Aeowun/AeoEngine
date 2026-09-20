# Development Debugging

AeoEngine debugging should begin with the actual failure and work toward the owning subsystem rather than modifying code based only on the first plausible theory.

---

# 1. Reproduce First

Before changing implementation, reproduce the issue consistently when possible.

Record:

* What action was performed.
* What was expected.
* What actually happened.
* Whether the issue occurs in Editor mode or Play mode.
* Whether the issue survives restarting the application.

---

# 2. Find the State Boundary

AeoEngine contains several state categories.

~~~text
Authored state
Runtime state
Editor state
Renderer state
Physics state
Character state
Script state
~~~

The first debugging question should be:

> Which system owns the incorrect value?

---

# 3. Trace Before Patching

When a failure appears to involve a subsystem, trace the actual control/data flow.

For example, a scripting failure may involve:

~~~text
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
~~~

Do not assume the component closest to the symptom owns the bug.

---

# 4. Reduce the Reproduction

Reduce a large problem to the smallest scene or script that still demonstrates the failure.

Examples:

~~~text
Physics problem
→ one floor + one body

Character problem
→ one SpawnPoint + one floor

Script problem
→ one entity + one function

Rendering problem
→ one Cell + one camera
~~~

A minimal reproduction makes ownership and causality much easier to inspect.

---

# 5. Instrumentation

Use targeted diagnostics rather than large amounts of permanent logging.

For AeoScript:

~~~aeoscript
debug.log("Reached update")
debug.log("Target ID:", target.id)
~~~

For Rust, temporary diagnostics should identify the actual state transition being investigated.

Remove diagnostic noise once the root cause is confirmed.

---

# 6. Root Cause First

The preferred debugging progression is:

~~~text
Symptom
 ↓
Reproduce
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
~~~

Avoid patching multiple unrelated systems just because they appear along the same execution path.

---

# 7. Working-Tree Safety

Engine development often occurs with several unrelated local edits present.

Never use destructive cleanup commands to make the working tree easier to reason about unless the affected work is intentionally disposable.

Preserve unrelated edits and isolate the intended fix instead.

---

# 8. AI-Assisted Debugging

AI tools can be useful for investigation, implementation, tests, documentation, repetitive refactors, and other development tasks.

The most reliable workflow is to provide a bounded task with concrete implementation direction.

A useful AI task specification should identify:

* The exact behavior that must change.
* The files or subsystem involved when known.
* The intended implementation approach.
* Important invariants that must not change.
* Existing tests or behavior that must remain intact.
* Required verification commands.

For example, instead of asking an AI tool to broadly "look into scripting," define the exact runtime behavior that needs to exist and the architectural constraints under which it should be implemented.

The AI should accelerate execution of a known task rather than silently inventing the architecture.

---

# 9. Small Tedious Tasks

Small, repetitive, fully scoped tasks are particularly suitable for AI assistance.

Examples include:

* Repetitive documentation formatting.
* Mechanical test additions following an established pattern.
* Straightforward file migrations.
* Consistent API documentation updates.
* Bounded refactors with an explicit desired result.

The task should have a clear definition of done and a predictable implementation shape.

---

# 10. Architectural Work

AI should not be given vague authority over major architectural decisions.

For larger changes, explicitly establish:

~~~text
Desired behavior
      ↓
Architectural direction
      ↓
Constraints/invariants
      ↓
Files/subsystems involved
      ↓
Implementation
      ↓
Verification
~~~

The developer remains responsible for deciding what the engine should become.

---

# 11. Verification of AI-Assisted Work

Never treat an AI tool's statement that a task is "fixed" as verification by itself.

Require evidence:

* Actual source changes.
* Actual command output.
* Passing focused tests.
* Passing full tests where appropriate.
* Manual runtime verification for affected workflows.

The repository and test/runtime behavior are the authority, not the tool's summary.

---

# 12. Root-Cause Lessons

AeoEngine development has already demonstrated several useful debugging patterns:

* A visible symptom may occur several layers away from the actual ownership bug.
* Authored/runtime state confusion can make a correct subsystem appear broken.
* Integration harnesses can reveal lifecycle bugs that isolated tests miss.
* Persistent state and execution state must have explicit ownership.
* Real gameplay workflows provide valuable bug reports that synthetic tests may not expose.

