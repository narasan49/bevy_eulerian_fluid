use bevy::{
    asset::{embedded_asset, embedded_path},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{
            AsBindGroup, BindGroup, BindGroupLayoutDescriptor, CachedComputePipelineId,
            ComputePass, PipelineCache,
        },
        renderer::RenderDevice,
        storage::ShaderStorageBuffer,
    },
};

use crate::{
    pipeline::{is_pipeline_loaded, queue_compute_pipeline, HasBindGroupLayout},
    plugin::FluidComputePass,
};

pub(crate) const PREFIX_SUM_BLOCK_SIZE: usize = 512;

pub(crate) struct PrefixSumPass;

impl FluidComputePass for PrefixSumPass {
    type Pipeline = PrefixSumPipeline;
    type Resource = PrefixSumResource;
    type BG = PrefixSumBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/prefix_sum.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct PrefixSumResource {
    #[storage(0, read_only, visibility(compute))]
    pub counts: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(compute))]
    pub offsets: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub block_scan_sums: Handle<ShaderStorageBuffer>,
}

#[derive(Component)]
pub(crate) struct PrefixSumBindGroup {
    pub _bind_group: BindGroup,
}

#[derive(Resource)]
pub(crate) struct PrefixSumPipeline {
    pub prefix_sum_block_pipeline: CachedComputePipelineId,
    pub prefix_sum_local_scans_pipeline: CachedComputePipelineId,
    pub add_scanned_block_sums_pipeline: CachedComputePipelineId,
    pub bind_group_layout: BindGroupLayoutDescriptor,
}

impl PrefixSumPipeline {
    pub fn new(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let bind_group_layout = PrefixSumResource::bind_group_layout_descriptor(render_device);

        let prefix_sum_block_pipeline = queue_compute_pipeline(
            world,
            "PrefixSumPerWorkgroupPipeline",
            embedded_path!("shaders/prefix_sum.wgsl"),
            "prefix_sum_per_workgroup",
            vec![bind_group_layout.clone()],
        );

        let prefix_sum_local_scans_pipeline = queue_compute_pipeline(
            world,
            "PrefixSumLocalScans",
            embedded_path!("shaders/prefix_sum.wgsl"),
            "prefix_sum_local_scans",
            vec![bind_group_layout.clone()],
        );

        let add_scanned_block_sums_pipeline = queue_compute_pipeline(
            world,
            "AddScannedBlockSums",
            embedded_path!("shaders/prefix_sum.wgsl"),
            "add_scanned_block_sums",
            vec![bind_group_layout.clone()],
        );

        Self {
            prefix_sum_block_pipeline,
            prefix_sum_local_scans_pipeline,
            add_scanned_block_sums_pipeline,
            bind_group_layout,
        }
    }
    pub fn is_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        is_pipeline_loaded(pipeline_cache, self.prefix_sum_block_pipeline)
            && is_pipeline_loaded(pipeline_cache, self.prefix_sum_local_scans_pipeline)
            && is_pipeline_loaded(pipeline_cache, self.add_scanned_block_sums_pipeline)
    }

    pub fn dispatch(
        &self,
        pipeline_cache: &PipelineCache,
        pass: &mut ComputePass,
        bind_group: &BindGroup,
        size: UVec2,
    ) {
        pass.set_bind_group(0, bind_group, &[]);
        let prefix_sum_block_pipeline = pipeline_cache
            .get_compute_pipeline(self.prefix_sum_block_pipeline)
            .unwrap();
        pass.set_pipeline(&prefix_sum_block_pipeline);
        pass.dispatch_workgroups(size.element_product() / PREFIX_SUM_BLOCK_SIZE as u32, 1, 1);

        let prefix_sum_local_scans_pipeline = pipeline_cache
            .get_compute_pipeline(self.prefix_sum_local_scans_pipeline)
            .unwrap();
        pass.set_pipeline(&prefix_sum_local_scans_pipeline);
        pass.dispatch_workgroups(1, 1, 1);

        let add_scanned_block_sums_pipeline = pipeline_cache
            .get_compute_pipeline(self.add_scanned_block_sums_pipeline)
            .unwrap();
        pass.set_pipeline(&add_scanned_block_sums_pipeline);
        pass.dispatch_workgroups(size.element_product() / PREFIX_SUM_BLOCK_SIZE as u32, 1, 1);
    }
}

impl FromWorld for PrefixSumPipeline {
    fn from_world(world: &mut World) -> Self {
        Self::new(world)
    }
}

impl HasBindGroupLayout for PrefixSumPipeline {
    fn bind_group_layout(&self) -> &bevy::render::render_resource::BindGroupLayoutDescriptor {
        &self.bind_group_layout
    }
}

impl From<BindGroup> for PrefixSumBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self {
            _bind_group: bind_group,
        }
    }
}
