# Building a Game

This guide describes the intended outside-in AeoEngine workflow: start with an authored World, get a playable loop working, then add gameplay systems and scripting.

The guide is deliberately practical. The best AeoEngine workflows should be validated by building real games rather than by assuming the engine API is complete.

If using the supplied AeoEngine custome character you can skip step 2
---

# 1. Start with the Playable Space

Create a small World first.

A useful initial scene might contain:

~~~text
Floor
Walls
A few platforms
SpawnPoint
Light
~~~

Keep the first test World small enough that editor and runtime behavior are easy to inspect.

---

# 2. Establish Character Compatability

Add a SpawnPoint and enter Play mode.

Verify:

* Character spawning.
* Movement.
* Gravity.
* Grounding.
* Jumping.
* Wall collision.
* Ceiling collision.
* Gameplay camera follow.

Do this before introducing complicated gameplay logic.

---

# 3. Add World Interaction

Use authored Blocks and Lights as gameplay targets.

Give objects meaningful identities when scripts will search for them.

For example:

~~~text
Door
Button
Platform
Goal
SpawnPoint
~~~

Identity names can be shared. Use Cell IDs whenever one exact authored instance must be targeted.

---

# 4. Add a Script

Create a script under:

~~~text
scripts/
~~~

Attach it to the intended Cell through the Properties panel.

A typical starting point is:

~~~aeoscript
entity Controller {
    fn update(dt) {
        debug.log("Controller running")
    }
}
~~~

---

# 5. Use Object Discovery

Scripts can find objects by identity:

~~~aeoscript
const doors = find("Door")
~~~

Or query all Cells of a class:

~~~aeoscript
const blocks = getAllCellsOfClass("Block")
~~~

Because names are not unique, always treat `find()` as a potentially multi-result query.

---

# 6. Add Runtime Interaction

Use runtime Cell properties to create temporary gameplay behavior.

Example:

~~~aeoscript
entity DoorController {
    fn open() {
        const doors = find("Door")

        for door in doors {
            door.visible = false
            door.solid = false
        }
    }
}
~~~

Those changes affect the running game without becoming authored edits.

---

# 7. Add Delayed Gameplay

Use `wait()` for simple time-based gameplay sequences.

~~~aeoscript
entity DoorController {
    open: bool = false

    fn temporarily_open() {
        const doors = find("Door")

        open = true

        for door in doors {
            door.solid = false
        }

        wait(2.0)

        for door in doors {
            door.solid = true
        }

        open = false
    }
}
~~~

A waiting script fiber resumes from the same execution point.

---

# 8. Build Gameplay in Small Systems

Prefer small gameplay scripts with clear responsibilities.

For example:

~~~text
DoorController
ButtonController
PlatformController
GoalController
EnemyController
GameState
~~~

---

# 9. Use Persistent Fields for Gameplay State

Entity fields are appropriate for state that must survive between update executions.

~~~aeoscript
entity Counter {
    score: number = 0
    active: bool = true
}
~~~

Function locals are appropriate for temporary execution state.

---

# 10. Test the Game Loop Early

A productive development sequence is:

~~~text
World
 ↓
Character
 ↓
Interaction
 ↓
Script
 ↓
Wait/timing
 ↓
Feedback
 ↓
Repeat
~~~

Don't wait until you're entire game exists before testing each layer.

---

# 11. Use the Engine as a Game Developer

The most valuable bugs are often found when the engine is used normally rather than when a subsystem is tested in isolation.

When a workflow fails, record:

* What you were trying to accomplish.
* What the editor showed.
* What Play mode did.
* Whether the problem was authored state or runtime state.
* Whether a script or engine system was involved.
* The smallest reproducible World.

---

# 12. Keep the First Game Small

The first AeoEngine game should be small enough to finish.

A useful target is a complete gameplay loop such as:

~~~text
Spawn
 ↓
Explore
 ↓
Interact
 ↓
Reach goal
 ↓
Win
~~~

The purpose is not production scale.

The purpose is to expose real engine workflow problems.

