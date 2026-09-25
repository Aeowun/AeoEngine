# Physics Architecture

AeoEngine uses a runtime physics system separate from authored World data.

`PhysicsWorld` owns runtime physical state while the authored World remains the source of static scene data.

---

# 1. PhysicsWorld

`PhysicsWorld` owns:

* Dynamic PhysicsBodies.
* Static collision representation.
* Physics IDs.
* Collision detection.
* Collision resolution.
* Support state.
* Sleeping state.
* Physics stepping state.

PhysicsWorld does not own persistent World Cell data.

---

# 2. Fixed-Step Simulation

Physics uses fixed-step simulation timing.

The physics clock accumulates frame time and advances physics in fixed-size steps.

This keeps physics behavior independent from variable render-frame timing.

The application coordinates the fixed-step execution.

---

# 3. World-to-Physics Conversion

At runtime initialization, authored World data is converted into the physics representation.

```text
Authored World
      ↓
PhysicsWorld registration
      ↓
Static collision
      ↓
Runtime simulation
```

Static collision is therefore derived from the current effective World state.

---

# 4. Static Collision

Solid World geometry provides static collision.

Anchored, solid Cells can contribute to static collision geometry.

Cell type alone does not automatically determine collision behavior; effective Cell properties such as `solid` and `anchored` are relevant.

Light and other non-solid Cells do not become collision geometry merely because they exist in the World.

---

# 5. Dynamic PhysicsBodies

Dynamic physical objects maintain runtime state such as:

* Position.
* Velocity.
* Support.
* Sleeping.
* Collision state.

Their runtime movement is independent of their authored World grid position.

The authored World remains unchanged while the body is simulated.

---

# 6. Runtime Property Overrides

Physics consumes effective runtime values.

For example, changing a Cell's `solid` or `anchored` property during Play can affect runtime collision behavior without changing the authored baseline.

The World tracks physics-relevant changes so PhysicsWorld can reconcile the affected state.

---

# 7. Change-Driven Static Synchronization

Static collision synchronization is change-driven.

The World tracks affected Cell IDs and marks physics-relevant changes as dirty.

```text
World change
      ↓
Physics-dirty Cell ID
      ↓
PhysicsWorld::sync_with_world
      ↓
Affected collision state reconciled
```

An unchanged scene does not require the entire static collision representation to be rebuilt every physics frame.

---

# 8. Dynamic Collision

Dynamic collision currently uses a simple candidate-generation approach with pairwise checks.

This is intentionally separate from the collision resolution logic.

Possible future broad-phase approaches include:

* Spatial grids.
* Spatial hashing.
* Sweep-and-prune.
* BVH-style structures.

A future broad phase can therefore be introduced without requiring a complete rewrite of the narrow-phase collision system.

---

# 9. Support and Sleeping

Physics tracks whether bodies are supported by static or dynamic collision geometry.

Supported bodies can enter a sleeping state when continued simulation is unnecessary.

Relevant changes can wake sleeping bodies.

Sleeping exists primarily to avoid unnecessary work while preserving stable resting behavior.

---

# 10. Gravity

World gravity is authored scene data.

The current default is:

```text
(0, -9.81, 0)
```

Physics applies the World gravity vector to dynamic runtime bodies.

Changing World gravity changes the runtime simulation without making PhysicsWorld the owner of the setting.

---

# 11. Discrete Collision

The current collision system uses discrete collision testing.

Continuous collision detection is therefore not currently guaranteed for high-speed objects.

If tunneling becomes a demonstrated gameplay problem, selective continuous collision detection can be introduced where needed.

---

# 12. Character Interaction

Characters are simulated by `CharacterSystem`, but interact with the same World collision environment.

The relationship is:

```text
World collision
      ↓
Physics / Character collision
      ↓
Runtime character state
```

Character-specific movement and collision response remain owned by CharacterSystem.

---

# 13. Runtime Boundary

Physics is a runtime subsystem.

```text
Authored Cell
      ↓
Physics initialization
      ↓
Runtime PhysicsBody / collision state
      ↓
Simulation
      ↓
Temporary runtime state
```

When Play mode stops, runtime physics state is discarded.

The authored World remains unchanged.