/// Prost message definitions matching gz-msgs protobuf schema.
/// Manually defined to avoid needing protoc and the full proto dependency tree.
/// Only the types needed for Scene deserialization are included.
///
/// Proto source: https://github.com/gazebosim/gz-msgs/tree/gz-msgs12/proto/gz/msgs

use prost::Message;

// ===== Primitive Types =====

#[derive(Clone, PartialEq, Message)]
pub struct Time {
    #[prost(int64, tag = "1")]
    pub sec: i64,
    #[prost(int32, tag = "2")]
    pub nsec: i32,
}

#[derive(Clone, PartialEq, Message)]
pub struct Vector2d {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(double, tag = "2")]
    pub x: f64,
    #[prost(double, tag = "3")]
    pub y: f64,
}

#[derive(Clone, PartialEq, Message)]
pub struct Vector3d {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(double, tag = "2")]
    pub x: f64,
    #[prost(double, tag = "3")]
    pub y: f64,
    #[prost(double, tag = "4")]
    pub z: f64,
}

#[derive(Clone, PartialEq, Message)]
pub struct Quaternion {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(double, tag = "2")]
    pub x: f64,
    #[prost(double, tag = "3")]
    pub y: f64,
    #[prost(double, tag = "4")]
    pub z: f64,
    #[prost(double, tag = "5")]
    pub w: f64,
}

#[derive(Clone, PartialEq, Message)]
pub struct Color {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(float, tag = "2")]
    pub r: f32,
    #[prost(float, tag = "3")]
    pub g: f32,
    #[prost(float, tag = "4")]
    pub b: f32,
    #[prost(float, tag = "5")]
    pub a: f32,
}

// ===== Header =====

#[derive(Clone, PartialEq, Message)]
pub struct Header {
    #[prost(message, optional, tag = "1")]
    pub stamp: Option<Time>,
    #[prost(message, repeated, tag = "2")]
    pub data: Vec<header::Map>,
}

pub mod header {
    use prost::Message;

    #[derive(Clone, PartialEq, Message)]
    pub struct Map {
        #[prost(string, tag = "1")]
        pub key: String,
        #[prost(string, repeated, tag = "2")]
        pub value: Vec<String>,
    }
}

// ===== Pose =====

#[derive(Clone, PartialEq, Message)]
pub struct Pose {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(uint32, tag = "3")]
    pub id: u32,
    #[prost(message, optional, tag = "4")]
    pub position: Option<Vector3d>,
    #[prost(message, optional, tag = "5")]
    pub orientation: Option<Quaternion>,
}

// ===== Geometry Types =====

#[derive(Clone, PartialEq, Message)]
pub struct BoxGeom {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(message, optional, tag = "2")]
    pub size: Option<Vector3d>,
}

#[derive(Clone, PartialEq, Message)]
pub struct CylinderGeom {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(double, tag = "2")]
    pub radius: f64,
    #[prost(double, tag = "3")]
    pub length: f64,
}

#[derive(Clone, PartialEq, Message)]
pub struct SphereGeom {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(double, tag = "2")]
    pub radius: f64,
}

#[derive(Clone, PartialEq, Message)]
pub struct PlaneGeom {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(message, optional, tag = "2")]
    pub normal: Option<Vector3d>,
    #[prost(message, optional, tag = "3")]
    pub size: Option<Vector2d>,
    #[prost(double, tag = "4")]
    pub d: f64,
}

#[derive(Clone, PartialEq, Message)]
pub struct ConeGeom {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(double, tag = "2")]
    pub radius: f64,
    #[prost(double, tag = "3")]
    pub length: f64,
}

#[derive(Clone, PartialEq, Message)]
pub struct CapsuleGeom {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(double, tag = "2")]
    pub radius: f64,
    #[prost(double, tag = "3")]
    pub length: f64,
}

#[derive(Clone, PartialEq, Message)]
pub struct EllipsoidGeom {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(message, optional, tag = "2")]
    pub radii: Option<Vector3d>,
}

#[derive(Clone, PartialEq, Message)]
pub struct MeshGeom {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(string, tag = "2")]
    pub filename: String,
    #[prost(message, optional, tag = "3")]
    pub scale: Option<Vector3d>,
    #[prost(string, tag = "4")]
    pub submesh: String,
    #[prost(bool, tag = "5")]
    pub center_submesh: bool,
}

#[derive(Clone, PartialEq, Message)]
pub struct ImageGeom {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(string, tag = "2")]
    pub uri: String,
    #[prost(double, tag = "3")]
    pub scale: f64,
    #[prost(int32, tag = "4")]
    pub threshold: i32,
    #[prost(double, tag = "5")]
    pub height: f64,
    #[prost(int32, tag = "6")]
    pub granularity: i32,
}

