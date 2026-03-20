use bevy::{
    asset::{embedded_asset, embedded_path},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, BindGroup, BindGroupLayout, CachedComputePipelineId, ComputePass,
            PipelineCache,
        },
        renderer::RenderDevice,
        storage::{GpuShaderStorageBuffer, ShaderStorageBuffer},
        texture::{FallbackImage, GpuImage},
    },
};

use crate::{
    pipeline::{is_pipeline_loaded, queue_compute_pipeline},
    plugin::FluidComputePass,
};

pub(crate) const PREFIX_SUM_BLOCK_SIZE: usize = 512;

pub(crate) struct PrefixSumPass;

impl FluidComputePass for PrefixSumPass {
    type P = PrefixSumPipeline;

    type Resource = PrefixSumResource;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/prefix_sum.wgsl");
    }

    fn prepare_bind_groups_system(
    ) -> bevy::ecs::schedule::ScheduleConfigs<bevy::ecs::system::ScheduleSystem> {
        prepare_bind_groups.into_configs()
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
    pub bind_group: BindGroup,
}

#[derive(Resource)]
pub(crate) struct PrefixSumPipeline {
    pub prefix_sum_block_pipeline: CachedComputePipelineId,
    pub prefix_sum_local_scans_pipeline: CachedComputePipelineId,
    pub add_scanned_block_sums_pipeline: CachedComputePipelineId,
    pub bind_group_layout: BindGroupLayout,
}

impl PrefixSumPipeline {
    pub fn new(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let bind_group_layout = PrefixSumResource::bind_group_layout(render_device);

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

fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipeline: Res<PrefixSumPipeline>,
    query: Query<(Entity, &PrefixSumResource)>,
    render_device: Res<RenderDevice>,
    mut param: (
        Res<'a, RenderAssets<GpuImage>>,
        Res<'a, FallbackImage>,
        Res<'a, RenderAssets<GpuShaderStorageBuffer>>,
    ),
) {
    for (entity, resource) in &query {
        let bind_group = resource
            .as_bind_group(&pipeline.bind_group_layout, &render_device, &mut param)
            .unwrap()
            .bind_group;

        commands
            .entity(entity)
            .insert(PrefixSumBindGroup { bind_group });
    }
}
