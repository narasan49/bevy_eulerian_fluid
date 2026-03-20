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

pub(crate) struct CorrectLevelSetPass;

impl FluidComputePass for CorrectLevelSetPass {
    type P = CorrectLevelSetPipeline;

    type Resource = CorrectLevelSetResource;

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups.into_configs()
    }

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/correct_levelset.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct CorrectLevelSetResource {
    #[storage_texture(0, image_format = R32Float, access = WriteOnly)]
    pub levelset_air: Handle<Image>,
    #[storage(1, read_only, visibility(compute))]
    pub phi_plus: Handle<ShaderStorageBuffer>,
    #[storage(2, read_only, visibility(compute))]
    pub phi_minus: Handle<ShaderStorageBuffer>,
}

impl CorrectLevelSetResource {
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
pub(crate) struct CorrectLevelSetPipeline {
    pub pipeline: SingleComputePipeline,
}

impl FromWorld for CorrectLevelSetPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<CorrectLevelSetResource>(
            world,
            "CorrectLevelSetPipeline",
            embedded_path!("shaders/correct_levelset.wgsl"),
            "correct_levelset",
        );

        Self { pipeline }
    }
}

#[derive(Component)]
pub(crate) struct CorrectLevelSetBindGroup {
    pub bind_group: BindGroup,
}

pub(super) fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipeline: Res<CorrectLevelSetPipeline>,
    query: Query<(Entity, &CorrectLevelSetResource)>,
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
            .insert(CorrectLevelSetBindGroup { bind_group });
    }
}
