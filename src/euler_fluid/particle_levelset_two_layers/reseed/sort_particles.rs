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

pub(super) struct SortPositiveParticlesPass;

impl FluidComputePass for SortPositiveParticlesPass {
    type Pipeline = SortParticlesPipeline;
    type Resource = SortPositiveParticlesResource;
    type BG = SortPositiveParticlesBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/sort_particles.wgsl");
    }
}

pub(super) struct SortNegativeParticlesPass;

impl FluidComputePass for SortNegativeParticlesPass {
    type Pipeline = SortParticlesPipeline;
    type Resource = SortNegativeParticlesResource;
    type BG = SortNegativeParticlesBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/sort_particles.wgsl");
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

impl HasBindGroupLayout for SortParticlesPipeline {
    fn bind_group_layout(&self) -> &bevy::render::render_resource::BindGroupLayoutDescriptor {
        &self.pipeline.bind_group_layout
    }
}

impl From<BindGroup> for SortPositiveParticlesBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}

impl From<BindGroup> for SortNegativeParticlesBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
