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

pub(crate) struct InitializeGridEdgePass;

impl FluidComputePass for InitializeGridEdgePass {
    type Pipeline = InitializeGridEdgePipeline;
    type Resource = InitializeGridEdgeResource;
    type BG = InitializeGridEdgeBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/initialize_grid_edge.wgsl");
    }
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct InitializeGridEdgeResource {
    #[storage_texture(0, image_format = R32Float, access = WriteOnly)]
    pub u0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = WriteOnly)]
    pub v0: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = WriteOnly)]
    pub u1: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, access = WriteOnly)]
    pub v1: Handle<Image>,
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
        let pipeline = SingleComputePipeline::new_with_uniform::<InitializeGridCenterResource>(
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

#[derive(Resource)]
pub(crate) struct InitializeGridEdgePipeline {
    pub pipeline: SingleComputePipeline,
}

impl FromWorld for InitializeGridEdgePipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<InitializeGridEdgeResource>(
            world,
            "InitializeGridEdgePipeline",
            embedded_path!("shaders/initialize_grid_edge.wgsl"),
            "initialize_grid_edge",
        );

        Self { pipeline }
    }
}

impl HasBindGroupLayout for InitializeGridEdgePipeline {
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

#[derive(Component)]
pub(crate) struct InitializeGridEdgeBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for InitializeGridEdgeBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
