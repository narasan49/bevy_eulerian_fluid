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

pub(super) struct AddPositiveParticlesPass;

impl FluidComputePass for AddPositiveParticlesPass {
    type P = AddParticlesPipeline;

    type Resource = AddPositiveParticlesResource;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/add_particles.wgsl");
    }

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups_positive.into_configs()
    }
}

pub(super) struct AddNegativeParticlesPass;

impl FluidComputePass for AddNegativeParticlesPass {
    type P = AddParticlesPipeline;

    type Resource = AddNegativeParticlesResource;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/add_particles.wgsl");
    }

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups_negative.into_configs()
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct AddPositiveParticlesResource {
    #[storage(0, read_only, visibility(compute))]
    pub positive_particles_to_be_added: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(compute))]
    pub positive_particles: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub positive_particles_count: Handle<ShaderStorageBuffer>,
    #[storage_texture(3, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[storage_texture(4, image_format = Rg32Float, access = ReadOnly)]
    pub grad_levelset_air: Handle<Image>,
    #[uniform(5)]
    pub sign: f32,
}

impl AddPositiveParticlesResource {
    pub fn new(
        pls_resources: &PLSResources,
        levelset_air: &Handle<Image>,
        grad_levelset_air: &Handle<Image>,
    ) -> Self {
        let positive_particles_to_be_added = pls_resources.positive_particles_to_be_added.clone();
        let positive_particles = pls_resources.positive_particles.clone();
        let positive_particles_count = pls_resources.positive_particles_count.clone();

        Self {
            positive_particles_to_be_added,
            positive_particles,
            positive_particles_count,
            levelset_air: levelset_air.clone(),
            grad_levelset_air: grad_levelset_air.clone(),
            sign: 1.0,
        }
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct AddNegativeParticlesResource {
    #[storage(0, read_only, visibility(compute))]
    pub negative_particles_to_be_added: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(compute))]
    pub negative_particles: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub negative_particles_count: Handle<ShaderStorageBuffer>,
    #[storage_texture(3, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[storage_texture(4, image_format = Rg32Float, access = ReadOnly)]
    pub grad_levelset_air: Handle<Image>,
    #[uniform(5)]
    pub sign: f32,
}

impl AddNegativeParticlesResource {
    pub fn new(
        pls_resources: &PLSResources,
        levelset_air: &Handle<Image>,
        grad_levelset_air: &Handle<Image>,
    ) -> Self {
        let negative_particles_to_be_added = pls_resources.negative_particles_to_be_added.clone();
        let negative_particles = pls_resources.negative_particles.clone();
        let negative_particles_count = pls_resources.negative_particles_count.clone();

        Self {
            negative_particles_to_be_added,
            negative_particles,
            negative_particles_count,
            levelset_air: levelset_air.clone(),
            grad_levelset_air: grad_levelset_air.clone(),
            sign: -1.0,
        }
    }
}

#[derive(Resource)]
pub(crate) struct AddParticlesPipeline {
    pub pipeline: SingleComputePipeline,
}

#[derive(Component)]
pub(crate) struct AddPositiveParticlesBindGroup {
    pub bind_group: BindGroup,
}

#[derive(Component)]
pub(crate) struct AddNegativeParticlesBindGroup {
    pub bind_group: BindGroup,
}

impl FromWorld for AddParticlesPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<AddPositiveParticlesResource>(
            world,
            "AddParticlesPipeline",
            embedded_path!("shaders/add_particles.wgsl"),
            "add_particles",
        );

        Self { pipeline }
    }
}

fn prepare_bind_groups_positive<'a>(
    mut commands: Commands,
    pipelines: Res<AddParticlesPipeline>,
    query: Query<(Entity, &AddPositiveParticlesResource)>,
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
            .insert(AddPositiveParticlesBindGroup { bind_group });
    }
}

fn prepare_bind_groups_negative<'a>(
    mut commands: Commands,
    pipelines: Res<AddParticlesPipeline>,
    query: Query<(Entity, &AddNegativeParticlesResource)>,
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
            .insert(AddNegativeParticlesBindGroup { bind_group });
    }
}
