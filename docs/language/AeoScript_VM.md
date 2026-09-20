# AeoScript VM Execution Model

The AeoScript VM is a tree-walking interpreter that executes parsed Abstract Syntax Trees (AST) within the engine's main tick.

---

# 1. Threading and Budget

*   **Synchronous**: Script execution is synchronous with the engine's physics and logic update.
*   **Fibers**: All execution occurs in persistent fibers. If a script calls `wait(n)`, its fiber is suspended and automatically resumed by the scheduler after `n` seconds.
*   **Operation Budget**: To ensure frame stability, each fiber has a maximum instruction budget per frame. Long-running loops or recursion that exceed this budget will cause the fiber to yield and continue on the next frame.

---

# 2. Memory Model

### Shared Reference Baskets
AeoScript `Basket` objects are backed by an atomic reference counter (`Arc`) and a `RefCell` in the host.
*   **Aliasing**: Passing a basket between functions or variables never copies the data; it copies the reference.
*   **Shared State**: Multiple script instances can share and mutate the same basket if they hold a reference to it.
*   **Frozen state**: Baskets carry a `frozen` flag. When set, any operation attempting a mutable borrow of the elements will trigger a VM runtime error.

### Handles
Handles (`Cell`, `Entity`) are small value types containing a kind and a unique ID. They do not hold raw pointers. The VM resolves handles against the engine's authoritative state (EntityManager and World) at the moment of access.

---

# 3. Standard Library Dispatch

The VM handles standard library calls (`math.*`, `basket.*`, `string.*`) through a specialized native module.
*   **Namespace Lookup**: Identifiers like `math` are reserved and resolve to internal namespace objects.
*   **Method Syntax**: Calls like `b.len()` or `b.insert(v)` are automatically routed to the corresponding `basket` namespace implementation with the object as the first argument.

---

# 4. Persistence of Logic (Bindings)

The VM loader reconciles authored world data with script logic using **Script Bindings**:
1.  The world file (`world.dat`) stores a list of bindings mapping a **u64 Cell ID** to a `.aeo` file path.
2.  During load, the VM verifies each ID exists in the world.
3.  The VM then creates a `ScriptInstance` for that cell.
4.  The instance logic is driven by the `entity` declaration in the script whose name matches the cell's authored `entity_identity` string.

---

# 5. Native Integration (EngineHost)

The VM communicates with the engine through the `EngineHost` trait. This abstraction ensures the interpreter remains decoupled from the specific physics or rendering implementations while allowing scripts to perform runtime property overrides and call engine-defined methods.
