use bevy::{
    asset::load_embedded_asset,
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, BindGroup, BindGroupLayout, CachedComputePipelineId,
            ComputePipelineDescriptor, PipelineCache,
        },
        renderer::RenderDevice,
        storage::{GpuShaderStorageBuffer, ShaderStorageBuffer},
        texture::{FallbackImage, GpuImage},
    },
};

use crate::pipeline::Pipeline;

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct InitializeParticlesResource {
    #[storage(0, visibility(compute))]
    pub count: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(compute))]
    pub levelset_particles: Handle<ShaderStorageBuffer>,
    #[storage_texture(2, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[storage_texture(3, image_format = Rg32Float, access = ReadOnly)]
    pub grad_levelset_air: Handle<Image>,
    #[storage_texture(4, image_format = R8Uint, access = ReadOnly)]
    pub near_interface: Handle<Image>,
}

#[derive(Resource)]
pub(crate) struct InitializeParticlesPipeline {
    pub pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayout,
}

#[derive(Component)]
pub(crate) struct InitializeParticlesBindGroups(pub BindGroup);

impl Pipeline for InitializeParticlesPipeline {
    fn is_pipeline_state_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        Self::is_pipeline_loaded(pipeline_cache, self.pipeline)
    }
}

impl FromWorld for InitializeParticlesPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let asset_server = world.resource::<AssetServer>();

        let bind_group_layout = InitializeParticlesResource::bind_group_layout(render_device);

        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("InitializeParticlesPipeline".into()),
            layout: vec![bind_group_layout.clone()],
            shader: load_embedded_asset!(asset_server, "shaders/initialize_particles.wgsl"),
            entry_point: Some("initialize_particles".into()),
            ..default()
        });

        InitializeParticlesPipeline {
            pipeline,
            bind_group_layout,
        }
    }
}

pub(super) fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipeline: Res<InitializeParticlesPipeline>,
    query: Query<(Entity, &InitializeParticlesResource)>,
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
            .insert(InitializeParticlesBindGroups(bind_group));
    }
}
