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

pub(crate) struct SolvePressurePlugin;

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct JacobiIterationResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub p0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = WriteOnly)]
    pub p1: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadOnly)]
    pub div: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, access = ReadOnly)]
    pub levelset_air0: Handle<Image>,
    #[storage_texture(4, image_format = R32Float, access = ReadOnly)]
    pub levelset_solid: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct JacobiIterationReverseResource {
    #[storage_texture(0, image_format = R32Float, access = WriteOnly)]
    pub p0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadOnly)]
    pub p1: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadOnly)]
    pub div: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, access = ReadOnly)]
    pub levelset_air0: Handle<Image>,
    #[storage_texture(4, image_format = R32Float, access = ReadOnly)]
    pub levelset_solid: Handle<Image>,
}

#[derive(Resource)]
pub(crate) struct SolvePressurePipeline {
    pub jacobi_iteration_pipeline: CachedComputePipelineId,
    pub jacobi_iteration_reverse_pipeline: CachedComputePipelineId,
    jacobi_iteration_bind_group_layout: BindGroupLayout,
    jacobi_iteration_reverse_bind_group_layout: BindGroupLayout,
}

#[derive(Component)]
pub(crate) struct SolvePressureBindGroups {
    pub jacobi_iteration_bind_group: BindGroup,
    pub jacobi_iteration_reverse_bind_group: BindGroup,
}

impl Plugin for SolvePressurePlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/jacobi_iteration.wgsl");

        app.add_plugins((
            ExtractComponentPlugin::<JacobiIterationResource>::default(),
            ExtractComponentPlugin::<JacobiIterationReverseResource>::default(),
        ));

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            prepare_bind_groups.in_set(RenderSystems::PrepareBindGroups),
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<SolvePressurePipeline>();
    }
}

impl Pipeline for SolvePressurePipeline {
    fn is_pipeline_state_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        Self::is_pipeline_loaded(pipeline_cache, self.jacobi_iteration_pipeline)
            && Self::is_pipeline_loaded(pipeline_cache, self.jacobi_iteration_reverse_pipeline)
    }
}

impl FromWorld for SolvePressurePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let asset_server = world.resource::<AssetServer>();

        let uniform_bind_group_layout = create_uniform_bind_group_layout(render_device);

        let jacobi_iteration_bind_group_layout =
            JacobiIterationResource::bind_group_layout(render_device);
        let jacobi_iteration_reverse_bind_group_layout =
            JacobiIterationReverseResource::bind_group_layout(render_device);

        let jacobi_iteration_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("JacobiIteration".into()),
                layout: vec![
                    jacobi_iteration_bind_group_layout.clone(),
                    uniform_bind_group_layout.clone(),
                ],
                shader: load_embedded_asset!(asset_server, "shaders/jacobi_iteration.wgsl"),
                entry_point: Some("jacobi_iteration".into()),
                ..default()
            });

        let jacobi_iteration_reverse_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("JacobiIterationReverse".into()),
                layout: vec![
                    jacobi_iteration_reverse_bind_group_layout.clone(),
                    uniform_bind_group_layout.clone(),
                ],
                shader: load_embedded_asset!(asset_server, "shaders/jacobi_iteration.wgsl"),
                shader_defs: vec!["REVERSE".into()],
                entry_point: Some("jacobi_iteration_reverse".into()),
                ..default()
            });

        SolvePressurePipeline {
            jacobi_iteration_pipeline,
            jacobi_iteration_reverse_pipeline,
            jacobi_iteration_bind_group_layout,
            jacobi_iteration_reverse_bind_group_layout,
        }
    }
}

fn prepare_bind_groups(
    mut commands: Commands,
    pipeline: Res<SolvePressurePipeline>,
    query: Query<(
        Entity,
        &JacobiIterationResource,
        &JacobiIterationReverseResource,
    )>,
    render_device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    fallback_image: Res<FallbackImage>,
    buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
) {
    let mut param = (gpu_images, fallback_image, buffers);
    for (entity, jacobi_iter_resource, jacobi_iter_rev_resource) in &query {
        let jacobi_iteration_bind_group = jacobi_iter_resource
            .as_bind_group(
                &pipeline.jacobi_iteration_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;
        let jacobi_iteration_reverse_bind_group = jacobi_iter_rev_resource
            .as_bind_group(
                &pipeline.jacobi_iteration_reverse_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        commands.entity(entity).insert(SolvePressureBindGroups {
            jacobi_iteration_bind_group,
            jacobi_iteration_reverse_bind_group,
        });
    }
}
