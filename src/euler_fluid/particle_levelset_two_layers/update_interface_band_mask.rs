use bevy::{
    asset::{embedded_asset, embedded_path},
    ecs::{schedule::ScheduleConfigs, system::ScheduleSystem},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_asset::RenderAssets,
        render_resource::{AsBindGroup, BindGroup},
        renderer::RenderDevice,
        storage::GpuShaderStorageBuffer,
        texture::{FallbackImage, GpuImage},
    },
};

use crate::{
    particle_levelset_two_layers::plugin::PLSResources, pipeline::SingleComputePipeline,
    plugin::FluidComputePass,
};

pub(super) struct UpdateInterfaceBandMaskPass;

impl FluidComputePass for UpdateInterfaceBandMaskPass {
    type P = UpdateInterfaceBandMaskPipeline;
    type Resource = UpdateInterfaceBandMaskResource;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/update_interface_band_mask.wgsl");
    }

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups.into_configs()
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct UpdateInterfaceBandMaskResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[storage_texture(1, image_format = R8Uint, access = WriteOnly)]
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

#[derive(Component)]
pub(crate) struct UpdateInterfaceBandMaskBindGroup {
    pub bind_group: BindGroup,
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

fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipelines: Res<UpdateInterfaceBandMaskPipeline>,
    query: Query<(Entity, &UpdateInterfaceBandMaskResource)>,
    render_device: Res<RenderDevice>,
    mut param: (
        Res<'a, RenderAssets<GpuImage>>,
        Res<'a, FallbackImage>,
        Res<'a, RenderAssets<GpuShaderStorageBuffer>>,
    ),
) {
    for (entity, resource) in &query {
        let bind_group = resource
            .as_bind_group(
                &pipelines.pipeline.bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        commands
            .entity(entity)
            .insert(UpdateInterfaceBandMaskBindGroup { bind_group });
    }
}
