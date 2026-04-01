use bevy::{
    asset::{embedded_asset, embedded_path},
    ecs::{schedule::ScheduleConfigs, system::ScheduleSystem},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_asset::RenderAssets,
        render_resource::{AsBindGroup, BindGroup},
        renderer::RenderDevice,
        storage::{GpuShaderStorageBuffer, ShaderStorageBuffer},
        texture::{FallbackImage, GpuImage},
    },
};

use crate::{
    particle_levelset_two_layers::plugin::PLSResources, pipeline::SingleComputePipeline,
    plugin::FluidComputePass,
};

pub(super) struct SortPositiveParticlesPass;

impl FluidComputePass for SortPositiveParticlesPass {
    type P = SortParticlesPipeline;

    type Resource = SortPositiveParticlesResource;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/sort_particles.wgsl");
    }

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups_positive.into_configs()
    }
}

pub(super) struct SortNegativeParticlesPass;

impl FluidComputePass for SortNegativeParticlesPass {
    type P = SortParticlesPipeline;

    type Resource = SortNegativeParticlesResource;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/sort_particles.wgsl");
    }

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups_negative.into_configs()
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct SortPositiveParticlesResource {
    #[storage(0, read_only, visibility(compute))]
    pub positive_particles: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub positive_particles_count: Handle<ShaderStorageBuffer>,
    #[storage(2, read_only, visibility(compute))]
    pub positive_cell_offsets: Handle<ShaderStorageBuffer>,
    #[storage(3, visibility(compute))]
    pub sorted_positive_particles: Handle<ShaderStorageBuffer>,
    #[uniform(4)]
    pub grid_size: UVec2,
    #[storage(5, visibility(compute))]
    pub positive_cell_cursor: Handle<ShaderStorageBuffer>,
}

impl SortPositiveParticlesResource {
    pub fn new(pls_resources: &PLSResources, grid_size: UVec2) -> Self {
        let positive_particles = pls_resources.positive_particles.clone();
        let positive_particles_count = pls_resources.positive_particles_count.clone();
        let positive_cell_offsets = pls_resources.positive_cell_offsets.clone();
        let sorted_positive_particles = pls_resources.sorted_positive_particles.clone();
        let positive_cell_cursor = pls_resources.positive_cell_cursor.clone();

        Self {
            positive_particles,
            positive_particles_count,
            positive_cell_offsets,
            sorted_positive_particles,
            grid_size,
            positive_cell_cursor,
        }
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct SortNegativeParticlesResource {
    #[storage(0, read_only, visibility(compute))]
    pub negative_particles: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub negative_particles_count: Handle<ShaderStorageBuffer>,
    #[storage(2, read_only, visibility(compute))]
    pub negative_cell_offsets: Handle<ShaderStorageBuffer>,
    #[storage(3, visibility(compute))]
    pub sorted_negative_particles: Handle<ShaderStorageBuffer>,
    #[uniform(4)]
    pub grid_size: UVec2,
    #[storage(5, visibility(compute))]
    pub negative_cell_cursor: Handle<ShaderStorageBuffer>,
}

impl SortNegativeParticlesResource {
    pub fn new(pls_resources: &PLSResources, grid_size: UVec2) -> Self {
        let negative_particles = pls_resources.negative_particles.clone();
        let negative_particles_count = pls_resources.negative_particles_count.clone();
        let negative_cell_offsets = pls_resources.negative_cell_offsets.clone();
        let sorted_negative_particles = pls_resources.sorted_negative_particles.clone();
        let negative_cell_cursor = pls_resources.negative_cell_cursor.clone();

        Self {
            negative_particles,
            negative_particles_count,
            negative_cell_offsets,
            sorted_negative_particles,
            grid_size,
            negative_cell_cursor,
        }
    }
}

#[derive(Resource)]
pub(crate) struct SortParticlesPipeline {
    pub pipeline: SingleComputePipeline,
}

#[derive(Component)]
pub(crate) struct SortPositiveParticlesBindGroup {
    pub bind_group: BindGroup,
}

#[derive(Component)]
pub(crate) struct SortNegativeParticlesBindGroup {
    pub bind_group: BindGroup,
}

impl FromWorld for SortParticlesPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<SortPositiveParticlesResource>(
            world,
            "SortParticlesPipeline",
            embedded_path!("shaders/sort_particles.wgsl"),
            "sort_particles",
        );

        Self { pipeline }
    }
}

fn prepare_bind_groups_positive<'a>(
    mut commands: Commands,
    pipelines: Res<SortParticlesPipeline>,
    query: Query<(Entity, &SortPositiveParticlesResource)>,
    render_device: Res<RenderDevice>,
    mut param: (
        Res<'a, RenderAssets<GpuImage>>,
        Res<'a, FallbackImage>,
        Res<'a, RenderAssets<GpuShaderStorageBuffer>>,
    ),
) {
    for (entity, resource) in &query {
        let bind_group = resource
            .as_bind_group(
                &pipelines.pipeline.bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        commands
            .entity(entity)
            .insert(SortPositiveParticlesBindGroup { bind_group });
    }
}

fn prepare_bind_groups_negative<'a>(
    mut commands: Commands,
    pipelines: Res<SortParticlesPipeline>,
    query: Query<(Entity, &SortNegativeParticlesResource)>,
    render_device: Res<RenderDevice>,
    mut param: (
        Res<'a, RenderAssets<GpuImage>>,
        Res<'a, FallbackImage>,
        Res<'a, RenderAssets<GpuShaderStorageBuffer>>,
    ),
) {
    for (entity, resource) in &query {
        let bind_group = resource
            .as_bind_group(
                &pipelines.pipeline.bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        commands
            .entity(entity)
            .insert(SortNegativeParticlesBindGroup { bind_group });
    }
}
