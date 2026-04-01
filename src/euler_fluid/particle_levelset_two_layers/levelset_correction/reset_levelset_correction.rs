use crate::{
    particle_levelset_two_layers::plugin::PLSResources, pipeline::SingleComputePipeline,
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

pub(crate) struct ResetLevelSetCorrectionPass;

impl FluidComputePass for ResetLevelSetCorrectionPass {
    type P = ResetLevelSetCorrectionPipeline;

    type Resource = ResetLevelSetCorrectionResource;

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups.into_configs()
    }

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/reset_levelset_correction.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct ResetLevelSetCorrectionResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[storage(1, visibility(compute))]
    pub phi_plus: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub phi_minus: Handle<ShaderStorageBuffer>,
}

impl ResetLevelSetCorrectionResource {
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
pub(crate) struct ResetLevelSetCorrectionPipeline {
    pub pipeline: SingleComputePipeline,
}

impl FromWorld for ResetLevelSetCorrectionPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<ResetLevelSetCorrectionResource>(
            world,
            "ResetLevelSetCorrectionPipeline",
            embedded_path!("shaders/reset_levelset_correction.wgsl"),
            "reset_levelset_correction",
        );

        Self { pipeline }
    }
}

#[derive(Component)]
pub(crate) struct ResetLevelSetCorrectionBindGroup {
    pub bind_group: BindGroup,
}

pub(super) fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipeline: Res<ResetLevelSetCorrectionPipeline>,
    query: Query<(Entity, &ResetLevelSetCorrectionResource)>,
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
            .insert(ResetLevelSetCorrectionBindGroup { bind_group });
    }
}
