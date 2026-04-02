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

pub(crate) struct CorrectLevelSetPass;

impl FluidComputePass for CorrectLevelSetPass {
    type Pipeline = CorrectLevelSetPipeline;
    type Resource = CorrectLevelSetResource;
    type BG = CorrectLevelSetBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/correct_levelset.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct CorrectLevelSetResource {
    #[storage_texture(0, image_format = R32Float, access = WriteOnly)]
    pub levelset_air: Handle<Image>,
    #[storage(1, read_only, visibility(compute))]
    pub phi_plus: Handle<ShaderStorageBuffer>,
    #[storage(2, read_only, visibility(compute))]
    pub phi_minus: Handle<ShaderStorageBuffer>,
}

impl CorrectLevelSetResource {
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
pub(crate) struct CorrectLevelSetPipeline {
    pub pipeline: SingleComputePipeline,
}

impl FromWorld for CorrectLevelSetPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<CorrectLevelSetResource>(
            world,
            "CorrectLevelSetPipeline",
            embedded_path!("shaders/correct_levelset.wgsl"),
            "correct_levelset",
        );

        Self { pipeline }
    }
}

#[derive(Component)]
pub(crate) struct CorrectLevelSetBindGroup {
    pub bind_group: BindGroup,
}

impl HasBindGroupLayout for CorrectLevelSetPipeline {
    fn bind_group_layout(&self) -> &bevy::render::render_resource::BindGroupLayoutDescriptor {
        &self.pipeline.bind_group_layout
    }
}

impl From<BindGroup> for CorrectLevelSetBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
