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

pub(crate) struct DivergencePass;

impl FluidComputePass for DivergencePass {
    type Pipeline = DivergencePipeline;
    type Resource = DivergenceResource;
    type BG = DivergenceBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "divergence.wgsl");
    }
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct DivergenceResource {
    #[storage_texture(0, image_format = R32Float, dimension = "3d", access = ReadOnly)]
    pub u1: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, dimension = "3d", access = ReadOnly)]
    pub v1: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, dimension = "3d", access = ReadOnly)]
    pub w1: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, dimension = "3d", access = ReadOnly)]
    pub u_solid: Handle<Image>,
    #[storage_texture(4, image_format = R32Float, dimension = "3d", access = ReadOnly)]
    pub v_solid: Handle<Image>,
    #[storage_texture(5, image_format = R32Float, dimension = "3d", access = ReadOnly)]
    pub w_solid: Handle<Image>,
    #[storage_texture(6, image_format = Rgba32Float, dimension = "3d", access = ReadOnly)]
    pub area_fraction_solid: Handle<Image>,
    #[storage_texture(7, image_format = R32Float, dimension = "3d", access = WriteOnly)]
    pub div: Handle<Image>,
}

impl DivergenceResource {
    pub fn new(resources: &FluidResources) -> Self {
        Self {
            u1: resources.u1.clone(),
            v1: resources.v1.clone(),
            w1: resources.w1.clone(),
            u_solid: resources.u_solid.clone(),
            v_solid: resources.v_solid.clone(),
            w_solid: resources.w_solid.clone(),
            area_fraction_solid: resources.area_fraction_solid.clone(),
            div: resources.div.clone(),
        }
    }
}

#[derive(Resource)]
pub(crate) struct DivergencePipeline {
    pub pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayoutDescriptor,
}

impl DivergencePipeline {
    pub fn is_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        is_pipeline_loaded(pipeline_cache, self.pipeline)
    }

    pub fn dispatch(
        &self,
        pipeline_cache: &PipelineCache,
        pass: &mut ComputePass,
        bind_group: &DivergenceBindGroup,
        uniform_bind_group: &FluidUniformBindGroup,
        workgroup_shape: &WorkgroupShape,
        size: UVec3,
    ) {
        pass.push_debug_group("divergence");
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

impl FromWorld for DivergencePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let workgroup_shape = world.resource::<WorkgroupShape>();

        let bind_group_layout = DivergenceResource::bind_group_layout_descriptor(render_device);
        let uniform_bind_group_layout = world.resource::<FluidUniformBindGroupLayout>();

        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("divergence_pipeline".into()),
            layout: vec![
                bind_group_layout.clone(),
                uniform_bind_group_layout.0.clone(),
            ],
            shader: load_embedded_asset!(world, "divergence.wgsl"),
            entry_point: Some("divergence".into()),
            shader_defs: vec![workgroup_shape.shader_def()],
            ..default()
        });

        DivergencePipeline {
            pipeline,
            bind_group_layout,
        }
    }
}

impl HasBindGroupLayout for DivergencePipeline {
    fn bind_group_layout(&self) -> &BindGroupLayoutDescriptor {
        &self.bind_group_layout
    }
}

#[derive(Component)]
pub(crate) struct DivergenceBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for DivergenceBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
