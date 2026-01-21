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
        storage::GpuShaderStorageBuffer,
        texture::{FallbackImage, GpuImage},
    },
};

use crate::pipeline::Pipeline;

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct InitializeInterfaceIndicesResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub levelset: Handle<Image>,
    /// 0 for far from interface, 1 for near interface
    #[storage_texture(1, image_format = R8Uint, access = WriteOnly)]
    pub near_interface: Handle<Image>,
}

#[derive(Resource)]
pub(crate) struct InitializeInterfaceIndicesPipeline {
    pub pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayout,
}

#[derive(Component)]
pub(crate) struct InitializeInterfaceIndicesBindGroups(pub BindGroup);

impl Pipeline for InitializeInterfaceIndicesPipeline {
    fn is_pipeline_state_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        Self::is_pipeline_loaded(pipeline_cache, self.pipeline)
    }
}

impl FromWorld for InitializeInterfaceIndicesPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let asset_server = world.resource::<AssetServer>();

        let bind_group_layout =
            InitializeInterfaceIndicesResource::bind_group_layout(render_device);

        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("InitializeInterfaceIndicesPipeline".into()),
            layout: vec![bind_group_layout.clone()],
            shader: load_embedded_asset!(asset_server, "shaders/initialize_interface_indices.wgsl"),
            entry_point: Some("initialize_interface_indices".into()),
            ..default()
        });

        InitializeInterfaceIndicesPipeline {
            pipeline,
            bind_group_layout,
        }
    }
}

pub(super) fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipeline: Res<InitializeInterfaceIndicesPipeline>,
    query: Query<(Entity, &InitializeInterfaceIndicesResource)>,
    render_device: Res<RenderDevice>,
    mut param: (
        Res<'a, RenderAssets<GpuImage>>,
        Res<'a, FallbackImage>,
        Res<'a, RenderAssets<GpuShaderStorageBuffer>>,
    ),
) {
    for (entity, initialize_interface_indices_resource) in &query {
        let bind_group = initialize_interface_indices_resource
            .as_bind_group(&pipeline.bind_group_layout, &render_device, &mut param)
            .unwrap()
            .bind_group;

        commands
            .entity(entity)
            .insert(InitializeInterfaceIndicesBindGroups(bind_group));
    }
}
