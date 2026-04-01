use bevy::{
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
    common_pass::prefix_sum::PrefixSumPipeline, particle_levelset_two_layers::plugin::PLSResources,
    plugin::FluidComputePass,
};

pub(crate) struct PrefixSumAlivePositiveParticlesPass;

impl FluidComputePass for PrefixSumAlivePositiveParticlesPass {
    type P = PrefixSumPipeline;

    type Resource = PrefixSumAlivePositiveParticlesResource;

    fn prepare_bind_groups_system(
    ) -> bevy::ecs::schedule::ScheduleConfigs<bevy::ecs::system::ScheduleSystem> {
        prepare_bind_groups_positive.into_configs()
    }
}

pub(crate) struct PrefixSumAliveNegativeParticlesPass;

impl FluidComputePass for PrefixSumAliveNegativeParticlesPass {
    type P = PrefixSumPipeline;

    type Resource = PrefixSumAliveNegativeParticlesResource;

    fn prepare_bind_groups_system(
    ) -> bevy::ecs::schedule::ScheduleConfigs<bevy::ecs::system::ScheduleSystem> {
        prepare_bind_groups_negative.into_configs()
    }
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

fn prepare_bind_groups_positive<'a>(
    mut commands: Commands,
    pipeline: Res<PrefixSumPipeline>,
    query: Query<(Entity, &PrefixSumAlivePositiveParticlesResource)>,
    render_device: Res<RenderDevice>,
    mut param: (
        Res<'a, RenderAssets<GpuImage>>,
        Res<'a, FallbackImage>,
        Res<'a, RenderAssets<GpuShaderStorageBuffer>>,
    ),
) {
    for (entity, resource) in &query {
        let bind_group = resource
            .as_bind_group(&pipeline.bind_group_layout, &render_device, &mut param)
            .unwrap()
            .bind_group;

        commands
            .entity(entity)
            .insert(PrefixSumAlivePositiveParticlesBindGroup { bind_group });
    }
}

fn prepare_bind_groups_negative<'a>(
    mut commands: Commands,
    pipeline: Res<PrefixSumPipeline>,
    query: Query<(Entity, &PrefixSumAliveNegativeParticlesResource)>,
    render_device: Res<RenderDevice>,
    mut param: (
        Res<'a, RenderAssets<GpuImage>>,
        Res<'a, FallbackImage>,
        Res<'a, RenderAssets<GpuShaderStorageBuffer>>,
    ),
) {
    for (entity, resource) in &query {
        let bind_group = resource
            .as_bind_group(&pipeline.bind_group_layout, &render_device, &mut param)
            .unwrap()
            .bind_group;

        commands
            .entity(entity)
            .insert(PrefixSumAliveNegativeParticlesBindGroup { bind_group });
    }
}
