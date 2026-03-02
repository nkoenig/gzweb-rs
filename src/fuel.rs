//! Fuel asset server integration for Gazebo.
//!
//! Gazebo simulations reference Fuel assets with URIs containing
//! `fuel.gazebosim.org`. These arrive as local filesystem paths from the
//! WebSocket and must be converted to HTTPS Fuel API URLs for fetching.
//! Assets are fetched over HTTP, kept in memory, and rendered.
//!
//! # Architecture
//!
//! A custom Bevy `AssetSource` named `"fuel"` is registered so that:
//!
//! ```text
//! asset_server.load("fuel://openrobotics/models/robot/1/files/meshes/body.stl")
//! ```
//!
//! triggers an HTTP fetch from:
//!
//! ```text
//! https://fuel.gazebosim.org/1.0/openrobotics/models/robot/1/files/meshes/body.stl
//! ```

use bevy::prelude::*;
use bevy::asset::io::{
    AssetReader, AssetReaderError, AssetSourceBuilder, PathStream, Reader, VecReader,
};
use std::path::Path;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Primary Fuel server hostname.
pub const FUEL_HOST: &str = "fuel.gazebosim.org";

/// Fuel API version used when constructing URLs.
pub const FUEL_VERSION: &str = "1.0";

/// Legacy Fuel server hostname (Ignition era).
pub const IGN_FUEL_HOST: &str = "fuel.ignitionrobotics.org";

// ─── URI helpers ─────────────────────────────────────────────────────────────

/// Returns `true` if the URI references a Gazebo Fuel asset.
pub fn is_fuel_uri(uri: &str) -> bool {
    uri.contains(FUEL_HOST) || uri.contains(IGN_FUEL_HOST)
}

/// Convert a Gazebo Fuel URI into a Bevy `fuel://` asset path.
///
/// Local paths like:
///   `/home/user/.gazebo/fuel/fuel.gazebosim.org/owner/models/name/1/meshes/body.stl`
/// become:
///   `fuel://owner/models/name/1/files/meshes/body.stl`
///
/// HTTPS URLs like:
///   `https://fuel.gazebosim.org/1.0/owner/models/name/1/files/meshes/body.stl`
/// become:
///   `fuel://owner/models/name/1/files/meshes/body.stl`
pub fn create_fuel_asset_path(uri: &str) -> String {
    // ── Already a proper HTTPS Fuel URL ──────────────────────────────────
    for host in [FUEL_HOST, IGN_FUEL_HOST] {
        let https_versioned = format!("https://{}/{}/", host, FUEL_VERSION);
        if uri.starts_with(&https_versioned) {
            return format!("fuel://{}", &uri[https_versioned.len()..]);
        }
        let https_host = format!("https://{}/", host);
        if uri.starts_with(&https_host) {
            return format!("fuel://{}", &uri[https_host.len()..]);
        }
    }

    // ── Local filesystem path containing the Fuel hostname ───────────────
    for host in [FUEL_HOST, IGN_FUEL_HOST] {
        if let Some(idx) = uri.find(host) {
            let after_host = &uri[idx + host.len()..];
            let parts: Vec<&str> = after_host
                .split('/')
                .filter(|s| !s.is_empty())
                .collect();

            // Expected: owner / models / name / version / <rest>
            if parts.len() >= 5 {
                let prefix = parts[..4].join("/");
                let suffix = parts[4..].join("/");
                return format!("fuel://{}/files/{}", prefix, suffix);
            } else if !parts.is_empty() {
                return format!("fuel://{}", parts.join("/"));
            }
        }
    }

    uri.to_string()
}

/// Reconstruct the full HTTPS Fuel URL from the asset path portion.
fn fuel_path_to_url(path: &Path) -> String {
    format!(
        "https://{}/{}/{}",
        FUEL_HOST,
        FUEL_VERSION,
        path.to_string_lossy()
    )
}

// ─── Custom AssetReader ──────────────────────────────────────────────────────

/// Fetches assets from the Fuel server over HTTPS.
struct FuelAssetReader;

