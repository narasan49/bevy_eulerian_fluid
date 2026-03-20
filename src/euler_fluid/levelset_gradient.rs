use crate::{pipeline::SingleComputePipeline, plugin::FluidComputePass};
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

pub(crate) struct LevelSetGradientPass;

impl FluidComputePass for LevelSetGradientPass {
    type P = LevelSetGradientPipeline;

    type Resource = LevelSetGradientResource;

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups.into_configs()
    }

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/levelset_gradient.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct LevelSetGradientResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[storage_texture(1, image_format = Rg32Float, access = WriteOnly)]
    pub grad_levelset: Handle<Image>,
}

impl LevelSetGradientResource {
    pub fn new(levelset_air: &Handle<Image>, grad_levelset: &Handle<Image>) -> Self {
        Self {
            levelset_air: levelset_air.clone(),
            grad_levelset: grad_levelset.clone(),
        }
    }
}

#[derive(Resource)]
pub(crate) struct LevelSetGradientPipeline {
    pub pipeline: SingleComputePipeline,
}

impl FromWorld for LevelSetGradientPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<LevelSetGradientResource>(
            world,
            "LevelSetGradientPipeline",
            embedded_path!("shaders/levelset_gradient.wgsl"),
            "levelset_gradient",
        );

        Self { pipeline }
    }
}

#[derive(Component)]
pub(crate) struct LevelSetGradientBindGroup {
    pub bind_group: BindGroup,
}

pub(super) fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipeline: Res<LevelSetGradientPipeline>,
    query: Query<(Entity, &LevelSetGradientResource)>,
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
            .insert(LevelSetGradientBindGroup { bind_group });
    }
}
