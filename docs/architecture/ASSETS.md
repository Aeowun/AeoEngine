# Asset Architecture

AeoEngine uses project-owned assets so authored Worlds can reference resources belonging to the project.

The current asset system includes texture and audio assets used by the renderer and runtime systems.

---

# 1. Project Asset Ownership

Project assets live inside the project directory.

Current asset areas include:

```text
.assets/
├── textures/
├── audio/
└── skybox/
```

The exact contents may grow as additional asset types are introduced.

The important rule is that authored project data references project-owned resources rather than arbitrary external runtime paths.

---

# 2. Texture Assets

Project textures are stored under:

```text
.assets/textures/
```

The editor can import supported PNG textures into the project.

The asset workflow:

```text
External texture
      ↓
Editor import
      ↓
.assets/textures/
      ↓
Authored asset reference
      ↓
Renderer
```

---

# 3. Texture References

Authored Cells reference project texture resources through project-owned identifiers or paths.

The original external source location is not the runtime dependency.

This allows a project to continue using an imported texture after the source file has been moved or is no longer part of the project workflow.

---

# 4. Texture Loading

The renderer resolves project texture references when resources are needed.

Loaded GPU textures are cached so repeated references can share runtime resources instead of creating duplicate GPU textures unnecessarily.

A fallback texture is used when a required texture cannot be loaded.

The fallback is a renderer resource; it does not modify the authored Cell's texture reference.

---

# 5. Skybox Assets

World sky environments use project-owned skybox assets.

Current skybox resources are stored under:

```text
.assets/skybox/
```

The renderer supports single-texture horizontal-cross cubemap layouts.

The World stores the authored sky configuration and renderer resources are derived from that reference.

---

# 6. Audio Assets

Audio assets are project-owned resources used by AudioEmitter Cells.

Current audio assets are stored under:

```text
.assets/audio/
```

The editor supports audio asset browsing/import workflows and runtime audio playback through the audio system.

The authored Cell stores the audio reference and playback-related authored properties.

Runtime playback state remains temporary.

---

# 7. Asset and Runtime Boundaries

Asset references are persistent project data.

Loaded renderer/audio/GPU resources are runtime resources.

```text
Project Asset
      ↓
Authored Reference
      ↓
Runtime Resource Loading
      ↓
GPU / Audio Runtime
```

Stopping Play mode does not delete the project asset or its authored reference.

---

# 8. Editor Integration

The editor uses asset workflows for supported resource types.

Current editor responsibilities include:

* Importing supported assets.
* Browsing project assets.
* Selecting asset references.
* Showing asset-related properties.
* Providing project paths to runtime systems.

The editor does not own the runtime GPU or audio resource itself.

---

# 9. Missing Assets

A missing asset should produce a safe runtime result.

For textures, the renderer uses a fallback resource when appropriate.

For other resources, the owning subsystem should report a useful diagnostic rather than silently creating unrelated project state.

---

# 10. Runtime Resource Ownership

Different systems own the resources they load:

```text
Renderer
→ GPU texture / mesh resources

AudioSystem
→ Runtime audio resources

Editor
→ Asset selection/import state

World
→ Authored asset references
```

The persistent asset reference remains part of project data.

---

# 11. Current Scope

The asset architecture is intentionally centered on concrete engine needs.

Current project-owned resource categories include:

* Textures.
* Skybox resources.
* Audio assets.

Additional asset types should follow the same general model:

```text
Project-owned resource
      ↓
Stable authored reference
      ↓
Owning runtime subsystem
      ↓
Runtime resource
```

Asset-system growth should follow actual engine requirements rather than introducing a general asset abstraction without a demonstrated need.