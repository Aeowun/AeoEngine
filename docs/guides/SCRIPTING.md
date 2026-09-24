# AeoScript Gameplay Scripting

AeoScript is AeoEngine's gameplay scripting language.

The scripting workflow connects `.aeo` source files, authored Cell bindings, persistent script fields, engine handles, and runtime fibers.

---

# 1. Create a Script

Create an `.aeo` file in the project's `scripts/` directory.

A simple script can look like:

~~~aeoscript
entity Controller {
    count: number = 0

    fn update(dt) {
        count += 1
        debug.log("Update:", count)
    }
}
~~~

---

# 2. Name the Scripted Object

Give the authored Cell a useful identity/name in the Properties panel.

The name is human-readable and may be shared by other Cells.

The actual script binding targets the Cell's persistent numeric ID.

---

# 3. Attach and Manage Scripts

Select the Cell and use the script controls in the Properties panel.

The engine creates a persistent binding between:

~~~text
Cell ID → script path
~~~

### Script Enable / Disable

Runtime controls are `script.enable(path)`, `script.disable(path)`, and `script.is_enabled(path)`. Disabled script paths are excluded from normal event dispatch, and disabling cancels their active fibers; this is file-level execution control, not a promise to recreate bindings on re-enable.

Removing a binding does not delete the source script.

---

# 4. Lifecycle Events

In addition to `update(dt)`, AeoScript supports contextual events.

### Contact Events

The `on_touch(cell)` function runs when the player character makes contact with a scripted object.

~~~aeoscript
entity Spike {
    fn on_touch(c) {
        debug.log("Player took damage!")
    }
}
~~~

The runtime dispatches supported contact and overlap callbacks through global handlers, dynamic handle properties, and bound entity handlers.

---

# 5. Use `update(dt)`

`update(dt)` is the normal per-frame lifecycle function.

Use the supplied delta time for time-scaled logic.

~~~aeoscript
entity Mover {
    position_x: number = 0
    speed: number = 2

    fn update(dt) {
        position_x += speed * dt
    }
}
~~~

---

# 5. Find World Objects

Use `find()` when you want name-based discovery.

~~~aeoscript
const doors = find("Door")
~~~

Use `getAllCellsOfClass()` when you want a class-based query.

~~~aeoscript
const blocks = getAllCellsOfClass("Block")
~~~

Both return baskets of handles. `find()` can return Entity and Cell-family handles: an authored Cell identity match is not an Entity handle.

---

# 6. Change Runtime Properties

Supported Cell runtime properties can be modified directly.

~~~aeoscript
for block in blocks {
    block.visible = false
    block.solid = false
    block.attributes["health"] = 100
}
~~~

These changes are temporary runtime overrides. Assign `nil` to an attribute to restore its authored value.

---

# 7. Dynamic Object Management

Use the `cell` namespace to create or remove objects during Play.

~~~aeoscript
const coin = cell.new("Block")
coin.position = [10, 5, 2]
coin.color = [1, 0.8, 0]

// Later...
cell.delete(coin)
~~~

---

# 8. Use Persistent Fields

Script fields persist across lifecycle executions.

~~~aeoscript
entity Counter {
    hits: number = 0

    fn update(dt) {
        hits += 1
    }
}
~~~

This is different from a local variable inside a function.

---

# 8. Use Functions

Break gameplay behavior into functions.

~~~aeoscript
entity Door {
    open: bool = false

    fn set_open(value: bool) {
        open = value
    }
}
~~~

Functions can call other functions and can yield with `wait()`.

---

# 9. Use `wait()`

Use `wait()` for delayed gameplay behavior.

~~~aeoscript
entity Blink {
    visible: bool = true

    fn update(dt) {
        visible = false
        wait(0.5)
        visible = true
    }
}
~~~

The same fiber resumes after the wait.

---

# 10. Nested Functions Can Yield

A nested function call can contain a wait.

~~~aeoscript
entity Example {
    value: number = 0

    fn delayed_change() {
        wait(0.25)
        value = 10
    }

    fn update(dt) {
        delayed_change()
    }
}
~~~

Local variables, call frames, and loop state are preserved by the fiber.

---

# 11. Collections

Baskets are zero-indexed and reference-backed.

~~~aeoscript
const items = ["A", "B", "C"]
items.insert(0, "Start")
~~~

Maps are key/value collections.

~~~aeoscript
const stats = {}
stats["health"] = 100
~~~

Missing map keys return `nil`.

---

# 12. Standard Library

