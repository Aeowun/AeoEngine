# Rendering Architecture

AeoEngine uses an OpenGL-based renderer to display authored World geometry, runtime physical objects, characters, lights, the editor grid, and other runtime/editor visuals.

Rendering consumes current engine state. It is not the authoritative owner of authored World data.

---

# 1. Renderer Ownership

The renderer owns GPU-facing resources and rendering operations.

The World, PhysicsWorld, CharacterSystem, and Editor provide the state that the renderer needs to display.

~~~text
World / Runtime State
        ↓
Renderer
        ↓
OpenGL
        ↓
Screen
~~~

---

# 2. Authored World Rendering

Authored World Cells are rendered from the current World state.

Runtime overrides are resolved when determining the effective values used for rendering.

For example, a runtime script can make a Cell invisible without changing its authored visibility.

---

# 3. Runtime Physics Rendering

Dynamic PhysicsBodies have runtime state independent of authored World coordinates.

The renderer uses the runtime body representation when displaying moving physical objects.

This prevents rendering from assuming that an authored grid position is always the object's current gameplay position.

---

# 4. Character Rendering

The CharacterSystem supplies evaluated runtime character state and mesh information to the renderer.

Character rendering includes the generated character geometry and evaluated animation pose.

---

# 5. Lighting

The renderer consumes authored and runtime lighting state.

Current lighting support includes directional and point lighting, plus World-level lighting configuration.

World lighting data includes settings such as:

* Global light enablement.
* Direction.
* Color.
* Intensity.
* Ambient intensity.
* Shadow-related settings.

---

# 6. Skybox and Environment

The renderer implements a seamless sky environment using OpenGL cubemaps (`GL_TEXTURE_CUBE_MAP`).

*   **Cubemap Geometry**: A standard skybox cube is rendered at "infinite" distance (depth value of 1.0).
*   **Asset Support**: Loads single-texture horizontal cross layouts from `.assets/skybox/`.
*   **Parallax Lock**: The skybox follows the active camera position to eliminate translation parallax while allowing rotation.

---

# 7. Gameplay Camera

The renderer uses the active camera state provided by the editor or GameplayCamera.

Editor mode uses the editor camera.

Play mode can use the GameplayCamera when a runtime character is active.

---

# 7. Textures

Project-owned textures are imported into the project's texture asset directory and referenced by asset identifiers.

The renderer loads imported textures dynamically and caches loaded texture resources.

A fallback texture is used when a requested asset is missing or unavailable.

---

# 8. Editor Rendering

The renderer also displays editor-specific visual state including:

* Editor grid.
* Cell selection outlines.
* Ghost previews.
* Center/anchor indicators.
* Other authoring overlays.

These visuals are editor representations and are not authored World content by themselves.

---

# 9. Performance & Spatial Chunking Architecture

Static voxel geometry is rendered using persistent spatial chunks (`CHUNK_SIZE = 16`).

The pipeline flow is:

~~~text
World (effective state & render revision)
        ↓
Spatial Chunking (ChunkCoord = WorldCoord / 16)
        ↓
Persistent GPU Chunk Meshes & Texture Batching
        ↓
Camera Frustum Chunk Culling
        ↓
GPU Draw Submission
~~~

### Key Architectural Boundaries:
* **Ownership**: `World` owns authored and runtime cell data. `Renderer` owns derived GPU chunk meshes (`ChunkMesh`). `PhysicsWorld` owns collision geometry independently.
* **Exposed-Face Meshing**: Each cell in a chunk is evaluated against Phase 1 exposed-face culling (`compute_exposed_faces_main` and `compute_exposed_faces_shadow`) across chunk boundaries without generating internal faces at chunk boundaries.
* **Texture Batching**: Voxel geometry within each chunk is grouped contiguous-by-texture-identifier, submitting a single draw call per texture batch per chunk.
* **Shadow Meshing**: Separate position-only shadow geometry is generated and persistently uploaded per chunk using shadow eligibility rules (`Block` and `SpawnPoint` with `solid == true`).
* **Coarse Invalidation Strategy**: When `world.render_revision()` changes, the renderer invalidates and rebuilds its GPU chunk cache. Unchanged frames issue zero CPU world scans or GPU buffer re-allocations.

Potential future rendering optimizations include:

* Selective dirty-chunk rebuilding (Phase 3).
* Greedy meshing (Phase 4).

---

# 10. Authority

The renderer represents state; it does not author it.

~~~text
Authored World
      ↓
Effective runtime/editor state
      ↓
Renderer
      ↓
GPU
~~~

