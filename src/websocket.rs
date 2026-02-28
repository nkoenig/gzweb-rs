use bevy::prelude::*;
use crossbeam_channel::{bounded, Receiver, Sender};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{MessageEvent, ErrorEvent, CloseEvent};

use crate::scene::SceneState;

/// Represents a message received from the websocket.
#[derive(Debug)]
pub enum WsMessage {
    Text(String),
    Binary(Vec<u8>),
}

#[derive(Component)]
pub struct WebsocketStatusText;

pub struct GzWebSocket {
    pub receiver: Receiver<WsMessage>,
    /// Channel for sending string messages to the websocket (used on both platforms).
    pub cmd_sender: Option<Sender<String>>,
    #[cfg(target_arch = "wasm32")]
    pub socket: Option<web_sys::WebSocket>,
    pub status: String,
    pub protos: Option<String>,
    /// Binary scene data received from the websocket, ready for processing.
    pub scene_data: Option<Vec<u8>>,
}

impl GzWebSocket {
    /// Send a string message through the websocket, abstracting over platform.
    pub fn send_message(&self, msg: &str) -> Result<(), String> {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(socket) = &self.socket {
                socket
                    .send_with_str(msg)
                    .map_err(|e| format!("{:?}", e))
            } else {
                Err("No socket".to_string())
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(sender) = &self.cmd_sender {
                sender
                    .send(msg.to_string())
                    .map_err(|e| format!("{}", e))
            } else {
                Err("No command sender".to_string())
            }
        }
    }
}

pub fn update_websocket_status(
    websocket: Option<NonSendMut<GzWebSocket>>,
    mut query: Query<(&mut Text, &mut TextColor), With<WebsocketStatusText>>,
    mut scene_state: ResMut<SceneState>,
) {
    if let Some(mut ws) = websocket {
        // Check for new messages from the receiver
        while let Ok(msg) = ws.receiver.try_recv() {
            match msg {
                WsMessage::Text(txt) => {
                    if txt.starts_with("STATUS:") {
                        ws.status = txt.replace("STATUS:", "").trim().to_string();
                    } else if txt == "authorized" {
                        info!("Received authorized message, re-requesting protos");
                        let request = serde_json::json!(["protos", "", "", ""]).to_string();
                        if let Err(e) = ws.send_message(&request) {
                            error!("Failed to send protos request: {}", e);
                        }
                        ws.status = "Authorized, Requesting Protos...".to_string();
                    } else if txt == "invalid" {
                        error!("Invalid key");
                        ws.status = "Auth Failed: Invalid Key".to_string();
                    } else {
                        // Assume it's the proto definition if we don't have it yet
                        if ws.protos.is_none() {
                            info!("Received protobuf definitions (length: {})", txt.len());
                            ws.protos = Some(txt);
                            ws.status = "Protos Received".to_string();
                        } else {
                            info!("WS Text Message (Ignored): {}", &txt[..txt.len().min(100)]);
                        }
                    }
                }
                WsMessage::Binary(data) => {
                    parse_binary_message(&data, &mut ws, &mut scene_state);
                }
            }
        }

        for (mut text, mut color) in &mut query {
            if scene_state.loaded {
                text.0 = "WS: Scene Loaded".to_string();
                color.0 = Color::srgb(0.0, 1.0, 0.0);
            } else if ws.protos.is_some() {
                text.0 = "WS: Connected (Protos Loaded)".to_string();
                color.0 = Color::srgb(0.0, 1.0, 0.0);
            } else {
                text.0 = format!("WS: {}", ws.status);
                if ws.status.contains("Connected") {
                    color.0 = Color::srgb(0.5, 1.0, 0.5);
                } else if ws.status.contains("Error") || ws.status.contains("Closed") {
                    color.0 = Color::srgb(1.0, 0.0, 0.0);
                } else {
                    color.0 = Color::srgb(1.0, 1.0, 0.0);
                }
            }
        }
    }
}

