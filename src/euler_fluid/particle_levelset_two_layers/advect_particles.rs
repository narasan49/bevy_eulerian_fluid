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

pub(crate) struct AdvectParticlesPass;

impl FluidComputePass for AdvectParticlesPass {
    type Pipeline = AdvectParticlesPipeline;
    type Resource = AdvectParticlesResource;
    type BG = AdvectParticlesBindGroup;

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

impl HasBindGroupLayout for AdvectParticlesPipeline {
    fn bind_group_layout(&self) -> &bevy::render::render_resource::BindGroupLayoutDescriptor {
        &self.pipeline.bind_group_layout
    }
}

impl From<BindGroup> for AdvectParticlesBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
