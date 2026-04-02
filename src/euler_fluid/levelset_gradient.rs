use crate::{
    pipeline::{HasBindGroupLayout, SingleComputePipeline},
    plugin::FluidComputePass,
};
use bevy::{
    asset::{embedded_asset, embedded_path},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{AsBindGroup, BindGroup},
    },
};

pub(crate) struct LevelSetGradientPass;

impl FluidComputePass for LevelSetGradientPass {
    type Pipeline = LevelSetGradientPipeline;
    type Resource = LevelSetGradientResource;
    type BG = LevelSetGradientBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/levelset_gradient.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct LevelSetGradientResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[storage_texture(1, image_format = Rg32Float, access = WriteOnly)]
    pub grad_levelset: Handle<Image>,
}

impl LevelSetGradientResource {
    pub fn new(levelset_air: &Handle<Image>, grad_levelset: &Handle<Image>) -> Self {
        Self {
            levelset_air: levelset_air.clone(),
            grad_levelset: grad_levelset.clone(),
        }
    }
}

#[derive(Resource)]
pub(crate) struct LevelSetGradientPipeline {
    pub pipeline: SingleComputePipeline,
}

impl FromWorld for LevelSetGradientPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<LevelSetGradientResource>(
            world,
            "LevelSetGradientPipeline",
            embedded_path!("shaders/levelset_gradient.wgsl"),
            "levelset_gradient",
        );

        Self { pipeline }
    }
}

impl HasBindGroupLayout for LevelSetGradientPipeline {
    fn bind_group_layout(&self) -> &bevy::render::render_resource::BindGroupLayoutDescriptor {
        &self.pipeline.bind_group_layout
    }
}

#[derive(Component)]
pub(crate) struct LevelSetGradientBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for LevelSetGradientBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
