use super::{
    types::{
        RayTraceCamera, RayTraceMaterial, RayTraceMaterials, RayTraceObject, RayTraceObjects,
        RayTraceQuad, RayTraceQuads, RayTraceSphere, RayTraceSpheres, SHAPE_QUAD, SHAPE_SPHERE,
    },
    GlobalRayTraceMeta, RTQuad, RTSphere, RayTracingSettings, RT_SHADER_HANDLE,
};

use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    ecs::query::QueryItem,
    prelude::*,
    render::{
        extract_component::ComponentUniforms,
        render_graph::{NodeRunError, RenderGraphContext, RenderLabel, ViewNode},
        render_resource::{
            binding_types::{storage_buffer, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::{ExtractedView, ViewTarget, ViewUniform, ViewUniforms},
        Extract,
    },
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct RayTraceLabel;

#[derive(Default)]
pub struct RayTraceNode;

impl ViewNode for RayTraceNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static RayTracingSettings,
        &'static RayTracePipelineId,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, rt_settings, rt_pipeline_id): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        if rt_settings.bounces == 0 || rt_settings.samples == 0 {
            return Ok(());
        }

        let pipelines = world.resource::<RayTracePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let view_uniforms_resource = world.resource::<ViewUniforms>();
        let view_uniforms = &view_uniforms_resource.uniforms;

        let Some(rt_pipeline) = pipeline_cache.get_render_pipeline(rt_pipeline_id.0) else {
            return Ok(());
        };

        let ray_trace_meta = world.resource::<GlobalRayTraceMeta>();
        if ray_trace_meta.camera.binding().is_none() || ray_trace_meta.spheres.binding().is_none() {
            return Ok(());
        }

        let settings_uniforms = world.resource::<ComponentUniforms<RayTracingSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();
        let bind_group = render_context.render_device().create_bind_group(
            "ray_trace_bind_group",
            &pipelines.rt_bind_group_layout,
            &BindGroupEntries::sequential((
                ray_trace_meta.camera.binding().unwrap(),
                ray_trace_meta.objects.binding().unwrap(),
                ray_trace_meta.spheres.binding().unwrap(),
                ray_trace_meta.quads.binding().unwrap(),
                ray_trace_meta.materials.binding().unwrap(),
                settings_binding.clone(),
                view_uniforms,
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("ray_trace_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(rt_pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
pub struct RayTracePipeline {
    rt_bind_group_layout: BindGroupLayout,
}

impl FromWorld for RayTracePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(
            "ray_trace_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    storage_buffer::<RayTraceCamera>(false),
                    storage_buffer::<RayTraceObjects>(false),
                    storage_buffer::<RayTraceSpheres>(false),
                    storage_buffer::<RayTraceQuads>(false),
                    storage_buffer::<RayTraceMaterials>(false),
                    uniform_buffer::<RayTracingSettings>(false),
                    uniform_buffer::<ViewUniform>(false),
                ),
            ),
        );

        Self {
            rt_bind_group_layout: layout,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct RayTracePipelineKey {
    hdr: bool,
}

impl SpecializedRenderPipeline for RayTracePipeline {
    type Key = RayTracePipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut shader_defs = vec![];
        let format = if key.hdr {
            shader_defs.push("TONEMAP".into());
            ViewTarget::TEXTURE_FORMAT_HDR
        } else {
            TextureFormat::bevy_default()
        };

        RenderPipelineDescriptor {
            label: Some("ray_trace_pipeline".into()),
            layout: vec![self.rt_bind_group_layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: RT_SHADER_HANDLE,
                shader_defs,
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),

            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: vec![],
        }
    }
}

#[derive(Component)]
pub struct RayTracePipelineId(CachedRenderPipelineId);

pub(super) fn prepare_rt_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<RayTracePipeline>>,
    pipeline: Res<RayTracePipeline>,
    views: Query<(Entity, &ExtractedView)>,
) {
    for (entity, view) in &views {
        let pipeline_key = RayTracePipelineKey { hdr: view.hdr };
        let pipeline_id = pipelines.specialize(&pipeline_cache, &pipeline, pipeline_key.clone());

        commands
            .entity(entity)
            .insert(RayTracePipelineId(pipeline_id));
    }
}

// ---- Extract ----
pub(super) fn prepare_ray_trace(
    mut global_ray_trace_meta: ResMut<GlobalRayTraceMeta>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    global_ray_trace_meta
        .camera
        .write_buffer(&render_device, &render_queue);
    global_ray_trace_meta
        .objects
        .write_buffer(&render_device, &render_queue);
    global_ray_trace_meta
        .spheres
        .write_buffer(&render_device, &render_queue);
    global_ray_trace_meta
        .quads
        .write_buffer(&render_device, &render_queue);
    global_ray_trace_meta
        .materials
        .write_buffer(&render_device, &render_queue);
}

pub(super) fn extract_ray_trace(
    camera_query: Extract<
        Query<(&Camera3d, &GlobalTransform), (Without<RTSphere>, Without<RTQuad>)>,
    >,
    sphere_query: Extract<
        Query<
            (&RTSphere, &Handle<StandardMaterial>, &GlobalTransform),
            (Without<Camera3d>, Without<RTQuad>),
        >,
    >,
    quad_query: Extract<
        Query<
            (&RTQuad, &Handle<StandardMaterial>, &GlobalTransform),
            (Without<Camera3d>, Without<RTSphere>),
        >,
    >,
    materials: Extract<Res<Assets<StandardMaterial>>>,
    mut global_ray_trace_meta: ResMut<GlobalRayTraceMeta>,
) {
    if let Ok((_camera, transform)) = camera_query.get_single() {
        global_ray_trace_meta.camera.set(RayTraceCamera {
            position: transform.translation(),
            forward: transform.forward(),
            right: transform.right(),
            up: transform.up(),
        });
    }

    let mut rt_objects = RayTraceObjects::default();
    let mut rt_materials = RayTraceMaterials::default();

    {
        let spheres: Vec<RayTraceSphere> = sphere_query
            .iter()
            .enumerate()
            .map(|(i, (sphere, material_handle, transform))| {
                let material = materials.get(material_handle).unwrap();
                let matindex = rt_materials.data.len();
                rt_materials.data.push(RayTraceMaterial {
                    color: material.base_color.rgba_to_vec4(),
                    emissive: material.emissive.rgba_to_vec4(),
                    roughness: material.perceptual_roughness,
                    metallic: material.metallic,
                    diffuse_transmission: material.diffuse_transmission,
                    specular_transmission: material.specular_transmission,
                    ior: material.ior,
                    double_sided: material.double_sided as u32,
                });

                rt_objects.data.push(RayTraceObject {
                    shape_type: SHAPE_SPHERE,
                    shape_index: i as i32,
                    material_index: matindex as i32,
                });

                RayTraceSphere {
                    position: transform.translation(),
                    radius: sphere.radius,
                }
            })
            .collect();

        global_ray_trace_meta
            .spheres
            .set(RayTraceSpheres { data: spheres });
    }

    {
        let quads: Vec<RayTraceQuad> = quad_query
            .iter()
            .enumerate()
            .map(|(i, (_quad, material_handle, transform))| {
                let material = materials.get(material_handle).unwrap();
                let matindex = rt_materials.data.len();
                rt_materials.data.push(RayTraceMaterial {
                    color: material.base_color.rgba_to_vec4(),
                    emissive: material.emissive.rgba_to_vec4(),
                    roughness: material.perceptual_roughness,
                    metallic: material.metallic,
                    diffuse_transmission: material.diffuse_transmission,
                    specular_transmission: material.specular_transmission,
                    ior: material.ior,
                    double_sided: material.double_sided as u32,
                });

                rt_objects.data.push(RayTraceObject {
                    shape_type: SHAPE_QUAD,
                    shape_index: i as i32,
                    material_index: matindex as i32,
                });

                RayTraceQuad {
                    position: transform.translation(),
                    model: transform.affine().matrix3.into(),
                }
            })
            .collect();

        global_ray_trace_meta
            .quads
            .set(RayTraceQuads { data: quads });
    }

    global_ray_trace_meta.materials.set(rt_materials);
    global_ray_trace_meta.objects.set(rt_objects);
}
