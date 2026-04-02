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

pub(super) struct DeletePositiveParticlesPass;

impl FluidComputePass for DeletePositiveParticlesPass {
    type Pipeline = DeleteParticlesPipeline;
    type Resource = DeletePositiveParticlesResource;
    type BG = DeletePositiveParticlesBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/delete_particles.wgsl");
    }
}

pub(super) struct DeleteNegativeParticlesPass;

impl FluidComputePass for DeleteNegativeParticlesPass {
    type Pipeline = DeleteParticlesPipeline;
    type Resource = DeleteNegativeParticlesResource;
    type BG = DeleteNegativeParticlesBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/delete_particles.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct DeletePositiveParticlesResource {
    #[storage(0, read_only, visibility(compute))]
    pub positive_alive_particles_mask: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub positive_alive_particles_mask_scan: Handle<ShaderStorageBuffer>,
    #[storage(2, read_only, visibility(compute))]
    pub sorted_positive_particles: Handle<ShaderStorageBuffer>,
    #[storage(3, visibility(compute))]
    pub positive_particles: Handle<ShaderStorageBuffer>,
}

impl DeletePositiveParticlesResource {
    pub fn new(pls_resources: &PLSResources) -> Self {
        let positive_alive_particles_mask = pls_resources.positive_alive_particles_mask.clone();
        let positive_alive_particles_mask_scan =
            pls_resources.positive_alive_particles_mask_scan.clone();
        let sorted_positive_particles = pls_resources.sorted_positive_particles.clone();
        let positive_particles = pls_resources.positive_particles.clone();

        Self {
            positive_alive_particles_mask,
            positive_alive_particles_mask_scan,
            sorted_positive_particles,
            positive_particles,
        }
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct DeleteNegativeParticlesResource {
    #[storage(0, read_only, visibility(compute))]
    pub negative_alive_particles_mask: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub negative_alive_particles_mask_scan: Handle<ShaderStorageBuffer>,
    #[storage(2, read_only, visibility(compute))]
    pub sorted_negative_particles: Handle<ShaderStorageBuffer>,
    #[storage(3, visibility(compute))]
    pub negative_particles: Handle<ShaderStorageBuffer>,
}

impl DeleteNegativeParticlesResource {
    pub fn new(pls_resources: &PLSResources) -> Self {
        let negative_alive_particles_mask = pls_resources.negative_alive_particles_mask.clone();
        let negative_alive_particles_mask_scan =
            pls_resources.negative_alive_particles_mask_scan.clone();
        let sorted_negative_particles = pls_resources.sorted_negative_particles.clone();
        let negative_particles = pls_resources.negative_particles.clone();

        Self {
            negative_alive_particles_mask,
            negative_alive_particles_mask_scan,
            sorted_negative_particles,
            negative_particles,
        }
    }
}

#[derive(Resource)]
pub(crate) struct DeleteParticlesPipeline {
    pub pipeline: SingleComputePipeline,
}

impl FromWorld for DeleteParticlesPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<DeletePositiveParticlesResource>(
            world,
            "DeleteParticlesPipeline",
            embedded_path!("shaders/delete_particles.wgsl"),
            "delete_particles",
        );

        Self { pipeline }
    }
}

impl HasBindGroupLayout for DeleteParticlesPipeline {
    fn bind_group_layout(&self) -> &bevy::render::render_resource::BindGroupLayoutDescriptor {
        &self.pipeline.bind_group_layout
    }
}

#[derive(Component)]
pub(crate) struct DeletePositiveParticlesBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for DeletePositiveParticlesBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}

#[derive(Component)]
pub(crate) struct DeleteNegativeParticlesBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for DeleteNegativeParticlesBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
