use crate::{
    particle_levelset_two_layers::{
        levelset_correction::accumulate_phi_correction::AccumulateLevelSetCorrectionPipeline,
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

pub(crate) struct AccumulateLevelSetCorrectionPlusSecondPass;

impl FluidComputePass for AccumulateLevelSetCorrectionPlusSecondPass {
    type Pipeline = AccumulateLevelSetCorrectionPipeline;
    type Resource = AccumulateLevelSetCorrectionPlusSecondResource;
    type BG = AccumulateLevelSetCorrectionPlusSecondBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/accumulate_levelset_correction.wgsl");
    }
}

pub(crate) struct AccumulateLevelSetCorrectionMinusSecondPass;

impl FluidComputePass for AccumulateLevelSetCorrectionMinusSecondPass {
    type Pipeline = AccumulateLevelSetCorrectionPipeline;
    type Resource = AccumulateLevelSetCorrectionMinusSecondResource;
    type BG = AccumulateLevelSetCorrectionMinusSecondBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/accumulate_levelset_correction.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct AccumulateLevelSetCorrectionPlusSecondResource {
    #[storage(0, read_only, visibility(compute))]
    pub positive_particles_count: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub positive_particles: Handle<ShaderStorageBuffer>,
    #[storage_texture(2, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[storage(3, visibility(compute))]
    pub phi_plus: Handle<ShaderStorageBuffer>,
}

impl AccumulateLevelSetCorrectionPlusSecondResource {
    pub fn new(pls_resources: &PLSResources, levelset_air: &Handle<Image>) -> Self {
        let positive_particles_count = pls_resources.positive_particles_count.clone();
        let positive_particles = pls_resources.positive_particles.clone();
        let phi_plus = pls_resources.phi_plus.clone();

        Self {
            positive_particles_count,
            positive_particles,
            levelset_air: levelset_air.clone(),
            phi_plus,
        }
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct AccumulateLevelSetCorrectionMinusSecondResource {
    #[storage(0, read_only, visibility(compute))]
    pub negative_particles_count: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub negative_particles: Handle<ShaderStorageBuffer>,
    #[storage_texture(2, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[storage(3, visibility(compute))]
    pub phi_minus: Handle<ShaderStorageBuffer>,
}

impl AccumulateLevelSetCorrectionMinusSecondResource {
    pub fn new(pls_resources: &PLSResources, levelset_air: &Handle<Image>) -> Self {
        let negative_particles_count = pls_resources.negative_particles_count.clone();
        let negative_particles = pls_resources.negative_particles.clone();
        let phi_minus = pls_resources.phi_minus.clone();

        Self {
            negative_particles_count,
            negative_particles,
            levelset_air: levelset_air.clone(),
            phi_minus,
        }
    }
}

#[derive(Component)]
pub(crate) struct AccumulateLevelSetCorrectionPlusSecondBindGroup {
    pub bind_group: BindGroup,
}

#[derive(Component)]
pub(crate) struct AccumulateLevelSetCorrectionMinusSecondBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for AccumulateLevelSetCorrectionPlusSecondBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}

impl From<BindGroup> for AccumulateLevelSetCorrectionMinusSecondBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
