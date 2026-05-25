use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    ecs::query::QueryData,
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{
            binding_types::uniform_buffer, AsBindGroup, BindGroup, BindGroupLayoutDescriptor,
            BindGroupLayoutEntries, CachedComputePipelineId, ComputePass,
            ComputePipelineDescriptor, PipelineCache, ShaderStages,
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
        fluid_source::fluid_source_uniform::{FluidSourceUniform, FluidSourceUniformBindGroup},
        workgroup::{workgroup_size_center, WorkgroupShape},
    },
    resource::FluidResources,
};

pub(crate) struct UpdateFluidSourcePass;

impl FluidComputePass for UpdateFluidSourcePass {
    type Pipeline = UpdateFluidSourcePipeline;
    type Resource = UpdateFluidSourceResource;
    type BG = UpdateFluidSourceBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "update_fluid_source.wgsl");
    }
}

#[derive(QueryData)]
pub(crate) struct UpdateFluidSourceBindGroupsQuery {
    bind_group: &'static UpdateFluidSourceBindGroup,
    uniform_bind_group: &'static FluidSourceUniformBindGroup,
}

#[derive(Resource)]
pub(crate) struct UpdateFluidSourcePipeline {
    pub pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayoutDescriptor,
}

impl HasBindGroupLayout for UpdateFluidSourcePipeline {
    fn bind_group_layout(&self) -> &BindGroupLayoutDescriptor {
        &self.bind_group_layout
    }
}

impl FromWorld for UpdateFluidSourcePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let workgroup_shape = world.resource::<WorkgroupShape>();

        let fluid_source_uniform_bind_group_layout = BindGroupLayoutDescriptor::new(
            "FluidSourceUniformBindGroupLayout",
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                uniform_buffer::<FluidSourceUniform>(true),
            ),
        );

        let bind_group_layout =
            UpdateFluidSourceResource::bind_group_layout_descriptor(render_device);

        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("update_fluid_source_pipeline".into()),
            layout: vec![
                bind_group_layout.clone(),
                fluid_source_uniform_bind_group_layout,
            ],
            shader: load_embedded_asset!(world, "update_fluid_source.wgsl"),
            entry_point: Some("update_fluid_source".into()),
            shader_defs: vec![workgroup_shape.shader_def()],
            ..default()
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }
}

impl UpdateFluidSourcePipeline {
    pub fn is_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        is_pipeline_loaded(pipeline_cache, self.pipeline)
    }

    pub fn dispatch(
        &self,
        pipeline_cache: &PipelineCache,
        pass: &mut ComputePass,
        bind_groups: &UpdateFluidSourceBindGroupsQueryItem,
        workgroup_shape: &WorkgroupShape,
        size: UVec3,
    ) {
        let pipeline = pipeline_cache.get_compute_pipeline(self.pipeline).unwrap();
        let num_workgroups = workgroup_size_center(size, workgroup_shape);

        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_groups.bind_group.bind_group, &[]);
        pass.set_bind_group(
            1,
            &bind_groups.uniform_bind_group.update_bind_group,
            &[bind_groups.uniform_bind_group.update_index],
        );

        pass.dispatch_workgroups(num_workgroups.x, num_workgroups.y, num_workgroups.z);
    }

    pub fn dispatch_init(
        &self,
        pipeline_cache: &PipelineCache,
        pass: &mut ComputePass,
        bind_groups: &UpdateFluidSourceBindGroupsQueryItem,
        workgroup_shape: &WorkgroupShape,
        size: UVec3,
    ) {
        let pipeline = pipeline_cache.get_compute_pipeline(self.pipeline).unwrap();
        let num_workgroups = workgroup_size_center(size, workgroup_shape);

        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_groups.bind_group.bind_group, &[]);
        pass.set_bind_group(
            1,
            &bind_groups.uniform_bind_group.init_bind_group,
            &[bind_groups.uniform_bind_group.init_index],
        );

        pass.dispatch_workgroups(num_workgroups.x, num_workgroups.y, num_workgroups.z);
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct UpdateFluidSourceResource {
    #[storage_texture(0, image_format = R32Float, dimension = "3d", access = ReadWrite)]
    pub levelset_air1: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, dimension = "3d", access = WriteOnly)]
    pub u0: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, dimension = "3d", access = WriteOnly)]
    pub v0: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, dimension = "3d", access = WriteOnly)]
    pub w0: Handle<Image>,
}

impl UpdateFluidSourceResource {
    pub fn new(resources: &FluidResources) -> Self {
        Self {
            levelset_air1: resources.levelset_air1.clone(),
            u0: resources.u0.clone(),
            v0: resources.v0.clone(),
            w0: resources.w0.clone(),
        }
    }
}

#[derive(Component)]
pub(crate) struct UpdateFluidSourceBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for UpdateFluidSourceBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
