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

use crate::{fluid_uniform::create_uniform_bind_group_layout, pipeline::Pipeline};

pub(crate) struct AdvectScalarPlugin;

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct AdvectLevelsetResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub u0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadOnly)]
    pub v0: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadOnly)]
    pub levelset_air0: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, access = WriteOnly)]
    pub levelset_air1: Handle<Image>,
}

#[derive(Resource)]
pub(crate) struct AdvectScalarPipeline {
    pub advect_levelset_pipeline: CachedComputePipelineId,
    advect_levelset_bind_group_layout: BindGroupLayout,
}

#[derive(Component)]
pub(crate) struct AdvectScalarBindGroups {
    pub advect_levelset_bind_group: BindGroup,
}

impl Plugin for AdvectScalarPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/advect_levelset.wgsl");

        app.add_plugins(ExtractComponentPlugin::<AdvectLevelsetResource>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            prepare_bind_groups.in_set(RenderSystems::PrepareBindGroups),
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<AdvectScalarPipeline>();
    }
}

impl Pipeline for AdvectScalarPipeline {
    fn is_pipeline_state_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        Self::is_pipeline_loaded(pipeline_cache, self.advect_levelset_pipeline)
    }
}

impl FromWorld for AdvectScalarPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let asset_server = world.resource::<AssetServer>();

        let uniform_bind_group_layout = create_uniform_bind_group_layout(render_device);
        let advect_levelset_bind_group_layout =
            AdvectLevelsetResource::bind_group_layout(render_device);

        let advect_levelset_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("AdvectLevelsetPipeline".into()),
                layout: vec![
                    advect_levelset_bind_group_layout.clone(),
                    uniform_bind_group_layout.clone(),
                ],
                shader: load_embedded_asset!(asset_server, "shaders/advect_levelset.wgsl"),
                entry_point: Some("advect_levelset".into()),
                ..default()
            });

        AdvectScalarPipeline {
            advect_levelset_pipeline,
            advect_levelset_bind_group_layout,
        }
    }
}

fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipeline: Res<AdvectScalarPipeline>,
    query: Query<(Entity, &AdvectLevelsetResource)>,
    render_device: Res<RenderDevice>,
    mut param: (
        Res<'a, RenderAssets<GpuImage>>,
        Res<'a, FallbackImage>,
        Res<'a, RenderAssets<GpuShaderStorageBuffer>>,
    ),
) {
    for (entity, advect_levelset_resource) in &query {
        let advect_levelset_bind_group = advect_levelset_resource
            .as_bind_group(
                &pipeline.advect_levelset_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        commands.entity(entity).insert(AdvectScalarBindGroups {
            advect_levelset_bind_group,
        });
    }
}