#[derive(Clone, PartialEq, Message)]
pub struct Polyline {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(double, tag = "2")]
    pub height: f64,
    #[prost(message, repeated, tag = "3")]
    pub point: Vec<Vector2d>,
}

// HeightmapGeom is stubbed — not used for rendering primitives
#[derive(Clone, PartialEq, Message)]
pub struct HeightmapGeom {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    // Remaining fields omitted — not needed for basic scene rendering
}

#[derive(Clone, PartialEq, Message)]
pub struct Geometry {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(enumeration = "geometry::Type", tag = "2")]
    pub r#type: i32,
    #[prost(message, optional, tag = "3")]
    pub r#box: Option<BoxGeom>,
    #[prost(message, optional, tag = "4")]
    pub cylinder: Option<CylinderGeom>,
    #[prost(message, optional, tag = "5")]
    pub plane: Option<PlaneGeom>,
    #[prost(message, optional, tag = "6")]
    pub sphere: Option<SphereGeom>,
    #[prost(message, optional, tag = "7")]
    pub image: Option<ImageGeom>,
    #[prost(message, optional, tag = "8")]
    pub heightmap: Option<HeightmapGeom>,
    #[prost(message, optional, tag = "9")]
    pub mesh: Option<MeshGeom>,
    #[prost(message, optional, tag = "10")]
    pub cone: Option<ConeGeom>,
    #[prost(message, repeated, tag = "11")]
    pub points: Vec<Vector3d>,
    #[prost(message, repeated, tag = "12")]
    pub polyline: Vec<Polyline>,
    #[prost(message, optional, tag = "13")]
    pub capsule: Option<CapsuleGeom>,
    #[prost(message, optional, tag = "14")]
    pub ellipsoid: Option<EllipsoidGeom>,
}

pub mod geometry {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
    #[repr(i32)]
    pub enum Type {
        Box = 0,
        Cylinder = 1,
        Sphere = 2,
        Plane = 3,
        Image = 4,
        Heightmap = 5,
        Mesh = 6,
        TriangleFan = 7,
        LineStrip = 8,
        Polyline = 9,
        Cone = 10,
        Empty = 11,
        Arrow = 12,
        Axis = 13,
        Capsule = 14,
        Ellipsoid = 15,
    }
}

// ===== Material =====

#[derive(Clone, PartialEq, Message)]
pub struct Material {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(message, optional, tag = "2")]
    pub script: Option<material::Script>,
    #[prost(enumeration = "material::ShaderType", tag = "3")]
    pub shader_type: i32,
    #[prost(string, tag = "4")]
    pub normal_map: String,
    #[prost(message, optional, tag = "5")]
    pub ambient: Option<Color>,
    #[prost(message, optional, tag = "6")]
    pub diffuse: Option<Color>,
    #[prost(message, optional, tag = "7")]
    pub specular: Option<Color>,
    #[prost(message, optional, tag = "8")]
    pub emissive: Option<Color>,
    #[prost(bool, tag = "9")]
    pub lighting: bool,
    #[prost(message, optional, tag = "10")]
    pub pbr: Option<material::Pbr>,
    #[prost(double, tag = "11")]
    pub render_order: f64,
    #[prost(bool, tag = "12")]
    pub double_sided: bool,
    #[prost(double, tag = "13")]
    pub shininess: f64,
}

pub mod material {
    use prost::Message;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
    #[repr(i32)]
    pub enum ShaderType {
        Vertex = 0,
        Pixel = 1,
        NormalMapObjectSpace = 2,
        NormalMapTangentSpace = 3,
    }

    #[derive(Clone, PartialEq, Message)]
    pub struct Script {
        #[prost(string, repeated, tag = "1")]
        pub uri: Vec<String>,
        #[prost(string, tag = "2")]
        pub name: String,
    }

    #[derive(Clone, PartialEq, Message)]
    pub struct Pbr {
        #[prost(enumeration = "pbr::WorkflowType", tag = "1")]
        pub r#type: i32,
        #[prost(string, tag = "2")]
        pub albedo_map: String,
        #[prost(string, tag = "3")]
        pub normal_map: String,
        #[prost(double, tag = "4")]
        pub metalness: f64,
        #[prost(string, tag = "5")]
        pub metalness_map: String,
        #[prost(double, tag = "6")]
        pub roughness: f64,
        #[prost(string, tag = "7")]
        pub roughness_map: String,
        #[prost(double, tag = "8")]
        pub glossiness: f64,
        #[prost(string, tag = "9")]
        pub glossiness_map: String,
        #[prost(string, tag = "10")]
        pub specular_map: String,
        #[prost(string, tag = "11")]
        pub environment_map: String,
        #[prost(string, tag = "12")]
        pub ambient_occlusion_map: String,
        #[prost(string, tag = "13")]
        pub emissive_map: String,
        #[prost(string, tag = "14")]
        pub light_map: String,
        #[prost(uint32, tag = "15")]
        pub light_map_texcoord_set: u32,
    }

