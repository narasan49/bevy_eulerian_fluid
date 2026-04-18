use crate::{
    pipeline::{HasBindGroupLayout, SingleComputePipeline},
    plugin::FluidComputePass,
};
use bevy::{
    asset::{embedded_asset, embedded_path},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{AsBindGroup, BindGroup},
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
    type Pipeline = FastIterativeInitializePipeline;
    type Resource = FastIterativeInitializeResource;
    type BG = FastIterativeInitializeBindGroup;

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
    #[storage_texture(2, image_format = R32Uint, access = WriteOnly)]
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
        let pipeline = SingleComputePipeline::new::<FastIterativeInitializeResource>(
            world,
            "FastIterativeInitializePipeline",
            embedded_path!("shaders/fast_iterative_method/initialize.wgsl"),
            "initialize",
        );

        Self { pipeline }
    }
}

impl HasBindGroupLayout for FastIterativeInitializePipeline {
    fn bind_group_layout(&self) -> &bevy::render::render_resource::BindGroupLayoutDescriptor {
        &self.pipeline.bind_group_layout
    }
}

#[derive(Component)]
pub(crate) struct FastIterativeInitializeBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for FastIterativeInitializeBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}

pub(crate) struct FastIterativeInitializeActiveLabelPass;

impl FluidComputePass for FastIterativeInitializeActiveLabelPass {
    type Pipeline = FastIterativeInitializeActiveLabelPipeline;
    type Resource = FastIterativeInitializeActiveLabelResource;
    type BG = FastIterativeInitializeActiveLabelBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(
            app,
            "shaders/fast_iterative_method/initialize_active_label.wgsl"
        );
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct FastIterativeInitializeActiveLabelResource {
    // workaround: Avoid DeviceRemoved error on WebGPU+aarch64 when using ReadWrite access.
    #[storage_texture(0, image_format = R32Uint, access = ReadOnly)]
    pub labels_in: Handle<Image>,
    #[storage_texture(1, image_format = R32Uint, access = WriteOnly)]
    pub labels_out: Handle<Image>,
}

impl FastIterativeInitializeActiveLabelResource {
    pub fn new(labels_in: &Handle<Image>, labels_out: &Handle<Image>) -> Self {
        Self {
            labels_in: labels_in.clone(),
            labels_out: labels_out.clone(),
        }
    }
}

#[derive(Resource)]
pub(crate) struct FastIterativeInitializeActiveLabelPipeline {
    pub pipeline: SingleComputePipeline,
}

impl FromWorld for FastIterativeInitializeActiveLabelPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new::<FastIterativeInitializeActiveLabelResource>(
            world,
            "FastIterativeInitializeActiveLabelPipeline",
            embedded_path!("shaders/fast_iterative_method/initialize_active_label.wgsl"),
            "initialize_active_label",
        );

        Self { pipeline }
    }
}

impl HasBindGroupLayout for FastIterativeInitializeActiveLabelPipeline {
    fn bind_group_layout(&self) -> &bevy::render::render_resource::BindGroupLayoutDescriptor {
        &self.pipeline.bind_group_layout
    }
}

#[derive(Component)]
pub(crate) struct FastIterativeInitializeActiveLabelBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for FastIterativeInitializeActiveLabelBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}

pub(crate) struct FastIterativeUpdatePass;

impl FluidComputePass for FastIterativeUpdatePass {
    type Pipeline = FastIterativeUpdatePipeline;
    type Resource = FastIterativeUpdateResource;
    type BG = FastIterativeUpdateBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/fast_iterative_method/update.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct FastIterativeUpdateResource {
    #[storage_texture(0, image_format = R32Uint, access = ReadWrite)]
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
        let pipeline = SingleComputePipeline::new::<FastIterativeUpdateResource>(
            world,
            "FastIterativeUpdatePipeline",
            embedded_path!("shaders/fast_iterative_method/update.wgsl"),
            "update",
        );

        Self { pipeline }
    }
}

impl HasBindGroupLayout for FastIterativeUpdatePipeline {
    fn bind_group_layout(&self) -> &bevy::render::render_resource::BindGroupLayoutDescriptor {
        &self.pipeline.bind_group_layout
    }
}

#[derive(Component)]
pub(crate) struct FastIterativeUpdateBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for FastIterativeUpdateBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
