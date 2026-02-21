use bevy::{
    asset::{embedded_path, AssetPath},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, BindGroup, BindGroupLayout, CachedComputePipelineId, ComputePass,
            ComputePipelineDescriptor, PipelineCache,
        },
        renderer::RenderDevice,
        storage::{GpuShaderStorageBuffer, ShaderStorageBuffer},
        texture::{FallbackImage, GpuImage},
    },
};

use crate::{pipeline::Pipeline, settings::FluidSettings};

pub(crate) const PREFIX_SUM_BLOCK_SIZE: usize = 512;

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct CountParticlesInCellResource {
    #[storage(0, read_only, visibility(compute))]
    pub particles: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub particle_count: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub cell_particle_counts: Handle<ShaderStorageBuffer>,
    #[uniform(3)]
    pub grid_size: UVec2,
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct PrefixSumParticleCountsResource {
    #[storage(0, visibility(compute))]
    pub cell_offsets: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(compute))]
    pub block_scan_sums: Handle<ShaderStorageBuffer>,
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct SortParticlesResource {
    #[storage(0, read_only, visibility(compute))]
    pub particles: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub cell_offsets: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub sorted_particles: Handle<ShaderStorageBuffer>,
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct DistributeParticlesResource {
    #[storage(0, read_only, visibility(compute))]
    pub particles: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub sorted_particles: Handle<ShaderStorageBuffer>,
}

#[derive(Component)]
pub(crate) struct DistributeParticlesToGridBindGroups {
    pub count_particles_bind_group: BindGroup,
    pub prefix_sum_bind_group: BindGroup,
    // pub sort_particles_bind_group: BindGroup,
    // pub distribute_paarticles_bind_group: BindGroup,
}

pub(crate) fn create_buffers(
    buffers: &mut ResMut<Assets<ShaderStorageBuffer>>,
    size: UVec2,
) -> (
    Handle<ShaderStorageBuffer>,
    Handle<ShaderStorageBuffer>,
    Handle<ShaderStorageBuffer>,
) {
    let size_grid = size.element_product() as usize;

    let cell_particle_counts = buffers.add(ShaderStorageBuffer::from(vec![0u32; size_grid]));
    let sorted_particles = buffers.add(ShaderStorageBuffer::from(vec![0u32; size_grid]));

    let block_scan_sums = buffers.add(ShaderStorageBuffer::from(vec![
        0u32;
        size_grid
            / PREFIX_SUM_BLOCK_SIZE
    ]));

    (cell_particle_counts, sorted_particles, block_scan_sums)
}

pub(crate) fn insert_distribute_particles_resources(
    commands: &mut Commands,
    entity: Entity,
    particles: Handle<ShaderStorageBuffer>,
    particle_count: Handle<ShaderStorageBuffer>,
    cell_particle_counts: Handle<ShaderStorageBuffer>,
    block_scan_sums: Handle<ShaderStorageBuffer>,
    sorted_particles: Handle<ShaderStorageBuffer>,
    grid_size: UVec2,
) {
    let count_particle_resource = CountParticlesInCellResource {
        particles: particles.clone(),
        particle_count: particle_count.clone(),
        cell_particle_counts: cell_particle_counts.clone(),
        grid_size,
    };

    let prefix_sum_particle_counts_resource = PrefixSumParticleCountsResource {
        cell_offsets: cell_particle_counts.clone(),
        block_scan_sums: block_scan_sums.clone(),
    };

    let sort_particles_resource = SortParticlesResource {
        particles: particles.clone(),
        cell_offsets: cell_particle_counts.clone(),
        sorted_particles: sorted_particles.clone(),
    };

    let distribute_particles_resource = DistributeParticlesResource {
        particles: particles.clone(),
        sorted_particles: sorted_particles.clone(),
    };

    commands.entity(entity).insert((
        count_particle_resource,
        prefix_sum_particle_counts_resource,
        sort_particles_resource,
        distribute_particles_resource,
    ));
}

#[derive(Resource)]
pub(crate) struct DistributeParticlesToGridPipelines {
    pub count_particles: CountParticlesPipeline,
    pub prefix_sum: PrefixSumParticleCountsPipeline,
    // pub sort_particles: SortParticlesPipeline,
    // pub distribute_particles: DistributeParticlesPipeline,
}

pub(crate) struct CountParticlesPipeline {
    pub pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayout,
}

pub(crate) struct PrefixSumParticleCountsPipeline {
    pub prefix_sum_block_pipeline: CachedComputePipelineId,
    pub prefix_sum_local_scans_pipeline: CachedComputePipelineId,
    pub add_scanned_block_sums_pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayout,
}

pub(crate) struct SortParticlesPipeline {
    pub pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayout,
}

pub(crate) struct DistributeParticlesPipeline {
    pub pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayout,
}

impl FromWorld for DistributeParticlesToGridPipelines {
    fn from_world(world: &mut World) -> Self {
        let count_particles = CountParticlesPipeline::from_world(world);
        let prefix_sum = PrefixSumParticleCountsPipeline::from_world(world);
        // let sort_particles = SortParticlesPipeline::from_world(world);
        // let distribute_particles = DistributeParticlesPipeline::from_world(world);

        DistributeParticlesToGridPipelines {
            count_particles,
            prefix_sum,
            // sort_particles,
            // distribute_particles,
        }
    }
}

impl Pipeline for CountParticlesPipeline {
    fn is_pipeline_state_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        Self::is_pipeline_loaded(pipeline_cache, self.pipeline)
    }
}