    pub mod pbr {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
        #[repr(i32)]
        pub enum WorkflowType {
            None = 0,
            Metal = 1,
            Specular = 2,
        }
    }
}

// ===== Plugin (stub, referenced by Visual) =====

#[derive(Clone, PartialEq, Message)]
pub struct Plugin {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(string, tag = "3")]
    pub filename: String,
    #[prost(string, tag = "4")]
    pub innerxml: String,
}

// ===== Visual =====

#[derive(Clone, PartialEq, Message)]
pub struct Visual {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(uint32, tag = "3")]
    pub id: u32,
    #[prost(string, tag = "4")]
    pub parent_name: String,
    #[prost(uint32, tag = "5")]
    pub parent_id: u32,
    #[prost(bool, tag = "6")]
    pub cast_shadows: bool,
    #[prost(double, tag = "7")]
    pub transparency: f64,
    #[prost(double, tag = "8")]
    pub laser_retro: f64,
    #[prost(message, optional, tag = "9")]
    pub pose: Option<Pose>,
    #[prost(message, optional, tag = "10")]
    pub geometry: Option<Geometry>,
    #[prost(message, optional, tag = "11")]
    pub material: Option<Material>,
    #[prost(bool, tag = "12")]
    pub visible: bool,
    #[prost(bool, tag = "13")]
    pub delete_me: bool,
    #[prost(bool, tag = "14")]
    pub is_static: bool,
    #[prost(message, repeated, tag = "15")]
    pub plugin: Vec<Plugin>,
    #[prost(message, optional, tag = "16")]
    pub scale: Option<Vector3d>,
    #[prost(message, optional, tag = "17")]
    pub meta: Option<visual::Meta>,
    #[prost(enumeration = "visual::Type", tag = "18")]
    pub r#type: i32,
}

pub mod visual {
    use prost::Message;

    #[derive(Clone, PartialEq, Message)]
    pub struct Meta {
        #[prost(int32, tag = "1")]
        pub layer: i32,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
    #[repr(i32)]
    pub enum Type {
        Entity = 0,
        Model = 1,
        Link = 2,
        Visual = 3,
        Collision = 4,
        Sensor = 5,
        Gui = 6,
        Physics = 7,
    }
}

// ===== Light =====

#[derive(Clone, PartialEq, Message)]
pub struct Light {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(enumeration = "light::LightType", tag = "3")]
    pub r#type: i32,
    #[prost(message, optional, tag = "4")]
    pub pose: Option<Pose>,
    #[prost(message, optional, tag = "5")]
    pub diffuse: Option<Color>,
    #[prost(message, optional, tag = "6")]
    pub specular: Option<Color>,
    #[prost(float, tag = "7")]
    pub attenuation_constant: f32,
    #[prost(float, tag = "8")]
    pub attenuation_linear: f32,
    #[prost(float, tag = "9")]
    pub attenuation_quadratic: f32,
    #[prost(message, optional, tag = "10")]
    pub direction: Option<Vector3d>,
    #[prost(float, tag = "11")]
    pub range: f32,
    #[prost(bool, tag = "12")]
    pub cast_shadows: bool,
    #[prost(float, tag = "13")]
    pub spot_inner_angle: f32,
    #[prost(float, tag = "14")]
    pub spot_outer_angle: f32,
    #[prost(float, tag = "15")]
    pub spot_falloff: f32,
    #[prost(uint32, tag = "16")]
    pub id: u32,
    #[prost(uint32, tag = "17")]
    pub parent_id: u32,
    #[prost(float, tag = "18")]
    pub intensity: f32,
    #[prost(bool, tag = "19")]
    pub is_light_off: bool,
    #[prost(bool, tag = "20")]
    pub visualize_visual: bool,
}

pub mod light {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
    #[repr(i32)]
    pub enum LightType {
        Point = 0,
        Spot = 1,
        Directional = 2,
    }
}

// ===== Link (simplified — only visual and pose needed for rendering) =====

