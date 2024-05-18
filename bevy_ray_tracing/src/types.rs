use bevy::{
    ecs::{
        system::Resource,
        world::{FromWorld, World},
    },
    math::{Mat3, Vec3, Vec4},
    render::render_resource::{ShaderType, StorageBuffer},
};

pub const SHAPE_SPHERE: u32 = 0;
pub const SHAPE_QUAD: u32 = 1;

#[derive(Default, Clone, Copy, ShaderType)]
pub struct RayTraceCamera {
    pub position: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}

#[derive(Default, Clone, Copy, ShaderType)]
pub struct RayTraceMaterial {
    pub color: Vec4,
    pub emissive: Vec4,
    pub roughness: f32,
    pub metallic: f32,
    pub diffuse_transmission: f32,
    pub specular_transmission: f32,
    pub ior: f32,
    pub double_sided: u32,
}

#[derive(Default, Clone, Copy, ShaderType)]
pub struct RayTraceObject {
    pub position: Vec3,
    pub shape_type: u32,
    pub shape_index: i32,
    pub material_index: i32,
}

#[derive(Default, Clone, Copy, ShaderType)]
pub struct RayTraceEmissive {
    pub index: i32,
}

#[derive(Default, Clone, Copy, ShaderType)]
pub struct RayTraceSphere {
    pub radius: f32,
}

#[derive(Default, Clone, Copy, ShaderType)]
pub struct RayTraceQuad {
    pub model: Mat3,
}

// Meta
#[derive(ShaderType, Default)]
pub struct RayTraceObjects {
    #[size(runtime)]
    pub data: Vec<RayTraceObject>,
}

#[derive(ShaderType, Default)]
pub struct RayTraceEmissives {
    #[size(runtime)]
    pub data: Vec<RayTraceEmissive>,
}

#[derive(ShaderType, Default)]
pub struct RayTraceSpheres {
    #[size(runtime)]
    pub data: Vec<RayTraceSphere>,
}

#[derive(ShaderType, Default)]
pub struct RayTraceQuads {
    #[size(runtime)]
    pub data: Vec<RayTraceQuad>,
}

#[derive(ShaderType, Default)]
pub struct RayTraceMaterials {
    #[size(runtime)]
    pub data: Vec<RayTraceMaterial>,
}

#[derive(Resource)]
pub struct GlobalRayTraceMeta {
    pub camera: StorageBuffer<RayTraceCamera>,
    pub objects: StorageBuffer<RayTraceObjects>,
    pub emissives: StorageBuffer<RayTraceEmissives>,
    pub spheres: StorageBuffer<RayTraceSpheres>,
    pub quads: StorageBuffer<RayTraceQuads>,
    pub materials: StorageBuffer<RayTraceMaterials>,
}

impl FromWorld for GlobalRayTraceMeta {
    fn from_world(_world: &mut World) -> Self {
        Self {
            camera: StorageBuffer::default(),
            objects: StorageBuffer::default(),
            emissives: StorageBuffer::default(),
            spheres: StorageBuffer::default(),
            quads: StorageBuffer::default(),
            materials: StorageBuffer::default(),
        }
    }
}
