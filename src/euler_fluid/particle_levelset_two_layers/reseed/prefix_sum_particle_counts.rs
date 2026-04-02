use bevy::{
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{AsBindGroup, BindGroup},
        storage::ShaderStorageBuffer,
    },
};

use crate::{
    common_pass::prefix_sum::PrefixSumPipeline, particle_levelset_two_layers::plugin::PLSResources,
    plugin::FluidComputePass,
};

pub(crate) struct PrefixSumPositiveParticlesCountPass;

impl FluidComputePass for PrefixSumPositiveParticlesCountPass {
    type Pipeline = PrefixSumPipeline;
    type Resource = PrefixSumPositiveParticlesCountResource;
    type BG = PrefixSumPositiveParticlesCountBindGroup;
}

pub(crate) struct PrefixSumNegativeParticlesCountPass;

impl FluidComputePass for PrefixSumNegativeParticlesCountPass {
    type Pipeline = PrefixSumPipeline;
    type Resource = PrefixSumNegativeParticlesCountResource;
    type BG = PrefixSumNegativeParticlesCountBindGroup;
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct PrefixSumPositiveParticlesCountResource {
    #[storage(0, read_only, visibility(compute))]
    num_positive_particles_in_cell: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(compute))]
    positive_cell_offsets: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    num_positive_particles_block_sums: Handle<ShaderStorageBuffer>,
}

impl PrefixSumPositiveParticlesCountResource {
    pub fn new(pls_resources: &PLSResources) -> Self {
        let num_positive_particles_in_cell = pls_resources.num_positive_particles_in_cell.clone();
        let positive_cell_offsets = pls_resources.positive_cell_offsets.clone();
        let num_positive_particles_block_sums =
            pls_resources.num_positive_particles_block_sums.clone();

        Self {
            num_positive_particles_in_cell,
            positive_cell_offsets,
            num_positive_particles_block_sums,
        }
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct PrefixSumNegativeParticlesCountResource {
    #[storage(0, read_only, visibility(compute))]
    num_negative_particles_in_cell: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(compute))]
    negative_cell_offsets: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    num_negative_particles_block_sums: Handle<ShaderStorageBuffer>,
}

impl PrefixSumNegativeParticlesCountResource {
    pub fn new(pls_resources: &PLSResources) -> Self {
        let num_negative_particles_in_cell = pls_resources.num_negative_particles_in_cell.clone();
        let negative_cell_offsets = pls_resources.negative_cell_offsets.clone();
        let num_negative_particles_block_sums =
            pls_resources.num_negative_particles_block_sums.clone();

        Self {
            num_negative_particles_in_cell,
            negative_cell_offsets,
            num_negative_particles_block_sums,
        }
    }
}

#[derive(Component)]
pub(crate) struct PrefixSumPositiveParticlesCountBindGroup {
    pub bind_group: BindGroup,
}

#[derive(Component)]
pub(crate) struct PrefixSumNegativeParticlesCountBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for PrefixSumPositiveParticlesCountBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}

impl From<BindGroup> for PrefixSumNegativeParticlesCountBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
