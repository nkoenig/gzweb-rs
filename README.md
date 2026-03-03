# Bevy WebGPU Demo

A Rust/Bevy application that connects to a Gazebo simulation via WebSocket, deserializes the scene (models, lights, geometry, materials), and renders it in 3D using Bevy's rendering engine. Targets both native desktop and WebAssembly (browser via WebGPU).

## Features

- Connects to the Gazebo websocket transport server
- Deserializes `gz.msgs.Scene` protobuf messages (using manually defined [prost](https://crates.io/crates/prost) structs)
- Spawns Bevy entities for lights (Point, Directional, Spot) and model visuals
- Supports primitive geometry: Box, Cylinder, Sphere, Plane, Capsule, Cone
- Applies materials (diffuse color, emissive, PBR metalness/roughness, transparency)
- Pan-orbit camera controls (left-drag to orbit, right-drag to pan, scroll to zoom)

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- `wasm32-unknown-unknown` target (for WASM builds): `rustup target add wasm32-unknown-unknown`
- A running [Gazebo](https://gazebosim.org/) simulation with the websocket server plugin

## Running Gazebo (Docker)

To easily run a Gazebo Harmonic simulation with the websocket server enabled, you can use the provided `Dockerfile.gazebo`. This creates an isolated environment with Gazebo configured to expose its websocket server on port `9002`, which is required for this application to connect.

### Build the Image

```bash
docker build -t gz-websocket -f Dockerfile.gazebo .
```

### Run the Container

You can start the simulation by running the container. By default, it will load `empty.sdf`:

```bash
docker run --rm -p 9002:9002 gz-websocket
```

To specify a different world (e.g., `shapes.sdf`), pass it as an argument:

```bash
docker run --rm -p 9002:9002 gz-websocket shapes.sdf
```

## Building

### Native (Desktop)

```bash
cargo build --release
```

### WebAssembly

```bash
./build.sh
```

This compiles to WASM, installs the required version of `wasm-bindgen-cli` if needed, and generates JS bindings in `target/wasm32-unknown-unknown/release/`.

## Running

### Native

```bash
cargo run
```

### WebAssembly

After building with `./build.sh`, serve the project directory with any HTTP server and open `index.html` in a WebGPU-capable browser (e.g. Chrome 113+).

## Configuring the WebSocket Port

The application connects to `ws://localhost:<port>`. The default port is **9002**.

### Compile-time (env var)

Set `GZ_WEBSOCKET_PORT` when building to bake in a different default:

```bash
GZ_WEBSOCKET_PORT=9090 cargo build --release
# or for WASM:
GZ_WEBSOCKET_PORT=9090 ./build.sh
```

### Runtime — Native (env var)

On native builds, `GZ_WEBSOCKET_PORT` is also checked at **runtime** and takes precedence over the compile-time default:

```bash
GZ_WEBSOCKET_PORT=9090 cargo run
```

### Runtime — WASM (URL query parameter)

When running in the browser, append `?port=<port>` to the URL:

```
http://localhost:8000/index.html?port=9090
```

The URL parameter takes precedence over the compile-time default.
