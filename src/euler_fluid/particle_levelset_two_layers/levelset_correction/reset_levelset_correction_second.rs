use crate::{
    particle_levelset_two_layers::{
        levelset_correction::reset_levelset_correction::ResetLevelSetCorrectionPipeline,
        plugin::PLSResources,
    },
    pipeline::SingleComputePipeline,
    plugin::FluidComputePass,
};
use bevy::{
    asset::{embedded_asset, embedded_path},
    ecs::{schedule::ScheduleConfigs, system::ScheduleSystem},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_asset::RenderAssets,
        render_resource::{AsBindGroup, BindGroup},
        renderer::RenderDevice,
        storage::{GpuShaderStorageBuffer, ShaderStorageBuffer},
        texture::{FallbackImage, GpuImage},
    },
};

pub(crate) struct ResetLevelSetCorrectionSecondPass;

impl FluidComputePass for ResetLevelSetCorrectionSecondPass {
    type P = ResetLevelSetCorrectionPipeline;

    type Resource = ResetLevelSetCorrectionSecondResource;

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups.into_configs()
    }

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

pub(super) fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipeline: Res<ResetLevelSetCorrectionPipeline>,
    query: Query<(Entity, &ResetLevelSetCorrectionSecondResource)>,
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
                &pipeline.pipeline.bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        commands
            .entity(entity)
            .insert(ResetLevelSetCorrectionSecondBindGroup { bind_group });
    }
}
