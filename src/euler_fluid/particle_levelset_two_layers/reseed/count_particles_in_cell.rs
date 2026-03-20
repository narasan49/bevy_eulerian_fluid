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

pub(super) struct CountPositiveParticlesInCellPass;

impl FluidComputePass for CountPositiveParticlesInCellPass {
    type P = CountParticlesInCellPipeline;

    type Resource = CountPositiveParticlesInCellResource;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/count_particles_in_cell.wgsl");
    }

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups_positive.into_configs()
    }
}

pub(super) struct CountNegativeParticlesInCellPass;

impl FluidComputePass for CountNegativeParticlesInCellPass {
    type P = CountParticlesInCellPipeline;

    type Resource = CountNegativeParticlesInCellResource;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/count_particles_in_cell.wgsl");
    }

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups_negative.into_configs()
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct CountPositiveParticlesInCellResource {
    #[storage(0, read_only, visibility(compute))]
    pub positive_particles: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub positive_particles_count: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub num_positive_particles_in_cell: Handle<ShaderStorageBuffer>,
    #[uniform(3)]
    pub grid_size: UVec2,
}

impl CountPositiveParticlesInCellResource {
    pub fn new(pls_resources: &PLSResources, grid_size: UVec2) -> Self {
        let positive_particles = pls_resources.positive_particles.clone();
        let positive_particles_count = pls_resources.positive_particles_count.clone();
        let num_positive_particles_in_cell = pls_resources.num_positive_particles_in_cell.clone();

        Self {
            positive_particles,
            positive_particles_count,
            num_positive_particles_in_cell,
            grid_size,
        }
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct CountNegativeParticlesInCellResource {
    #[storage(0, read_only, visibility(compute))]
    pub negative_particles: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub negative_particles_count: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub num_negative_particles_in_cell: Handle<ShaderStorageBuffer>,
    #[uniform(3)]
    pub grid_size: UVec2,
}

impl CountNegativeParticlesInCellResource {
    pub fn new(pls_resources: &PLSResources, grid_size: UVec2) -> Self {
        let negative_particles = pls_resources.negative_particles.clone();
        let negative_particles_count = pls_resources.negative_particles_count.clone();
        let num_negative_particles_in_cell = pls_resources.num_negative_particles_in_cell.clone();

        Self {
            negative_particles,
            negative_particles_count,
            num_negative_particles_in_cell,
            grid_size,
        }
    }
}

#[derive(Resource)]
pub(crate) struct CountParticlesInCellPipeline {
    pub pipeline: SingleComputePipeline,
}

#[derive(Component)]
pub(crate) struct CountPositiveParticlesInCellBindGroup {
    pub bind_group: BindGroup,
}

#[derive(Component)]
pub(crate) struct CountNegativeParticlesInCellBindGroup {
    pub bind_group: BindGroup,
}

impl FromWorld for CountParticlesInCellPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<CountPositiveParticlesInCellResource>(
            world,
            "CountParticlesInCellPipeline",
            embedded_path!("shaders/count_particles_in_cell.wgsl"),
            "count_particles_in_cell",
        );

        Self { pipeline }
    }
}

fn prepare_bind_groups_positive<'a>(
    mut commands: Commands,
    pipelines: Res<CountParticlesInCellPipeline>,
    query: Query<(Entity, &CountPositiveParticlesInCellResource)>,
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
            .insert(CountPositiveParticlesInCellBindGroup { bind_group });
    }
}

fn prepare_bind_groups_negative<'a>(
    mut commands: Commands,
    pipelines: Res<CountParticlesInCellPipeline>,
    query: Query<(Entity, &CountNegativeParticlesInCellResource)>,
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
            .insert(CountNegativeParticlesInCellBindGroup { bind_group });
    }
}
