# Textures

AeoEngine supports project-owned texture assets through the editor's asset workflow.

---

# 1. Project Texture Location

Imported textures are stored in:

~~~text
.assets/textures/
~~~

The directory is created automatically when needed.

---

# 2. Importing a Texture

Use the editor's texture import workflow.

The current import path supports PNG texture files.

The importer copies the selected resource into the project instead of relying on the original external file location.

---

# 3. Texture References

Imported textures receive project asset identifiers.

Authored Cells can refer to those identifiers where texture properties are supported.

This makes the project asset, rather than the original source path, the important resource reference.

---

# 4. Renderer Loading

At runtime the renderer resolves the project texture reference and loads the corresponding GPU texture resource.

Loaded textures are cached for reuse.

---

# 5. Missing Textures

If a texture asset cannot be found or loaded, the renderer can use a fallback texture rather than leaving the render path without a valid texture resource.

This makes missing-asset behavior visible without making the renderer depend on a successful external file lookup.

---

# 6. Authored vs Runtime

The authored texture reference is project data.

The loaded OpenGL texture resource is runtime rendering state.

~~~text
Project asset
      ↓
Authored texture reference
      ↓
Renderer loading
      ↓
GPU texture resource
~~~

Stopping Play does not delete project texture assets.

---

# 7. Texture Workflow

A typical workflow is:

~~~text
Prepare PNG
 ↓
Import into project
 ↓
Select Cell
 ↓
Assign texture
 ↓
Save World
 ↓
Run Play
 ↓
Verify rendering
~~~

---

# 8. Debugging Texture Problems

When a texture does not appear correctly, verify:

1. The file exists in `.assets/textures`.
2. The authored asset reference is correct.
3. The renderer can resolve the asset identifier.
4. The renderer does not report a missing resource.
5. The fallback texture behavior is not masking a missing asset.

---

# 9. Current Scope

The current project-owned asset workflow is centered on textures.

Additional asset types should follow the same general pattern of project ownership, stable references, runtime loading, and explicit failure/fallback behavior.

