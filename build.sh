#!/bin/bash
set -e

# Check if wasm-bindgen is installed and matches version
REQUIRED_VERSION="0.2.108"

install_wasm_bindgen() {
    echo "Installing wasm-bindgen-cli v$REQUIRED_VERSION..."
    cargo install wasm-bindgen-cli --version $REQUIRED_VERSION
}

if ! command -v wasm-bindgen &> /dev/null; then
    install_wasm_bindgen
else
    INSTALLED_VERSION=$(wasm-bindgen --version | awk '{print $2}')
    if [ "$INSTALLED_VERSION" != "$REQUIRED_VERSION" ]; then
        echo "wasm-bindgen version mismatch. Installed: $INSTALLED_VERSION, Required: $REQUIRED_VERSION"
        install_wasm_bindgen
    fi
fi

echo "Building WASM binary..."
cargo build --target wasm32-unknown-unknown --release

echo "Generating JS bindings..."
wasm-bindgen --out-dir ./target/wasm32-unknown-unknown/release/ --target web ./target/wasm32-unknown-unknown/release/bevy_webgpu_demo.wasm

echo "Build complete."
