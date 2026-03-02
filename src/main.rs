use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    input::mouse::{MouseButton},
    render::{
        renderer::RenderAdapter,
    },
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

mod gz_msgs;
mod websocket;
mod scene;
mod fuel;
mod asset_proxy;
use websocket::*;
use scene::*;

fn main() {
    // Early status update to prove WASM is running
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(element) = document.get_element_by_id("bevy-status") {
                let _ = element.set_inner_html("Bevy Status: WASM Loaded (Initializing App...)");
            }
        }
    }

    // Custom panic hook to report errors to the DOM
    std::panic::set_hook(Box::new(|_info| {
        // Print to console trace
        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::hook(_info); 
        
        // Report to DOM
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(element) = document.get_element_by_id("bevy-status") {
                        let msg = if let Some(s) = _info.payload().downcast_ref::<&str>() {
                            format!("Bevy Panic: {}", s)
                        } else if let Some(s) = _info.payload().downcast_ref::<String>() {
                            format!("Bevy Panic: {}", s)
                        } else {
                            "Bevy Panic: Unknown Error".to_string()
                        };
                        let _ = element.set_inner_html(&msg);
                        let _ = element.set_attribute("style", "position: absolute; top: 40px; right: 10px; color: white; font-family: monospace; font-size: 16px; background: black; padding: 5px;");
                    }
                }
            }
        }
    }));

    App::new()
        .add_plugins(fuel::FuelPlugin) // Must be before DefaultPlugins (registers asset source)
        .add_plugins(asset_proxy::AssetProxyPlugin) // Must be before DefaultPlugins (registers wsasset:// source)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Gazebo Web - WebGPU".into(),
                canvas: Some("#bevy-canvas".into()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }).set(bevy::log::LogPlugin {
            level: bevy::log::Level::INFO,
            filter: "wgpu=warn,bevy_render=info,bevy_ecs=info".to_string(),
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(bevy_obj::ObjPlugin)
        .add_plugins(bevy_stl::StlPlugin)
        .insert_resource(ClearColor(Color::WHITE)) // White background until scene loads
        .init_resource::<SceneState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (update_fps, update_adapter_info, update_dom_status, update_orbit_focus, update_websocket_status, process_scene, apply_dynamic_poses))
        .add_systems(Startup, setup_websocket_system) // Separate system to ensuring it runs
        .run();
}

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct AdapterText;

#[derive(Component)]
struct MainCamera;

fn setup(
    mut commands: Commands,
) {
    info!("Bevy WebGPU Demo Starting (3D)...");

    // Update status to show ECS is running
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(element) = document.get_element_by_id("bevy-status") {
                let _ = element.set_inner_html("Bevy Status: ECS Started (3D Setup)");
            }
        }
    }

    // Default ambient light until the scene data arrives
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 200.0,
        affects_lightmapped_meshes: true,
    });

    // 3D Camera with PanOrbitCamera
    commands.spawn((
        Camera3d::default(),
        Camera {
            // Explicitly set a white clear color for this camera. This ensures its
            // background is not affected by scene-wide background color changes.
            clear_color: ClearColorConfig::Custom(Color::WHITE),
        },
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera {
            button_orbit: MouseButton::Left,
            button_pan: MouseButton::Right,
            modifier_orbit: None,
            modifier_pan: None,
            ..default()
        },
        MainCamera,
    ));

    // UI
    commands.spawn((
        Text::new("FPS: N/A"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        FpsText,
    ));

    commands.spawn((
        Text::new("Adapter: Discovering..."),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Px(10.0),
            ..default()
        },
        AdapterText,
    ));

    commands.spawn((
        Text::new("WS: Initializing..."),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(70.0),
            left: Val::Px(10.0),
            ..default()
        },
        WebsocketStatusText,
    ));
    
    commands.spawn((
        Text::new("Controls: Left Drag to Orbit, Right Drag to Pan, Scroll to Zoom\nLeft Click to Orbit Around Cursor"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

fn update_orbit_focus(
    mut camera_query: Query<(&mut PanOrbitCamera, &Camera, &GlobalTransform)>,
    mut gizmos: Gizmos,
    windows: Query<&Window>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(cursor_position) = window.cursor_position() {
            let Ok((mut pan_orbit, camera, camera_transform)) = camera_query.single_mut() else {
                return;
            };
            
            // Calculate a ray from the cursor
            let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
                return;
            };

            // Raycast against ground plane (Normal: Y, Distance: 0)
            // Plane equation: n . x = d. Here n = (0,1,0), d = 0.
            // Ray: Origin + t * Dir
            // (Origin + t*Dir) . n = 0
            // Origin.y + t * Dir.y = 0 => t = -Origin.y / Dir.y
            
            let t = -ray.origin.y / ray.direction.y;
            
            if t > 0.0 {
                let intersection = ray.origin + ray.direction * t;
                
                // Update focus
                pan_orbit.target_focus = intersection;
                
                // Optional: Update radius to keep camera in place
                pan_orbit.radius = Some((camera_transform.translation() - intersection).length());
                
                // Visualize the click
                gizmos.sphere(intersection, 0.2, Color::srgb(1.0, 0.0, 0.0));
            }
        }
    }
}

fn update_dom_status(_diagnostics: Res<DiagnosticsStore>) {
    // This system directly updates the HTML DOM to prove the app is running
    // even if the canvas is black/transparent.
    #[cfg(target_arch = "wasm32")]
    if let Some(fps) = _diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(element) = document.get_element_by_id("bevy-status") {
                        let _ = element.set_inner_html(&format!("Bevy Status: Running (FPS: {:.2})", value));
                    }
                }
            }
        }
    }
}

fn update_fps(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                text.0 = format!("FPS: {value:.2}");
            }
        }
    }
}

fn update_adapter_info(
    render_adapter: Option<Res<RenderAdapter>>,
    mut query: Query<(&mut Text, &mut TextColor), With<AdapterText>>,
) {
    for (mut text, mut color) in &mut query {
        if let Some(adapter) = &render_adapter {
            let info = adapter.get_info();
            let backend_str = format!("{:?}", info.backend);
            
            text.0 = format!("Backend: {} ({})", backend_str, info.name);
            
            // Color code based on backend
            // Note: Bevy re-exports wgpu, but we need to check the specific Backend enum
            match info.backend {
                wgpu::Backend::Vulkan | 
                wgpu::Backend::Metal | 
                wgpu::Backend::Dx12 | 
                wgpu::Backend::BrowserWebGpu => {
                     color.0 = Color::srgb(0.0, 1.0, 0.0); // Green for Modern APIs
                }
                wgpu::Backend::Gl => {
                     color.0 = Color::srgb(0.0, 0.5, 1.0); // Blue for WebGL
                }
                _ => {
                     color.0 = Color::srgb(1.0, 1.0, 0.0); // Yellow for Other
                }
            }
        } else {
            text.0 = "Backend: None (No GPU Found)".to_string();
            color.0 = Color::srgb(1.0, 0.0, 0.0); // Red for error
        }
    }
}