impl AssetReader for FuelAssetReader {
    async fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<impl Reader + 'a, AssetReaderError> {
        let url = fuel_path_to_url(path);
        info!("FuelAssetReader: fetching {}", url);

        let bytes = fetch_bytes(&url).await.map_err(|e| {
            warn!("FuelAssetReader: failed to fetch '{}': {}", url, e);
            AssetReaderError::NotFound(path.to_path_buf())
        })?;

        Ok(VecReader::new(bytes))
    }

    async fn read_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<impl Reader + 'a, AssetReaderError> {
        // No .meta files on Fuel server
        Err::<VecReader, _>(AssetReaderError::NotFound(
            path.to_path_buf(),
        ))
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

// ─── Platform-specific HTTP fetch ────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    // ureq is blocking, but Bevy runs AssetReader futures on IoTaskPool
    // threads where blocking is acceptable.
    let response = ureq::get(url)
        .call()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let bytes = response
        .into_body()
        .read_to_vec()
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    Ok(bytes)
}

#[cfg(target_arch = "wasm32")]
async fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, Response};

    let mut opts = RequestInit::new();
    opts.method("GET");

    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;

    let window = web_sys::window().ok_or("No window object")?;
    let resp_js = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;

    let resp: Response = resp_js
        .dyn_into()
        .map_err(|_| "Response is not a Response object".to_string())?;

    if !resp.ok() {
        return Err(format!("HTTP {} for {}", resp.status(), url));
    }

    let array_buffer = JsFuture::from(
        resp.array_buffer()
            .map_err(|e| format!("Failed to get array buffer: {:?}", e))?,
    )
    .await
    .map_err(|e| format!("Failed to read array buffer: {:?}", e))?;

    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let mut bytes = vec![0u8; uint8_array.length() as usize];
    uint8_array.copy_to(&mut bytes);

    Ok(bytes)
}

// ─── Bevy Plugin ─────────────────────────────────────────────────────────────

/// Registers the `fuel://` asset source so that Bevy's `AssetServer`
/// can load assets from the Gazebo Fuel server over HTTPS.
///
/// **Must be added before `DefaultPlugins`** (since `AssetPlugin` builds
/// sources during init).
pub struct FuelPlugin;

impl Plugin for FuelPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_source(
            "fuel",
            AssetSourceBuilder::new(|| Box::new(FuelAssetReader)),
        );
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_fuel_uri() {
        assert!(is_fuel_uri(
            "https://fuel.gazebosim.org/1.0/openrobotics/models/robot/1/files/body.stl"
        ));
        assert!(is_fuel_uri(
            "/home/user/.gazebo/fuel/fuel.gazebosim.org/openrobotics/models/robot/1/meshes/body.stl"
        ));
        assert!(is_fuel_uri(
            "/tmp/fuel.ignitionrobotics.org/owner/models/robot/1/meshes/body.glb"
        ));
        assert!(!is_fuel_uri("/home/user/my_model/meshes/body.stl"));
        assert!(!is_fuel_uri("file:///absolute/path.stl"));
    }

    #[test]
    fn test_local_path_to_fuel_asset_path() {
        let input = "/home/user/.gazebo/fuel/fuel.gazebosim.org/openrobotics/models/robot/1/meshes/body.stl";
        assert_eq!(
            create_fuel_asset_path(input),
            "fuel://openrobotics/models/robot/1/files/meshes/body.stl"
        );
    }

    #[test]
    fn test_https_url_to_fuel_asset_path() {
        let input = "https://fuel.gazebosim.org/1.0/openrobotics/models/robot/1/files/meshes/body.stl";
        assert_eq!(
            create_fuel_asset_path(input),
            "fuel://openrobotics/models/robot/1/files/meshes/body.stl"
        );
    }

    #[test]
    fn test_non_fuel_passthrough() {
        let input = "file:///home/user/model/meshes/body.stl";
        assert_eq!(create_fuel_asset_path(input), input);
    }
}
