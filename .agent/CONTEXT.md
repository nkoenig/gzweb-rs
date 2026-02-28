# gzweb-rs — Agent Context

## Project Overview

**gzweb-rs** is a modern Rust application that renders live [Gazebo](https://gazebosim.org/) robotic simulations in 3D. It connects to a running Gazebo instance via WebSocket, receives the scene description as protobuf messages, and renders it using [Bevy](https://bevyengine.org/) with WebGPU.

The application is designed to run **both in a web browser (WebAssembly/WASM) and as a native desktop application** from a single Rust codebase.

### Key Goals

- **WebGPU-first rendering** — performance and high-quality graphics are the top priority.
- **Dual-target**: browser (WASM + WebGPU) and native desktop (Vulkan/Metal/DX12 via wgpu).
- **Real-time** connection to a live Gazebo simulation websocket server.
- **Clean ECS architecture** using Bevy's Entity-Component-System model.

---

## Technology Stack

| Component | Technology | Version |
|-----------|-----------|---------|
| Language | Rust | stable (edition 2021) |
| Game engine / ECS | [Bevy](https://bevyengine.org/) | 0.15 |
| GPU rendering | WebGPU via `wgpu` | 23.0 |
| Web target | `wasm32-unknown-unknown` | — |
| JS/WASM bridge | `wasm-bindgen`, `web-sys`, `js-sys` | 0.2/0.3 |
| Protobuf decode | `prost` | 0.13 |
| Serialisation | `serde` + `serde_json` | 1.0 |
| Camera controls | `bevy_panorbit_camera` | 0.21.0 |
| Native WebSocket | `tungstenite` | 0.26 |
| Channel (threading) | `crossbeam-channel` | 0.5 |
| UUID generation | `uuid` (v4 + js feature) | 1.21 |

---

## Repository Layout

```
gzweb-rs/
├── .agent/              # Agent context & design docs (you are here)
│   ├── CONTEXT.md       # This file — project overview and patterns
│   └── DESIGN.md        # Architecture patterns and design decisions
├── src/
│   ├── main.rs          # App entry point, Bevy App setup, UI overlay systems
│   ├── websocket.rs     # WebSocket abstraction (native + WASM), message routing
│   ├── scene.rs         # Gazebo scene → Bevy entity spawning
│   └── gz_msgs.rs       # Hand-written prost protobuf structs (gz.msgs.*)
├── index.html           # Browser shell (canvas, status overlay, JS init)
├── build.sh             # WASM build script (wasm-bindgen, cargo)
├── run.sh               # Dev HTTP server (python3, port 8000)
├── Cargo.toml           # Workspace manifest
└── Cargo.lock
```

---

## Build & Run

### Native (desktop)

```bash
cargo run
# or
cargo build --release && ./target/release/bevy_webgpu_demo
```

### WASM (browser)

```bash
./build.sh            # compiles to WASM + generates JS bindings
./run.sh              # starts python HTTP server on :8000
# open http://localhost:8000 in Chrome 113+ (WebGPU required)
```

### Configuration — WebSocket Port

| Method | How |
|--------|-----|
| Compile-time default | `GZ_WEBSOCKET_PORT=9002 ./build.sh` |
| Native runtime | `GZ_WEBSOCKET_PORT=9090 cargo run` |
| WASM runtime | `http://localhost:8000?port=9090` |

Default port: **9002**

---

## Prerequisites

- Rust stable + `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`)
- `wasm-bindgen-cli` 0.2.108 (auto-installed by `build.sh`)
- A running **Gazebo simulation** with `gz-transport` websocket server plugin enabled
- Chrome 113+ or any WebGPU-capable browser for the WASM target

---

## See Also

- [`DESIGN.md`](./DESIGN.md) — architecture patterns, ECS design, platform abstractions, and known limitations
