use crate::{
    particle_levelset_two_layers::plugin::PLSResources,
    pipeline::{HasBindGroupLayout, SingleComputePipeline},
    plugin::FluidComputePass,
};
use bevy::{
    asset::{embedded_asset, embedded_path},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{AsBindGroup, BindGroup},
        storage::ShaderStorageBuffer,
    },
};

pub(crate) struct MarkEscapedParticlesPass;

impl FluidComputePass for MarkEscapedParticlesPass {
    type Pipeline = MarkEscapedParticlesPipeline;
    type Resource = MarkEscapedParticlesResource;
    type BG = MarkEscapedParticlesBindGroup;

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

impl HasBindGroupLayout for MarkEscapedParticlesPipeline {
    fn bind_group_layout(&self) -> &bevy::render::render_resource::BindGroupLayoutDescriptor {
        &self.pipeline.bind_group_layout
    }
}

impl From<BindGroup> for MarkEscapedParticlesBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
