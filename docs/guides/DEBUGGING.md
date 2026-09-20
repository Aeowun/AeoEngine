# Debugging AeoEngine

AeoEngine contains several interacting systems, so the most effective debugging approach is to identify which state boundary or subsystem is failing before changing code.

---

# 1. Start with the Symptom

Write down the smallest concrete failure.

Examples:

~~~text
A Block does not render.
A character spawns inside a wall.
A script field resets every frame.
A script binding becomes stale.
A physics body does not wake.
An imported texture is missing.
~~~

Avoid starting with a speculative implementation theory.

---

# 2. Determine Authored vs Runtime

Ask first:

> Is the wrong value in the authored World, or only during Play mode?

If it is wrong in the editor before Play, inspect World/editor/persistence behavior.

If it is correct in the editor but wrong during Play, inspect runtime systems.

---

# 3. Script Diagnostics

Use `debug.log()` for targeted script instrumentation.

~~~aeoscript
debug.log("Reached update")
debug.log("Target ID:", target.id)
debug.log("Health:", health)
~~~

The PRINT OUTPUT system provides runtime script messages and diagnostics.

---

# 4. Script Runtime Failures

For script problems, separate the possibilities:

~~~text
Parse problem
    ↓
Lexer / Grammar / Parser

Runtime language problem
    ↓
Interpreter / Fiber / Standard Library

Engine API problem
    ↓
Handle / Host integration

Lifecycle problem
    ↓
ScriptScene / Scheduler / ScriptInstance
~~~

---

# 5. Fiber and `wait()` Problems

If a script appears to run but does not resume after `wait()`, inspect:

* Whether the function actually reached `wait()`.
* Whether the lifecycle task remains active.
* Whether the fiber is waiting rather than completed.
* Whether the same fiber resumes.
* Whether persistent entity fields are synchronized.

The important distinction is:

~~~text
Fiber
└── execution state

ScriptInstance
└── persistent fields
~~~

---

# 6. Binding Problems

For a stale script binding, check:

~~~text
Binding target Cell ID
        ↓
Does Cell ID exist?
        ↓
Does the script path exist?
        ↓
Does the script entity match the intended authored object?
~~~

Do not assume that a matching name means the binding target is correct.

Names can be duplicated.

---

# 7. Physics Problems

Reduce the scene.

Start with:

~~~text
One floor
One dynamic object
No scripts
No character
~~~

Then test:

* Gravity.
* Resting contact.
* Collision.
* Support.
* Sleeping.
* Wake behavior.

Add other systems only after the base physics behavior is understood.

---

# 8. Character Problems

Reduce character problems to:

~~~text
SpawnPoint
Floor
Wall
Character
GameplayCamera
~~~

Then test:

* Spawn.
* Movement.
* Gravity.
* Grounding.
* Jumping.
* Wall collision.
* Camera follow.

This separates CharacterSystem problems from World and scripting problems.

---

# 9. Rendering Problems

Determine whether the problem is:

~~~text
Authored object missing
Runtime object missing
Wrong transform
Wrong material/texture
Wrong lighting
Camera problem
GPU resource problem
~~~

Check the renderer only after confirming that the expected runtime object actually exists.

---

# 10. Persistence Problems

When a change does not survive a save/reload cycle, determine whether it was supposed to be authored data or runtime data.

Runtime changes are not expected to persist simply because they were visible during Play.

For authored persistence, verify:

* The editor actually changed the authored World.
* Save was performed.
* The correct project/world was saved.
* Reload reconstructs the expected authored state.

---

# 11. Rust Verification

From the project root:

~~~text
cargo check
~~~

Then run the test suite:

~~~text
cargo test
~~~

A successful test suite provides evidence that the code-level regression tests still pass, but manual Play-mode behavior remains important for editor and runtime integration.

---

# 12. Integration Harnesses

AeoScript integration scripts can exercise several engine features together.

A good integration test can verify:

* Language semantics.
* Collections.
* Standard library behavior.
* World lookup.
* Runtime overrides.
* Lifecycle behavior.
* Fiber yielding.
* Persistence of script fields.

This catches problems that isolated unit tests may miss.

---

# 13. Root-Cause Workflow

Use this sequence when debugging a new engine problem:

~~~text
Reproduce
   ↓
Reduce
   ↓
Identify state boundary
   ↓
Identify owning subsystem
   ↓
Trace actual data/control flow
   ↓
Fix smallest correct root cause
   ↓
Add regression test
   ↓
Run cargo check
   ↓
Run cargo test
   ↓
Manually verify in AeoEngine
~~~

Avoid changing multiple unrelated systems simply because they appear near the symptom.

---

# 14. Real-Game Debugging

Once AeoEngine is being used to build actual games, real workflow failures become especially valuable.

Record:

* What the developer was trying to do.
* What was expected.
* What actually happened.
* Whether the problem reproduced in a minimal World.
* Whether scripts were involved.
* Whether the problem survived a restart.

The goal is to turn a gameplay failure into a small reproducible engine problem.

---

# 15. Documentation Rule

When a debugging session establishes a durable architectural rule, document it in the appropriate architecture or language reference rather than leaving the knowledge only in a bug-fix commit.

