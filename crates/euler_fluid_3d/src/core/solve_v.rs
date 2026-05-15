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
        workgroup::{workgroup_size_y, WorkgroupShape},
    },
    resource::FluidResources,
};

pub(crate) struct SolveVPass;

impl FluidComputePass for SolveVPass {
    type Pipeline = SolveVPipeline;
    type Resource = SolveVResource;
    type BG = SolveVBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "solve_v.wgsl");
    }
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct SolveVResource {
    #[storage_texture(0, image_format = R32Float, dimension = "3d", access = WriteOnly)]
    pub v0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, dimension = "3d", access = ReadOnly)]
    pub v1: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, dimension = "3d", access = ReadOnly)]
    pub v_solid: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, dimension = "3d", access = ReadOnly)]
    pub p: Handle<Image>,
    #[storage_texture(4, image_format = R32Float, dimension = "3d", access = ReadOnly)]
    pub levelset_air0: Handle<Image>,
    #[storage_texture(5, image_format = Rgba32Float, dimension = "3d", access = ReadOnly)]
    pub area_fraction_solid: Handle<Image>,
}

impl SolveVResource {
    pub fn new(resources: &FluidResources) -> Self {
        Self {
            v0: resources.v0.clone(),
            v1: resources.v1.clone(),
            v_solid: resources.v_solid.clone(),
            p: resources.p.clone(),
            levelset_air0: resources.levelset_air0.clone(),
            area_fraction_solid: resources.area_fraction_solid.clone(),
        }
    }
}

#[derive(Resource)]
pub(crate) struct SolveVPipeline {
    pub pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayoutDescriptor,
}

impl SolveVPipeline {
    pub fn is_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        is_pipeline_loaded(pipeline_cache, self.pipeline)
    }

    pub fn dispatch(
        &self,
        pipeline_cache: &PipelineCache,
        pass: &mut ComputePass,
        bind_group: &SolveVBindGroup,
        uniform_bind_group: &FluidUniformBindGroup,
        workgroup_shape: &WorkgroupShape,
        size: UVec3,
    ) {
        pass.push_debug_group("solve_u");
        let pipeline = pipeline_cache.get_compute_pipeline(self.pipeline).unwrap();

        let num_workgroups = workgroup_size_y(size, workgroup_shape);

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

impl FromWorld for SolveVPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let uniform_bind_group_layout = world.resource::<FluidUniformBindGroupLayout>();

        let bind_group_layout = SolveVResource::bind_group_layout_descriptor(render_device);

        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("solve_v_pipeline".into()),
            layout: vec![
                bind_group_layout.clone(),
                uniform_bind_group_layout.0.clone(),
            ],
            shader: load_embedded_asset!(world, "solve_v.wgsl"),
            entry_point: Some("solve_v".into()),
            ..default()
        });

        SolveVPipeline {
            pipeline,
            bind_group_layout,
        }
    }
}

impl HasBindGroupLayout for SolveVPipeline {
    fn bind_group_layout(&self) -> &BindGroupLayoutDescriptor {
        &self.bind_group_layout
    }
}

#[derive(Component)]
pub(crate) struct SolveVBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for SolveVBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
