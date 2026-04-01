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

pub(crate) struct MarkEscapedParticlesPass;

impl FluidComputePass for MarkEscapedParticlesPass {
    type P = MarkEscapedParticlesPipeline;

    type Resource = MarkEscapedParticlesResource;

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups.into_configs()
    }

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/mark_escaped_particles.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct MarkEscapedParticlesResource {
    #[storage(0, read_only, visibility(compute))]
    pub positive_particles_count: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(compute))]
    pub positive_particles: Handle<ShaderStorageBuffer>,
    #[storage(2, read_only, visibility(compute))]
    pub negative_particles_count: Handle<ShaderStorageBuffer>,
    #[storage(3, visibility(compute))]
    pub negative_particles: Handle<ShaderStorageBuffer>,
    #[storage_texture(4, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
}

impl MarkEscapedParticlesResource {
    pub fn new(pls_resources: &PLSResources, levelset_air: &Handle<Image>) -> Self {
        let positive_particles_count = pls_resources.positive_particles_count.clone();
        let positive_particles = pls_resources.positive_particles.clone();
        let negative_particles_count = pls_resources.negative_particles_count.clone();
        let negative_particles = pls_resources.negative_particles.clone();

        Self {
            positive_particles_count,
            positive_particles,
            negative_particles_count,
            negative_particles,
            levelset_air: levelset_air.clone(),
        }
    }
}

#[derive(Resource)]
pub(crate) struct MarkEscapedParticlesPipeline {
    pub pipeline: SingleComputePipeline,
}

impl FromWorld for MarkEscapedParticlesPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<MarkEscapedParticlesResource>(
            world,
            "MarkEscapedParticlesPipeline",
            embedded_path!("shaders/mark_escaped_particles.wgsl"),
            "mark_escaped_particles",
        );

        Self { pipeline }
    }
}

#[derive(Component)]
pub(crate) struct MarkEscapedParticlesBindGroup {
    pub bind_group: BindGroup,
}

pub(super) fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipeline: Res<MarkEscapedParticlesPipeline>,
    query: Query<(Entity, &MarkEscapedParticlesResource)>,
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
            .insert(MarkEscapedParticlesBindGroup { bind_group });
    }
}
