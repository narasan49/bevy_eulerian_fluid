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

pub(crate) struct UpdatePositiveParticleRadiiPass;

impl FluidComputePass for UpdatePositiveParticleRadiiPass {
    type Pipeline = UpdateParticleRadiiPipeline;
    type Resource = UpdatePositiveParticleRadiiResource;
    type BG = UpdatePositiveParticleRadiiBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/update_particle_radii.wgsl");
    }
}

pub(crate) struct UpdateNegativeParticleRadiiPass;

impl FluidComputePass for UpdateNegativeParticleRadiiPass {
    type Pipeline = UpdateParticleRadiiPipeline;
    type Resource = UpdateNegativeParticleRadiiResource;
    type BG = UpdateNegativeParticleRadiiBindGroup;

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

impl HasBindGroupLayout for UpdateParticleRadiiPipeline {
    fn bind_group_layout(&self) -> &bevy::render::render_resource::BindGroupLayoutDescriptor {
        &self.pipeline.bind_group_layout
    }
}

impl From<BindGroup> for UpdateNegativeParticleRadiiBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}

impl From<BindGroup> for UpdatePositiveParticleRadiiBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
