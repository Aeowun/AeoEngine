# Physics Gameplay

AeoEngine physics is intended to provide gameplay interactions between authored World geometry and runtime physical bodies.

---

# 1. Static World Collision

An authored Cell can participate in static collision when it is solid and anchored.

This gives the runtime a persistent collision surface for floors, walls, platforms, and other structures.

---

# 2. Dynamic Bodies

Non-anchored physical objects can become dynamic PhysicsBodies.

During Play they can move, fall, collide, sleep, and wake.

Their runtime positions are separate from authored World coordinates.

---

# 3. Gravity

World gravity is authored scene configuration.

The current default is:

~~~text
(0, -9.81, 0)
~~~

Dynamic bodies are affected by runtime gravity.

---

# 4. Resting Objects

Physics tracks support between runtime bodies and static geometry.

Supported objects can sleep rather than being updated as though they were still moving.

If support is lost, a sleeping body can be awakened.

---

# 5. Building a Physics Test

A useful test scene contains:

~~~text
Large anchored floor
      ↓
Stack of non-anchored blocks
      ↓
Player nearby
~~~

Enter Play and verify that:

* Blocks fall.
* Blocks collide with the floor.
* Stacks settle.
* Moving or removing support causes appropriate bodies to wake.

---

# 6. Runtime Properties

Scripts can change supported Cell physics properties during Play.

For example:

~~~aeoscript
const targets = find("Platform")

for target in targets {
    target.solid = false
    target.anchored = false
}
~~~

Physics synchronization responds to physics-relevant runtime changes without rebuilding every static collider every frame.

---

# 7. Character Interaction

Characters use the World collision environment for:

* Floor contact.
* Walls.
* Ceilings.
* Grounded state.

Character gameplay remains separate from ordinary dynamic PhysicsBody behavior.

---

# 8. Physics and Authored Data

Physics is runtime state.

When a dynamic object moves, the physics simulation does not automatically move the authored Cell in the saved World.

This means a test such as dropping a block in Play does not permanently relocate the authored block.

---

# 9. Debugging Physics

When physics behaves unexpectedly, isolate the scene.

Start with:

~~~text
One floor
One dynamic body
One character
No scripts
~~~

Then add complexity one system at a time.

This makes it easier to distinguish a physics problem from a script, character, editor, or persistence problem.

---

# 10. Current Physics Boundaries

The current system uses discrete collision testing and a simple dynamic-body candidate strategy.

If gameplay demonstrates tunneling or scaling problems, those should become concrete optimization or feature tasks rather than assumptions about future requirements.

