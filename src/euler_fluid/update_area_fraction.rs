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

pub(crate) struct UpdateAreaFractionPass;

impl FluidComputePass for UpdateAreaFractionPass {
    type P = UpdateAreaFractionPipeline;

    type Resource = UpdateAreaFractionResource;

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups.into_configs()
    }

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/update_area_fraction.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct UpdateAreaFractionResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub levelset_solid: Handle<Image>,
    #[storage_texture(1, image_format = Rgba32Float, access = WriteOnly)]
    pub area_fraction_solid: Handle<Image>,
}

impl UpdateAreaFractionResource {
    pub fn new(levelset_solid: &Handle<Image>, area_fraction_solid: &Handle<Image>) -> Self {
        Self {
            levelset_solid: levelset_solid.clone(),
            area_fraction_solid: area_fraction_solid.clone(),
        }
    }
}

#[derive(Resource)]
pub(crate) struct UpdateAreaFractionPipeline {
    pub pipeline: SingleComputePipeline,
}

impl FromWorld for UpdateAreaFractionPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<UpdateAreaFractionResource>(
            world,
            "UpdateAreaFractionPipeline",
            embedded_path!("shaders/update_area_fraction.wgsl"),
            "update_area_fraction",
        );

        Self { pipeline }
    }
}

#[derive(Component)]
pub(crate) struct UpdateAreaFractionBindGroup {
    pub bind_group: BindGroup,
}

pub(super) fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipeline: Res<UpdateAreaFractionPipeline>,
    query: Query<(Entity, &UpdateAreaFractionResource)>,
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
            .insert(UpdateAreaFractionBindGroup { bind_group });
    }
}
