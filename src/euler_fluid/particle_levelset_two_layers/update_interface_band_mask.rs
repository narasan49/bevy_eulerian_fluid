use bevy::{
    asset::{embedded_asset, embedded_path},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{AsBindGroup, BindGroup},
    },
};

use crate::{
    particle_levelset_two_layers::plugin::PLSResources,
    pipeline::{HasBindGroupLayout, SingleComputePipeline},
    plugin::FluidComputePass,
};

pub(super) struct UpdateInterfaceBandMaskPass;

impl FluidComputePass for UpdateInterfaceBandMaskPass {
    type Pipeline = UpdateInterfaceBandMaskPipeline;
    type Resource = UpdateInterfaceBandMaskResource;
    type BG = UpdateInterfaceBandMaskBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/update_interface_band_mask.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct UpdateInterfaceBandMaskResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[storage_texture(1, image_format = R32Uint, access = WriteOnly)]
    pub interface_band_mask: Handle<Image>,
}

impl UpdateInterfaceBandMaskResource {
    pub fn new(pls_resources: &PLSResources, levelset_air: &Handle<Image>) -> Self {
        let interface_band_mask = pls_resources.interface_band_mask.clone();

        Self {
            levelset_air: levelset_air.clone(),
            interface_band_mask,
        }
    }
}

#[derive(Resource)]
pub(crate) struct UpdateInterfaceBandMaskPipeline {
    pub pipeline: SingleComputePipeline,
}

impl HasBindGroupLayout for UpdateInterfaceBandMaskPipeline {
    fn bind_group_layout(&self) -> &bevy::render::render_resource::BindGroupLayoutDescriptor {
        &self.pipeline.bind_group_layout
    }
}

impl FromWorld for UpdateInterfaceBandMaskPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<UpdateInterfaceBandMaskResource>(
            world,
            "UpdateInterfaceBandMaskPipeline",
            embedded_path!("shaders/update_interface_band_mask.wgsl"),
            "update_interface_band_mask",
        );

        Self { pipeline }
    }
}

#[derive(Component)]
pub(crate) struct UpdateInterfaceBandMaskBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for UpdateInterfaceBandMaskBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
