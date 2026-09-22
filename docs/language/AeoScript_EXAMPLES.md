# AeoScript Examples

This document contains small examples built from the currently implemented AeoScript language and engine API.

The examples focus on patterns already supported by the runtime rather than future APIs.

---

# 1. Simple State

~~~aeoscript
entity Counter {
    count: number = 0

    fn update(dt) {
        count += 1
    }
}
~~~

The `count` field persists across update executions.

---

# 2. Runtime Cell Visibility

~~~aeoscript
entity Controller {
    fn update(dt) {
        const blocks = find("Target")

        for block in blocks {
            block.visible = false
        }
    }
}
~~~

The visibility change is a runtime override. It does not rewrite the authored World value.

---

# 3. Changing Block Color

~~~aeoscript
entity ColorController {
    fn update(dt) {
        const blocks = find("RedTarget")

        for block in blocks {
            block.color = [1, 0, 0]
        }
    }
}
~~~

---

# 4. Working with IDs

~~~aeoscript
entity Inspect {
    fn update(dt) {
        const blocks = find("Target")

        for block in blocks {
            debug.log("Cell ID:", block.id)
            debug.log("Name:", block.name)
        }
    }
}
~~~

The ID identifies the specific Cell instance. The name is human-readable and may not be unique.

---

# 5. Basket Aliasing

~~~aeoscript
fn example() {
    const items = [1, 2, 3]
    const alias = items

    alias[0] = 99

    debug.log(items[0])
}
~~~

Both variables refer to the same basket.

---

# 6. Basket Cloning

~~~aeoscript
fn example() {
    const original = [1, 2, 3]
    const copy = basket.clone(original)

    copy[0] = 99

    debug.log(original[0])
    debug.log(copy[0])
}
~~~

The outer baskets are distinct.

---

# 7. Basket Utilities

~~~aeoscript
fn inventory() {
    items: basket = ["Sword", "Potion", "Key"]

    items.sort()
    items.insert(0, "Map")

    const index = items.find("Potion")

    debug.log("Potion index:", index)
}
~~~

---

# 8. Maps

~~~aeoscript
fn stats_example() {
    const stats = {}

    stats["health"] = 100
    stats["score"] = 250
    stats[1] = "numeric key"

    debug.log(stats["health"])
    debug.log(stats[1])
}
~~~

Map keys are not positional indexes.

---

# 9. Missing Map Key

~~~aeoscript
fn lookup_example() {
    const values = {}

    if values["missing"] == nil {
        debug.log("No value exists")
    }
}
~~~

Reading a missing key produces `nil`.

---

# 10. Map Key Deletion

~~~aeoscript
fn remove_example() {
    const values = {}

    values["name"] = "Ghost"
    values["name"] = nil
}
~~~

Assigning `nil` removes the map entry.

---

# 11. Math

~~~aeoscript
fn math_example(value: number): number {
    const clamped = math.clamp(value, 0, 100)
    const radians = math.deg_to_rad(90)

    return math.sin(radians) * clamped
}
~~~

---

# 12. Unicode Strings

~~~aeoscript
fn string_example() {
    const text = "é"

    debug.log(string.len(text))

    const characters = string.split(text)
    debug.log(characters[0])
}
~~~

String operations use Unicode scalar values rather than treating each UTF-8 byte as a character.

---

# 13. Runtime Offset

~~~aeoscript
entity PlatformController {
    fn update(dt) {
        const platforms = find("Platform")

        for platform in platforms {
            platform.offset = [0, 2, 0]
        }
    }
}
~~~

The visual offset is a runtime override.

---

# 14. Delayed Action

~~~aeoscript
entity DelayedAction {
    triggered: bool = false

    fn update(dt) {
        if triggered == false {
            triggered = true
            wait(1.0)
            debug.log("One second elapsed")
        }
    }
}
~~~

The script field prevents another delayed action from starting every frame.

The fiber itself remains suspended during the wait.

---

# 15. Nested Wait

~~~aeoscript
entity NestedExample {
    finished: bool = false

    fn delayed_step() {
        step: number = 10

        wait(0.05)

        step += 5
        finished = step == 15
    }

    fn update(dt) {
        if finished == false {
            delayed_step()
        }
    }
}
~~~

The local `step` survives the wait because it belongs to the active fiber call frame.

---

# 16. Loop with Wait

~~~aeoscript
entity Sequence {
    running: bool = false
    total: number = 0

    fn play() {
        running = true
        total = 0

        for value in [1, 2, 3] {
            total += value
            wait(0.1)
        }

        running = false
    }
}
~~~

The loop resumes from the correct iteration after each wait.

---

# 17. Finding All Blocks

~~~aeoscript
entity BlockInspector {
    fn update(dt) {
        const blocks = getAllCellsOfClass("Block")

        debug.log("Block count:", blocks.len())
    }
}
~~~

---

# 18. Runtime Property Restoration Pattern

Runtime overrides normally disappear when Play mode stops. During a running game, a script can still preserve the original runtime values when it needs temporary behavior:

~~~aeoscript
entity TemporaryVisibility {
    active: bool = false

    fn hide_target() {
        const matches = find("Target")

        for target in matches {
            target.visible = false
        }

        wait(1.0)

        for target in matches {
            target.visible = true
        }
    }
}
~~~

For persistent authored changes, use the editor rather than relying on runtime overrides.

---

# 19. Debugging with `debug.log`

Structured script output can make gameplay debugging easier:

~~~aeoscript
debug.log("Started")
debug.log("Health:", health)
debug.log("Target ID:", target.id)
~~~

Runtime diagnostics are shown through the engine's script output system.

---

# 20. Example Design Principle

The best current AeoScript pattern is to combine persistent entity fields with short runtime operations and explicit waits:

~~~text
Entity fields
      ↓
Gameplay decision
      ↓
Engine handle access
      ↓
Runtime modification
      ↓
wait()
      ↓
Same fiber resumes
~~~

This keeps gameplay logic readable while using the engine's runtime state model correctly.

---

# 21. Contact Events

Use `on_touch(cell)` to react when the player makes contact with an object.

~~~aeoscript
entity Lava {
    fn on_touch(player_char) {
        debug.log("Player touched lava!")
    }
}
~~~

The `cell` parameter provides a handle to the object being touched.

---

# 22. Dynamic Spawning

Use the `cell` namespace to manage objects at runtime.

~~~aeoscript
fn spawn_collectible(x, y, z) {
    const star = cell.new("Block")
    star.position = [x, y, z]
    star.color = [1, 1, 0]
    star.attributes["type"] = "collectible"
}
~~~

---

# 23. The "Manager" Pattern (The Roblox Way)

Rather than attaching scripts to every individual object in the editor, you can use a single script to manage multiple objects by their IDs.

~~~aeoscript
// Manager Script
const door1 = cell.get(10552341)
const door2 = cell.get(10552342)

const onTouchDoor = fn(c) {
    debug.log("Door touched by:", c.name)
    self.visible = false
    self.solid = false
    wait(1)
    self.visible = true
    self.solid = true
}

if door1 { door1.on_touch = onTouchDoor }
if door2 { door2.on_touch = onTouchDoor }
~~~
