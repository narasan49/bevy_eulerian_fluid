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

pub(crate) struct ResetLevelSetCorrectionPass;

impl FluidComputePass for ResetLevelSetCorrectionPass {
    type Pipeline = ResetLevelSetCorrectionPipeline;
    type Resource = ResetLevelSetCorrectionResource;
    type BG = ResetLevelSetCorrectionBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/reset_levelset_correction.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct ResetLevelSetCorrectionResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[storage(1, visibility(compute))]
    pub phi_plus: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub phi_minus: Handle<ShaderStorageBuffer>,
}

impl ResetLevelSetCorrectionResource {
    pub fn new(pls_resources: &PLSResources, levelset_air: &Handle<Image>) -> Self {
        let phi_plus = pls_resources.phi_plus.clone();
        let phi_minus = pls_resources.phi_minus.clone();

        Self {
            levelset_air: levelset_air.clone(),
            phi_plus,
            phi_minus,
        }
    }
}

#[derive(Resource)]
pub(crate) struct ResetLevelSetCorrectionPipeline {
    pub pipeline: SingleComputePipeline,
}

impl FromWorld for ResetLevelSetCorrectionPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<ResetLevelSetCorrectionResource>(
            world,
            "ResetLevelSetCorrectionPipeline",
            embedded_path!("shaders/reset_levelset_correction.wgsl"),
            "reset_levelset_correction",
        );

        Self { pipeline }
    }
}

#[derive(Component)]
pub(crate) struct ResetLevelSetCorrectionBindGroup {
    pub bind_group: BindGroup,
}

impl HasBindGroupLayout for ResetLevelSetCorrectionPipeline {
    fn bind_group_layout(&self) -> &bevy::render::render_resource::BindGroupLayoutDescriptor {
        &self.pipeline.bind_group_layout
    }
}

impl From<BindGroup> for ResetLevelSetCorrectionBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
