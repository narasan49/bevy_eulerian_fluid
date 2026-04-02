use crate::{
    particle_levelset_two_layers::{
        levelset_correction::reset_levelset_correction::ResetLevelSetCorrectionPipeline,
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

pub(crate) struct ResetLevelSetCorrectionSecondPass;

impl FluidComputePass for ResetLevelSetCorrectionSecondPass {
    type Pipeline = ResetLevelSetCorrectionPipeline;
    type Resource = ResetLevelSetCorrectionSecondResource;
    type BG = ResetLevelSetCorrectionSecondBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/reset_levelset_correction.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct ResetLevelSetCorrectionSecondResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[storage(1, visibility(compute))]
    pub phi_plus: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub phi_minus: Handle<ShaderStorageBuffer>,
}

impl ResetLevelSetCorrectionSecondResource {
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

#[derive(Component)]
pub(crate) struct ResetLevelSetCorrectionSecondBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for ResetLevelSetCorrectionSecondBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
