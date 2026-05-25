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
    core::workgroup::{workgroup_size_xyz, WorkgroupShape},
    resource::FluidResources,
};

pub(crate) struct InitializeResourcesPass;

impl FluidComputePass for InitializeResourcesPass {
    type Pipeline = InitializeResourcesPipeline;
    type Resource = InitializeResourcesResource;
    type BG = InitializeResourcesBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "initialize_resources.wgsl");
    }
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct InitializeResourcesResource {
    #[storage_texture(0, image_format = R32Float, dimension = "3d", access = WriteOnly)]
    pub levelset_air0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, dimension = "3d", access = WriteOnly)]
    pub levelset_air1: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, dimension = "3d", access = WriteOnly)]
    pub u0: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, dimension = "3d", access = WriteOnly)]
    pub v0: Handle<Image>,
    #[storage_texture(4, image_format = R32Float, dimension = "3d", access = WriteOnly)]
    pub w0: Handle<Image>,
}

impl InitializeResourcesResource {
    pub fn new(resources: &FluidResources) -> Self {
        Self {
            levelset_air0: resources.levelset_air0.clone(),
            levelset_air1: resources.levelset_air1.clone(),
            u0: resources.u0.clone(),
            v0: resources.v0.clone(),
            w0: resources.w0.clone(),
        }
    }
}

#[derive(Resource)]
pub(crate) struct InitializeResourcesPipeline {
    pub pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayoutDescriptor,
}

impl InitializeResourcesPipeline {
    pub fn is_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        is_pipeline_loaded(pipeline_cache, self.pipeline)
    }

    pub fn dispatch(
        &self,
        pipeline_cache: &PipelineCache,
        pass: &mut ComputePass,
        bind_group: &InitializeResourcesBindGroup,
        workgroup_shape: &WorkgroupShape,
        size: UVec3,
    ) {
        pass.push_debug_group("initialize_resources");
        let pipeline = pipeline_cache.get_compute_pipeline(self.pipeline).unwrap();

        let num_workgroups = workgroup_size_xyz(size, workgroup_shape);

        pass.set_bind_group(0, &bind_group.bind_group, &[]);

        pass.set_pipeline(pipeline);
        pass.dispatch_workgroups(num_workgroups.x, num_workgroups.y, num_workgroups.z);

        pass.pop_debug_group();
    }
}

impl FromWorld for InitializeResourcesPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let workgroup_shape = world.resource::<WorkgroupShape>();

        let bind_group_layout =
            InitializeResourcesResource::bind_group_layout_descriptor(render_device);

        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("initialize_resources_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            shader: load_embedded_asset!(world, "initialize_resources.wgsl"),
            entry_point: Some("initialize_resources".into()),
            shader_defs: vec![workgroup_shape.shader_def()],
            ..default()
        });

        InitializeResourcesPipeline {
            pipeline,
            bind_group_layout,
        }
    }
}

impl HasBindGroupLayout for InitializeResourcesPipeline {
    fn bind_group_layout(&self) -> &BindGroupLayoutDescriptor {
        &self.bind_group_layout
    }
}

#[derive(Component)]
pub(crate) struct InitializeResourcesBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for InitializeResourcesBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
