use crate::{
    particle_levelset_two_layers::{
        levelset_correction::correct_levelset::CorrectLevelSetPipeline, plugin::PLSResources,
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

pub(crate) struct CorrectLevelSetSecondPass;

impl FluidComputePass for CorrectLevelSetSecondPass {
    type Pipeline = CorrectLevelSetPipeline;
    type Resource = CorrectLevelSetSecondResource;
    type BG = CorrectLevelSetSecondBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/correct_levelset.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct CorrectLevelSetSecondResource {
    #[storage_texture(0, image_format = R32Float, access = WriteOnly)]
    pub levelset_air: Handle<Image>,
    #[storage(1, read_only, visibility(compute))]
    pub phi_plus: Handle<ShaderStorageBuffer>,
    #[storage(2, read_only, visibility(compute))]
    pub phi_minus: Handle<ShaderStorageBuffer>,
}

impl CorrectLevelSetSecondResource {
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
pub(crate) struct CorrectLevelSetSecondBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for CorrectLevelSetSecondBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