/// Parse a binary websocket message with the gz-transport frame format:
/// `operation,topic,msgType,<protobuf payload>`
fn parse_binary_message(
    data: &[u8],
    ws: &mut GzWebSocket,
    scene_state: &mut SceneState,
) {
    // Find the first three commas to split the header
    let mut comma_positions = Vec::new();
    for (i, &byte) in data.iter().enumerate() {
        if byte == b',' {
            comma_positions.push(i);
            if comma_positions.len() == 3 {
                break;
            }
        }
    }

    if comma_positions.len() < 3 {
        warn!("Binary message too short or missing header commas ({} bytes)", data.len());
        return;
    }

    let operation = std::str::from_utf8(&data[..comma_positions[0]]).unwrap_or("");
    let topic = std::str::from_utf8(&data[comma_positions[0] + 1..comma_positions[1]]).unwrap_or("");
    let msg_type = std::str::from_utf8(&data[comma_positions[1] + 1..comma_positions[2]]).unwrap_or("");
    let payload = &data[comma_positions[2] + 1..];

    info!(
        "Binary msg: op='{}', topic='{}', type='{}', payload={} bytes",
        operation, topic, msg_type, payload.len()
    );

    match operation {
        "pub" => {
            match topic {
                "worlds" => {
                    match prost::Message::decode(payload) {
                        Ok(worlds_msg) => {
                            let worlds_msg: crate::gz_msgs::StringMsgV = worlds_msg;
                            if let Some(world_name) = worlds_msg.data.first() {
                                info!("World name: {}", world_name);
                                scene_state.world_name = Some(world_name.clone());
                                ws.status = format!("World: {}", world_name);
                            } else {
                                error!("Worlds message contained no worlds");
                            }
                        }
                        Err(e) => error!("Failed to decode worlds message: {:?}", e),
                    }
                }
                "scene" => {
                    info!("Received scene data ({} bytes)", payload.len());
                    ws.scene_data = Some(payload.to_vec());
                    ws.status = "Scene Received, Processing...".to_string();
                }
                _ => {
                    info!("Unhandled pub topic: '{}'", topic);
                }
            }
        }
        _ => {
            info!("Unhandled binary operation: '{}'", operation);
        }
    }
}

pub fn setup_websocket_system(world: &mut World) {
    let port = resolve_port();
    let url = format!("ws://localhost:{}", port);
    info!("Connecting to WebSocket: {}", url);

    #[cfg(target_arch = "wasm32")]
    setup_websocket_wasm(world, &url);

    #[cfg(not(target_arch = "wasm32"))]
    setup_websocket_native(world, &url);
}

/// Resolve the websocket port from env var (compile-time on wasm, runtime on native),
/// URL query param (wasm only), or default to 9002.
fn resolve_port() -> String {
    // Compile-time default from env var
    let mut port = option_env!("GZ_WEBSOCKET_PORT")
        .unwrap_or("9002")
        .to_string();

    // On native, also check runtime env var (takes precedence over compile-time)
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(env_port) = std::env::var("GZ_WEBSOCKET_PORT") {
            if !env_port.is_empty() {
                port = env_port;
            }
        }
    }

    // On wasm, check URL query parameter (takes precedence)
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(location) = window.location().search() {
                if let Some(idx) = location.find("port=") {
                    let (_, p) = location.split_at(idx + 5);
                    let end = p.find('&').unwrap_or(p.len());
                    let p_str = &p[..end];
                    if !p_str.is_empty() {
                        port = p_str.to_string();
                    }
                }
            }
        }
    }

    port
}

// ===== Native WebSocket (tungstenite) =====

#[cfg(not(target_arch = "wasm32"))]
fn setup_websocket_native(world: &mut World, url: &str) {
    use std::net::TcpStream;
    use std::thread;
    use tungstenite::Message;

    let (msg_tx, msg_rx) = bounded::<WsMessage>(100);
    let (cmd_tx, cmd_rx) = bounded::<String>(100);

    let url_owned = url.to_string();
    let tx = msg_tx.clone();

    thread::spawn(move || {
        info!("Native WS thread: connecting to {}", url_owned);
        let _ = tx.send(WsMessage::Text("STATUS: Connecting...".to_string()));

        // Parse the URL to get host:port for TcpStream
        let addr = url_owned
            .trim_start_matches("ws://")
            .trim_start_matches("wss://");

        // Connect raw TCP (no TLS needed for localhost)
        let tcp_stream = match TcpStream::connect(addr) {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to connect TCP to {}: {:?}", addr, e);
                let _ = tx.send(WsMessage::Text(format!("STATUS: Error: {}", e)));
                return;
            }
        };

        // Clone for non-blocking control (we need the raw stream reference)
        let nb_stream = match tcp_stream.try_clone() {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to clone TCP stream: {:?}", e);
                return;
            }
        };

        // Perform websocket handshake over the TCP stream
        let (mut socket, _response) = match tungstenite::client(&url_owned, tcp_stream) {
            Ok(pair) => {
                let _ = tx.send(WsMessage::Text("STATUS: Connected".to_string()));
                pair
            }
            Err(e) => {
                error!("WebSocket handshake failed: {:?}", e);
                let _ = tx.send(WsMessage::Text(format!("STATUS: Error: {}", e)));
                return;
            }
        };

        // Send initial protos request
        let request = serde_json::json!(["protos", "", "", ""]).to_string();
        if let Err(e) = socket.send(Message::Text(request.into())) {
            error!("Failed to send protos request: {:?}", e);
            return;
        }
        info!("Sent protos request");

        // Switch to non-blocking for the read loop
        let _ = nb_stream.set_nonblocking(true);

        loop {
            // Check for outgoing commands
            while let Ok(cmd) = cmd_rx.try_recv() {
                // Temporarily set blocking for reliable send
                let _ = nb_stream.set_nonblocking(false);
                if let Err(e) = socket.send(Message::Text(cmd.into())) {
                    error!("Failed to send command: {:?}", e);
                    let _ = tx.send(WsMessage::Text("STATUS: Send Error".to_string()));
                    return;
                }
                let _ = nb_stream.set_nonblocking(true);
            }

            // Try to read a message (non-blocking)
            match socket.read() {
                Ok(msg) => match msg {
                    Message::Text(txt) => {
                        let _ = tx.send(WsMessage::Text(txt.to_string()));
                    }
                    Message::Binary(data) => {
                        let _ = tx.send(WsMessage::Binary(data.to_vec()));
                    }
                    Message::Ping(data) => {
                        let _ = nb_stream.set_nonblocking(false);
                        let _ = socket.send(Message::Pong(data));
                        let _ = nb_stream.set_nonblocking(true);
                    }
                    Message::Close(_) => {
                        info!("WebSocket closed by server");
                        let _ = tx.send(WsMessage::Text("STATUS: Closed".to_string()));
                        return;
                    }
                    _ => {}
                },
                Err(tungstenite::Error::Io(ref e))
                    if e.kind() == std::io::ErrorKind::WouldBlock =>
                {
                    // No data available yet, sleep briefly to avoid busy-spinning
                    thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(e) => {
                    error!("WebSocket read error: {:?}", e);
                    let _ = tx.send(WsMessage::Text(format!("STATUS: Error: {}", e)));
                    return;
                }
            }
        }
    });

    world.insert_non_send_resource(GzWebSocket {
        receiver: msg_rx,
        cmd_sender: Some(cmd_tx),
        status: format!("Connecting to {}...", url),
        protos: None,
        scene_data: None,
    });
}

