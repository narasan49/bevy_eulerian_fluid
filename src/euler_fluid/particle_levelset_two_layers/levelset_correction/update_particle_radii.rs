use crate::{
    particle_levelset_two_layers::plugin::PLSResources, pipeline::SingleComputePipeline,
    plugin::FluidComputePass,
};
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

pub(crate) struct UpdatePositiveParticleRadiiPass;

impl FluidComputePass for UpdatePositiveParticleRadiiPass {
    type P = UpdateParticleRadiiPipeline;

    type Resource = UpdatePositiveParticleRadiiResource;

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups_positive.into_configs()
    }

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/update_particle_radii.wgsl");
    }
}

pub(crate) struct UpdateNegativeParticleRadiiPass;

impl FluidComputePass for UpdateNegativeParticleRadiiPass {
    type P = UpdateParticleRadiiPipeline;

    type Resource = UpdateNegativeParticleRadiiResource;

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups_negative.into_configs()
    }

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/update_particle_radii.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct UpdatePositiveParticleRadiiResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[storage(1, read_only, visibility(compute))]
    pub positive_particles_count: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub positive_particles: Handle<ShaderStorageBuffer>,
}

impl UpdatePositiveParticleRadiiResource {
    pub fn new(pls_resources: &PLSResources, levelset_air: &Handle<Image>) -> Self {
        let positive_particles_count = pls_resources.positive_particles_count.clone();
        let positive_particles = pls_resources.positive_particles.clone();
        Self {
            levelset_air: levelset_air.clone(),
            positive_particles_count,
            positive_particles,
        }
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct UpdateNegativeParticleRadiiResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[storage(1, read_only, visibility(compute))]
    pub negative_particles_count: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub negative_particles: Handle<ShaderStorageBuffer>,
}

impl UpdateNegativeParticleRadiiResource {
    pub fn new(pls_resources: &PLSResources, levelset_air: &Handle<Image>) -> Self {
        let negative_particles_count = pls_resources.negative_particles_count.clone();
        let negative_particles = pls_resources.negative_particles.clone();
        Self {
            levelset_air: levelset_air.clone(),
            negative_particles_count,
            negative_particles,
        }
    }
}

#[derive(Resource)]
pub(crate) struct UpdateParticleRadiiPipeline {
    pub pipeline: SingleComputePipeline,
}

impl FromWorld for UpdateParticleRadiiPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<UpdatePositiveParticleRadiiResource>(
            world,
            "UpdateParticleRadiiPipeline",
            embedded_path!("shaders/update_particle_radii.wgsl"),
            "update_particle_radii",
        );

        Self { pipeline }
    }
}

#[derive(Component)]
pub(crate) struct UpdatePositiveParticleRadiiBindGroup {
    pub bind_group: BindGroup,
}

#[derive(Component)]
pub(crate) struct UpdateNegativeParticleRadiiBindGroup {
    pub bind_group: BindGroup,
}

pub(super) fn prepare_bind_groups_positive<'a>(
    mut commands: Commands,
    pipeline: Res<UpdateParticleRadiiPipeline>,
    query: Query<(Entity, &UpdatePositiveParticleRadiiResource)>,
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
                &pipeline.pipeline.bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        commands
            .entity(entity)
            .insert(UpdatePositiveParticleRadiiBindGroup { bind_group });
    }
}

pub(super) fn prepare_bind_groups_negative<'a>(
    mut commands: Commands,
    pipeline: Res<UpdateParticleRadiiPipeline>,
    query: Query<(Entity, &UpdateNegativeParticleRadiiResource)>,
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
                &pipeline.pipeline.bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        commands
            .entity(entity)
            .insert(UpdateNegativeParticleRadiiBindGroup { bind_group });
    }
}
