````markdown
# Rendering Architecture

AeoEngine uses an OpenGL-based renderer to display authored World geometry, runtime physical objects, characters, lights, the editor grid, and other runtime/editor visuals.

Rendering consumes current engine state. It is not the authoritative owner of authored World data.

---

# 1. Renderer Ownership

The renderer owns GPU-facing resources and rendering operations.

The World, PhysicsWorld, CharacterSystem, and Editor provide the state that the renderer needs to display.

```text
World / Runtime State
        ↓
Renderer
        ↓
OpenGL
        ↓
Screen
```

The renderer may maintain derived runtime resources such as:

* Spatial render chunks.
* Chunk meshes.
* GPU vertex and index buffers.
* Texture resources.
* Shadow geometry.
* Other graphics resources.

These resources are derived from current engine state and are not authoritative authored World data.

---

# 2. Authored World Rendering

Authored World Cells are rendered from the current effective World state.

Runtime overrides are resolved when determining the effective values used for rendering.

For example, a runtime script can make a Cell invisible without changing its authored visibility.

The renderer therefore displays the effective current state while preserving the distinction between authored values and temporary runtime overrides.

---

# 3. Runtime Physics Rendering

Dynamic PhysicsBodies have runtime state independent of authored World coordinates.

The renderer uses the runtime body representation when displaying moving physical objects.

This prevents rendering from assuming that an authored grid position is always the object's current gameplay position.

Static World collision remains owned by `PhysicsWorld` and is not derived from renderer geometry resources.

---

# 4. Character Rendering

The CharacterSystem supplies evaluated runtime character state and mesh information to the renderer.

Character rendering includes the generated character geometry and evaluated animation pose.

Character runtime state remains owned by the CharacterSystem rather than the renderer.

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

Lighting representation used for rendering is derived from the current effective World and runtime state.

---

# 6. Skybox and Environment

The renderer implements a seamless sky environment using OpenGL cubemaps (`GL_TEXTURE_CUBE_MAP`).

* **Cubemap Geometry**: A standard skybox cube is rendered at "infinite" distance with the depth value configured to keep the environment behind World geometry.
* **Asset Support**: Loads single-texture horizontal cross layouts from `.assets/skybox/`.
* **Parallax Lock**: The skybox follows the active camera position to eliminate translation parallax while allowing camera rotation.

The sky environment is a rendering representation and does not become part of authored Cell geometry.

---

# 7. Gameplay Camera

The renderer uses the active camera state provided by the editor or `GameplayCamera`.

Editor mode uses the editor camera.

Play mode can use the `GameplayCamera` when a runtime gameplay camera is active.

Editor camera state and gameplay camera state remain separate.

---

# 8. Textures

Project-owned textures are imported into the project's texture asset directory and referenced by asset identifiers.

The renderer loads imported textures dynamically and caches loaded texture resources.

A fallback texture is used when a requested asset is missing or unavailable.

Texture resources are runtime renderer resources derived from project asset references.

---

# 9. Editor Rendering

The renderer also displays editor-specific visual state including:

* Editor grid.
* Cell selection outlines.
* Ghost previews.
* Center/anchor indicators.
* Other authoring overlays.

These visuals are editor representations and are not authored World content by themselves.

Editor overlays may be rebuilt independently of persistent World geometry.

---

# 10. Performance and Spatial Chunking Architecture

Static voxel geometry is rendered using persistent spatial chunks with:

```text
CHUNK_SIZE = 16
```

The current rendering pipeline is:

```text
World
  ↓
Effective Cell State
  ↓
Spatial Chunking
  ↓
Affected Chunk Detection
  ↓
Chunk Mesh Construction
  ↓
Texture Batching
  ↓
Camera Frustum Culling
  ↓
GPU Draw Submission
```

The renderer maintains persistent chunk mesh resources rather than reconstructing the complete World render representation for every localized change.

### Key Architectural Boundaries

