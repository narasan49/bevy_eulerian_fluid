use bevy::{
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
    common_pass::prefix_sum::PrefixSumPipeline, particle_levelset_two_layers::plugin::PLSResources,
    plugin::FluidComputePass,
};

pub(crate) struct PrefixSumPositiveParticlesCountPass;

impl FluidComputePass for PrefixSumPositiveParticlesCountPass {
    type P = PrefixSumPipeline;

    type Resource = PrefixSumPositiveParticlesCountResource;

    fn prepare_bind_groups_system(
    ) -> bevy::ecs::schedule::ScheduleConfigs<bevy::ecs::system::ScheduleSystem> {
        prepare_bind_groups_positive.into_configs()
    }
}

pub(crate) struct PrefixSumNegativeParticlesCountPass;

impl FluidComputePass for PrefixSumNegativeParticlesCountPass {
    type P = PrefixSumPipeline;

    type Resource = PrefixSumNegativeParticlesCountResource;

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups_negative.into_configs()
    }

    // fn register_assets(app: &mut App) {
    //     embedded_asset!(app, "shaders/reseed_particles.wgsl");
    // }
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

fn prepare_bind_groups_positive<'a>(
    mut commands: Commands,
    pipeline: Res<PrefixSumPipeline>,
    query: Query<(Entity, &PrefixSumNegativeParticlesCountResource)>,
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
            .insert(PrefixSumNegativeParticlesCountBindGroup { bind_group });
    }
}

fn prepare_bind_groups_negative<'a>(
    mut commands: Commands,
    pipeline: Res<PrefixSumPipeline>,
    query: Query<(Entity, &PrefixSumPositiveParticlesCountResource)>,
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
            .insert(PrefixSumPositiveParticlesCountBindGroup { bind_group });
    }
}
