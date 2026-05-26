use bevy::{
    asset::{embedded_asset, embedded_path},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{AsBindGroup, BindGroup, BindGroupLayoutDescriptor},
    },
};

use crate::{
    pipeline::{HasBindGroupLayout, SingleComputePipeline},
    plugin::FluidComputePass,
};

pub(crate) struct InitializeGridCenterPass;

impl FluidComputePass for InitializeGridCenterPass {
    type Pipeline = InitializeGridCenterPipeline;
    type Resource = InitializeGridCenterResource;
    type BG = InitializeGridCenterBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/initialize_grid_center.wgsl");
    }
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct InitializeGridCenterResource {
    #[storage_texture(0, image_format = R32Float, access = WriteOnly)]
    pub levelset_air0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = WriteOnly)]
    pub levelset_air1: Handle<Image>,
    #[storage_texture(2, image_format = Rg32Float, access = WriteOnly)]
    pub grad_levelset_air: Handle<Image>,
}

#[derive(Resource)]
pub(crate) struct InitializeGridCenterPipeline {
    pub pipeline: SingleComputePipeline,
}

impl FromWorld for InitializeGridCenterPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<InitializeGridCenterResource>(
            world,
            "InitializeGridCenterPipeline",
            embedded_path!("shaders/initialize_grid_center.wgsl"),
            "initialize_grid_center",
        );

        Self { pipeline }
    }
}

impl HasBindGroupLayout for InitializeGridCenterPipeline {
    fn bind_group_layout(&self) -> &BindGroupLayoutDescriptor {
        &self.pipeline.bind_group_layout
    }
}
#[derive(Component)]
pub(crate) struct InitializeGridCenterBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for InitializeGridCenterBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
