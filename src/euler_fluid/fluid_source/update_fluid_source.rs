use bevy::{
    asset::{embedded_asset, embedded_path},
    ecs::query::QueryData,
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{
            binding_types::uniform_buffer, AsBindGroup, BindGroup, BindGroupLayoutDescriptor,
            BindGroupLayoutEntries, CachedComputePipelineId, ComputePass, PipelineCache,
            ShaderStages,
        },
        renderer::RenderDevice,
    },
};

use crate::{
    fluid_source::fluid_source_uniform::{FluidSourceUniform, FluidSourceUniformBindGroup},
    pipeline::{is_pipeline_loaded, queue_compute_pipeline, HasBindGroupLayout},
    plugin::FluidComputePass,
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

        let fluid_source_uniform_bind_group_layout = BindGroupLayoutDescriptor::new(
            "FluidSourceUniformBindGroupLayout",
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                uniform_buffer::<FluidSourceUniform>(true),
            ),
        );

        let bind_group_layout =
            UpdateFluidSourceResource::bind_group_layout_descriptor(render_device);

        let pipeline = queue_compute_pipeline(
            world,
            "UpdateFluidSourcePipeline",
            embedded_path!("update_fluid_source.wgsl"),
            "update_fluid_source",
            vec![
                bind_group_layout.clone(),
                fluid_source_uniform_bind_group_layout,
            ],
        );

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
        num_workgroups: UVec3,
    ) {
        let pipeline = pipeline_cache.get_compute_pipeline(self.pipeline).unwrap();

        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_groups.bind_group.bind_group, &[]);
        pass.set_bind_group(
            1,
            &bind_groups.uniform_bind_group.bind_group,
            &[bind_groups.uniform_bind_group.index],
        );

        pass.dispatch_workgroups(num_workgroups.x, num_workgroups.y, num_workgroups.z);
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct UpdateFluidSourceResource {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub levelset_air: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = WriteOnly)]
    pub u: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = WriteOnly)]
    pub v: Handle<Image>,
}

impl UpdateFluidSourceResource {
    pub fn new(levelset_air: &Handle<Image>, u: &Handle<Image>, v: &Handle<Image>) -> Self {
        Self {
            levelset_air: levelset_air.clone(),
            u: u.clone(),
            v: v.clone(),
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
