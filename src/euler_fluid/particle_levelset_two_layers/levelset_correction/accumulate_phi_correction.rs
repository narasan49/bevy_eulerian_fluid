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

pub(crate) struct AccumulateLevelSetCorrectionPlusPass;

impl FluidComputePass for AccumulateLevelSetCorrectionPlusPass {
    type Pipeline = AccumulateLevelSetCorrectionPipeline;
    type Resource = AccumulateLevelSetCorrectionPlusResource;
    type BG = AccumulateLevelSetCorrectionPlusBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/accumulate_levelset_correction.wgsl");
    }
}

pub(crate) struct AccumulateLevelSetCorrectionMinusPass;

impl FluidComputePass for AccumulateLevelSetCorrectionMinusPass {
    type Pipeline = AccumulateLevelSetCorrectionPipeline;
    type Resource = AccumulateLevelSetCorrectionMinusResource;
    type BG = AccumulateLevelSetCorrectionMinusBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/accumulate_levelset_correction.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct AccumulateLevelSetCorrectionPlusResource {
    #[storage(0, read_only, visibility(compute))]
    pub positive_particles_count: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub positive_particles: Handle<ShaderStorageBuffer>,
    #[storage_texture(2, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[storage(3, visibility(compute))]
    pub phi_plus: Handle<ShaderStorageBuffer>,
}

impl AccumulateLevelSetCorrectionPlusResource {
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
pub(crate) struct AccumulateLevelSetCorrectionMinusResource {
    #[storage(0, read_only, visibility(compute))]
    pub negative_particles_count: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub negative_particles: Handle<ShaderStorageBuffer>,
    #[storage_texture(2, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[storage(3, visibility(compute))]
    pub phi_minus: Handle<ShaderStorageBuffer>,
}

impl AccumulateLevelSetCorrectionMinusResource {
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

#[derive(Resource)]
pub(crate) struct AccumulateLevelSetCorrectionPipeline {
    pub pipeline: SingleComputePipeline,
}

impl FromWorld for AccumulateLevelSetCorrectionPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<AccumulateLevelSetCorrectionPlusResource>(
            world,
            "AccumulateLevelSetCorrectionPipeline",
            embedded_path!("shaders/accumulate_levelset_correction.wgsl"),
            "accumulate_levelset_correction",
        );

        Self { pipeline }
    }
}

#[derive(Component)]
pub(crate) struct AccumulateLevelSetCorrectionPlusBindGroup {
    pub bind_group: BindGroup,
}

#[derive(Component)]
pub(crate) struct AccumulateLevelSetCorrectionMinusBindGroup {
    pub bind_group: BindGroup,
}

impl HasBindGroupLayout for AccumulateLevelSetCorrectionPipeline {
    fn bind_group_layout(&self) -> &bevy::render::render_resource::BindGroupLayoutDescriptor {
        &self.pipeline.bind_group_layout
    }
}

impl From<BindGroup> for AccumulateLevelSetCorrectionPlusBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}

impl From<BindGroup> for AccumulateLevelSetCorrectionMinusBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
