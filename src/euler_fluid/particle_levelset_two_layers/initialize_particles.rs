use bevy::{
    asset::{embedded_asset, embedded_path},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{AsBindGroup, BindGroup},
        storage::ShaderStorageBuffer,
    },
};

use crate::{
    particle_levelset_two_layers::plugin::PLSResources,
    pipeline::{HasBindGroupLayout, SingleComputePipeline},
    plugin::FluidComputePass,
};

pub(super) struct InitializeParticlesPass;

impl FluidComputePass for InitializeParticlesPass {
    type Pipeline = InitializeParticlesPipeline;
    type Resource = InitializeParticlesResource;
    type BG = InitializeParticlesBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/initialize_particles.wgsl");
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

impl HasBindGroupLayout for InitializeParticlesPipeline {
    fn bind_group_layout(&self) -> &bevy::render::render_resource::BindGroupLayoutDescriptor {
        &self.pipeline.bind_group_layout
    }
}

impl From<BindGroup> for InitializeParticlesBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
