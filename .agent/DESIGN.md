# gzweb-rs — Architecture & Design Patterns

## Core Architecture

The application is structured as a **Bevy ECS application** with three functional layers:

```
┌─────────────────────────────────────────────────────┐
│  Bevy App (ECS)                                     │
│  ┌───────────┐  ┌──────────────┐  ┌─────────────┐  │
│  │  main.rs  │  │  scene.rs    │  │websocket.rs │  │
│  │ App setup │  │ Scene →      │  │ WS bridge   │  │
│  │ UI overlay│  │ Bevy entities│  │ (native/wasm│  │
│  └───────────┘  └──────────────┘  └─────────────┘  │
│                        ↑                  ↓         │
│                  gz_msgs.rs        crossbeam channel │
│              (protobuf structs)                      │
└─────────────────────────────────────────────────────┘
         ↕ WebSocket (ws://localhost:<port>)
┌─────────────────────────────────────────────────────┐
│  Gazebo Simulation  (gz-transport websocket plugin)  │
└─────────────────────────────────────────────────────┘
```

---

## Platform Abstraction Pattern

The same codebase targets **WASM** and **native** using Rust's `cfg` conditional compilation. Use the following idiom for platform-specific code:

```rust
// Native-only block
#[cfg(not(target_arch = "wasm32"))]
{
    // tungstenite, std::thread, std::env, etc.
}

// WASM-only block
#[cfg(target_arch = "wasm32")]
{
    // web-sys, wasm-bindgen, JS callbacks, etc.
}
```

**Native-only Cargo dependencies** go under:
```toml
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
```

**WASM-only Cargo dependencies** go under:
```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
```

---

## Bevy ECS Conventions

### Resources

| Resource | Type | Purpose |
|----------|------|---------|
| `GzWebSocket` | `NonSendResource` | WebSocket state; **must be NonSend** (contains JS callbacks on WASM) |
| `SceneState` | `Resource` | Scene loading state machine |
| `AmbientLight` | `Resource` | Set from gz scene data |
| `ClearColor` | `Resource` | Background colour from gz scene |

> **IMPORTANT**: `GzWebSocket` must always be registered as a `NonSendResource` (`world.insert_non_send_resource(...)`) and accessed via `Option<NonSendMut<GzWebSocket>>`. This is required because the WASM `web_sys::WebSocket` is not `Send`.

### Components

| Component | Purpose |
|-----------|---------|
| `GzSceneEntity { gz_name }` | Marker on all entities spawned from Gazebo scene |
| `FpsText` | Marker for FPS overlay text |
| `AdapterText` | Marker for GPU backend overlay text |
| `WebsocketStatusText` | Marker for WS status overlay text |
| `MainCamera` | Marker for the primary 3D camera |
| `PanOrbitCamera` | Camera controller (from `bevy_panorbit_camera`) |

### Systems and Ordering

Systems are registered in `main.rs`:

```
Startup:
  setup                   — camera, lights, UI text entities
  setup_websocket_system  — WebSocket init (NonSend resource)

Update (every frame):
  update_fps              — FPS Bevy overlay
  update_adapter_info     — GPU backend Bevy overlay
  update_dom_status       — DOM status element (WASM only)
  update_orbit_focus      — raycast-based orbit focus on left click
  update_websocket_status — drain WS channel, update status text
  process_scene           — scene loading state machine
```

---

## WebSocket Message Protocol

Gazebo uses `gz-transport` over WebSocket with a custom framing format.

### Text messages (JSON)
Outgoing requests are JSON arrays:
```json
["protos", "", "", ""]      // Request proto definitions
["worlds", "", "", ""]      // Request list of worlds
["scene", "<world>", "", ""] // Request scene for a world
```

Incoming:
- Proto definition text (stored in `GzWebSocket.protos`)
- `"authorized"` — re-request protos after auth
- `"invalid"` — auth failed

### Binary messages (gz-transport frame)
```
<operation>,<topic>,<msgType>,<protobuf_payload>
```
Where the first three comma-delimited fields are ASCII and the remainder is a raw protobuf payload.

Current handled topics:
- `pub/worlds` → decodes `gz.msgs.StringMsgV` → stores world name in `SceneState`
- `pub/scene`  → stores raw bytes in `GzWebSocket.scene_data` for protobuf decode

### Scene Loading State Machine (`SceneState`)

