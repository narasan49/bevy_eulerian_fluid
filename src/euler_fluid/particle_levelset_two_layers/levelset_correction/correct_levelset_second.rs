use crate::{
    particle_levelset_two_layers::{
        levelset_correction::correct_levelset::CorrectLevelSetPipeline, plugin::PLSResources,
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

pub(crate) struct CorrectLevelSetSecondPass;

impl FluidComputePass for CorrectLevelSetSecondPass {
    type P = CorrectLevelSetPipeline;

    type Resource = CorrectLevelSetSecondResource;

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups.into_configs()
    }

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

pub(super) fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipeline: Res<CorrectLevelSetPipeline>,
    query: Query<(Entity, &CorrectLevelSetSecondResource)>,
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
            .insert(CorrectLevelSetSecondBindGroup { bind_group });
    }
}
