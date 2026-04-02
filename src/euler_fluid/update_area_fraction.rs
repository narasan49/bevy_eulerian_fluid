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

pub(crate) struct UpdateAreaFractionPass;

impl FluidComputePass for UpdateAreaFractionPass {
    type Pipeline = UpdateAreaFractionPipeline;
    type Resource = UpdateAreaFractionResource;
    type BG = UpdateAreaFractionBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/update_area_fraction.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct UpdateAreaFractionResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub levelset_solid: Handle<Image>,
    #[storage_texture(1, image_format = Rgba32Float, access = WriteOnly)]
    pub area_fraction_solid: Handle<Image>,
}

impl UpdateAreaFractionResource {
    pub fn new(levelset_solid: &Handle<Image>, area_fraction_solid: &Handle<Image>) -> Self {
        Self {
            levelset_solid: levelset_solid.clone(),
            area_fraction_solid: area_fraction_solid.clone(),
        }
    }
}

#[derive(Resource)]
pub(crate) struct UpdateAreaFractionPipeline {
    pub pipeline: SingleComputePipeline,
}

impl FromWorld for UpdateAreaFractionPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<UpdateAreaFractionResource>(
            world,
            "UpdateAreaFractionPipeline",
            embedded_path!("shaders/update_area_fraction.wgsl"),
            "update_area_fraction",
        );

        Self { pipeline }
    }
}

impl HasBindGroupLayout for UpdateAreaFractionPipeline {
    fn bind_group_layout(&self) -> &bevy::render::render_resource::BindGroupLayoutDescriptor {
        &self.pipeline.bind_group_layout
    }
}

#[derive(Component)]
pub(crate) struct UpdateAreaFractionBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for UpdateAreaFractionBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
