# AeoScript VM

The AeoScript VM executes parsed script code and provides the interface between AeoScript and AeoEngine.

---

# 1. Execution Model

AeoScript is interpreted from an Abstract Syntax Tree (AST).

* **Fibers**: Scripts execute in persistent fibers. A fiber can yield, such as when `wait()` is called, and resume during a later frame.
* **Instruction Budget**: Each fiber has a maximum number of operations it can execute during one resume. If the limit is reached, the fiber yields and resumes later.
* **Single-Threaded**: Script execution runs synchronously during the engine tick.

---

# 2. Memory and Scoping

The VM manages two types of storage:

1. **Script Instance State**: Fields declared inside an `entity` block. These persist for the lifetime of the script instance.
2. **Call Scopes**: Local variables declared inside functions or blocks. These exist only for their enclosing scope and follow the VM's stack-based scope model.

---

# 3. Handle System

Scripts use handles to reference engine objects. Rust pointers and memory addresses are not exposed to AeoScript.

* **Cell Handles**: Identified by a stable 8-digit decimal ID. The VM uses the ID to resolve the cell in the world.
* **Entity Handles**: Identified by a unique runtime ID assigned by the engine.
* **Validation**: Handles are checked before operations are performed. A handle becomes invalid when its referenced object is deleted or destroyed.

---

# 4. Runtime State

The VM stores changes made to cells during Play mode separately from the authored world data.

* **Writes**: When a script changes a cell property such as `.color`, the new value is stored in the world's temporary `runtime_state` map.
* **Reads**: When the engine needs the current value of a property, it checks `runtime_state` first. If no runtime value exists, it uses the authored cell data.
* **Cleanup**: `runtime_state` is discarded when Play mode stops. The authored world remains unchanged.

---

# 5. Native Bridge

The VM communicates with AeoEngine through the `EngineHost` trait.

* **Function Dispatch**: Global functions such as `getAllCellsOfClass` are routed to the host implementation.
* **Property and Method Resolution**: Member access on handles is resolved by the host. The host determines which properties can be read, written, or called.

---

# 6. Diagnostics and Error Handling

The VM generates structured `LogRecord` diagnostics.

* **Context Tracking**: Log entries include the originating script path, entity name, and function or event context.
* **Fiber Errors**: If a fiber encounters an error, such as division by zero or an invalid API call, the VM logs the error and terminates that fiber. Other fibers continue running.
* **Terminal Integration**: Log records are collected from the scene and displayed in the editor terminal with severity-based coloring.
