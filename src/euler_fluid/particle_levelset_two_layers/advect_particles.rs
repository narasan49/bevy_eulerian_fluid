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

pub(crate) struct AdvectParticlesPass;

impl FluidComputePass for AdvectParticlesPass {
    type P = AdvectParticlesPipeline;

    type Resource = AdvectParticlesResource;

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups.into_configs()
    }

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/advect_particles.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct AdvectParticlesResource {
    #[storage(0, read_only, visibility(compute))]
    pub positive_particles_count: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(compute))]
    pub positive_particles: Handle<ShaderStorageBuffer>,
    #[storage(2, read_only, visibility(compute))]
    pub negative_particles_count: Handle<ShaderStorageBuffer>,
    #[storage(3, visibility(compute))]
    pub negative_particles: Handle<ShaderStorageBuffer>,
    #[storage_texture(4, image_format = R32Float, access = ReadOnly)]
    pub u0: Handle<Image>,
    #[storage_texture(5, image_format = R32Float, access = ReadOnly)]
    pub v0: Handle<Image>,
    #[storage_texture(6, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
}

impl AdvectParticlesResource {
    pub fn new(
        pls_resources: &PLSResources,
        u0: &Handle<Image>,
        v0: &Handle<Image>,
        levelset_air: &Handle<Image>,
    ) -> Self {
        let positive_particles_count = pls_resources.positive_particles_count.clone();
        let positive_particles = pls_resources.positive_particles.clone();
        let negative_particles_count = pls_resources.negative_particles_count.clone();
        let negative_particles = pls_resources.negative_particles.clone();

        Self {
            positive_particles_count,
            positive_particles,
            negative_particles_count,
            negative_particles,
            u0: u0.clone(),
            v0: v0.clone(),
            levelset_air: levelset_air.clone(),
        }
    }
}

#[derive(Resource)]
pub(crate) struct AdvectParticlesPipeline {
    pub pipeline: SingleComputePipeline,
}

impl FromWorld for AdvectParticlesPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new_with_uniform::<AdvectParticlesResource>(
            world,
            "AdvectParticlesPipeline",
            embedded_path!("shaders/advect_particles.wgsl"),
            "advect_particles",
        );

        Self { pipeline }
    }
}

#[derive(Component)]
pub(crate) struct AdvectParticlesBindGroup {
    pub bind_group: BindGroup,
}

pub(super) fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipeline: Res<AdvectParticlesPipeline>,
    query: Query<(Entity, &AdvectParticlesResource)>,
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
            .insert(AdvectParticlesBindGroup { bind_group });
    }
}
