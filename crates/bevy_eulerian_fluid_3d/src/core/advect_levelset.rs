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
    core::{
        fluid_uniform::{FluidUniformBindGroup, FluidUniformBindGroupLayout},
        workgroup::{workgroup_size_center, WorkgroupShape},
    },
    resource::FluidResources,
};

pub(crate) struct AdvectLevelSetPass;

impl FluidComputePass for AdvectLevelSetPass {
    type Pipeline = AdvectLevelSetPipeline;
    type Resource = AdvectLevelSetResource;
    type BG = AdvectLevelSetBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "advect_levelset.wgsl");
    }
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct AdvectLevelSetResource {
    #[storage_texture(0, image_format = R32Float, dimension = "3d", access = ReadOnly)]
    pub u0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, dimension = "3d", access = ReadOnly)]
    pub v0: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, dimension = "3d", access = ReadOnly)]
    pub w0: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, dimension = "3d", access = ReadOnly)]
    pub levelset_air0: Handle<Image>,
    #[storage_texture(4, image_format = R32Float, dimension = "3d", access = WriteOnly)]
    pub levelset_air1: Handle<Image>,
}

impl AdvectLevelSetResource {
    pub fn new(resources: &FluidResources) -> Self {
        Self {
            u0: resources.u0.clone(),
            v0: resources.v0.clone(),
            w0: resources.w0.clone(),
            levelset_air0: resources.levelset_air0.clone(),
            levelset_air1: resources.levelset_air1.clone(),
        }
    }
}

#[derive(Resource)]
pub(crate) struct AdvectLevelSetPipeline {
    pub pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayoutDescriptor,
}

impl AdvectLevelSetPipeline {
    pub fn is_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        is_pipeline_loaded(pipeline_cache, self.pipeline)
    }

    pub fn dispatch(
        &self,
        pipeline_cache: &PipelineCache,
        pass: &mut ComputePass,
        bind_group: &AdvectLevelSetBindGroup,
        uniform_bind_group: &FluidUniformBindGroup,
        workgroup_shape: &WorkgroupShape,
        size: UVec3,
    ) {
        pass.push_debug_group("advect_levelset");
        let pipeline = pipeline_cache.get_compute_pipeline(self.pipeline).unwrap();

        let num_workgroups = workgroup_size_center(size, workgroup_shape);

        pass.set_bind_group(0, &bind_group.bind_group, &[]);
        pass.set_bind_group(
            1,
            &uniform_bind_group.bind_group,
            &[uniform_bind_group.index],
        );

        pass.set_pipeline(pipeline);
        pass.dispatch_workgroups(num_workgroups.x, num_workgroups.y, num_workgroups.z);

        pass.pop_debug_group();
    }
}

impl FromWorld for AdvectLevelSetPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let workgroup_shape = world.resource::<WorkgroupShape>();

        let bind_group_layout = AdvectLevelSetResource::bind_group_layout_descriptor(render_device);
        let uniform_bind_group_layout = world.resource::<FluidUniformBindGroupLayout>();

        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("advect_levelset_pipeline".into()),
            layout: vec![
                bind_group_layout.clone(),
                uniform_bind_group_layout.0.clone(),
            ],
            shader: load_embedded_asset!(world, "advect_levelset.wgsl"),
            shader_defs: vec!["CUBIC".into(), workgroup_shape.shader_def()],
            entry_point: Some("advect_levelset".into()),
            ..default()
        });

        AdvectLevelSetPipeline {
            pipeline,
            bind_group_layout,
        }
    }
}

impl HasBindGroupLayout for AdvectLevelSetPipeline {
    fn bind_group_layout(&self) -> &BindGroupLayoutDescriptor {
        &self.bind_group_layout
    }
}

#[derive(Component)]
pub(crate) struct AdvectLevelSetBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for AdvectLevelSetBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
