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

#[derive(Clone, Debug)]
pub struct FastIterativeMethodConfig {
    pub num_iterations: u32,
}

impl Default for FastIterativeMethodConfig {
    fn default() -> Self {
        Self { num_iterations: 10 }
    }
}

pub(crate) struct FastIterativeInitializePass;

impl FluidComputePass for FastIterativeInitializePass {
    type P = FastIterativeInitializePipeline;

    type Resource = FastIterativeInitializeResource;

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups_initialize.into_configs()
    }

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/fast_iterative_method/initialize.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct FastIterativeInitializeResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = WriteOnly)]
    pub phi: Handle<Image>,
    #[storage_texture(2, image_format = R8Uint, access = WriteOnly)]
    pub labels: Handle<Image>,
}

impl FastIterativeInitializeResource {
    pub fn new(levelset_air: &Handle<Image>, phi: &Handle<Image>, labels: &Handle<Image>) -> Self {
        Self {
            levelset_air: levelset_air.clone(),
            phi: phi.clone(),
            labels: labels.clone(),
        }
    }
}

#[derive(Resource)]
pub(crate) struct FastIterativeInitializePipeline {
    pub pipeline: SingleComputePipeline,
}

impl FromWorld for FastIterativeInitializePipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new_with_uniform::<FastIterativeInitializeResource>(
            world,
            "FastIterativeInitializePipeline",
            embedded_path!("shaders/fast_iterative_method/initialize.wgsl"),
            "initialize",
        );

        Self { pipeline }
    }
}

#[derive(Component)]
pub(crate) struct FastIterativeInitializeBindGroup {
    pub bind_group: BindGroup,
}

pub(super) fn prepare_bind_groups_initialize<'a>(
    mut commands: Commands,
    pipeline: Res<FastIterativeInitializePipeline>,
    query: Query<(Entity, &FastIterativeInitializeResource)>,
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
            .insert(FastIterativeInitializeBindGroup { bind_group });
    }
}

pub(crate) struct FastIterativeInitializeActiveLabelPass;

impl FluidComputePass for FastIterativeInitializeActiveLabelPass {
    type P = FastIterativeInitializeActiveLabelPipeline;

    type Resource = FastIterativeInitializeActiveLabelResource;

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups_initialize_active_label.into_configs()
    }

    fn register_assets(app: &mut App) {
        embedded_asset!(
            app,
            "shaders/fast_iterative_method/initialize_active_label.wgsl"
        );
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct FastIterativeInitializeActiveLabelResource {
    #[storage_texture(0, image_format = R8Uint, access = ReadWrite)]
    pub labels: Handle<Image>,
}

impl FastIterativeInitializeActiveLabelResource {
    pub fn new(labels: &Handle<Image>) -> Self {
        Self {
            labels: labels.clone(),
        }
    }
}

#[derive(Resource)]
pub(crate) struct FastIterativeInitializeActiveLabelPipeline {
    pub pipeline: SingleComputePipeline,
}

impl FromWorld for FastIterativeInitializeActiveLabelPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline =
            SingleComputePipeline::new_with_uniform::<FastIterativeInitializeActiveLabelResource>(
                world,
                "FastIterativeInitializeActiveLabelPipeline",
                embedded_path!("shaders/fast_iterative_method/initialize_active_label.wgsl"),
                "initialize_active_label",
            );

        Self { pipeline }
    }
}

#[derive(Component)]
pub(crate) struct FastIterativeInitializeActiveLabelBindGroup {
    pub bind_group: BindGroup,
}

pub(super) fn prepare_bind_groups_initialize_active_label<'a>(
    mut commands: Commands,
    pipeline: Res<FastIterativeInitializeActiveLabelPipeline>,
    query: Query<(Entity, &FastIterativeInitializeActiveLabelResource)>,
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
            .insert(FastIterativeInitializeActiveLabelBindGroup { bind_group });
    }
}

pub(crate) struct FastIterativeUpdatePass;

impl FluidComputePass for FastIterativeUpdatePass {
    type P = FastIterativeUpdatePipeline;

    type Resource = FastIterativeUpdateResource;

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups_update.into_configs()
    }

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/fast_iterative_method/update.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct FastIterativeUpdateResource {
    #[storage_texture(0, image_format = R8Uint, access = ReadWrite)]
    pub labels: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub phi: Handle<Image>,
}

impl FastIterativeUpdateResource {
    pub fn new(phi: &Handle<Image>, labels: &Handle<Image>) -> Self {
        Self {
            phi: phi.clone(),
            labels: labels.clone(),
        }
    }
}

#[derive(Resource)]
pub(crate) struct FastIterativeUpdatePipeline {
    pub pipeline: SingleComputePipeline,
}

impl FromWorld for FastIterativeUpdatePipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new_with_uniform::<FastIterativeUpdateResource>(
            world,
            "FastIterativeUpdatePipeline",
            embedded_path!("shaders/fast_iterative_method/update.wgsl"),
            "update",
        );

        Self { pipeline }
    }
}

#[derive(Component)]
pub(crate) struct FastIterativeUpdateBindGroup {
    pub bind_group: BindGroup,
}

pub(super) fn prepare_bind_groups_update<'a>(
    mut commands: Commands,
    pipeline: Res<FastIterativeUpdatePipeline>,
    query: Query<(Entity, &FastIterativeUpdateResource)>,
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
            .insert(FastIterativeUpdateBindGroup { bind_group });
    }
}
