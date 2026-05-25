use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{
            AsBindGroup, BindGroup, BindGroupLayoutDescriptor, CachedComputePipelineId,
            ComputePass, ComputePipelineDescriptor, PipelineCache,
        },
        renderer::RenderDevice,
    },
};
use bevy_eulerian_fluid_common::{
    fluid_compute_pass::FluidComputePass,
    fluid_compute_pipeline::{is_pipeline_loaded, HasBindGroupLayout},
};

use crate::{
    core::workgroup::{workgroup_size_center, WorkgroupShape},
    resource::FluidResources,
};

pub(crate) struct LevelSetGradientPass;

impl FluidComputePass for LevelSetGradientPass {
    type Pipeline = LevelSetGradientPipeline;
    type Resource = LevelSetGradientResource;
    type BG = LevelSetGradientBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "levelset_gradient.wgsl");
    }
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct LevelSetGradientResource {
    #[storage_texture(0, image_format = R32Float, dimension = "3d", access = ReadOnly)]
    pub levelset_air0: Handle<Image>,
    #[storage_texture(1, image_format = Rgba32Float, dimension = "3d", access = WriteOnly)]
    pub grad_levelset_air: Handle<Image>,
}

impl LevelSetGradientResource {
    pub fn new(resources: &FluidResources) -> Self {
        Self {
            levelset_air0: resources.levelset_air0.clone(),
            grad_levelset_air: resources.grad_levelset_air.clone(),
        }
    }
}

#[derive(Resource)]
pub(crate) struct LevelSetGradientPipeline {
    pub pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayoutDescriptor,
}

impl LevelSetGradientPipeline {
    pub fn is_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        is_pipeline_loaded(pipeline_cache, self.pipeline)
    }

    pub fn dispatch(
        &self,
        pipeline_cache: &PipelineCache,
        pass: &mut ComputePass,
        bind_group: &LevelSetGradientBindGroup,
        workgroup_shape: &WorkgroupShape,
        size: UVec3,
    ) {
        pass.push_debug_group("levelset_gradient");
        let pipeline = pipeline_cache.get_compute_pipeline(self.pipeline).unwrap();

        let num_workgroups = workgroup_size_center(size, workgroup_shape);

        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group.bind_group, &[]);
        pass.dispatch_workgroups(num_workgroups.x, num_workgroups.y, num_workgroups.z);

        pass.pop_debug_group();
    }
}

impl FromWorld for LevelSetGradientPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let workgroup_shape = world.resource::<WorkgroupShape>();

        let bind_group_layout =
            LevelSetGradientResource::bind_group_layout_descriptor(render_device);
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("levelset_gradient_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            shader: load_embedded_asset!(world, "levelset_gradient.wgsl"),
            entry_point: Some("levelset_gradient".into()),
            shader_defs: vec![workgroup_shape.shader_def()],
            ..default()
        });

        LevelSetGradientPipeline {
            pipeline,
            bind_group_layout,
        }
    }
}

impl HasBindGroupLayout for LevelSetGradientPipeline {
    fn bind_group_layout(&self) -> &BindGroupLayoutDescriptor {
        &self.bind_group_layout
    }
}

#[derive(Component)]
pub(crate) struct LevelSetGradientBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for LevelSetGradientBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
