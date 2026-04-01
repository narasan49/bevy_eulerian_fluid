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

pub(super) struct UpdatePositiveParticlesCountPass;

impl FluidComputePass for UpdatePositiveParticlesCountPass {
    type P = UpdateParticlesCountPipeline;

    type Resource = UpdatePositiveParticlesCountResource;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/update_particles_count.wgsl");
    }

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups_positive.into_configs()
    }
}

pub(super) struct UpdateNegativeParticlesCountPass;

impl FluidComputePass for UpdateNegativeParticlesCountPass {
    type P = UpdateParticlesCountPipeline;

    type Resource = UpdateNegativeParticlesCountResource;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/update_particles_count.wgsl");
    }

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups_negative.into_configs()
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct UpdatePositiveParticlesCountResource {
    #[storage(0, read_only, visibility(compute))]
    pub positive_alive_particles_mask: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub positive_alive_particles_mask_scan: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub positive_particles_count: Handle<ShaderStorageBuffer>,
}

impl UpdatePositiveParticlesCountResource {
    pub fn new(pls_resources: &PLSResources) -> Self {
        let positive_alive_particles_mask = pls_resources.positive_alive_particles_mask.clone();
        let positive_alive_particles_mask_scan =
            pls_resources.positive_alive_particles_mask_scan.clone();
        let positive_particles_count = pls_resources.positive_particles_count.clone();

        Self {
            positive_alive_particles_mask,
            positive_alive_particles_mask_scan,
            positive_particles_count,
        }
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct UpdateNegativeParticlesCountResource {
    #[storage(0, read_only, visibility(compute))]
    pub negative_alive_particles_mask: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub negative_alive_particles_mask_scan: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub negative_particles_count: Handle<ShaderStorageBuffer>,
}

impl UpdateNegativeParticlesCountResource {
    pub fn new(pls_resources: &PLSResources) -> Self {
        let negative_alive_particles_mask = pls_resources.negative_alive_particles_mask.clone();
        let negative_alive_particles_mask_scan =
            pls_resources.negative_alive_particles_mask_scan.clone();
        let negative_particles_count = pls_resources.negative_particles_count.clone();

        Self {
            negative_alive_particles_mask,
            negative_alive_particles_mask_scan,
            negative_particles_count,
        }
    }
}

#[derive(Resource)]
pub(crate) struct UpdateParticlesCountPipeline {
    pub pipeline: SingleComputePipeline,
}

#[derive(Component)]
pub(crate) struct UpdatePositiveParticlesCountBindGroup {
    pub bind_group: BindGroup,
}

#[derive(Component)]
pub(crate) struct UpdateNegativeParticlesCountBindGroup {
    pub bind_group: BindGroup,
}

impl FromWorld for UpdateParticlesCountPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<UpdatePositiveParticlesCountResource>(
            world,
            "UpdateParticlesCountPipeline",
            embedded_path!("shaders/update_particles_count.wgsl"),
            "update_particles_count",
        );

        Self { pipeline }
    }
}

fn prepare_bind_groups_positive<'a>(
    mut commands: Commands,
    pipelines: Res<UpdateParticlesCountPipeline>,
    query: Query<(Entity, &UpdatePositiveParticlesCountResource)>,
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
            .insert(UpdatePositiveParticlesCountBindGroup { bind_group });
    }
}

fn prepare_bind_groups_negative<'a>(
    mut commands: Commands,
    pipelines: Res<UpdateParticlesCountPipeline>,
    query: Query<(Entity, &UpdateNegativeParticlesCountResource)>,
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
            .insert(UpdateNegativeParticlesCountBindGroup { bind_group });
    }
}
