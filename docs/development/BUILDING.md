# Building AeoEngine

This document describes the development build workflow for AeoEngine itself.

---

# 1. Platform

AeoEngine currently targets:

* Windows 10.
* Windows 11.

The engine is developed and tested using the Rust stable toolchain.

---

# 2. Repository Root

Run development commands from the AeoEngine repository root.

~~~text
C:\Dev\AeoEngine
~~~

The exact local path may differ.

---

# 3. Check the Project

Use Cargo to verify that the project compiles:

~~~text
cargo check
~~~

`cargo check` is the fast compile-oriented verification step and does not produce the final executable.

---

# 4. Build and Run

Run the engine with:

~~~text
cargo run
~~~

This is the normal development path for manually exercising the engine and editor.

---

# 5. Tests

Run the full Rust test suite with:

~~~text
cargo test
~~~

The full suite should be run before committing substantial runtime changes.

---

# 6. Development Loop

A practical engine development loop is:

~~~text
Change
  ↓
cargo check
  ↓
Focused tests
  ↓
Full cargo test
  ↓
Manual AeoEngine run
  ↓
Inspect behavior
  ↓
Repeat
~~~

Manual Play-mode behavior is especially important for editor, rendering, physics, character, and scripting integration.

---

# 7. Generated and Local Files

Development may produce generated artifacts, imported assets, temporary project state, and local configuration files.

Before committing, inspect:

~~~text
git status
~~~

Do not assume every untracked file belongs in the commit.

Generated artifacts and unrelated local files should remain outside feature commits unless they are intentionally part of the change.

---

# 8. Preserving the Working Tree

AeoEngine development frequently contains multiple related changes at once.

Do not use destructive commands to clean up unrelated work.

Avoid commands such as:

~~~text
git reset --hard
git restore .
~~~

unless there is an explicit decision to discard the affected work.

When an unrelated change is present, preserve it and stage only the intended files.

---

# 9. Inspecting Changes

Use Git to understand what actually changed.

~~~text
git status --short
git --no-pager diff
git --no-pager diff --cached
~~~

`git --no-pager diff` is useful in environments where the default Git pager makes the command appear to stop responding.

---

# 10. Build Verification Principle

A successful build is necessary but not sufficient.

For runtime work, the expected verification sequence is:

~~~text
Compiles
  +
Tests pass
  +
Manual behavior works
  =
Verified change
~~~

The last step is obviously important for editor and gameplay behavior that is difficult to express entirely through unit tests.

