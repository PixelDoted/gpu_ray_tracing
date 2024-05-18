#define_import_path bevy_ray_tracing::types

const PI: f32 = 3.14159265359;
const EPSILON: f32 = 1e-6;
const SHAPE_SPHERE: u32 = 0;
const SHAPE_QUAD: u32 = 1;

struct RTSettings {
    bounces: i32,
    samples: i32,
    sky: vec3<f32>,
}

struct Camera {
    position: vec3<f32>,
    forward: vec3<f32>,
    right: vec3<f32>,
    up: vec3<f32>,
}

struct Object {
    position: vec3<f32>,
    shape_type: u32,
    shape_index: i32,
    material_index: i32,
}

struct Sphere {
    radius: f32,
}

struct Quad {
    model: mat3x3<f32>,
}

struct Material {
    color: vec4<f32>,
    emissive: vec4<f32>,
    roughness: f32,
    metallic: f32,
    diffuse_transmission: f32,
    specular_transmission: f32,
    ior: f32,
    double_sided: u32,
}

// ---- variables ----
var<private> hit_record: HitRecord;
var<private> rng_state: u32;

struct HitRecord {
    t: f32,
    p: vec3<f32>,
    n: vec3<f32>,
    uv: vec2<f32>,
    front_face: bool,

    object_index: i32,
    material_index: i32,
}

struct Ray {
    pos: vec3<f32>,
    dir: vec3<f32>,
}

struct Surface {
    material: Material,
    normal: vec3<f32>,
    light: vec3<f32>,
}