impl FromWorld for CountParticlesPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout = CountParticlesInCellResource::bind_group_layout(render_device);

        let pipeline = create_pipeline(
            world,
            "CountParticlesPipeline",
            "shaders/distribute/count_particles_in_cell.wgsl",
            "count_particles_in_cell",
            vec![bind_group_layout.clone()],
        );

        CountParticlesPipeline {
            pipeline,
            bind_group_layout,
        }
    }
}

impl Pipeline for PrefixSumParticleCountsPipeline {
    fn is_pipeline_state_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        Self::is_pipeline_loaded(pipeline_cache, self.prefix_sum_block_pipeline)
            && Self::is_pipeline_loaded(pipeline_cache, self.prefix_sum_local_scans_pipeline)
            && Self::is_pipeline_loaded(pipeline_cache, self.add_scanned_block_sums_pipeline)
    }
}

impl FromWorld for PrefixSumParticleCountsPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout = PrefixSumParticleCountsResource::bind_group_layout(render_device);

        let prefix_sum_block_pipeline = create_pipeline(
            world,
            "PrefixSumPerWorkgroupPipeline",
            "shaders/distribute/prefix_sum_particle_counts.wgsl",
            "prefix_sum_particle_counts_per_workgroup",
            vec![bind_group_layout.clone()],
        );

        let prefix_sum_local_scans_pipeline = create_pipeline(
            world,
            "PrefixSumLocalScans",
            "shaders/distribute/prefix_sum_particle_counts.wgsl",
            "prefix_sum_local_scans",
            vec![bind_group_layout.clone()],
        );

        let add_scanned_block_sums_pipeline = create_pipeline(
            world,
            "AddScannedBlockSums",
            "shaders/distribute/prefix_sum_particle_counts.wgsl",
            "add_scanned_block_sums",
            vec![bind_group_layout.clone()],
        );

        PrefixSumParticleCountsPipeline {
            prefix_sum_block_pipeline,
            prefix_sum_local_scans_pipeline,
            add_scanned_block_sums_pipeline,
            bind_group_layout,
        }
    }
}

impl Pipeline for SortParticlesPipeline {
    fn is_pipeline_state_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        Self::is_pipeline_loaded(pipeline_cache, self.pipeline)
    }
}

impl FromWorld for SortParticlesPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout = SortParticlesResource::bind_group_layout(render_device);

        let pipeline = create_pipeline(
            world,
            "SortParticlesPipeline",
            "shaders/distribute/sort_particles.wgsl",
            "sort_particles",
            vec![bind_group_layout.clone()],
        );

        SortParticlesPipeline {
            pipeline,
            bind_group_layout,
        }
    }
}

impl Pipeline for DistributeParticlesPipeline {
    fn is_pipeline_state_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        Self::is_pipeline_loaded(pipeline_cache, self.pipeline)
    }
}

impl FromWorld for DistributeParticlesPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout = DistributeParticlesResource::bind_group_layout(render_device);

        let pipeline = create_pipeline(
            world,
            "DistributeParticlesPipeline",
            "shaders/distribute/distribute_particles.wgsl",
            "distribute_particles",
            vec![bind_group_layout.clone()],
        );

        DistributeParticlesPipeline {
            pipeline,
            bind_group_layout,
        }
    }
}

