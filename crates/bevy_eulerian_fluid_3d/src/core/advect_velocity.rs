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
        workgroup::{workgroup_size_x, workgroup_size_y, workgroup_size_z, WorkgroupShape},
    },
    resource::FluidResources,
};

pub(crate) struct AdvectionPass;

impl FluidComputePass for AdvectionPass {
    type Pipeline = AdvectVelocityPipeline;
    type Resource = AdvectVelocityResource;
    type BG = AdvectVelocityBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "advect_velocity.wgsl");
    }
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct AdvectVelocityResource {
    #[storage_texture(0, image_format = R32Float, dimension = "3d", access = ReadOnly)]
    pub u0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, dimension = "3d", access = ReadOnly)]
    pub v0: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, dimension = "3d", access = ReadOnly)]
    pub w0: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, dimension = "3d", access = WriteOnly)]
    pub u1: Handle<Image>,
    #[storage_texture(4, image_format = R32Float, dimension = "3d", access = WriteOnly)]
    pub v1: Handle<Image>,
    #[storage_texture(5, image_format = R32Float, dimension = "3d", access = WriteOnly)]
    pub w1: Handle<Image>,
}

impl AdvectVelocityResource {
    pub fn new(resources: &FluidResources) -> Self {
        Self {
            u0: resources.u0.clone(),
            v0: resources.v0.clone(),
            w0: resources.w0.clone(),
            u1: resources.u1.clone(),
            v1: resources.v1.clone(),
            w1: resources.w1.clone(),
        }
    }
}

#[derive(Resource)]
pub(crate) struct AdvectVelocityPipeline {
    pub advect_u_pipeline: CachedComputePipelineId,
    pub advect_v_pipeline: CachedComputePipelineId,
    pub advect_w_pipeline: CachedComputePipelineId,
    advect_velocity_bind_group_layout: BindGroupLayoutDescriptor,
}

impl AdvectVelocityPipeline {
    pub fn is_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        is_pipeline_loaded(pipeline_cache, self.advect_u_pipeline)
            && is_pipeline_loaded(pipeline_cache, self.advect_v_pipeline)
            && is_pipeline_loaded(pipeline_cache, self.advect_w_pipeline)
    }

    pub fn dispatch(
        &self,
        pipeline_cache: &PipelineCache,
        pass: &mut ComputePass,
        bind_group: &AdvectVelocityBindGroup,
        uniform_bind_group: &FluidUniformBindGroup,
        workgroup_shape: &WorkgroupShape,
        size: UVec3,
    ) {
        pass.push_debug_group("advect_velocity");
        let advect_u_pipeline = pipeline_cache
            .get_compute_pipeline(self.advect_u_pipeline)
            .unwrap();
        let advect_v_pipeline = pipeline_cache
            .get_compute_pipeline(self.advect_v_pipeline)
            .unwrap();
        let advect_w_pipeline = pipeline_cache
            .get_compute_pipeline(self.advect_w_pipeline)
            .unwrap();

        let num_workgroups_x = workgroup_size_x(size, workgroup_shape);
        let num_workgroups_y = workgroup_size_y(size, workgroup_shape);
        let num_workgroups_z = workgroup_size_z(size, workgroup_shape);

        pass.set_bind_group(0, &bind_group.bind_group, &[]);
        pass.set_bind_group(
            1,
            &uniform_bind_group.bind_group,
            &[uniform_bind_group.index],
        );

        pass.set_pipeline(advect_u_pipeline);
        pass.dispatch_workgroups(num_workgroups_x.x, num_workgroups_x.y, num_workgroups_x.z);

        pass.set_pipeline(advect_v_pipeline);
        pass.dispatch_workgroups(num_workgroups_y.x, num_workgroups_y.y, num_workgroups_y.z);

        pass.set_pipeline(advect_w_pipeline);
        pass.dispatch_workgroups(num_workgroups_z.x, num_workgroups_z.y, num_workgroups_z.z);

        pass.pop_debug_group();
    }
}

impl FromWorld for AdvectVelocityPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let workgroup_shape = world.resource::<WorkgroupShape>();

        let advect_velocity_bind_group_layout =
            AdvectVelocityResource::bind_group_layout_descriptor(render_device);
        let uniform_bind_group_layout = world.resource::<FluidUniformBindGroupLayout>();

        let advect_u_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("advect_u_pipeline".into()),
            layout: vec![
                advect_velocity_bind_group_layout.clone(),
                uniform_bind_group_layout.0.clone(),
            ],
            shader: load_embedded_asset!(world, "advect_velocity.wgsl"),
            entry_point: Some("advect_u".into()),
            shader_defs: vec![workgroup_shape.shader_def()],
            ..default()
        });
        let advect_v_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("advect_v_pipeline".into()),
            layout: vec![
                advect_velocity_bind_group_layout.clone(),
                uniform_bind_group_layout.0.clone(),
            ],
            shader: load_embedded_asset!(world, "advect_velocity.wgsl"),
            entry_point: Some("advect_v".into()),
            shader_defs: vec![workgroup_shape.shader_def()],
            ..default()
        });
        let advect_w_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("advect_w_pipeline".into()),
            layout: vec![
                advect_velocity_bind_group_layout.clone(),
                uniform_bind_group_layout.0.clone(),
            ],
            shader: load_embedded_asset!(world, "advect_velocity.wgsl"),
            entry_point: Some("advect_w".into()),
            shader_defs: vec![workgroup_shape.shader_def()],
            ..default()
        });

        Self {
            advect_u_pipeline,
            advect_v_pipeline,
            advect_w_pipeline,
            advect_velocity_bind_group_layout,
        }
    }
}

impl HasBindGroupLayout for AdvectVelocityPipeline {
    fn bind_group_layout(&self) -> &BindGroupLayoutDescriptor {
        &self.advect_velocity_bind_group_layout
    }
}

#[derive(Component)]
pub(crate) struct AdvectVelocityBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for AdvectVelocityBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
