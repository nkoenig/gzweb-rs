use bevy::prelude::*;
use prost::Message;

use crate::gz_msgs;
use crate::websocket::GzWebSocket;

/// Marker component for entities spawned from the Gazebo scene.
#[derive(Component)]
pub struct GzSceneEntity {
    #[allow(dead_code)]
    pub gz_name: String,
}

/// Tracks whether the scene has been loaded.
#[derive(Resource, Default)]
pub struct SceneState {
    pub loaded: bool,
    pub scene_requested: bool,
    pub worlds_requested: bool,
    pub world_name: Option<String>,
}

/// Bevy system: drives the scene request state machine and processes the scene
/// when it arrives.
pub fn process_scene(
    websocket: Option<NonSendMut<GzWebSocket>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut scene_state: ResMut<SceneState>,
) {
    let Some(mut ws) = websocket else { return };

    // Skip if already loaded
    if scene_state.loaded {
        return;
    }

    // Step 1: Once protos are received, request worlds
    if ws.protos.is_some() && !scene_state.worlds_requested {
        let request = serde_json::json!(["worlds", "", "", ""]).to_string();
        match ws.send_message(&request) {
            Ok(_) => {
                info!("Sent worlds request");
                scene_state.worlds_requested = true;
                ws.status = "Requesting Worlds...".to_string();
            }
            Err(e) => error!("Failed to send worlds request: {}", e),
        }
        return;
    }

    // Step 2: Once world name is received, request scene
    if let Some(ref world_name) = scene_state.world_name {
        if !scene_state.scene_requested {
            let request =
                serde_json::json!(["scene", world_name, "", ""]).to_string();
            match ws.send_message(&request) {
                Ok(_) => {
                    info!("Sent scene request for world: {}", world_name);
                    scene_state.scene_requested = true;
                    ws.status = "Requesting Scene...".to_string();
                }
                Err(e) => error!("Failed to send scene request: {}", e),
            }
        }
    }

    // Step 3: Process received scene data
    if let Some(scene_bytes) = ws.scene_data.take() {
        info!("Processing scene data ({} bytes)", scene_bytes.len());

        match gz_msgs::Scene::decode(scene_bytes.as_slice()) {
            Ok(scene) => {
                info!(
                    "Scene '{}': {} models, {} lights",
                    scene.name,
                    scene.model.len(),
                    scene.light.len()
                );
                debug!("Decoded Scene message: {:#?}", scene);

                // Set ambient light from scene
                if let Some(ambient) = &scene.ambient {
                    commands.insert_resource(AmbientLight {
                        color: gz_color_to_bevy(ambient),
                        brightness: 300.0,
                    });
                }

                // Set background color
                if let Some(bg) = &scene.background {
                    commands.insert_resource(ClearColor(gz_color_to_bevy(bg)));
                }

                // Spawn lights
                for light in &scene.light {
                    spawn_light(&mut commands, light, Transform::IDENTITY);
                }

                // Spawn models
                for model in &scene.model {
                    spawn_model(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        model,
                        Transform::IDENTITY,
                    );
                }

                scene_state.loaded = true;
                ws.status = "Scene Loaded".to_string();
                info!("Scene processing complete");
            }
            Err(e) => {
                error!("Failed to decode Scene protobuf: {:?}", e);
                ws.status = format!("Scene Decode Error: {}", e);
            }
        }
    }
}

// ===== Light spawning =====

