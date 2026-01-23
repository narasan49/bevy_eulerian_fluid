use crate::fluid_uniform::create_uniform_bind_group_layout;
use crate::pipeline::Pipeline;
use bevy::asset::load_embedded_asset;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{
    AsBindGroup, BindGroup, BindGroupLayout, CachedComputePipelineId, ComputePipelineDescriptor,
    PipelineCache,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::storage::{GpuShaderStorageBuffer, ShaderStorageBuffer};
use bevy::render::texture::{FallbackImage, GpuImage};

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct AdvectParticlesResource {
    #[storage(0, read_only, visibility(compute))]
    pub count: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(compute))]
    pub levelset_particles: Handle<ShaderStorageBuffer>,
    #[storage_texture(2, image_format = R32Float, access = ReadOnly)]
    pub u0: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, access = ReadOnly)]
    pub v0: Handle<Image>,
}

#[derive(Resource)]
pub(crate) struct AdvectParticlesPipeline {
    pub pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayout,
}

#[derive(Component)]
pub(crate) struct AdvectParticlesBindGroups(pub BindGroup);

impl Pipeline for AdvectParticlesPipeline {
    fn is_pipeline_state_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        Self::is_pipeline_loaded(pipeline_cache, self.pipeline)
    }
}

impl FromWorld for AdvectParticlesPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let asset_server = world.resource::<AssetServer>();

        let uniform_bind_group_layout = create_uniform_bind_group_layout(render_device);
        let bind_group_layout = AdvectParticlesResource::bind_group_layout(render_device);

        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("AdvectParticlesPipeline".into()),
            layout: vec![bind_group_layout.clone(), uniform_bind_group_layout],
            shader: load_embedded_asset!(asset_server, "shaders/advect_particles.wgsl"),
            entry_point: Some("advect_particles".into()),
            ..default()
        });

        AdvectParticlesPipeline {
            pipeline,
            bind_group_layout,
        }
    }
}

pub(super) fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipeline: Res<AdvectParticlesPipeline>,
    query: Query<(Entity, &AdvectParticlesResource)>,
    render_device: Res<RenderDevice>,
    mut param: (
        Res<'a, RenderAssets<GpuImage>>,
        Res<'a, FallbackImage>,
        Res<'a, RenderAssets<GpuShaderStorageBuffer>>,
    ),
) {
    for (entity, resource) in &query {
        let bind_group = resource
            .as_bind_group(&pipeline.bind_group_layout, &render_device, &mut param)
            .unwrap()
            .bind_group;

        commands
            .entity(entity)
            .insert(AdvectParticlesBindGroups(bind_group));
    }
}
