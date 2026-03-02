//! WebSocket-based asset proxy for loading `file://` assets in WASM.
//!
//! In the browser, the app cannot access the local filesystem. When a `file://`
//! URI is encountered on WASM, this module:
//!
//! 1. Converts it to a `wsasset://` path so Bevy's `AssetServer` routes it
//!    to our custom [`WsAssetReader`].
//! 2. The reader sends an `"asset"` request to the Gazebo WebSocket server.
//! 3. A Bevy system receives the response bytes and inserts them into the
//!    shared [`WsAssetResponseStore`].
//! 4. The reader picks up the bytes and returns them to Bevy's loader pipeline.
//!
//! This matches the existing Gazebo WebSocket protocol used by the JS frontend
//! (`Transport.ts` `getAsset` method):
//!
//! ```text
//! Client → Server:  "asset,,,file:///home/user/model/meshes/body.stl"
//! Server → Client:  binary frame "asset,<uri>,<type>,<payload>"
//! ```

use bevy::prelude::*;
use bevy::asset::io::{
    AssetReader, AssetReaderError, AssetSourceBuilder, PathStream, Reader, VecReader,
};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

// ─── Shared response store ──────────────────────────────────────────────────

/// Thread-safe store for asset bytes received from the WebSocket server.
///
/// Shared (via `Arc`) between the [`WsAssetReader`] and the Bevy system that
/// processes incoming WebSocket messages. Both sides use the `file://` URI
/// (without the `file://` prefix) as the key.
pub type SharedResponseStore = Arc<Mutex<HashMap<String, Vec<u8>>>>;

/// Bevy resource wrapper so systems can access the shared response store.
#[derive(Resource, Clone)]
pub struct WsAssetResponseStore(pub SharedResponseStore);

impl Default for WsAssetResponseStore {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }
}

/// Bevy resource: set of `file://` URIs that need to be requested from
/// the WebSocket server. Each URI is added by `scene.rs` when it encounters
/// a WASM `file://` path. A system drains this and sends WebSocket requests.
#[derive(Resource, Default)]
pub struct PendingWsAssetRequests {
    /// URIs waiting to be sent. Key = file path, Value = true if request sent.
    pub requests: HashMap<String, bool>,
}

// ─── Custom AssetReader ─────────────────────────────────────────────────────

/// Reads asset bytes from the shared response store, which is populated
/// by WebSocket "asset" responses.
struct WsAssetReader {
    store: SharedResponseStore,
}

impl AssetReader for WsAssetReader {
    async fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<impl Reader + 'a, AssetReaderError> {
        let key = format!("file:///{}", path.to_string_lossy());
        info!("WsAssetReader: waiting for bytes for '{}'", key);

        // Poll the shared store until the WebSocket system delivers our bytes.
        // On WASM, yield_now() gives control back to the browser event loop so
        // WebSocket callbacks and Bevy systems can run between polls.
        let mut polls = 0u32;
        loop {
            {
                let mut store = self.store.lock().unwrap();
                if let Some(bytes) = store.remove(&key) {
                    info!("WsAssetReader: received {} bytes for '{}'", bytes.len(), key);
                    return Ok(VecReader::new(bytes));
                }
            }
            polls += 1;
            if polls % 100 == 0 {
                info!("WsAssetReader: still waiting for '{}' (poll #{})", key, polls);
            }
            if polls > 6000 {
                // ~60 seconds at 10ms per poll
                warn!("WsAssetReader: timeout waiting for '{}'", key);
                return Err(AssetReaderError::NotFound(path.to_path_buf()));
            }
            bevy::tasks::futures_lite::future::yield_now().await;
        }
    }

    async fn read_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<impl Reader + 'a, AssetReaderError> {
        Err::<VecReader, _>(AssetReaderError::NotFound(path.to_path_buf()))
    }

    async fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<Box<PathStream>, AssetReaderError> {
        Err(AssetReaderError::NotFound(path.to_path_buf()))
    }

    async fn is_directory<'a>(
        &'a self,
        _path: &'a Path,
    ) -> Result<bool, AssetReaderError> {
        Ok(false)
    }
}

// ─── System: send asset requests via WebSocket ──────────────────────────────

/// Sends pending asset requests to the Gazebo WebSocket server.
///
/// For each unrequested URI in [`PendingWsAssetRequests`], sends:
/// `"asset,,,<file://uri>"` matching the Gazebo WS protocol.
pub fn send_ws_asset_requests_system(
    mut pending: ResMut<PendingWsAssetRequests>,
    websocket: Option<NonSendMut<crate::websocket::GzWebSocket>>,
) {
    let Some(ws) = websocket else { return };

    for (uri, sent) in pending.requests.iter_mut() {
        if *sent {
            continue;
        }
        let msg = format!("asset,,,{}", uri);
        match ws.send_message(&msg) {
            Ok(()) => {
                info!("AssetProxy: requested '{}' via WebSocket", uri);
                *sent = true;
            }
            Err(e) => {
                warn!("AssetProxy: failed to request '{}': {}", uri, e);
            }
        }
    }
}

// ─── Plugin ─────────────────────────────────────────────────────────────────

/// Registers the `wsasset://` asset source and supporting resources/systems.
///
/// **Must be added before `DefaultPlugins`** (asset sources are built during
/// `AssetPlugin` init).
pub struct AssetProxyPlugin;

impl Plugin for AssetProxyPlugin {
    fn build(&self, app: &mut App) {
        // Create the shared response store
        let store = WsAssetResponseStore::default();
        let reader_store = store.0.clone();

        // Register the "wsasset" source with our custom reader
        app.register_asset_source(
            "wsasset",
            AssetSourceBuilder::new(move || {
                Box::new(WsAssetReader {
                    store: reader_store.clone(),
                })
            }),
        );

        // Insert resources and systems
        app.insert_resource(store)
            .init_resource::<PendingWsAssetRequests>()
            .add_systems(Update, send_ws_asset_requests_system);
    }
}
