use bevy::{
    asset::{embedded_asset, embedded_path},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{
            AsBindGroup, BindGroup, BindGroupLayoutDescriptor, BufferUsages,
            CachedComputePipelineId, ComputePass, PipelineCache,
        },
        renderer::RenderDevice,
        storage::ShaderStorageBuffer,
    },
};

use crate::{
    pipeline::{is_pipeline_loaded, queue_compute_pipeline, HasBindGroupLayout},
    plugin::FluidComputePass,
    settings::FluidTextures,
};

const WG_SIZE: UVec2 = UVec2::splat(16);

pub(crate) struct MinVelocityPass;

impl FluidComputePass for MinVelocityPass {
    type Pipeline = MinVelocityPipeline;
    type Resource = MinVelocityResource;
    type BG = MinVelocityBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "min_velocity.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct MinVelocityResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub u: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadOnly)]
    pub v: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[storage(3, visibility(compute))] // array<f32, 16*16>
    pub partial_sums: Handle<ShaderStorageBuffer>,
    #[storage(4, visibility(compute))] // f32
    pub sum: Handle<ShaderStorageBuffer>,
}

impl MinVelocityResource {
    pub fn new(
        buffers: &mut Assets<ShaderStorageBuffer>,
        fluid_textures: &FluidTextures,
        grid_size: UVec2,
    ) -> Self {
        let size_partial_sums = (grid_size / WG_SIZE).element_product() as usize;
        let partial_sums = buffers.add(ShaderStorageBuffer::from(vec![0.0; size_partial_sums]));
        let mut sum_buffer = ShaderStorageBuffer::from(0.0);
        sum_buffer.buffer_description.usage |= BufferUsages::COPY_SRC;
        let sum = buffers.add(sum_buffer);

        Self {
            u: fluid_textures.u.clone(),
            v: fluid_textures.v.clone(),
            levelset_air: fluid_textures.levelset_air.clone(),
            partial_sums,
            sum,
        }
    }
}

#[derive(Resource)]
pub(crate) struct MinVelocityPipeline {
    pub partial_reduction_pipeline: CachedComputePipelineId,
    pub reduction_pipeline: CachedComputePipelineId,
    pub bind_group_layout: BindGroupLayoutDescriptor,
}

impl FromWorld for MinVelocityPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout = MinVelocityResource::bind_group_layout_descriptor(render_device);

        let partial_reduction_pipeline = queue_compute_pipeline(
            world,
            "PartialReductionPipeline",
            embedded_path!("min_velocity.wgsl"),
            "partial_reduction",
            vec![bind_group_layout.clone()],
        );

        let reduction_pipeline = queue_compute_pipeline(
            world,
            "ReductionPipeline",
            embedded_path!("min_velocity.wgsl"),
            "reduction",
            vec![bind_group_layout.clone()],
        );

        Self {
            partial_reduction_pipeline,
            reduction_pipeline,
            bind_group_layout,
        }
    }
}

impl HasBindGroupLayout for MinVelocityPipeline {
    fn bind_group_layout(&self) -> &BindGroupLayoutDescriptor {
        &self.bind_group_layout
    }
}

impl MinVelocityPipeline {
    pub fn is_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        is_pipeline_loaded(pipeline_cache, self.partial_reduction_pipeline)
            && is_pipeline_loaded(pipeline_cache, self.reduction_pipeline)
    }

    pub fn dispatch(
        &self,
        pipeline_cache: &PipelineCache,
        pass: &mut ComputePass,
        bind_group: &MinVelocityBindGroup,
        grid_size: UVec2,
    ) {
        pass.push_debug_group("Calculate volume");
        let partial_reduction_pipeline = pipeline_cache
            .get_compute_pipeline(self.partial_reduction_pipeline)
            .unwrap();

        let reduction_pipeline = pipeline_cache
            .get_compute_pipeline(self.reduction_pipeline)
            .unwrap();

        pass.set_pipeline(partial_reduction_pipeline);
        pass.set_bind_group(0, &bind_group.bind_group, &[]);

        let num_workgroups = grid_size / WG_SIZE;
        pass.dispatch_workgroups(num_workgroups.x, num_workgroups.y, 1);

        pass.set_pipeline(reduction_pipeline);
        pass.dispatch_workgroups(1, 1, 1);

        pass.pop_debug_group();
    }
}

#[derive(Component)]
pub(crate) struct MinVelocityBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for MinVelocityBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