// ===== WASM WebSocket (web-sys) =====

#[cfg(target_arch = "wasm32")]
fn setup_websocket_wasm(world: &mut World, url: &str) {
    match web_sys::WebSocket::new(url) {
        Ok(ws) => {
            ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

            let (tx, rx) = bounded::<WsMessage>(100);
            let (cmd_tx, _cmd_rx) = bounded::<String>(100); // Not used on wasm (direct socket send)
            let tx_open = tx.clone();
            let tx_msg = tx.clone();
            let tx_err = tx.clone();
            let tx_close = tx.clone();

            let ws_clone = ws.clone();

            // OnOpen
            let on_open = Closure::<dyn FnMut()>::new(move || {
                let _ = tx_open.send(WsMessage::Text("STATUS: Connected".to_string()));

                let request = serde_json::json!(["protos", "", "", ""]).to_string();
                match ws_clone.send_with_str(&request) {
                    Ok(_) => info!("Sent protos request"),
                    Err(e) => error!("Failed to send protos request: {:?}", e),
                }
            });
            ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
            on_open.forget();

            // OnMessage
            let on_message = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
                if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                    let _ = tx_msg.send(WsMessage::Text(format!("{}", txt)));
                } else if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                    let array = js_sys::Uint8Array::new(&abuf);
                    let mut data = vec![0u8; array.length() as usize];
                    array.copy_to(&mut data);
                    let _ = tx_msg.send(WsMessage::Binary(data));
                } else if let Ok(blob) = e.data().dyn_into::<web_sys::Blob>() {
                    let reader = web_sys::FileReader::new().unwrap();
                    let tx_blob = tx_msg.clone();
                    let reader_clone = reader.clone();

                    let onload = Closure::<dyn FnMut()>::new(move || {
                        let result = reader_clone.result().unwrap();
                        if let Ok(txt) = result.dyn_into::<js_sys::JsString>() {
                            let _ = tx_blob.send(WsMessage::Text(format!("{}", txt)));
                        }
                    });

                    reader.set_onloadend(Some(onload.as_ref().unchecked_ref()));
                    let _ = reader.read_as_text(&blob);
                    onload.forget();
                }
            });
            ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
            on_message.forget();

            // OnError
            let on_error = Closure::<dyn FnMut(ErrorEvent)>::new(move |_e: ErrorEvent| {
                let _ = tx_err.send(WsMessage::Text("STATUS: Error".to_string()));
            });
            ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
            on_error.forget();

            // OnClose
            let on_close = Closure::<dyn FnMut(CloseEvent)>::new(move |_e: CloseEvent| {
                let _ = tx_close.send(WsMessage::Text("STATUS: Closed".to_string()));
            });
            ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
            on_close.forget();

            let url_str = url.to_string();
            world.insert_non_send_resource(GzWebSocket {
                receiver: rx,
                cmd_sender: Some(cmd_tx),
                socket: Some(ws),
                status: format!("Connecting to {}...", url_str),
                protos: None,
                scene_data: None,
            });
        }
        Err(e) => {
            error!("Failed to create WebSocket: {:?}", e);
            world.insert_non_send_resource(GzWebSocket {
                receiver: bounded(1).1,
                cmd_sender: None,
                socket: None,
                status: "Failed to Create Socket".to_string(),
                protos: None,
                scene_data: None,
            });
        }
    }
}
