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

pub(super) struct InitializeParticlesPass;

impl FluidComputePass for InitializeParticlesPass {
    type P = InitializeParticlesPipeline;

    type Resource = InitializeParticlesResource;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/initialize_particles.wgsl");
    }

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups.into_configs()
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct InitializeParticlesResource {
    #[storage(0, visibility(compute))]
    pub positive_particles_count: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(compute))]
    pub positive_particles: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub negative_particles_count: Handle<ShaderStorageBuffer>,
    #[storage(3, visibility(compute))]
    pub negative_particles: Handle<ShaderStorageBuffer>,
    #[storage_texture(4, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[storage_texture(5, image_format = Rg32Float, access = ReadOnly)]
    pub grad_levelset_air: Handle<Image>,
    #[storage_texture(6, image_format = R32Uint, access = ReadOnly)]
    pub interface_band_mask: Handle<Image>,
}

impl InitializeParticlesResource {
    pub fn new(
        pls_resources: &PLSResources,
        levelset_air: &Handle<Image>,
        grad_levelset_air: &Handle<Image>,
    ) -> Self {
        let positive_particles_count = pls_resources.positive_particles_count.clone();
        let positive_particles = pls_resources.positive_particles.clone();
        let negative_particles_count = pls_resources.negative_particles_count.clone();
        let negative_particles = pls_resources.negative_particles.clone();
        let interface_band_mask = pls_resources.interface_band_mask.clone();

        Self {
            positive_particles_count,
            positive_particles,
            negative_particles_count,
            negative_particles,
            levelset_air: levelset_air.clone(),
            grad_levelset_air: grad_levelset_air.clone(),
            interface_band_mask,
        }
    }
}

#[derive(Resource)]
pub(crate) struct InitializeParticlesPipeline {
    pub pipeline: SingleComputePipeline,
}

#[derive(Component)]
pub(crate) struct InitializeParticlesBindGroup {
    pub bind_group: BindGroup,
}

impl FromWorld for InitializeParticlesPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<InitializeParticlesResource>(
            world,
            "InitializeParticlesPipeline",
            embedded_path!("shaders/initialize_particles.wgsl"),
            "initialize_particles",
        );

        Self { pipeline }
    }
}

fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipelines: Res<InitializeParticlesPipeline>,
    query: Query<(Entity, &InitializeParticlesResource)>,
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
            .insert(InitializeParticlesBindGroup { bind_group });
    }
}
