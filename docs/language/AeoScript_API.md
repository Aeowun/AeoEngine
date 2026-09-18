# AeoScript API

This document describes the interface between AeoScript and AeoEngine.

The API should follow the engine as it exists.

This is not a promise that every function below exists yet.

---

# 1. General Rule

AeoScript should call the engine through small, obvious APIs.

Prefer:

```aeoscript
character.jump()
```

over exposing the character controller internals.

Prefer:

```aeoscript
transform.position
```

over exposing the internal transform component layout.

---

# 2. Entity

The `entity` API deals with the entity the script belongs to and other entities.

Possible operations:

```aeoscript
entity.name
entity.destroy()
entity.has_tag("Player")
```

Possible lookup:

```aeoscript
world.find("Door")
```

The exact lookup model is not final.

---

# 3. Transform

Initial planned properties:

```aeoscript
transform.position
transform.rotation
transform.scale
```

Possible operations:

```aeoscript
transform.move(offset)
transform.move_forward(amount)
transform.rotate(rotation)
transform.look_at(position)
```

The implementation must use the actual AeoEngine transform system.

---

# 4. Character

Initial planned operations:

```aeoscript
character.move(direction)
character.jump()
character.look(direction)
character.is_grounded()
```

The script should provide intent.

The engine handles movement, collision, gravity, and the rest of the character system.

---

# 5. Physics

Possible properties:

```aeoscript
physics.velocity
```

Possible operations:

```aeoscript
physics.apply_force(force)
physics.apply_impulse(impulse)
```

The scripting API should not expose collision solver internals.

---

# 6. Input

Initial direction:

```aeoscript
input.is_down("W")
input.was_pressed("Space")
input.was_released("Space")
```

Eventually named actions:

```aeoscript
input.is_down("move_forward")
input.was_pressed("jump")
```

Raw keys and named actions may coexist.

---

# 7. Audio

Possible API:

```aeoscript
audio.play("door_open")
audio.stop("alarm")
```

Later:

```aeoscript
audio.play("door_open", volume)
```

Do not expose low-level audio objects until there is a reason to.

---

# 8. Camera

Possible operations:

```aeoscript
camera.look_at(position)
camera.set_mode("follow")
```

The final camera API depends on how the gameplay camera is exposed by AeoEngine.

---

# 9. World

Possible operations:

```aeoscript
world.find("Door")
world.spawn("Enemy", position)
```

Possible future operations:

```aeoscript
world.get_entities()
world.load_scene("Level2")
```

These need to be designed around actual engine behavior.

---

# 10. Debug

Development scripts should be able to write useful information.

Possible API:

```aeoscript
debug.log("Enemy spawned")
debug.warn("No target")
```

Possible visual debugging:

```aeoscript
debug.draw_line(start, finish)
debug.draw_point(position)
```

These should be development tools, not gameplay dependencies.

---

# 11. Time

The engine already provides `dt` to `update`.

Example:

```aeoscript
fn update(dt: number) {
    cooldown -= dt
}
```

A future `time` API may expose:

```aeoscript
time.now()
time.delta()
time.scale
```

There is no need to add this until it is useful.

---

# 12. Vectors

Initial vector constructors:

```aeoscript
vec2(x, y)
vec3(x, y, z)
vec4(x, y, z, w)
```

Possible vector operations:

```aeoscript
direction.length()
direction.normalized()
```

The vector API should remain close to the math functionality already used by AeoEngine.

---

# 13. Entity Properties

The engine may expose convenient properties directly:

```aeoscript
target.position
target.name
```

This is preferable to forcing scripts to repeatedly request components.

---

# 14. Tags

Entities should eventually support tags.

Example:

```aeoscript
if other.has_tag("Player") {
    start_alarm()
}
```

Tags need to be an engine feature first.

AeoScript should not invent a separate tag system.

---

# 15. Script State

Fields defined on the entity are script state.

Example:

```aeoscript
entity Enemy {

    health: number = 100
    target: Entity? = null
}
```

Every script instance has its own values.

---

# 16. Native API Boundary

AeoScript can only call native engine functions that have been explicitly registered.

Conceptually:

```text
AeoScript
    |
    v
VM
    |
    v
Registered native function
    |
    v
AeoEngine
```

There is no generic "call Rust" operation.

---

# 17. API Naming

Use names that describe what the game developer wants to do.

Good:

```aeoscript
character.jump()
audio.play("alarm")
entity.destroy()
```

Bad:

```aeoscript
physics_controller.invoke_jump()
audio_manager.submit_event()
entity_manager.remove_handle()
```

The scripting API is a user-facing interface.

---

# 18. Return Values

Where useful, APIs may return results.

Example:

```aeoscript
if character.jump() {
    print("Jump accepted")
}
```

Not every engine function needs a return value.

---

# 19. Errors

A bad API call should produce a script error that points to the script.

Example:

```text
AeoScript Runtime Error

Door.aeo:27

Entity no longer exists.
```

It should not crash the engine.

---

# 20. API Growth

New APIs should be added when an actual gameplay need appears.

Do not build a giant standard API before the scripting runtime exists.

First make:

```text
entity
transform
input
debug
character
```

work well.

Then expand.

