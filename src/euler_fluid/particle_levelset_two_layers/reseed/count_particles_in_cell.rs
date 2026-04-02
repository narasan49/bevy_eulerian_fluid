use bevy::{
    asset::{embedded_asset, embedded_path},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{AsBindGroup, BindGroup},
        storage::ShaderStorageBuffer,
    },
};

use crate::{
    particle_levelset_two_layers::plugin::PLSResources,
    pipeline::{HasBindGroupLayout, SingleComputePipeline},
    plugin::FluidComputePass,
};

pub(super) struct CountPositiveParticlesInCellPass;

impl FluidComputePass for CountPositiveParticlesInCellPass {
    type Pipeline = CountParticlesInCellPipeline;
    type Resource = CountPositiveParticlesInCellResource;
    type BG = CountPositiveParticlesInCellBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/count_particles_in_cell.wgsl");
    }
}

pub(super) struct CountNegativeParticlesInCellPass;

impl FluidComputePass for CountNegativeParticlesInCellPass {
    type Pipeline = CountParticlesInCellPipeline;
    type Resource = CountNegativeParticlesInCellResource;
    type BG = CountNegativeParticlesInCellBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/count_particles_in_cell.wgsl");
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

impl HasBindGroupLayout for CountParticlesInCellPipeline {
    fn bind_group_layout(&self) -> &bevy::render::render_resource::BindGroupLayoutDescriptor {
        &self.pipeline.bind_group_layout
    }
}

#[derive(Component)]
pub(crate) struct CountPositiveParticlesInCellBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for CountPositiveParticlesInCellBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}

#[derive(Component)]
pub(crate) struct CountNegativeParticlesInCellBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for CountNegativeParticlesInCellBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
