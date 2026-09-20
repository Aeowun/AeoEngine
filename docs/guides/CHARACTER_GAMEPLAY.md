# Character Gameplay

AeoEngine includes a dedicated runtime character system with movement, collision, jumping, animation, and a third-person gameplay camera.

---

# 1. Character Spawn

Create a SpawnPoint in the authored World.

Enter Play mode and the runtime attempts to create the character at a valid location.

If the initial location is obstructed, nearby clearance search will be used to find a valid spawn location.

---

# 2. Movement

The runtime character supports gameplay movement using fixed-step simulation.

The character system tracks movement state including Idle and Walk behavior.

---

# 3. Gravity

Characters are affected by runtime gravity.

Grounded state is maintained separately so jumping can require valid floor contact.

---

# 4. Jumping

Jumping is gated by grounded state.

This prevents repeated mid-air jumps through ordinary input while the character is not supported by the ground.

---

# 5. Collision

The character can collide with solid World geometry.

The runtime handles:

* Floors.
* Walls.
* Ceilings.

The character's gameplay collision dimensions are kept separate from visual character scaling.

---

# 6. Character Orientation

Character facing follows movement direction.

It is not simply forced to match the camera's orientation.

This allows third-person camera orbit to remain independent from character movement direction.

---

# 7. Animation

The runtime character system includes a dedicated rig and skeleton.

Current animation includes:

* Idle.
* Walk.
* Pose evaluation.
* Animation blending.

Animation updates use the fixed runtime timestep.

---

# 8. Appearance

Character appearance uses generated character geometry and appearance customization data.

The character system owns this runtime visual representation.

---

# 9. Gameplay Camera

The GameplayCamera is a basic dedicated third-person camera.

It provides:

* Character follow.
* Mouse orbit.
* Pitch limits.
* Collision/obstruction handling against solid World geometry.

The GameplayCamera is separate from the Editor camera and can serve as an example for more complex setups

---

# 10. Testing a New Character

A small character test World should contain:

~~~text
SpawnPoint
Floor
Wall
Platform
Ceiling
~~~

Test:

1. Spawn.
2. Walk.
3. Stop at walls.
4. Walk off a platform.
5. Land on the floor.
6. Jump.
7. Test ceiling collision.
8. Orbit the camera.
9. Return to Editor mode.

---

# 11. Character and Scripts

AeoScript can interact with engine objects through the supported runtime API.

As the scripting API expands, character gameplay logic can be layered over the CharacterSystem rather than duplicating movement and collision logic inside scripts.

The CharacterSystem remains responsible for core character simulation.