#[derive(Clone, PartialEq, Message)]
pub struct Link {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(uint32, tag = "2")]
    pub id: u32,
    #[prost(string, tag = "3")]
    pub name: String,
    #[prost(bool, tag = "4")]
    pub self_collide: bool,
    #[prost(bool, tag = "5")]
    pub gravity: bool,
    #[prost(bool, tag = "6")]
    pub kinematic: bool,
    #[prost(bool, tag = "7")]
    pub enabled: bool,
    // density (tag 8) and inertial (tag 9) omitted — not needed for rendering
    #[prost(message, optional, tag = "10")]
    pub pose: Option<Pose>,
    #[prost(message, repeated, tag = "11")]
    pub visual: Vec<Visual>,
    // collision (tag 12), sensor (tag 13), projector (tag 14) omitted
    #[prost(bool, tag = "15")]
    pub canonical: bool,
    // battery (tag 16) omitted
    #[prost(message, repeated, tag = "17")]
    pub light: Vec<Light>,
}

// ===== Axis (stub for Joint) =====

#[derive(Clone, PartialEq, Message)]
pub struct Axis {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(message, optional, tag = "2")]
    pub xyz: Option<Vector3d>,
    #[prost(double, tag = "3")]
    pub limit_lower: f64,
    #[prost(double, tag = "4")]
    pub limit_upper: f64,
    #[prost(double, tag = "5")]
    pub limit_effort: f64,
    #[prost(double, tag = "6")]
    pub limit_velocity: f64,
    #[prost(double, tag = "7")]
    pub damping: f64,
    #[prost(double, tag = "8")]
    pub friction: f64,
}

// ===== Joint (simplified for Scene) =====

#[derive(Clone, PartialEq, Message)]
pub struct Joint {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(uint32, tag = "3")]
    pub id: u32,
    #[prost(enumeration = "joint::Type", tag = "4")]
    pub r#type: i32,
    #[prost(string, tag = "5")]
    pub parent: String,
    #[prost(uint32, tag = "6")]
    pub parent_id: u32,
    #[prost(string, tag = "7")]
    pub child: String,
    #[prost(uint32, tag = "8")]
    pub child_id: u32,
    #[prost(message, optional, tag = "9")]
    pub pose: Option<Pose>,
    #[prost(message, optional, tag = "10")]
    pub axis1: Option<Axis>,
    #[prost(message, optional, tag = "11")]
    pub axis2: Option<Axis>,
}

pub mod joint {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
    #[repr(i32)]
    pub enum Type {
        Revolute = 0,
        Revolute2 = 1,
        Prismatic = 2,
        Universal = 3,
        Ball = 4,
        Screw = 5,
        Gearbox = 6,
        Fixed = 7,
        Continuous = 8,
    }
}

// ===== AxisAlignedBox =====

#[derive(Clone, PartialEq, Message)]
pub struct AxisAlignedBox {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(message, optional, tag = "2")]
    pub min_corner: Option<Vector3d>,
    #[prost(message, optional, tag = "3")]
    pub max_corner: Option<Vector3d>,
}

// ===== Model =====

#[derive(Clone, PartialEq, Message)]
pub struct Model {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(uint32, tag = "3")]
    pub id: u32,
    #[prost(bool, tag = "4")]
    pub is_static: bool,
    #[prost(message, optional, tag = "5")]
    pub pose: Option<Pose>,
    #[prost(message, repeated, tag = "6")]
    pub joint: Vec<Joint>,
    #[prost(message, repeated, tag = "7")]
    pub link: Vec<Link>,
    #[prost(bool, tag = "8")]
    pub deleted: bool,
    #[prost(message, repeated, tag = "9")]
    pub visual: Vec<Visual>,
    #[prost(message, optional, tag = "10")]
    pub scale: Option<Vector3d>,
    #[prost(bool, tag = "11")]
    pub self_collide: bool,
    #[prost(message, repeated, tag = "12")]
    pub model: Vec<Model>,
    #[prost(message, optional, tag = "13")]
    pub bounding_box: Option<AxisAlignedBox>,
}

// ===== Fog =====

#[derive(Clone, PartialEq, Message)]
pub struct Fog {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(enumeration = "fog::FogType", tag = "2")]
    pub r#type: i32,
    #[prost(message, optional, tag = "3")]
    pub color: Option<Color>,
    #[prost(float, tag = "4")]
    pub density: f32,
    #[prost(float, tag = "5")]
    pub start: f32,
    #[prost(float, tag = "6")]
    pub end: f32,
}

pub mod fog {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
    #[repr(i32)]
    pub enum FogType {
        None = 0,
        Linear = 1,
        Exponential = 2,
        Exponential2 = 3,
    }
}

