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

pub(crate) struct ReseedPositiveParticlesPass;

impl FluidComputePass for ReseedPositiveParticlesPass {
    type P = ReseedParticlesPipeline;

    type Resource = ReseedPositiveParticlesResource;

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups_positive.into_configs()
    }

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/reseed_particles.wgsl");
    }
}

pub(crate) struct ReseedNegativeParticlesPass;

impl FluidComputePass for ReseedNegativeParticlesPass {
    type P = ReseedParticlesPipeline;

    type Resource = ReseedNegativeParticlesResource;

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups_negative.into_configs()
    }

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/reseed_particles.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct ReseedPositiveParticlesResource {
    #[storage(0, read_only, visibility(compute))]
    pub sorted_particles: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub num_perticles_in_cell: Handle<ShaderStorageBuffer>,
    #[storage(2, read_only, visibility(compute))]
    pub cell_offsets: Handle<ShaderStorageBuffer>,
    #[storage(3, visibility(compute))]
    pub alive_particles_mask: Handle<ShaderStorageBuffer>,
    #[storage(4, visibility(compute))]
    pub particles_to_be_added: Handle<ShaderStorageBuffer>,
    #[storage_texture(5, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[uniform(6)]
    pub grid_size: UVec2,
}

impl ReseedPositiveParticlesResource {
    pub fn new(
        pls_rersources: &PLSResources,
        levelset_air: &Handle<Image>,
        grid_size: UVec2,
    ) -> Self {
        let sorted_particles = pls_rersources.sorted_positive_particles.clone();
        let alive_particles_mask = pls_rersources.positive_alive_particles_mask.clone();
        let num_perticles_in_cell = pls_rersources.num_positive_particles_in_cell.clone();
        let particles_to_be_added = pls_rersources.positive_particles_to_be_added.clone();
        let cell_offsets = pls_rersources.positive_cell_offsets.clone();

        Self {
            sorted_particles,
            alive_particles_mask,
            num_perticles_in_cell,
            particles_to_be_added,
            cell_offsets,
            levelset_air: levelset_air.clone(),
            grid_size,
        }
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct ReseedNegativeParticlesResource {
    #[storage(0, read_only, visibility(compute))]
    pub sorted_particles: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub num_perticles_in_cell: Handle<ShaderStorageBuffer>,
    #[storage(2, read_only, visibility(compute))]
    pub cell_offsets: Handle<ShaderStorageBuffer>,
    #[storage(3, visibility(compute))]
    pub alive_particles_mask: Handle<ShaderStorageBuffer>,
    #[storage(4, visibility(compute))]
    pub particles_to_be_added: Handle<ShaderStorageBuffer>,
    #[storage_texture(5, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[uniform(6)]
    pub grid_size: UVec2,
}

impl ReseedNegativeParticlesResource {
    pub fn new(
        pls_resources: &PLSResources,
        levelset_air: &Handle<Image>,
        grid_size: UVec2,
    ) -> Self {
        let sorted_particles = pls_resources.sorted_negative_particles.clone();
        let alive_particles_mask = pls_resources.negative_alive_particles_mask.clone();
        let num_perticles_in_cell = pls_resources.num_negative_particles_in_cell.clone();
        let particles_to_be_added = pls_resources.negative_particles_to_be_added.clone();
        let cell_offsets = pls_resources.negative_cell_offsets.clone();

        Self {
            sorted_particles,
            alive_particles_mask,
            num_perticles_in_cell,
            particles_to_be_added,
            cell_offsets,
            levelset_air: levelset_air.clone(),
            grid_size,
        }
    }
}

#[derive(Resource)]
pub(crate) struct ReseedParticlesPipeline {
    pub pipeline: SingleComputePipeline,
}

impl FromWorld for ReseedParticlesPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new_with_uniform::<ReseedPositiveParticlesResource>(
            world,
            "ReseedParticlesPipeline",
            embedded_path!("shaders/reseed_particles.wgsl"),
            "reseed_particles",
        );

        Self { pipeline }
    }
}

#[derive(Component)]
pub(crate) struct ReseedPositiveParticlesBindGroup {
    pub bind_group: BindGroup,
}

#[derive(Component)]
pub(crate) struct ReseedNegativeParticlesBindGroup {
    pub bind_group: BindGroup,
}

pub(super) fn prepare_bind_groups_positive<'a>(
    mut commands: Commands,
    pipeline: Res<ReseedParticlesPipeline>,
    query: Query<(Entity, &ReseedPositiveParticlesResource)>,
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
            .insert(ReseedPositiveParticlesBindGroup { bind_group });
    }
}

pub(super) fn prepare_bind_groups_negative<'a>(
    mut commands: Commands,
    pipeline: Res<ReseedParticlesPipeline>,
    query: Query<(Entity, &ReseedNegativeParticlesResource)>,
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
            .insert(ReseedNegativeParticlesBindGroup { bind_group });
    }
}
