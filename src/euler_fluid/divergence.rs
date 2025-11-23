use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, BindGroup, BindGroupLayout, CachedComputePipelineId,
            ComputePipelineDescriptor, PipelineCache,
        },
        renderer::RenderDevice,
        storage::GpuShaderStorageBuffer,
        texture::{FallbackImage, GpuImage},
        Render, RenderApp, RenderSystems,
    },
};

use crate::pipeline::Pipeline;

pub(crate) struct DivergencePlugin;

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct DivergenceResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub u1: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadOnly)]
    pub v1: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadOnly)]
    pub u_solid: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, access = ReadOnly)]
    pub v_solid: Handle<Image>,
    #[storage_texture(4, image_format = R32Float, access = ReadOnly)]
    pub levelset_solid: Handle<Image>,
    #[storage_texture(5, image_format = R32Float, access = WriteOnly)]
    pub div: Handle<Image>,
}

#[derive(Resource)]
pub(crate) struct DivergencePipeline {
    pub divergence_pipeline: CachedComputePipelineId,
    divergence_bind_group_layout: BindGroupLayout,
}

#[derive(Component)]
pub(crate) struct DivergenceBindGroups {
    pub divergence_bind_group: BindGroup,
}

impl Plugin for DivergencePlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/divergence.wgsl");

        app.add_plugins(ExtractComponentPlugin::<DivergenceResource>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            prepare_bind_group.in_set(RenderSystems::PrepareBindGroups),
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<DivergencePipeline>();
    }
}

impl Pipeline for DivergencePipeline {
    fn is_pipeline_state_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        Self::is_pipeline_loaded(pipeline_cache, self.divergence_pipeline)
    }
}

impl FromWorld for DivergencePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let asset_server = world.resource::<AssetServer>();

        let divergence_bind_group_layout = DivergenceResource::bind_group_layout(render_device);

        let divergence_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("DivergencePipeline".into()),
                layout: vec![divergence_bind_group_layout.clone()],
                shader: load_embedded_asset!(asset_server, "shaders/divergence.wgsl"),
                entry_point: Some("divergence".into()),
                ..default()
            });

        DivergencePipeline {
            divergence_pipeline,
            divergence_bind_group_layout,
        }
    }
}

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<DivergencePipeline>,
    query: Query<(Entity, &DivergenceResource)>,
    render_device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    fallback_image: Res<FallbackImage>,
    buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
) {
    let mut param = (gpu_images, fallback_image, buffers);
    for (entity, divergence_resource) in &query {
        let divergence_bind_group = divergence_resource
            .as_bind_group(
                &pipeline.divergence_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        commands.entity(entity).insert(DivergenceBindGroups {
            divergence_bind_group,
        });
    }
}
