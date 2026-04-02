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

pub(super) struct UpdatePositiveParticlesCountPass;

impl FluidComputePass for UpdatePositiveParticlesCountPass {
    type Pipeline = UpdateParticlesCountPipeline;
    type Resource = UpdatePositiveParticlesCountResource;
    type BG = UpdatePositiveParticlesCountBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/update_particles_count.wgsl");
    }
}

pub(super) struct UpdateNegativeParticlesCountPass;

impl FluidComputePass for UpdateNegativeParticlesCountPass {
    type Pipeline = UpdateParticlesCountPipeline;
    type Resource = UpdateNegativeParticlesCountResource;
    type BG = UpdateNegativeParticlesCountBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/update_particles_count.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct UpdatePositiveParticlesCountResource {
    #[storage(0, read_only, visibility(compute))]
    pub positive_alive_particles_mask: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub positive_alive_particles_mask_scan: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub positive_particles_count: Handle<ShaderStorageBuffer>,
}

impl UpdatePositiveParticlesCountResource {
    pub fn new(pls_resources: &PLSResources) -> Self {
        let positive_alive_particles_mask = pls_resources.positive_alive_particles_mask.clone();
        let positive_alive_particles_mask_scan =
            pls_resources.positive_alive_particles_mask_scan.clone();
        let positive_particles_count = pls_resources.positive_particles_count.clone();

        Self {
            positive_alive_particles_mask,
            positive_alive_particles_mask_scan,
            positive_particles_count,
        }
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct UpdateNegativeParticlesCountResource {
    #[storage(0, read_only, visibility(compute))]
    pub negative_alive_particles_mask: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub negative_alive_particles_mask_scan: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub negative_particles_count: Handle<ShaderStorageBuffer>,
}

impl UpdateNegativeParticlesCountResource {
    pub fn new(pls_resources: &PLSResources) -> Self {
        let negative_alive_particles_mask = pls_resources.negative_alive_particles_mask.clone();
        let negative_alive_particles_mask_scan =
            pls_resources.negative_alive_particles_mask_scan.clone();
        let negative_particles_count = pls_resources.negative_particles_count.clone();

        Self {
            negative_alive_particles_mask,
            negative_alive_particles_mask_scan,
            negative_particles_count,
        }
    }
}

#[derive(Resource)]
pub(crate) struct UpdateParticlesCountPipeline {
    pub pipeline: SingleComputePipeline,
}

#[derive(Component)]
pub(crate) struct UpdatePositiveParticlesCountBindGroup {
    pub bind_group: BindGroup,
}

#[derive(Component)]
pub(crate) struct UpdateNegativeParticlesCountBindGroup {
    pub bind_group: BindGroup,
}

impl FromWorld for UpdateParticlesCountPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<UpdatePositiveParticlesCountResource>(
            world,
            "UpdateParticlesCountPipeline",
            embedded_path!("shaders/update_particles_count.wgsl"),
            "update_particles_count",
        );

        Self { pipeline }
    }
}

impl HasBindGroupLayout for UpdateParticlesCountPipeline {
    fn bind_group_layout(&self) -> &bevy::render::render_resource::BindGroupLayoutDescriptor {
        &self.pipeline.bind_group_layout
    }
}

impl From<BindGroup> for UpdatePositiveParticlesCountBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}

impl From<BindGroup> for UpdateNegativeParticlesCountBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