fn spawn_light(commands: &mut Commands, light: &gz_msgs::Light, parent_transform: Transform) {
    if light.is_light_off {
        return;
    }

    let color = light
        .diffuse
        .as_ref()
        .map(gz_color_to_bevy)
        .unwrap_or(Color::WHITE);
    let intensity = if light.intensity > 0.0 {
        light.intensity
    } else {
        1.0
    };
    let transform = combine_transforms(parent_transform, gz_pose_to_transform(light.pose.as_ref()));

    let light_type = gz_msgs::light::LightType::try_from(light.r#type)
        .unwrap_or(gz_msgs::light::LightType::Point);

    match light_type {
        gz_msgs::light::LightType::Point => {
            commands.spawn((
                PointLight {
                    color,
                    intensity: intensity * 800.0,
                    range: if light.range > 0.0 {
                        light.range
                    } else {
                        20.0
                    },
                    shadows_enabled: light.cast_shadows,
                    ..default()
                },
                transform,
                GzSceneEntity {
                    gz_name: light.name.clone(),
                },
            ));
            info!("Spawned PointLight: '{}'", light.name);
        }
        gz_msgs::light::LightType::Directional => {
            // Directional lights use the transform's forward direction
            let mut dir_transform = transform;
            if let Some(dir) = &light.direction {
                let direction = Vec3::new(dir.x as f32, dir.y as f32, dir.z as f32);
                if direction.length_squared() > 0.0 {
                    dir_transform =
                        Transform::from_translation(dir_transform.translation)
                            .looking_to(direction, Vec3::Y);
                }
            }
            commands.spawn((
                DirectionalLight {
                    color,
                    illuminance: intensity * 10000.0,
                    shadows_enabled: light.cast_shadows,
                    ..default()
                },
                dir_transform,
                GzSceneEntity {
                    gz_name: light.name.clone(),
                },
            ));
            info!("Spawned DirectionalLight: '{}'", light.name);
        }
        gz_msgs::light::LightType::Spot => {
            commands.spawn((
                SpotLight {
                    color,
                    intensity: intensity * 800.0,
                    range: if light.range > 0.0 {
                        light.range
                    } else {
                        20.0
                    },
                    inner_angle: light.spot_inner_angle,
                    outer_angle: light.spot_outer_angle,
                    shadows_enabled: light.cast_shadows,
                    ..default()
                },
                transform,
                GzSceneEntity {
                    gz_name: light.name.clone(),
                },
            ));
            info!("Spawned SpotLight: '{}'", light.name);
        }
    }
}

// ===== Model spawning =====

fn spawn_model(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    model: &gz_msgs::Model,
    parent_transform: Transform,
) {
    if model.deleted {
        return;
    }

    let model_transform =
        combine_transforms(parent_transform, gz_pose_to_transform(model.pose.as_ref()));

    info!(
        "Processing model '{}': {} links, {} visuals, {} nested models",
        model.name,
        model.link.len(),
        model.visual.len(),
        model.model.len()
    );

    // Process model-level visuals
    for visual in &model.visual {
        spawn_visual(commands, meshes, materials, visual, model_transform);
    }

    // Process links
    for link in &model.link {
        let link_transform =
            combine_transforms(model_transform, gz_pose_to_transform(link.pose.as_ref()));

        for visual in &link.visual {
            spawn_visual(commands, meshes, materials, visual, link_transform);
        }

        // Lights attached to links
        for link_light in &link.light {
            spawn_light(commands, link_light, link_transform);
        }
    }

    // Process nested models recursively
    for nested_model in &model.model {
        spawn_model(commands, meshes, materials, nested_model, model_transform);
    }
}

// ===== Visual spawning =====

fn spawn_visual(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    visual: &gz_msgs::Visual,
    parent_transform: Transform,
) {
    if !visual.visible && visual.id != 0 {
        // If id == 0, it's likely a default value and visible was just unset
        // Only skip if explicitly set to invisible
        return;
    }

    let visual_transform =
        combine_transforms(parent_transform, gz_pose_to_transform(visual.pose.as_ref()));

    let Some(geometry) = &visual.geometry else {
        info!("Visual '{}' has no geometry, skipping", visual.name);
        return;
    };

    let geom_type = gz_msgs::geometry::Type::try_from(geometry.r#type)
        .unwrap_or(gz_msgs::geometry::Type::Empty);

    let mesh_handle: Option<Handle<Mesh>> = match geom_type {
        gz_msgs::geometry::Type::Box => {
            if let Some(box_geom) = &geometry.r#box {
                if let Some(size) = &box_geom.size {
                    Some(meshes.add(Cuboid::new(
                        size.x as f32,
                        size.y as f32,
                        size.z as f32,
                    )))
                } else {
                    Some(meshes.add(Cuboid::new(1.0, 1.0, 1.0)))
                }
            } else {
                Some(meshes.add(Cuboid::new(1.0, 1.0, 1.0)))
            }
        }
        gz_msgs::geometry::Type::Cylinder => {
            if let Some(cyl) = &geometry.cylinder {
                Some(meshes.add(Cylinder::new(cyl.radius as f32, cyl.length as f32)))
            } else {
                Some(meshes.add(Cylinder::new(0.5, 1.0)))
            }
        }
        gz_msgs::geometry::Type::Sphere => {
            if let Some(sph) = &geometry.sphere {
                Some(meshes.add(Sphere::new(sph.radius as f32)))
            } else {
                Some(meshes.add(Sphere::new(0.5)))
            }
        }
        gz_msgs::geometry::Type::Plane => {
            if let Some(plane) = &geometry.plane {
                let size_x = plane.size.as_ref().map(|s| s.x as f32).unwrap_or(10.0);
                let size_y = plane.size.as_ref().map(|s| s.y as f32).unwrap_or(10.0);
                Some(meshes.add(Plane3d::default().mesh().size(size_x, size_y)))
            } else {
                Some(meshes.add(Plane3d::default().mesh().size(10.0, 10.0)))
            }
        }
        gz_msgs::geometry::Type::Capsule => {
            if let Some(cap) = &geometry.capsule {
                Some(meshes.add(Capsule3d::new(cap.radius as f32, cap.length as f32)))
            } else {
                Some(meshes.add(Capsule3d::new(0.5, 1.0)))
            }
        }
        gz_msgs::geometry::Type::Cone => {
            if let Some(cone) = &geometry.cone {
                Some(meshes.add(Cone::new(cone.radius as f32, cone.length as f32)))
            } else {
                Some(meshes.add(Cone::new(0.5, 1.0)))
            }
        }
        gz_msgs::geometry::Type::Mesh => {
            if let Some(mesh_geom) = &geometry.mesh {
                info!(
                    "Visual '{}': Mesh geometry '{}' (mesh loading not yet implemented)",
                    visual.name, mesh_geom.filename
                );
            }
            None
        }
        _ => {
            info!(
                "Visual '{}': unsupported geometry type {:?}",
                visual.name, geom_type
            );
            None
        }
    };

    if let Some(mesh) = mesh_handle {
        let mat = gz_material_to_bevy(visual.material.as_ref(), visual.transparency);
        let mat_handle = materials.add(mat);

        // Apply scale from visual if present
        let mut final_transform = visual_transform;
        if let Some(scale) = &visual.scale {
            final_transform.scale = Vec3::new(
                scale.x as f32 * final_transform.scale.x,
                scale.y as f32 * final_transform.scale.y,
                scale.z as f32 * final_transform.scale.z,
            );
        }

        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(mat_handle),
            final_transform,
            GzSceneEntity {
                gz_name: visual.name.clone(),
            },
        ));
        info!("Spawned visual: '{}' ({:?})", visual.name, geom_type);
    }
}