Core utility namespaces are:

~~~text
math
basket
string
~~~

Engine-integrated namespaces are `cell`, `entity`, `script`, `event`, and `test`; `debug` is the logging namespace, and `time.delta` is a host-provided property.

Examples:

~~~aeoscript
const speed = math.clamp(velocity, 0, 10)
items.sort()
const words = string.split("one,two,three", ",")
~~~

---

# 13. Debugging Scripts

Use structured output during gameplay development.

~~~aeoscript
debug.log("Started")
debug.log("Health:", health)
debug.log("Target ID:", target.id)
~~~

Use the PRINT OUTPUT panel to inspect runtime messages and errors.

---

# 14. Script State Model

A useful mental model is:

~~~text
Authored Cell
    ↓
Script Binding
    ↓
ScriptInstance
    ↓
ScriptFiber
    ├── Call stack
    ├── Locals
    ├── Loop state
    └── wait state
~~~

Persistent fields belong to the ScriptInstance.

Execution state belongs to the Fiber.

---

# 15. Controller & Camera Script Profiles

In AeoEngine 0.7.3, player movement and camera control can be delegated to script-defined controller and camera profiles.

### Controller Scripts (`controllers/`)

Controller scripts live in `controllers/` (e.g., `controllers/thirdPerson_Controller/thirdPersonController.aeo`).

```aeoscript
entity ThirdPersonController {
    player = nil
    camera = nil
    speed = 5.0
    jump_impulse = 6.5

    fn constructor(player_object, camera_object, input_system) {
        player = player_object
        camera = camera_object
    }

    fn update(dt) {
        if player == nil { return }

        const move = input.get_move_vector()
        const basis = camera.get_horizontal_basis()

        const forward = basis[0]
        const right = basis[1]

        const move_x = forward[0] * move[1] + right[0] * move[0]
        const move_z = forward[2] * move[1] + right[2] * move[0]

        player.set_horizontal_velocity(move_x * speed, move_z * speed)

        if input.is_jump_pressed() && player.is_grounded() {
            player.apply_vertical_impulse(jump_impulse)
        }
    }
}
```

The controller provides movement decisions while AeoEngine handles physics, gravity, and collision resolution.

### Camera Scripts (`cameras/`)

Camera scripts live in `cameras/` (e.g., `cameras/thirdPerson/thirdPersonCamera.aeo`).

```aeoscript
entity ThirdPersonCamera {
    player = nil
    distance = 6.0
    yaw = 0.785
    pitch = 0.35

    fn constructor(player_object) {
        player = player_object
    }

    fn update(dt) {
        if player == nil { return }

        const orbit_delta = input.get_orbit_delta()
        yaw = yaw - orbit_delta[0] * 0.015
        pitch = math.clamp(pitch + orbit_delta[1] * 0.015, -0.2, 1.2)

        const char_pos = player.position
        const target = [char_pos[0], char_pos[1] + 1.0, char_pos[2]]

        const desired_pos = [
            char_pos[0] + distance * math.cos(pitch) * math.sin(yaw),
            char_pos[1] + distance * math.sin(pitch) + 1.5,
            char_pos[2] + distance * math.cos(pitch) * math.cos(yaw)
        ]

        const actual_pos = physics.resolve_camera_collision(target, desired_pos)

        camera.set_position(actual_pos[0], actual_pos[1], actual_pos[2])
        camera.set_target(target[0], target[1], target[2])
        camera.set_orientation(yaw, pitch)
    }
}
```

---

# 16. Audio Emitter & UI Workflows

### Audio Emitter Workflow

Find an `AudioEmitter` cell by name and access its `.sound` handle:

```aeoscript
const sound_emitter = find("DoorSound")[0]
sound_emitter.sound.play()
wait(1.0)
sound_emitter.sound.stop()
```

### Runtime UI Workflow

Create HUD overlays dynamically at runtime:

```aeoscript
const panel = ui.new("Panel")
panel.size = [200, 80]

const text = ui.new("Text")
text.text = "HEALTH: 100"

const btn = ui.new("Button")
btn.text = "Click Me"
btn.on_click = fn() {
    debug.log("Button clicked!")
}
```

---

# 17. Recommended Workflow

Use this development loop:

~~~text
Write script
 ↓
Save
 ↓
Run Play
 ↓
Observe output
 ↓
Test gameplay
 ↓
Stop Play
 ↓
Fix script
 ↓
Repeat
~~~

Build the smallest working behavior first, then add complexity.