// ===== Sky =====

#[derive(Clone, PartialEq, Message)]
pub struct Sky {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(double, tag = "2")]
    pub time: f64,
    #[prost(double, tag = "3")]
    pub sunrise: f64,
    #[prost(double, tag = "4")]
    pub sunset: f64,
    #[prost(double, tag = "5")]
    pub wind_speed: f64,
    #[prost(double, tag = "6")]
    pub wind_direction: f64,
    #[prost(message, optional, tag = "7")]
    pub cloud_ambient: Option<Color>,
    #[prost(double, tag = "8")]
    pub humidity: f64,
    #[prost(double, tag = "9")]
    pub mean_cloud_size: f64,
    #[prost(string, tag = "10")]
    pub cubemap_uri: String,
}

// ===== Scene =====

#[derive(Clone, PartialEq, Message)]
pub struct Scene {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(message, optional, tag = "3")]
    pub ambient: Option<Color>,
    #[prost(message, optional, tag = "4")]
    pub background: Option<Color>,
    #[prost(message, optional, tag = "5")]
    pub sky: Option<Sky>,
    #[prost(bool, tag = "6")]
    pub shadows: bool,
    #[prost(message, optional, tag = "7")]
    pub fog: Option<Fog>,
    #[prost(bool, tag = "8")]
    pub grid: bool,
    #[prost(message, repeated, tag = "9")]
    pub model: Vec<Model>,
    #[prost(message, repeated, tag = "10")]
    pub light: Vec<Light>,
    #[prost(message, repeated, tag = "11")]
    pub joint: Vec<Joint>,
    #[prost(bool, tag = "12")]
    pub origin_visual: bool,
}

// ===== StringMsg (for world name response) =====

#[allow(dead_code)]
#[derive(Clone, PartialEq, Message)]
pub struct StringMsg {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(string, tag = "2")]
    pub data: String,
}

// ===== StringMsg_V (for worlds list) =====

#[allow(dead_code)]
#[derive(Clone, PartialEq, Message)]
pub struct StringMsgV {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(string, repeated, tag = "2")]
    pub data: Vec<String>,
}

// ===== Pose_V (for dynamic_pose/info) =====

#[derive(Clone, PartialEq, Message)]
pub struct PoseV {
    #[prost(message, optional, tag = "1")]
    pub header: Option<Header>,
    #[prost(message, repeated, tag = "2")]
    pub pose: Vec<Pose>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;

    #[test]
    fn test_scene_parsing_roundtrip() {
        let mut original_scene = Scene {
            header: None,
            name: "test_scene".to_string(),
            ambient: None,
            background: None,
            sky: None,
            shadows: false,
            fog: None,
            grid: false,
            model: vec![],
            light: vec![],
            joint: vec![],
            origin_visual: false,
        };
        
        let mut model = Model {
            header: None,
            name: "test_model".to_string(),
            id: 1,
            is_static: false,
            pose: None,
            joint: vec![],
            link: vec![],
            deleted: false,
            visual: vec![],
            scale: None,
            self_collide: false,
            model: vec![],
            bounding_box: None,
        };
        
        let mut visual = Visual {
            header: None,
            name: "test_visual".to_string(),
            id: 2,
            parent_name: "test_model".to_string(),
            parent_id: 1,
            cast_shadows: true,
            transparency: 0.0,
            laser_retro: 0.0,
            pose: None,
            geometry: None,
            material: None,
            visible: true,
            delete_me: false,
            is_static: false,
            plugin: vec![],
            scale: None,
            meta: None,
            r#type: visual::Type::Visual as i32,
        };
        
        let geom = Geometry {
            header: None,
            r#type: geometry::Type::Box as i32,
            r#box: Some(BoxGeom {
                header: None,
                size: Some(Vector3d { header: None, x: 1.0, y: 2.0, z: 3.0 }),
            }),
            cylinder: None,
            plane: None,
            sphere: None,
            image: None,
            heightmap: None,
            mesh: None,
            cone: None,
            points: vec![],
            polyline: vec![],
            capsule: None,
            ellipsoid: None,
        };
        
        visual.geometry = Some(geom);
        model.visual.push(visual);
        original_scene.model.push(model);

        // Encode to bytes
        let mut buf = Vec::new();
        original_scene.encode(&mut buf).expect("Failed to encode scene");

        // Decode back
        let decoded_scene = Scene::decode(buf.as_slice()).expect("Failed to decode scene");

        assert_eq!(decoded_scene, original_scene);
    }
}
