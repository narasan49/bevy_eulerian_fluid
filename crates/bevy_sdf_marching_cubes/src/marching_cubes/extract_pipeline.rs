use bevy::{
    asset::load_embedded_asset,
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, BindGroup, BindGroupLayoutDescriptor, CachedComputePipelineId,
            ComputePipelineDescriptor, PipelineCache,
        },
        renderer::RenderDevice,
        storage::GpuShaderStorageBuffer,
        texture::{FallbackImage, GpuImage},
    },
};

use crate::marching_cubes::resource::MarchingCubesExtractResource;

#[derive(Resource)]
pub struct MarchingCubesExtractPipeline {
    pub pipeline_id: CachedComputePipelineId,
    pub bind_group_layout: BindGroupLayoutDescriptor,
}

impl FromWorld for MarchingCubesExtractPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let bind_group_layout =
            MarchingCubesExtractResource::bind_group_layout_descriptor(render_device);
        let pipeline_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("MarchingCubesExtractPipeline".into()),
            layout: vec![bind_group_layout.clone()],
            push_constant_ranges: vec![],
            shader: load_embedded_asset!(world, "marching_cubes_extract.wgsl"),
            entry_point: Some("extract".into()),
            shader_defs: vec![],
            zero_initialize_workgroup_memory: true,
        });

        Self {
            pipeline_id,
            bind_group_layout,
        }
    }
}

#[derive(Component)]
pub struct MarchingCubesExtractBindGroup {
    pub bind_group: BindGroup,
}

pub fn prepare_marching_cubes_extract_bind_groups<'a>(
    mut commands: Commands,
    fluids: Query<(Entity, &MarchingCubesExtractResource)>,
    pipeline: Res<MarchingCubesExtractPipeline>,
    pipeline_cache: Res<PipelineCache>,
    render_device: Res<RenderDevice>,
    mut param: (
        Res<'a, RenderAssets<GpuImage>>,
        Res<'a, FallbackImage>,
        Res<'a, RenderAssets<GpuShaderStorageBuffer>>,
    ),
) {
    for (entity, mc_resource) in &fluids {
        info_once!("[once] prepare extract bind group");
        let bind_group = mc_resource
            .as_bind_group(
                &pipeline.bind_group_layout,
                &render_device,
                &pipeline_cache,
                &mut param,
            )
            .unwrap()
            .bind_group;

        commands
            .entity(entity)
            .insert(MarchingCubesExtractBindGroup { bind_group });
    }
}
