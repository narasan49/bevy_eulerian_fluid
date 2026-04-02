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

pub(crate) struct DivergencePass;

impl FluidComputePass for DivergencePass {
    type Pipeline = DivergencePipeline;
    type Resource = DivergenceResource;
    type BG = DivergenceBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/divergence.wgsl");
    }
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct DivergenceResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub u1: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadOnly)]
    pub v1: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadOnly)]
    pub u_solid: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, access = ReadOnly)]
    pub v_solid: Handle<Image>,
    #[storage_texture(4, image_format = R32Float, access = ReadOnly)]
    pub levelset_solid: Handle<Image>,
    #[storage_texture(5, image_format = R32Float, access = WriteOnly)]
    pub div: Handle<Image>,
}

#[derive(Resource)]
pub(crate) struct DivergencePipeline {
    pub pipeline: SingleComputePipeline,
}

#[derive(Component)]
pub(crate) struct DivergenceBindGroup {
    pub bind_group: BindGroup,
}

impl FromWorld for DivergencePipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<DivergenceResource>(
            world,
            "DivergencePipeline",
            embedded_path!("shaders/divergence.wgsl"),
            "divergence",
        );

        DivergencePipeline { pipeline }
    }
}

impl HasBindGroupLayout for DivergencePipeline {
    fn bind_group_layout(&self) -> &BindGroupLayoutDescriptor {
        &self.pipeline.bind_group_layout
    }
}

impl From<BindGroup> for DivergenceBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
