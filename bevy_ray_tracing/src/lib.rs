mod shader;
mod types;

use crate::types::GlobalRayTraceMeta;
use shader::{
    extract_ray_trace, prepare_ray_trace, prepare_rt_pipelines, RayTraceLabel, RayTraceNode,
    RayTracePipeline,
};

use bevy::{
    asset::load_internal_asset,
    core_pipeline::core_3d::graph::{Core3d, Node3d},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin},
        render_graph::{RenderGraphApp, ViewNodeRunner},
        render_resource::*,
        Render, RenderApp, RenderSet,
    },
};

#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
pub struct RayTracingSettings {
    pub bounces: u32,
    pub samples: u32,
    pub sky: Vec3,
}

#[derive(Component, Clone, Copy, ExtractComponent)]
pub struct RTSphere {
    pub radius: f32,
}

#[derive(Component, Clone, Copy, ExtractComponent)]
pub struct RTQuad;

// ---- Plugin ----
pub const RT_TYPES_HANDLE: Handle<Shader> = Handle::weak_from_u128(9475836894214873755);
pub const RT_HIT_HANDLE: Handle<Shader> = Handle::weak_from_u128(5959859852532537293);
pub const RT_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(5832768451236749832);

pub struct RayTracingPlugin;

impl Plugin for RayTracingPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, RT_TYPES_HANDLE, "types.wgsl", Shader::from_wgsl);
        // load_internal_asset!(app, RT_HIT_HANDLE, "hit.wgsl", Shader::from_wgsl);
        load_internal_asset!(app, RT_SHADER_HANDLE, "raytrace.wgsl", Shader::from_wgsl);

        app.add_plugins((
            ExtractComponentPlugin::<RayTracingSettings>::default(),
            UniformComponentPlugin::<RayTracingSettings>::default(),
        ));

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<GlobalRayTraceMeta>()
            .init_resource::<SpecializedRenderPipelines<RayTracePipeline>>()
            .add_plugins(ExtractComponentPlugin::<RTSphere>::default())
            .add_plugins(ExtractComponentPlugin::<RTQuad>::default())
            .add_systems(ExtractSchedule, extract_ray_trace)
            .add_systems(
                Render,
                (
                    prepare_ray_trace.in_set(RenderSet::ManageViews),
                    prepare_rt_pipelines.in_set(RenderSet::Prepare),
                ),
            )
            .add_render_graph_node::<ViewNodeRunner<RayTraceNode>>(Core3d, RayTraceLabel)
            .add_render_graph_edges(Core3d, (Node3d::EndMainPass, RayTraceLabel, Node3d::Bloom));
    }

    fn finish(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<RayTracePipeline>();
    }
}
