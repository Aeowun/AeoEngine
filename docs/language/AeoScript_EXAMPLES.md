# AeoScript Examples

These examples demonstrate currently supported AeoScript language and engine APIs.

They are intended as small reference examples rather than a complete gameplay tutorial.

---

# 1. Persistent Script State

```aeoscript
entity Counter {
    count: number = 0

    fn update(dt) {
        count += 1
    }
}
```

The `count` field persists across update executions.

---

# 2. Find by Identity

```aeoscript
entity Hider {
    fn update(dt) {
        const targets = find("Target")

        for target in targets {
            target.visible = false
        }
    }
}
```

Names are not unique, so `find()` may return multiple objects.

---

# 3. Cell IDs

```aeoscript
entity Inspector {
    fn update(dt) {
        const targets = find("Target")

        for target in targets {
            debug.log("ID:", target.id)
            debug.log("Name:", target.name)
        }
    }
}
```

The ID identifies one specific Cell instance.

---

# 4. Cell Attributes

```aeoscript
entity Door {
    open: bool = false

    fn update(dt) {
        if open {
            self_cell.attributes["state"] = "open"
        }
    }
}
```

Runtime attribute writes are temporary overrides.

---

# 5. Baskets

```aeoscript
fn inventory() {
    const items = ["Sword", "Potion", "Key"]

    items.insert(0, "Map")
    items.sort()

    debug.log(items)
}
```

---

# 6. Basket Aliasing

```aeoscript
fn aliasing() {
    const items = [1, 2, 3]
    const alias = items

    alias[0] = 99

    debug.log(items[0])
}
```

The two variables reference the same basket.

---

# 7. Basket Cloning

```aeoscript
fn cloning() {
    const original = [1, 2, 3]
    const copy = basket.clone(original)

    copy[0] = 99

    debug.log(original[0])
    debug.log(copy[0])
}
```

The outer baskets are separate.

---

# 8. Maps

```aeoscript
fn stats() {
    const values = {}

    values["health"] = 100
    values["score"] = 250
    values[1] = "numeric"

    debug.log(values["health"])
    debug.log(values[1])
}
```

---

# 9. Missing Map Values

```aeoscript
fn lookup() {
    const values = {}

    if values["missing"] == nil {
        debug.log("No value")
    }
}
```

---

# 10. Math

```aeoscript
fn calculate(value: number): number {
    const clamped = math.clamp(value, 0, 100)
    const radians = math.deg_to_rad(90)

    return math.sin(radians) * clamped
}
```

---

# 11. Unicode Strings

```aeoscript
fn text_example() {
    const text = "é"

    debug.log(string.len(text))

    const characters = string.split(text)
    debug.log(characters[0])
}
```

String functions operate on Unicode scalar values.

---

# 12. Runtime Color

```aeoscript
entity ColorController {
    fn update(dt) {
        const targets = find("RedTarget")

        for target in targets {
            target.color = [1, 0, 0]
        }
    }
}
```

---

# 13. Runtime Offset

```aeoscript
entity PlatformController {
    fn update(dt) {
        const platforms = find("Platform")

        for platform in platforms {
            platform.offset = [0, 2, 0]
        }
    }
}
```

---

# 14. Waiting

```aeoscript
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
```

The same fiber resumes after the wait.

---

# 15. Nested Waiting

```aeoscript
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
```

The nested call frame survives the wait.

---

# 16. Loop with `wait()`

```aeoscript
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
```

---

# 17. Query by Class

```aeoscript
entity BlockInspector {
    fn update(dt) {
        const blocks = getAllCellsOfClass("Block")

        debug.log("Block count:", blocks.len())
    }
}
```

---

# 18. Runtime Cell Creation

```aeoscript
fn spawn_collectible(x, y, z) {
    const star = cell.new("Block")

    star.position = [x, y, z]
    star.color = [1, 1, 0]
    star.attributes["type"] = "collectible"
}
```

The Cell exists only during runtime.

---

# 19. Direct Cell Lookup

```aeoscript
const door = cell.get(1001)

if door {
    door.visible = false
}
```

The persistent ID identifies a specific authored or current runtime Cell.

---

# 20. Contact Event

```aeoscript
entity Lava {
    fn on_touch(other) {
        debug.log("Contact:", other.name)
    }
}
```

The Cell's collision-event setting controls whether relevant contact events are dispatched.

---

# 21. Overlap Event

```aeoscript
entity Region {
    fn on_overlap(overlapping, cell) {
        if overlapping {
            debug.log("Entered region")
        } else {
            debug.log("Left region")
        }
    }
}
```

---

# 22. Runtime UI

```aeoscript
const panel = ui.new("Panel")
panel.size = [200, 60]

const health = ui.new("Text")
health.text = "HEALTH: 100"

const button = ui.new("Button")
button.text = "Heal"

button.on_click = fn() {
    debug.log("Heal clicked")
}
```

---

# 23. Input and Player Movement

```aeoscript
const move = input.get_move_vector()
const basis = camera.get_horizontal_basis()

const forward = basis[0]
const right = basis[1]

const move_x = forward[0] * move[1] + right[0] * move[0]
const move_z = forward[2] * move[1] + right[2] * move[0]

if math.sqrt(move_x * move_x + move_z * move_z) > 0.001 {
    player.set_horizontal_velocity(move_x * 5.0, move_z * 5.0)
    player.set_facing_direction(move_x, move_z)
    player.select_animation("Walk")
} else {
    player.set_horizontal_velocity(0.0, 0.0)
    player.select_animation("Idle")
}
```

---

# 24. Camera Control

```aeoscript
const basis = camera.get_horizontal_basis()
const forward = basis[0]

const target = player.position

camera.set_target(
    target[0] + forward[0],
    target[1],
    target[2] + forward[2]
)
```

---

# 25. Audio

```aeoscript
const speaker = find("DoorSound")[0]

speaker.sound.play()
```

Delayed audio can use the same fiber model:

```aeoscript
fn open_door_sound() {
    speaker.sound.play()

    wait(1.0)

    speaker.sound.stop()
}
```

---

# 26. Script Control

```aeoscript
script.disable("scripts/traps.aeo")

wait(2.0)

script.enable("scripts/traps.aeo")
```

This controls runtime execution of the script.

---

# 27. Closure Callback

```aeoscript
const multiplier = 2

const multiply = fn(value) {
    return value * multiplier
}

debug.log(multiply(10))
```

The closure retains access to `multiplier`.

---

# 28. Runtime Test

```aeoscript
fn verify_cell() {
    const blocks = getAllCellsOfClass("Block")

    if blocks.len() > 0 {
        test.complete("block_exists", true)
    } else {
        test.complete("block_exists", false)
    }
}
```

The `test` API is intended for the in-game AeoScript integration test suite.

---

# 29. Gameplay Pattern

A common runtime pattern is:

```text
Persistent script field
      ↓
Gameplay decision
      ↓
Engine handle
      ↓
Runtime change
      ↓
wait()
      ↓
Same fiber resumes
```

This keeps gameplay state explicit while allowing long-running actions to suspend without losing their execution state.