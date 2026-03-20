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

pub(super) struct DeletePositiveParticlesPass;

impl FluidComputePass for DeletePositiveParticlesPass {
    type P = DeleteParticlesPipeline;

    type Resource = DeletePositiveParticlesResource;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/delete_particles.wgsl");
    }

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups_positive.into_configs()
    }
}

pub(super) struct DeleteNegativeParticlesPass;

impl FluidComputePass for DeleteNegativeParticlesPass {
    type P = DeleteParticlesPipeline;

    type Resource = DeleteNegativeParticlesResource;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/delete_particles.wgsl");
    }

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups_negative.into_configs()
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct DeletePositiveParticlesResource {
    #[storage(0, read_only, visibility(compute))]
    pub positive_alive_particles_mask: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub positive_alive_particles_mask_scan: Handle<ShaderStorageBuffer>,
    #[storage(2, read_only, visibility(compute))]
    pub sorted_positive_particles: Handle<ShaderStorageBuffer>,
    #[storage(3, visibility(compute))]
    pub positive_particles: Handle<ShaderStorageBuffer>,
}

impl DeletePositiveParticlesResource {
    pub fn new(pls_resources: &PLSResources) -> Self {
        let positive_alive_particles_mask = pls_resources.positive_alive_particles_mask.clone();
        let positive_alive_particles_mask_scan =
            pls_resources.positive_alive_particles_mask_scan.clone();
        let sorted_positive_particles = pls_resources.sorted_positive_particles.clone();
        let positive_particles = pls_resources.positive_particles.clone();

        Self {
            positive_alive_particles_mask,
            positive_alive_particles_mask_scan,
            sorted_positive_particles,
            positive_particles,
        }
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct DeleteNegativeParticlesResource {
    #[storage(0, read_only, visibility(compute))]
    pub negative_alive_particles_mask: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub negative_alive_particles_mask_scan: Handle<ShaderStorageBuffer>,
    #[storage(2, read_only, visibility(compute))]
    pub sorted_negative_particles: Handle<ShaderStorageBuffer>,
    #[storage(3, visibility(compute))]
    pub negative_particles: Handle<ShaderStorageBuffer>,
}

impl DeleteNegativeParticlesResource {
    pub fn new(pls_resources: &PLSResources) -> Self {
        let negative_alive_particles_mask = pls_resources.negative_alive_particles_mask.clone();
        let negative_alive_particles_mask_scan =
            pls_resources.negative_alive_particles_mask_scan.clone();
        let sorted_negative_particles = pls_resources.sorted_negative_particles.clone();
        let negative_particles = pls_resources.negative_particles.clone();

        Self {
            negative_alive_particles_mask,
            negative_alive_particles_mask_scan,
            sorted_negative_particles,
            negative_particles,
        }
    }
}

#[derive(Resource)]
pub(crate) struct DeleteParticlesPipeline {
    pub pipeline: SingleComputePipeline,
}

#[derive(Component)]
pub(crate) struct DeletePositiveParticlesBindGroup {
    pub bind_group: BindGroup,
}

#[derive(Component)]
pub(crate) struct DeleteNegativeParticlesBindGroup {
    pub bind_group: BindGroup,
}

impl FromWorld for DeleteParticlesPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<DeletePositiveParticlesResource>(
            world,
            "DeleteParticlesPipeline",
            embedded_path!("shaders/delete_particles.wgsl"),
            "delete_particles",
        );

        Self { pipeline }
    }
}

fn prepare_bind_groups_positive<'a>(
    mut commands: Commands,
    pipelines: Res<DeleteParticlesPipeline>,
    query: Query<(Entity, &DeletePositiveParticlesResource)>,
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
            .insert(DeletePositiveParticlesBindGroup { bind_group });
    }
}

fn prepare_bind_groups_negative<'a>(
    mut commands: Commands,
    pipelines: Res<DeleteParticlesPipeline>,
    query: Query<(Entity, &DeleteNegativeParticlesResource)>,
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
            .insert(DeleteNegativeParticlesBindGroup { bind_group });
    }
}
