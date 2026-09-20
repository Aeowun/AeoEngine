# Asset Architecture

AeoEngine uses project-owned assets so authored Worlds can refer to resources belonging to the project rather than relying on arbitrary external filesystem paths.

The current asset workflow is most developed around project-owned textures.

---

# 1. Project Asset Ownership

Project resources live inside the project directory.

Texture assets are stored under:

~~~text
.assets/textures/
~~~

This keeps imported project resources alongside the World and other project data.

---

# 2. Texture Import

The editor can import PNG textures through the asset workflow.

The importer:

1. Ensures the project texture directory exists.
2. Copies the selected texture into `.assets/textures`.
3. Assigns a project texture identifier.
4. Allows authored Cells to reference the imported asset.

---

# 3. Texture Identifiers

Authored texture references use project asset identifiers rather than depending on the original external file location.

This allows the project to own the resource and continue using it after import.

---

# 4. Renderer Integration

The renderer resolves project texture identifiers when loading visual resources.

Loaded textures are cached so repeated use does not require recreating the same GPU texture resource for every reference.

A fallback texture is used when a referenced asset cannot be loaded.

---

# 5. Editor Integration

The Properties and Build workflows can expose texture selection for supported Cell types.

The asset importer filters the texture workflow to supported PNG files.

---

# 6. Runtime Boundary

Asset references are authored project data.

The renderer's loaded GPU texture objects are runtime resources.

~~~text
Project asset
      ↓
Authored asset reference
      ↓
Renderer resource loading
      ↓
GPU texture
~~~

Stopping Play mode does not remove the project asset or its authored reference.

---

# 7. Current Scope

The current asset system is centered on project-owned textures.

Future asset types can use the same principle:

* Project-owned resource.
* Stable project reference.
* Runtime resource loading.
* Explicit fallback/error behavior.

The asset system should grow only as concrete engine features require additional resource types.

