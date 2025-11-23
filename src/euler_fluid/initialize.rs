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

pub(crate) struct InitializePlugin;

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct InitializeVelocityResource {
    #[storage_texture(0, image_format = R32Float, access = WriteOnly)]
    pub u0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = WriteOnly)]
    pub v0: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = WriteOnly)]
    pub u1: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, access = WriteOnly)]
    pub v1: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct InitializeGridCenterResource {
    #[storage_texture(0, image_format = R32Float, access = WriteOnly)]
    pub levelset_air0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = WriteOnly)]
    pub levelset_air1: Handle<Image>,
}

#[derive(Resource)]
pub(crate) struct InitializePipeline {
    pub init_velocity_pipeline: CachedComputePipelineId,
    pub init_grid_center_pipeline: CachedComputePipelineId,
    init_velocity_bind_group_layout: BindGroupLayout,
    init_grid_center_bind_group_layout: BindGroupLayout,
}

#[derive(Component)]
pub(crate) struct InitializeBindGroups {
    pub init_velocity_bind_group: BindGroup,
    pub init_grid_center_bind_group: BindGroup,
}

impl Plugin for InitializePlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/initialize_velocity.wgsl");
        embedded_asset!(app, "shaders/initialize_grid_center.wgsl");

        app.add_plugins((
            ExtractComponentPlugin::<InitializeVelocityResource>::default(),
            ExtractComponentPlugin::<InitializeGridCenterResource>::default(),
        ));

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            prepare_bind_groups.in_set(RenderSystems::PrepareBindGroups),
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<InitializePipeline>();
    }
}

impl Pipeline for InitializePipeline {
    fn is_pipeline_state_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        Self::is_pipeline_loaded(pipeline_cache, self.init_velocity_pipeline)
            && Self::is_pipeline_loaded(pipeline_cache, self.init_grid_center_pipeline)
    }
}

impl FromWorld for InitializePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let asset_server = world.resource::<AssetServer>();

        let uniform_bind_group_layout = create_uniform_bind_group_layout(render_device);
        let init_velocity_bind_group_layout =
            InitializeVelocityResource::bind_group_layout(render_device);
        let init_grid_center_bind_group_layout =
            InitializeGridCenterResource::bind_group_layout(render_device);

        let init_velocity_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("InitializeVelocityPipeline".into()),
                layout: vec![init_velocity_bind_group_layout.clone()],
                shader: load_embedded_asset!(asset_server, "shaders/initialize_velocity.wgsl"),
                entry_point: Some("initialize_velocity".into()),
                ..default()
            });

        let init_grid_center_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("InitializeGridCenterPipeline".into()),
                layout: vec![
                    init_grid_center_bind_group_layout.clone(),
                    uniform_bind_group_layout.clone(),
                ],
                shader: load_embedded_asset!(asset_server, "shaders/initialize_grid_center.wgsl"),
                entry_point: Some("initialize_grid_center".into()),
                ..default()
            });

        InitializePipeline {
            init_velocity_pipeline,
            init_grid_center_pipeline,
            init_velocity_bind_group_layout,
            init_grid_center_bind_group_layout,
        }
    }
}

fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipeline: Res<InitializePipeline>,
    query: Query<(
        Entity,
        &InitializeVelocityResource,
        &InitializeGridCenterResource,
    )>,
    render_device: Res<RenderDevice>,
    mut param: (
        Res<'a, RenderAssets<GpuImage>>,
        Res<'a, FallbackImage>,
        Res<'a, RenderAssets<GpuShaderStorageBuffer>>,
    ),
) {
    for (entity, init_velocity_resource, init_grid_center_resource) in &query {
        let init_velocity_bind_group = init_velocity_resource
            .as_bind_group(
                &pipeline.init_velocity_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let init_grid_center_bind_group = init_grid_center_resource
            .as_bind_group(
                &pipeline.init_grid_center_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        commands.entity(entity).insert(InitializeBindGroups {
            init_velocity_bind_group,
            init_grid_center_bind_group,
        });
    }
}