```
[Initial]
    │ protos received
    ▼
[Request Worlds] → worlds_requested = true
    │ world name received
    ▼
[Request Scene]  → scene_requested = true
    │ scene_data received
    ▼
[Decode & Spawn] → loaded = true
```

---

## Protobuf Structs (`gz_msgs.rs`)

Protobuf structs are **hand-written** using `prost` derive macros — there is no `.proto` file compile step. When adding new message types, follow the existing pattern in `gz_msgs.rs`:

```rust
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MyNewMsg {
    #[prost(string, tag = "1")]
    pub name: String,
    // ...
}
```

Field tag numbers must match the actual `gz.msgs` proto definitions from the Gazebo source.

---

## Scene → Bevy Entity Mapping

Scene spawning follows the hierarchy in `scene.rs`:

```
gz::Scene
├── Light[]         → PointLight / DirectionalLight / SpotLight
└── Model[]
    ├── Visual[]    → Mesh3d + MeshMaterial3d
    ├── Link[]
    │   ├── Visual[]
    │   └── Light[]
    └── Model[]     (nested, recursive)
```

### Geometry Support

| gz geometry | Bevy mesh |
|-------------|-----------|
| Box | `Cuboid` |
| Cylinder | `Cylinder` |
| Sphere | `Sphere` |
| Plane | `Plane3d` |
| Capsule | `Capsule3d` |
| Cone | `Cone` |
| Mesh | ⚠️ Not yet implemented |

### Coordinate Transforms

Gazebo uses right-hand Z-up; Bevy uses right-hand Y-up. Transforms are applied using `combine_transforms(parent, child)` which correctly composes position + rotation + scale through the model/link/visual hierarchy.

> **TODO**: A coordinate system swap (Gz → Bevy) may be required for correct orientation. Currently transforms are passed through without axis remapping.

### Materials

Materials are mapped from `gz.msgs.Material` to Bevy `StandardMaterial` (PBR):
- Diffuse → `base_color`
- Emissive → `emissive`
- PBR metalness/roughness → `metallic` / `perceptual_roughness`
- Transparency → `AlphaMode::Blend`
- Double-sided → `double_sided` + `cull_mode: None`

---

## Rendering — WebGPU Priority

- Bevy is configured with both `webgpu` AND `webgl2` features enabled.
- **WebGPU is preferred**: the backend is selected by the browser; Chrome 113+ will use WebGPU.
- On native, wgpu selects Vulkan/Metal/DX12 automatically.
- The GPU backend is displayed in the HUD (green = modern API, blue = WebGL, yellow = other).
- Release profile uses `opt-level = "s"`, `lto = true`, `codegen-units = 1` for compact WASM size.

Do **not** remove the `webgpu` Bevy feature — it is a hard project requirement.

---

## UI Overlay

Two parallel overlay systems exist:

1. **Bevy UI** (works on both platforms): FPS, GPU backend, WS status, controls hint — spawned as Bevy `Text` entities with `Node` positioning.
2. **DOM manipulation** (WASM only): `#bevy-status` element in `index.html` is updated directly via `web_sys` to show status even before the canvas renders.

---

## Known Limitations & Future Work

- **Mesh loading**: `gz::geometry::Mesh` (external `.dae`/`.obj` files) is not yet implemented.
- **Dynamic updates**: Only the initial scene load is handled. Live entity updates (model pose changes, spawns/deletions) are not yet supported.
- **Coordinate system**: Potential Y/Z axis swap between Gazebo (Z-up) and Bevy (Y-up) not yet applied.
- **Authentication**: Partial support — handles `authorized`/`invalid` messages but no credential input UI.
- **Texture/material loading**: Only vertex colors and PBR properties; no texture maps yet.
- **Model file loading**: Gazebo model resources (meshes, textures) need a resource server or bundling strategy for WASM.

---

## Code Style Guidelines

- Use Bevy's standard `prelude::*` imports.
- All platform-specific code must use `#[cfg(...)]` — no runtime feature flags.
- New Gazebo message types go in `gz_msgs.rs` following the existing prost pattern.
- New scene geometry types go in `scene.rs` in the `spawn_visual` match block.
- Use `info!` / `warn!` / `error!` from `bevy::prelude` (re-exports `tracing`).
- `GzWebSocket` must always remain a `NonSendResource`.
- Prefer `Option<NonSendMut<GzWebSocket>>` in system parameters to gracefully handle init.