* **Ownership**: `World` owns authored and runtime Cell data. `Renderer` owns derived GPU chunk meshes (`ChunkMesh`). `PhysicsWorld` owns collision geometry independently.
* **Exposed-Face Meshing**: Each Cell in a chunk is evaluated using exposed-face culling (`compute_exposed_faces_main` and `compute_exposed_faces_shadow`) across chunk boundaries. Internal faces are not generated when neighboring geometry makes them unnecessary.
* **Texture Batching**: Voxel geometry within each chunk is grouped by texture identifier so compatible geometry can be submitted as a texture batch rather than issuing an independent draw for every Cell.
* **Shadow Meshing**: Separate position-only shadow geometry is generated and persistently uploaded per chunk using shadow eligibility rules (`Block` and `SpawnPoint` with `solid == true`).
* **Selective Rebuilding**: World render changes identify affected chunks so localized edits rebuild only the required chunk meshes rather than reconstructing the complete World render representation.
* **Persistent GPU Resources**: Unchanged chunk meshes remain available for reuse across frames and localized World edits.
* **Visibility Culling**: Camera frustum culling prevents chunks outside the active view from being submitted for rendering.
* **Stable Unchanged Frames**: When no World render changes occur, the renderer does not need to rescan the complete World or reallocate GPU buffers for unchanged chunk geometry.

The important separation is:

```text
Authored World
      ↓
Derived Chunk Representation
      ↓
GPU Resources
```

The chunk representation is a rendering cache. It can be destroyed and rebuilt without changing the authored World.

---

# 11. Renderer Update Behavior

Renderer updates are driven by changes to effective World state and runtime rendering state.

A localized World edit should follow the general path:

```text
World Change
     ↓
Affected Chunk(s)
     ↓
Rebuild Required Chunk Meshes
     ↓
Upload Updated GPU Resources
```

An unchanged World does not require a complete World mesh reconstruction every frame.

Runtime state changes that affect only dynamic objects can update those runtime render representations without rebuilding unrelated static voxel chunks.

This keeps static World geometry and dynamic gameplay rendering on separate update paths.

---

# 12. Renderer Performance Characteristics

The renderer benchmark demonstrates the intended scaling behavior of selective chunk rebuilding.

Representative 3D cube results:

| Shape | Size | Full CPU Build | Selective Rebuild |
| ----- | ----: | -------------: | ----------------: |
| 3D cube | 50K | 230.6 ms | 17.46 ms |
| 3D cube | 75K | 334.9 ms | 17.89 ms |
| 3D cube | 100K | 436.8 ms | 17.51 ms |
| 3D cube | 200K | 876.0 ms | 17.92 ms |

Representative 2D plane results:

| Shape | Size | Full CPU Build | Selective Rebuild |
| ----- | ----: | -------------: | ----------------: |
| 2D plane | 50K | 458.6 ms | 2.37 ms |
| 2D plane | 100K | 931.2 ms | 2.30 ms |
| 2D plane | 200K | 1.85 s | 2.32 ms |

These results show the distinction between:

* Full reconstruction, whose cost increases with total World size.
* Selective rebuilding, whose cost is primarily associated with the affected render region.

The renderer benchmark currently contains a known pathological 1D line distribution at large sizes:

```text
30K → approximately 2.19 s full build
50K → benchmark cutoff
```

This case is not representative of normal dense voxel World construction and remains a pathological geometry-distribution workload.

The renderer benchmark should continue to be used when making rendering performance changes.

A successful benchmark run is expected to report:

```text
test_benchmark_large_world_performance_cliff ... ok
1 passed, 0 failed
```

---

# 13. Authority

The renderer represents state; it does not author it.

```text
Authored World
      ↓
Effective runtime/editor state
      ↓
Renderer
      ↓
GPU
```

The renderer may cache and transform that state into chunk meshes, GPU buffers, texture resources, and other derived representations.

Those representations are temporary runtime resources.

The authored World remains authoritative for persistent scene data.

---

# 14. Architectural Principle

The renderer is a consumer and transformer of engine state.

Its responsibilities are:

* Determine the graphics representation of current state.
* Maintain derived render resources.
* Build and rebuild spatial chunk meshes.
* Perform render visibility decisions.
* Submit graphics work to OpenGL.
* Maintain rendering-specific caches and GPU resources.

Its responsibilities do not include:

* Persisting authored World data.
* Owning physics state.
* Owning character simulation state.
* Owning script execution state.
* Converting temporary runtime state into authored World state.

The rendering architecture therefore follows:

```text
Authored World
      ↓
Effective State
      ↓
Render Representation
      ↓
Spatial Chunks
      ↓
Chunk Meshes
      ↓
GPU
```

while runtime simulation follows its own derived path:

```text
Authored World
      ↓
Runtime Conversion
      ↓
Physics / Characters / Scripts / Camera
      ↓
Effective Runtime State
      ↓
Renderer
```

The renderer remains a derived representation layer between engine state and the GPU.
````
