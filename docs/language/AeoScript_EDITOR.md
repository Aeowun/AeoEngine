# AeoScript Editor Workflow

The AeoScript workflow is split between the integrated Monaco script editor and the object-level properties in the AeoEngine editor.

---

# 1. Attaching Scripts (Properties Panel)

Scripts are attached to authored objects (Blocks, Lights, etc.) via the **PROPERTIES** panel.

1.  **Select a Cell**: Click on an object in the world or hierarchy.
2.  **Set Identity**: Assign a gameplay name (e.g., "Door"). This name determines which `entity` block in the script will control this object.
3.  **Attach Script**: Use the "Attach Script" dropdown to choose an `.aeo` file.
4.  **Unique ID Binding**: The engine creates a binding using the object's unique 8-digit ID. This ensures that even if you rename the object or create duplicates, the script remains bound only to that specific instance.

---

# 2. Script Workspace

The **Script Workspace** provides a full-screen environment for authoring logic.

*   **File Explorer**: Lists all `.aeo` files in the project's `scripts/` directory.
*   **Monaco Editor**: Provides standard editing features like syntax highlighting and basic diagnostics.
*   **Save**: Saving a script automatically updates the loaded VM logic. If the game is running (Play Mode), the VM attempts to hot-reload the script without stopping the simulation.

---

# 3. Diagnostics and Output

Diagnostics are routed to two primary locations:

### Integrated Monaco Diagnostics
Syntax and parsing errors appear directly on the lines of code in the editor workspace.

### Print Output Panel
The **PRINT OUTPUT** panel (Terminal) displays runtime messages:
*   `print()` and `debug.log()` output.
*   VM Runtime Errors (e.g., "cannot mutate frozen basket").
*   Engine Warnings (e.g., "[STALE] binding found for missing ID").

---

# 4. Identity Management

*   **Authored Identities**: Multiple objects can share the same name (e.g., "FencePost").
*   **Mass Finding**: Scripts can use `find("FencePost")` to get a collection of all objects sharing that name.
*   **Instance Removal**: Removing a script from one "FencePost" in the Properties panel does not affect other objects with the same name, because the editor manages bindings by unique instance ID.


