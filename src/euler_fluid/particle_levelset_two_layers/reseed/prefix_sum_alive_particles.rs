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

pub(crate) struct PrefixSumAlivePositiveParticlesPass;

impl FluidComputePass for PrefixSumAlivePositiveParticlesPass {
    type Pipeline = PrefixSumPipeline;
    type Resource = PrefixSumAlivePositiveParticlesResource;
    type BG = PrefixSumAlivePositiveParticlesBindGroup;
}

pub(crate) struct PrefixSumAliveNegativeParticlesPass;

impl FluidComputePass for PrefixSumAliveNegativeParticlesPass {
    type Pipeline = PrefixSumPipeline;
    type Resource = PrefixSumAliveNegativeParticlesResource;
    type BG = PrefixSumAliveNegativeParticlesBindGroup;
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct PrefixSumAlivePositiveParticlesResource {
    #[storage(0, read_only, visibility(compute))]
    pub positive_alive_particles_mask: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(compute))]
    pub positive_alive_particles_mask_scan: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub positive_alive_particles_mask_block_sums: Handle<ShaderStorageBuffer>,
}

impl PrefixSumAlivePositiveParticlesResource {
    pub fn new(pls_resources: &PLSResources) -> Self {
        let positive_alive_particles_mask = pls_resources.positive_alive_particles_mask.clone();
        let positive_alive_particles_mask_scan =
            pls_resources.positive_alive_particles_mask_scan.clone();
        let positive_alive_particles_mask_block_sums = pls_resources
            .positive_alive_particles_mask_block_sums
            .clone();

        Self {
            positive_alive_particles_mask,
            positive_alive_particles_mask_scan,
            positive_alive_particles_mask_block_sums,
        }
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct PrefixSumAliveNegativeParticlesResource {
    #[storage(0, read_only, visibility(compute))]
    pub negative_alive_particles_mask: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(compute))]
    pub negative_alive_particles_mask_scan: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub negative_alive_particles_mask_block_sums: Handle<ShaderStorageBuffer>,
}

impl PrefixSumAliveNegativeParticlesResource {
    pub fn new(pls_resources: &PLSResources) -> Self {
        let negative_alive_particles_mask = pls_resources.negative_alive_particles_mask.clone();
        let negative_alive_particles_mask_scan =
            pls_resources.negative_alive_particles_mask_scan.clone();
        let negative_alive_particles_mask_block_sums = pls_resources
            .negative_alive_particles_mask_block_sums
            .clone();

        Self {
            negative_alive_particles_mask,
            negative_alive_particles_mask_scan,
            negative_alive_particles_mask_block_sums,
        }
    }
}

#[derive(Component)]
pub(crate) struct PrefixSumAlivePositiveParticlesBindGroup {
    pub bind_group: BindGroup,
}

#[derive(Component)]
pub(crate) struct PrefixSumAliveNegativeParticlesBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for PrefixSumAlivePositiveParticlesBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}

impl From<BindGroup> for PrefixSumAliveNegativeParticlesBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
