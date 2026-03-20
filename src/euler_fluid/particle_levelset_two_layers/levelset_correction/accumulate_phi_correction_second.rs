use crate::{
    particle_levelset_two_layers::{
        levelset_correction::accumulate_phi_correction::AccumulateLevelSetCorrectionPipeline,
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

pub(crate) struct AccumulateLevelSetCorrectionPlusSecondPass;

impl FluidComputePass for AccumulateLevelSetCorrectionPlusSecondPass {
    type P = AccumulateLevelSetCorrectionPipeline;

    type Resource = AccumulateLevelSetCorrectionPlusSecondResource;

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups_plus.into_configs()
    }

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/accumulate_levelset_correction.wgsl");
    }
}

pub(crate) struct AccumulateLevelSetCorrectionMinusSecondPass;

impl FluidComputePass for AccumulateLevelSetCorrectionMinusSecondPass {
    type P = AccumulateLevelSetCorrectionPipeline;

    type Resource = AccumulateLevelSetCorrectionMinusSecondResource;

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups_minus.into_configs()
    }

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/accumulate_levelset_correction.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct AccumulateLevelSetCorrectionPlusSecondResource {
    #[storage(0, read_only, visibility(compute))]
    pub positive_particles_count: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub positive_particles: Handle<ShaderStorageBuffer>,
    #[storage_texture(2, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[storage(3, visibility(compute))]
    pub phi_plus: Handle<ShaderStorageBuffer>,
}

impl AccumulateLevelSetCorrectionPlusSecondResource {
    pub fn new(pls_resources: &PLSResources, levelset_air: &Handle<Image>) -> Self {
        let positive_particles_count = pls_resources.positive_particles_count.clone();
        let positive_particles = pls_resources.positive_particles.clone();
        let phi_plus = pls_resources.phi_plus.clone();

        Self {
            positive_particles_count,
            positive_particles,
            levelset_air: levelset_air.clone(),
            phi_plus,
        }
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct AccumulateLevelSetCorrectionMinusSecondResource {
    #[storage(0, read_only, visibility(compute))]
    pub negative_particles_count: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub negative_particles: Handle<ShaderStorageBuffer>,
    #[storage_texture(2, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[storage(3, visibility(compute))]
    pub phi_minus: Handle<ShaderStorageBuffer>,
}

impl AccumulateLevelSetCorrectionMinusSecondResource {
    pub fn new(pls_resources: &PLSResources, levelset_air: &Handle<Image>) -> Self {
        let negative_particles_count = pls_resources.negative_particles_count.clone();
        let negative_particles = pls_resources.negative_particles.clone();
        let phi_minus = pls_resources.phi_minus.clone();

        Self {
            negative_particles_count,
            negative_particles,
            levelset_air: levelset_air.clone(),
            phi_minus,
        }
    }
}

#[derive(Component)]
pub(crate) struct AccumulateLevelSetCorrectionPlusSecondBindGroup {
    pub bind_group: BindGroup,
}

impl FluidSingleBindGroup for AccumulateLevelSetCorrectionPlusSecondBindGroup {
    fn new(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}

#[derive(Component)]
pub(crate) struct AccumulateLevelSetCorrectionMinusSecondBindGroup {
    pub bind_group: BindGroup,
}

trait FluidSingleBindGroup: Component + Sized {
    fn new(bind_group: BindGroup) -> Self;
}

fn prepare_bind_groups<
    'a,
    A: AsBindGroup<
            Param = (
                Res<'a, RenderAssets<GpuImage>>,
                Res<'a, FallbackImage>,
                Res<'a, RenderAssets<GpuShaderStorageBuffer>>,
            ),
        > + Component,
    B: FluidSingleBindGroup,
>(
    mut commands: Commands,
    pipeline: Res<AccumulateLevelSetCorrectionPipeline>,
    query: Query<(Entity, &A)>,
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

        commands.entity(entity).insert(B::new(bind_group));
    }
}

pub(super) fn prepare_bind_groups_plus<'a>(
    mut commands: Commands,
    pipeline: Res<AccumulateLevelSetCorrectionPipeline>,
    query: Query<(Entity, &AccumulateLevelSetCorrectionPlusSecondResource)>,
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
            .insert(AccumulateLevelSetCorrectionPlusSecondBindGroup { bind_group });
    }
}

pub(super) fn prepare_bind_groups_minus<'a>(
    mut commands: Commands,
    pipeline: Res<AccumulateLevelSetCorrectionPipeline>,
    query: Query<(Entity, &AccumulateLevelSetCorrectionMinusSecondResource)>,
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
            .insert(AccumulateLevelSetCorrectionMinusSecondBindGroup { bind_group });
    }
}
