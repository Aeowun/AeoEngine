# Physics Architecture

AeoEngine uses a runtime physics system separate from authored World data.

Physics simulation runs at a fixed timestep and maintains runtime bodies for moving objects while using authored World Cells for static collision geometry.

---

# 1. Fixed Timestep

Physics simulation uses a fixed timestep of 60 Hz.

The physics clock accumulates elapsed frame time and advances the simulation in fixed-size steps.

This keeps physics behavior independent of render-frame variability.

---

# 2. PhysicsWorld

`PhysicsWorld` owns runtime physics state.

It manages:

* Dynamic PhysicsBodies.
* Static collision representation.
* Collision detection.
* Penetration resolution.
* Support tracking.
* Sleeping and waking.

The runtime PhysicsWorld does not become the authoritative source for authored Cell position or properties.

---

# 3. Static Collision

Anchored, solid authored Cells provide static collision geometry.

Light Cells are not treated as solid physics geometry simply because they exist in the World.

Static collision state is derived from authored World data plus supported runtime overrides.

---

# 4. Dynamic Bodies

Non-anchored physical Cells can become dynamic PhysicsBodies.

A dynamic body maintains runtime values such as:

* Position.
* Velocity.
* Support state.
* Sleeping state.

Its movement does not rewrite the authored World coordinate every physics step.

---

# 5. Runtime Property Snapshots

Runtime physics bodies retain the properties required for correct simulation and rendering.

This prevents moving bodies from needing to query authored World data as their sole source of runtime visual state.

---

# 6. Change-Driven Static Synchronization

Static collision synchronization is change-driven.

The World tracks physics-relevant Cell changes using a dirty-cell set.

A Cell ID to coordinate index allows affected Cells to be located efficiently.

~~~text
Runtime property change
        ↓
Mark Cell ID dirty
        ↓
PhysicsWorld::sync_with_world
        ↓
Reconcile only changed static colliders
~~~

The system does not rebuild the entire static collider collection every physics frame when nothing has changed.

Idle scenes therefore avoid unnecessary full-scene static collision reconciliation.

---

# 7. Dirty-State Sources

Physics-relevant runtime changes can mark a Cell dirty, including changes to properties such as:

* Solidity.
* Anchored state.
* Visual offset where it affects effective collision position.

Authored World operations also maintain the Cell-ID to coordinate index used during synchronization.

---

# 8. Dynamic Collision

Dynamic bodies are checked for collisions during physics updates.

The current broad-phase behavior is intentionally simple and currently performs pairwise dynamic-body candidate checks.

Future broad-phase optimization can be introduced without rewriting the narrow-phase collision logic.

Possible future approaches include:

* Spatial grids.
* Spatial hashing.
* Sweep-and-prune.
* BVH-style broad phases.

---

# 9. Support and Sleeping

Physics tracks when dynamic bodies are supported by other bodies or static collision geometry.

Supported bodies can enter a sleeping state.

When support is lost or relevant physical conditions change, sleeping bodies can be awakened.

The goal is stable resting behavior without continuously integrating unnecessary motion.

---

# 10. Gravity

World gravity is authored scene data and is supplied to runtime physics.

The default gravity is:

~~~text
(0, -9.81, 0)
~~~

Runtime physics integrates gravity into dynamic-body motion.

---

# 11. Discrete Collision

The current physics system uses discrete collision testing.

High-speed tunneling is therefore a known architectural boundary rather than a hidden guarantee.

Continuous collision detection can be introduced later where profiling and gameplay demonstrate a need for it.

---

# 12. Runtime Boundary

Physics is a runtime system.

~~~text
Authored Cell
      ↓
Physics initialization
      ↓
Runtime PhysicsBody
      ↓
Simulation
      ↓
Temporary runtime state
~~~

When Play mode stops, the runtime physics state is discarded and the authored World remains unchanged.

