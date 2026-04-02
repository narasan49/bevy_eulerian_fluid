use crate::{
    particle_levelset_two_layers::{
        levelset_correction::mark_escaped_particles::MarkEscapedParticlesPipeline,
        plugin::PLSResources,
    },
    plugin::FluidComputePass,
};
use bevy::{
    asset::embedded_asset,
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{AsBindGroup, BindGroup},
        storage::ShaderStorageBuffer,
    },
};

pub(crate) struct MarkEscapedParticlesSecondPass;

impl FluidComputePass for MarkEscapedParticlesSecondPass {
    type Pipeline = MarkEscapedParticlesPipeline;
    type Resource = MarkEscapedParticlesSecondResource;
    type BG = MarkEscapedParticlesSecondBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/mark_escaped_particles.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct MarkEscapedParticlesSecondResource {
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

impl MarkEscapedParticlesSecondResource {
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

#[derive(Component)]
pub(crate) struct MarkEscapedParticlesSecondBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for MarkEscapedParticlesSecondBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