fn create_pipeline(
    world: &mut World,
    label: &'static str,
    shader: &'static str,
    entry_point: &'static str,
    bind_group_layouts: Vec<BindGroupLayout>,
) -> CachedComputePipelineId {
    let pipeline_cache = world.resource::<PipelineCache>();
    let asset_server = world.resource::<AssetServer>();

    let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some(label.into()),
        layout: bind_group_layouts,
        shader: asset_server
            .load(AssetPath::from_path_buf(embedded_path!(shader)).with_source("embedded")),
        entry_point: Some(entry_point.into()),
        ..default()
    });

    pipeline
}

pub(super) fn reset_buffers(
    query: Query<(&PrefixSumParticleCountsResource, &FluidSettings)>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    for (resource, settings) in &query {
        let grid_size = settings.size.element_product() as usize;

        let cell_offsets = buffers.get_mut(&resource.cell_offsets).unwrap();
        cell_offsets.set_data(vec![0u32; grid_size]);

        let block_scan_sums = buffers.get_mut(&resource.block_scan_sums).unwrap();
        block_scan_sums.set_data(vec![0u32; grid_size / PREFIX_SUM_BLOCK_SIZE]);
    }
}

pub(super) fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipelines: Res<DistributeParticlesToGridPipelines>,
    query: Query<(
        Entity,
        &CountParticlesInCellResource,
        &PrefixSumParticleCountsResource,
    )>,
    render_device: Res<RenderDevice>,
    mut param: (
        Res<'a, RenderAssets<GpuImage>>,
        Res<'a, FallbackImage>,
        Res<'a, RenderAssets<GpuShaderStorageBuffer>>,
    ),
) {
    for (entity, count_particles_resource, prefix_sum_resource) in &query {
        let count_particles_bind_group = count_particles_resource
            .as_bind_group(
                &pipelines.count_particles.bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let prefix_sum_bind_group = prefix_sum_resource
            .as_bind_group(
                &pipelines.prefix_sum.bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        commands
            .entity(entity)
            .insert(DistributeParticlesToGridBindGroups {
                count_particles_bind_group,
                prefix_sum_bind_group,
            });
    }
}

pub(crate) fn dispatch(
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    bind_groups: &DistributeParticlesToGridBindGroups,
    pipelines: &DistributeParticlesToGridPipelines,
    size: UVec2,
) {
    pass.push_debug_group("Distribute particles to grid");
    let count_particles_pipeline = pipeline_cache
        .get_compute_pipeline(pipelines.count_particles.pipeline)
        .unwrap();

    pass.set_pipeline(&count_particles_pipeline);
    pass.set_bind_group(0, &bind_groups.count_particles_bind_group, &[]);
    pass.dispatch_workgroups(size.element_product() / PREFIX_SUM_BLOCK_SIZE as u32, 1, 1);

    pass.push_debug_group("Prefix-sum particles");
    {
        let prefix_sum_block_pipeline = pipeline_cache
            .get_compute_pipeline(pipelines.prefix_sum.prefix_sum_block_pipeline)
            .unwrap();
        pass.set_pipeline(&prefix_sum_block_pipeline);
        pass.set_bind_group(0, &bind_groups.prefix_sum_bind_group, &[]);
        pass.dispatch_workgroups(size.element_product() / PREFIX_SUM_BLOCK_SIZE as u32, 1, 1);

        let prefix_sum_local_scans_pipeline = pipeline_cache
            .get_compute_pipeline(pipelines.prefix_sum.prefix_sum_local_scans_pipeline)
            .unwrap();
        // let size_scan_block = size.element_product() / PREFIX_SUM_BLOCK_SIZE as u32;
        pass.set_pipeline(&prefix_sum_local_scans_pipeline);
        pass.dispatch_workgroups(1, 1, 1);

        let add_scanned_block_sums_pipeline = pipeline_cache
            .get_compute_pipeline(pipelines.prefix_sum.add_scanned_block_sums_pipeline)
            .unwrap();
        pass.set_pipeline(&add_scanned_block_sums_pipeline);
        pass.dispatch_workgroups(size.element_product() / PREFIX_SUM_BLOCK_SIZE as u32, 1, 1);
    }
    pass.pop_debug_group();

    pass.pop_debug_group();
}