// ===== Conversion helpers =====

fn gz_color_to_bevy(color: &gz_msgs::Color) -> Color {
    Color::srgba(color.r, color.g, color.b, color.a)
}

fn gz_pose_to_transform(pose: Option<&gz_msgs::Pose>) -> Transform {
    let Some(pose) = pose else {
        return Transform::IDENTITY;
    };

    let translation = pose
        .position
        .as_ref()
        .map(|p| Vec3::new(p.x as f32, p.y as f32, p.z as f32))
        .unwrap_or(Vec3::ZERO);

    let rotation = pose
        .orientation
        .as_ref()
        .map(|q| Quat::from_xyzw(q.x as f32, q.y as f32, q.z as f32, q.w as f32).normalize())
        .unwrap_or(Quat::IDENTITY);

    Transform {
        translation,
        rotation,
        scale: Vec3::ONE,
    }
}

fn combine_transforms(parent: Transform, child: Transform) -> Transform {
    Transform {
        translation: parent.translation
            + parent.rotation * (parent.scale * child.translation),
        rotation: parent.rotation * child.rotation,
        scale: parent.scale * child.scale,
    }
}

fn gz_material_to_bevy(
    material: Option<&gz_msgs::Material>,
    transparency: f64,
) -> StandardMaterial {
    let Some(mat) = material else {
        return StandardMaterial {
            base_color: Color::srgb(0.8, 0.8, 0.8),
            ..default()
        };
    };

    let base_color = mat
        .diffuse
        .as_ref()
        .map(|c| {
            let alpha = if transparency > 0.0 {
                (1.0 - transparency as f32).max(0.0)
            } else {
                c.a
            };
            Color::srgba(c.r, c.g, c.b, alpha)
        })
        .unwrap_or(Color::srgb(0.8, 0.8, 0.8));

    let emissive = mat
        .emissive
        .as_ref()
        .map(|c| LinearRgba::new(c.r, c.g, c.b, c.a))
        .unwrap_or(LinearRgba::BLACK);

    // PBR properties
    let (metallic, perceptual_roughness) = if let Some(pbr) = &mat.pbr {
        (pbr.metalness as f32, pbr.roughness as f32)
    } else {
        (0.0, 0.5)
    };

    let alpha_mode = if transparency > 0.0 {
        AlphaMode::Blend
    } else {
        AlphaMode::Opaque
    };

    StandardMaterial {
        base_color,
        emissive,
        metallic,
        perceptual_roughness,
        double_sided: mat.double_sided,
        cull_mode: if mat.double_sided {
            None
        } else {
            Some(bevy::render::render_resource::Face::Back)
        },
        alpha_mode,
        ..default()
    }
}
