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

pub(crate) struct SolveVelocityPlugin;

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct SolveUResource {
    #[storage_texture(0, image_format = R32Float, access = WriteOnly)]
    pub u0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadOnly)]
    pub u1: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadOnly)]
    pub u_solid: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, access = ReadOnly)]
    pub p1: Handle<Image>,
    #[storage_texture(4, image_format = R32Float, access = ReadOnly)]
    pub levelset_air0: Handle<Image>,
    #[storage_texture(5, image_format = R32Float, access = ReadOnly)]
    pub levelset_solid: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct SolveVResource {
    #[storage_texture(0, image_format = R32Float, access = WriteOnly)]
    pub v0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadOnly)]
    pub v1: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadOnly)]
    pub v_solid: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, access = ReadOnly)]
    pub p1: Handle<Image>,
    #[storage_texture(4, image_format = R32Float, access = ReadOnly)]
    pub levelset_air0: Handle<Image>,
    #[storage_texture(5, image_format = R32Float, access = ReadOnly)]
    pub levelset_solid: Handle<Image>,
}

#[derive(Resource)]
pub(crate) struct SolveVelocityPipeline {
    pub solve_u_pipeline: CachedComputePipelineId,
    pub solve_v_pipeline: CachedComputePipelineId,
    solve_u_bind_group_layout: BindGroupLayout,
    solve_v_bind_group_layout: BindGroupLayout,
}

#[derive(Component)]
pub(crate) struct SolveVelocityBindGroups {
    pub solve_u_bind_group: BindGroup,
    pub solve_v_bind_group: BindGroup,
}

impl Plugin for SolveVelocityPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/solve_velocity_u.wgsl");
        embedded_asset!(app, "shaders/solve_velocity_v.wgsl");

        app.add_plugins((
            ExtractComponentPlugin::<SolveUResource>::default(),
            ExtractComponentPlugin::<SolveVResource>::default(),
        ));

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            prepare_bind_groups.in_set(RenderSystems::PrepareBindGroups),
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<SolveVelocityPipeline>();
    }
}

impl Pipeline for SolveVelocityPipeline {
    fn is_pipeline_state_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        Self::is_pipeline_loaded(pipeline_cache, self.solve_u_pipeline)
            && Self::is_pipeline_loaded(pipeline_cache, self.solve_v_pipeline)
    }
}

impl FromWorld for SolveVelocityPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let asset_server = world.resource::<AssetServer>();

        let uniform_bind_group_layout = create_uniform_bind_group_layout(render_device);
        let solve_u_bind_group_layout = SolveUResource::bind_group_layout(render_device);
        let solve_v_bind_group_layout = SolveVResource::bind_group_layout(render_device);

        let solve_u_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("SolveUPipeline".into()),
            layout: vec![
                solve_u_bind_group_layout.clone(),
                uniform_bind_group_layout.clone(),
            ],
            shader: load_embedded_asset!(asset_server, "shaders/solve_velocity_u.wgsl"),
            entry_point: Some("solve_velocity_u".into()),
            ..default()
        });

        let solve_v_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("SolveVPipeline".into()),
            layout: vec![
                solve_v_bind_group_layout.clone(),
                uniform_bind_group_layout.clone(),
            ],
            shader: load_embedded_asset!(asset_server, "shaders/solve_velocity_v.wgsl"),
            entry_point: Some("solve_velocity_v".into()),
            ..default()
        });

        SolveVelocityPipeline {
            solve_u_pipeline,
            solve_v_pipeline,
            solve_u_bind_group_layout,
            solve_v_bind_group_layout,
        }
    }
}

fn prepare_bind_groups(
    mut commands: Commands,
    pipeline: Res<SolveVelocityPipeline>,
    query: Query<(Entity, &SolveUResource, &SolveVResource)>,
    render_device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    fallback_image: Res<FallbackImage>,
    buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
) {
    let mut param = (gpu_images, fallback_image, buffers);
    for (entity, solve_u_resource, solve_v_resource) in &query {
        let solve_u_bind_group = solve_u_resource
            .as_bind_group(
                &pipeline.solve_u_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let solve_v_bind_group = solve_v_resource
            .as_bind_group(
                &pipeline.solve_v_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        commands.entity(entity).insert(SolveVelocityBindGroups {
            solve_u_bind_group,
            solve_v_bind_group,
        });
    }
}
