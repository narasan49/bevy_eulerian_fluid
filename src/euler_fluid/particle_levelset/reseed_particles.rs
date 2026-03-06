use bevy::{
    asset::{embedded_asset, embedded_path, AssetPath},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, BindGroup, BindGroupLayout, CachedComputePipelineId, ComputePass,
            ComputePipelineDescriptor, PipelineCache,
        },
        renderer::RenderDevice,
        storage::{GpuShaderStorageBuffer, ShaderStorageBuffer},
        texture::{FallbackImage, GpuImage},
        Render, RenderApp, RenderSystems,
    },
};

use crate::{
    pipeline::{DispatchFluidPass, Pipeline},
    settings::FluidSettings,
};

const PREFIX_SUM_BLOCK_SIZE: usize = 512;

pub(crate) struct ReseedParticlesPlugin;

impl Plugin for ReseedParticlesPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/reseed_particles/mark_as_remove.wgsl");
        embedded_asset!(
            app,
            "shaders/reseed_particles/prefix_sum_alive_particles.wgsl"
        );
        embedded_asset!(app, "shaders/reseed_particles/remove_particles.wgsl");
        embedded_asset!(app, "shaders/reseed_particles/update_particles_count.wgsl");
        embedded_asset!(app, "shaders/reseed_particles/add_particles.wgsl");

        app.add_plugins((
            ExtractComponentPlugin::<MarkAsRemoveResource>::default(),
            ExtractComponentPlugin::<PrefixSumAliveParticlesResource>::default(),
            ExtractComponentPlugin::<RemoveParticlesResource>::default(),
            ExtractComponentPlugin::<UpdateParticlesCountResource>::default(),
            ExtractComponentPlugin::<AddParticlesResource>::default(),
        ));

        app.add_systems(Update, reset_buffers);

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            prepare_bind_groups.in_set(RenderSystems::PrepareBindGroups),
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<ReseedParticlesPipelines>();
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct MarkAsRemoveResource {
    #[storage(0, visibility(compute))]
    sorted_particles: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(compute))]
    alive_particles_mask: Handle<ShaderStorageBuffer>,
    #[storage(2, read_only, visibility(compute))]
    cell_particle_counts: Handle<ShaderStorageBuffer>,
    #[storage_texture(3, image_format = R8Uint, access = ReadOnly)]
    interface_band_mask: Handle<Image>,
    #[storage(4, read_only, visibility(compute))]
    cell_offsets: Handle<ShaderStorageBuffer>,
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct PrefixSumAliveParticlesResource {
    #[storage(0, read_only, visibility(compute))]
    alive_particles_mask: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(compute))]
    alive_particles_mask_scan: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    sums: Handle<ShaderStorageBuffer>,
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct RemoveParticlesResource {
    #[storage(0, read_only, visibility(compute))]
    alive_particles_mask: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    alive_particles_mask_scan: Handle<ShaderStorageBuffer>,
    #[storage(2, read_only, visibility(compute))]
    sorted_particles: Handle<ShaderStorageBuffer>,
    #[storage(3, visibility(compute))]
    particles: Handle<ShaderStorageBuffer>,
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct UpdateParticlesCountResource {
    #[storage(0, read_only, visibility(compute))]
    alive_particles_mask: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    alive_particles_mask_scan: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    particle_count: Handle<ShaderStorageBuffer>,
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct AddParticlesResource {
    #[storage(0, visibility(compute))]
    particles: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(compute))]
    particle_count: Handle<ShaderStorageBuffer>,
    #[storage(2, read_only, visibility(compute))]
    cell_particle_counts: Handle<ShaderStorageBuffer>,
    #[storage_texture(3, image_format = R8Uint, access = ReadOnly)]
    interface_band_mask: Handle<Image>,
    #[storage_texture(4, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[storage_texture(5, image_format = Rg32Float, access = ReadOnly)]
    pub grad_levelset_air: Handle<Image>,
}

#[derive(Component)]
pub(crate) struct ReseedParticlesBindGroups {
    mark_as_remove_bind_group: BindGroup,
    prefix_sum_alive_particles_bind_group: BindGroup,
    remove_particles_bind_group: BindGroup,
    update_particles_count_bind_group: BindGroup,
    add_particles_bind_group: BindGroup,
}

#[derive(Bundle)]
pub(crate) struct ReseedParticlesBundle {
    mark_as_remove: MarkAsRemoveResource,
    remove_particles: RemoveParticlesResource,
    prefix_sum_alive_particles: PrefixSumAliveParticlesResource,
    update_particles_count: UpdateParticlesCountResource,
    add_particles: AddParticlesResource,
}

impl ReseedParticlesBundle {
    pub fn new(
        sorted_particles: &Handle<ShaderStorageBuffer>,
        alive_particles_mask: &Handle<ShaderStorageBuffer>,
        alive_particles_mask_scan: &Handle<ShaderStorageBuffer>,
        sums: &Handle<ShaderStorageBuffer>,
        particles: &Handle<ShaderStorageBuffer>,
        particle_count: &Handle<ShaderStorageBuffer>,
        cell_particle_counts: &Handle<ShaderStorageBuffer>,
        cell_offsets: &Handle<ShaderStorageBuffer>,
        interface_band_mask: &Handle<Image>,
        levelset_air: &Handle<Image>,
        grad_levelset_air: &Handle<Image>,
    ) -> Self {
        let mark_as_remove = MarkAsRemoveResource {
            sorted_particles: sorted_particles.clone(),
            alive_particles_mask: alive_particles_mask.clone(),
            cell_particle_counts: cell_particle_counts.clone(),
            interface_band_mask: interface_band_mask.clone(),
            cell_offsets: cell_offsets.clone(),
        };

        let remove_particles = RemoveParticlesResource {
            alive_particles_mask: alive_particles_mask.clone(),
            alive_particles_mask_scan: alive_particles_mask_scan.clone(),
            sorted_particles: sorted_particles.clone(),
            particles: particles.clone(),
        };

        let prefix_sum_alive_particles = PrefixSumAliveParticlesResource {
            alive_particles_mask: alive_particles_mask.clone(),
            alive_particles_mask_scan: alive_particles_mask_scan.clone(),
            sums: sums.clone(),
        };

        let update_particles_count = UpdateParticlesCountResource {
            alive_particles_mask: alive_particles_mask.clone(),
            alive_particles_mask_scan: alive_particles_mask_scan.clone(),
            particle_count: particle_count.clone(),
        };

        let add_particles = AddParticlesResource {
            particles: particles.clone(),
            particle_count: particle_count.clone(),
            cell_particle_counts: cell_particle_counts.clone(),
            interface_band_mask: interface_band_mask.clone(),
            levelset_air: levelset_air.clone(),
            grad_levelset_air: grad_levelset_air.clone(),
        };

        Self {
            mark_as_remove,
            remove_particles,
            prefix_sum_alive_particles,
            update_particles_count,
            add_particles,
        }
    }
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
    let alive_particles_mask = buffers.add(ShaderStorageBuffer::from(vec![0u32; size_grid]));
    let alive_particles_mask_scan = buffers.add(ShaderStorageBuffer::from(vec![0u32; size_grid]));
    let sums = buffers.add(ShaderStorageBuffer::from(vec![
        0u32;
        size_grid
            / PREFIX_SUM_BLOCK_SIZE
    ]));

    (alive_particles_mask, alive_particles_mask_scan, sums)
}

#[derive(Resource)]
pub(crate) struct ReseedParticlesPipelines {
    mark_as_remove: MarkAsRemovePipeline,
    prefix_sum_alive_particles: PrefixSumAliveParticlesPipeline,
    remove_particles: RemoveParticlesPipeline,
    update_particles_count: UpdateParticlesCountPipeline,
    add_particles: AddParticlesPipeline,
}

struct MarkAsRemovePipeline {
    pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayout,
}

struct PrefixSumAliveParticlesPipeline {
    prefix_sum_block_pipeline: CachedComputePipelineId,
    prefix_sum_local_scans_pipeline: CachedComputePipelineId,
    add_scanned_block_sums_pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayout,
}

struct RemoveParticlesPipeline {
    pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayout,
}
struct UpdateParticlesCountPipeline {
    pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayout,
}
struct AddParticlesPipeline {
    pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayout,
}

impl Pipeline for ReseedParticlesPipelines {
    fn is_pipeline_state_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        Self::is_pipeline_loaded(pipeline_cache, self.mark_as_remove.pipeline)
            && Self::is_pipeline_loaded(
                pipeline_cache,
                self.prefix_sum_alive_particles.prefix_sum_block_pipeline,
            )
            && Self::is_pipeline_loaded(
                pipeline_cache,
                self.prefix_sum_alive_particles
                    .prefix_sum_local_scans_pipeline,
            )
            && Self::is_pipeline_loaded(
                pipeline_cache,
                self.prefix_sum_alive_particles
                    .add_scanned_block_sums_pipeline,
            )
            && Self::is_pipeline_loaded(pipeline_cache, self.remove_particles.pipeline)
            && Self::is_pipeline_loaded(pipeline_cache, self.update_particles_count.pipeline)
            && Self::is_pipeline_loaded(pipeline_cache, self.add_particles.pipeline)
    }
}

impl FromWorld for ReseedParticlesPipelines {
    fn from_world(world: &mut World) -> Self {
        let mark_as_remove = MarkAsRemovePipeline::from_world(world);
        let prefix_sum_alive_particles = PrefixSumAliveParticlesPipeline::from_world(world);
        let remove_particles = RemoveParticlesPipeline::from_world(world);
        let update_particles_count = UpdateParticlesCountPipeline::from_world(world);
        let add_particles = AddParticlesPipeline::from_world(world);

        Self {
            mark_as_remove,
            prefix_sum_alive_particles,
            remove_particles,
            update_particles_count,
            add_particles,
        }
    }
}

impl FromWorld for MarkAsRemovePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout = MarkAsRemoveResource::bind_group_layout(render_device);

        let pipeline = create_pipeline(
            world,
            "MarkAsRemovePipeline",
            "shaders/reseed_particles/mark_as_remove.wgsl",
            "mark_as_remove",
            vec![bind_group_layout.clone()],
        );

        Self {
            pipeline,
            bind_group_layout,
        }
    }
}

impl FromWorld for PrefixSumAliveParticlesPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout = PrefixSumAliveParticlesResource::bind_group_layout(render_device);

        let prefix_sum_block_pipeline = create_pipeline(
            world,
            "PrefixSumBlockPipeline",
            "shaders/reseed_particles/prefix_sum_alive_particles.wgsl",
            "prefix_sum_per_workgroup",
            vec![bind_group_layout.clone()],
        );

        let prefix_sum_local_scans_pipeline = create_pipeline(
            world,
            "PrefixSumBlockPipeline",
            "shaders/reseed_particles/prefix_sum_alive_particles.wgsl",
            "prefix_sum_local_scans",
            vec![bind_group_layout.clone()],
        );

        let add_scanned_block_sums_pipeline = create_pipeline(
            world,
            "PrefixSumBlockPipeline",
            "shaders/reseed_particles/prefix_sum_alive_particles.wgsl",
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
}

impl FromWorld for RemoveParticlesPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout = RemoveParticlesResource::bind_group_layout(render_device);

        let pipeline = create_pipeline(
            world,
            "RemoveParticlesPipeline",
            "shaders/reseed_particles/remove_particles.wgsl",
            "remove_particles",
            vec![bind_group_layout.clone()],
        );

        Self {
            pipeline,
            bind_group_layout,
        }
    }
}

impl FromWorld for UpdateParticlesCountPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout = UpdateParticlesCountResource::bind_group_layout(render_device);

        let pipeline = create_pipeline(
            world,
            "UpdateParticlesCountPipeline",
            "shaders/reseed_particles/update_particles_count.wgsl",
            "update_particles_count",
            vec![bind_group_layout.clone()],
        );

        Self {
            pipeline,
            bind_group_layout,
        }
    }
}

impl FromWorld for AddParticlesPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout = AddParticlesResource::bind_group_layout(render_device);

        let pipeline = create_pipeline(
            world,
            "AddParticlesPipeline",
            "shaders/reseed_particles/add_particles.wgsl",
            "add_particles",
            vec![bind_group_layout.clone()],
        );

        Self {
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

fn reset_buffers(
    query: Query<(&PrefixSumAliveParticlesResource, &FluidSettings)>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    for (resource, settings) in &query {
        let grid_size = settings.size.element_product() as usize;

        let alive_particles_mask = buffers.get_mut(&resource.alive_particles_mask).unwrap();
        alive_particles_mask.set_data(vec![0u32; grid_size]);

        let alive_particles_mask_scan = buffers
            .get_mut(&resource.alive_particles_mask_scan)
            .unwrap();
        alive_particles_mask_scan.set_data(vec![0u32; grid_size]);

        let sums = buffers.get_mut(&resource.sums).unwrap();
        sums.set_data(vec![0u32; grid_size / PREFIX_SUM_BLOCK_SIZE]);
    }
}

fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipelines: Res<ReseedParticlesPipelines>,
    query: Query<(
        Entity,
        &MarkAsRemoveResource,
        &PrefixSumAliveParticlesResource,
        &RemoveParticlesResource,
        &UpdateParticlesCountResource,
        &AddParticlesResource,
    )>,
    render_device: Res<RenderDevice>,
    mut param: (
        Res<'a, RenderAssets<GpuImage>>,
        Res<'a, FallbackImage>,
        Res<'a, RenderAssets<GpuShaderStorageBuffer>>,
    ),
) {
    for q in &query {
        let mark_as_remove_bind_group =
            q.1.as_bind_group(
                &pipelines.mark_as_remove.bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;
        let prefix_sum_alive_particles_bind_group =
            q.2.as_bind_group(
                &pipelines.prefix_sum_alive_particles.bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;
        let remove_particles_bind_group =
            q.3.as_bind_group(
                &pipelines.remove_particles.bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;
        let update_particles_count_bind_group =
            q.4.as_bind_group(
                &pipelines.update_particles_count.bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;
        let add_particles_bind_group =
            q.5.as_bind_group(
                &pipelines.add_particles.bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        commands.entity(q.0).insert(ReseedParticlesBindGroups {
            mark_as_remove_bind_group,
            prefix_sum_alive_particles_bind_group,
            remove_particles_bind_group,
            update_particles_count_bind_group,
            add_particles_bind_group,
        });
    }
}

pub(crate) fn dispatch(
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    bind_groups: &ReseedParticlesBindGroups,
    pipelines: &ReseedParticlesPipelines,
    size: UVec2,
) {
    pass.push_debug_group("Reseed particles");
    let mark_as_remove_pipeline = pipeline_cache
        .get_compute_pipeline(pipelines.mark_as_remove.pipeline)
        .unwrap();
    pass.set_pipeline(&mark_as_remove_pipeline);
    pass.set_bind_group(0, &bind_groups.mark_as_remove_bind_group, &[]);
    pass.dispatch_center(size);

    pass.push_debug_group("Prefix-sum alive particles");
    {
        let prefix_sum_block_pipeline = pipeline_cache
            .get_compute_pipeline(
                pipelines
                    .prefix_sum_alive_particles
                    .prefix_sum_block_pipeline,
            )
            .unwrap();
        pass.set_pipeline(&prefix_sum_block_pipeline);
        pass.set_bind_group(0, &bind_groups.prefix_sum_alive_particles_bind_group, &[]);
        pass.dispatch_workgroups(size.element_product() / PREFIX_SUM_BLOCK_SIZE as u32, 1, 1);

        let prefix_sum_local_scans_pipeline = pipeline_cache
            .get_compute_pipeline(
                pipelines
                    .prefix_sum_alive_particles
                    .prefix_sum_local_scans_pipeline,
            )
            .unwrap();
        pass.set_pipeline(&prefix_sum_local_scans_pipeline);
        pass.dispatch_workgroups(1, 1, 1);

        let add_scanned_block_sums_pipeline = pipeline_cache
            .get_compute_pipeline(
                pipelines
                    .prefix_sum_alive_particles
                    .add_scanned_block_sums_pipeline,
            )
            .unwrap();
        pass.set_pipeline(&add_scanned_block_sums_pipeline);
        pass.dispatch_workgroups(size.element_product() / PREFIX_SUM_BLOCK_SIZE as u32, 1, 1);
    }
    pass.pop_debug_group();

    let remove_particles_pipeline = pipeline_cache
        .get_compute_pipeline(pipelines.remove_particles.pipeline)
        .unwrap();
    pass.set_pipeline(&remove_particles_pipeline);
    pass.set_bind_group(0, &bind_groups.remove_particles_bind_group, &[]);
    pass.dispatch_workgroups(size.element_product() / 256, 1, 1);

    let update_particles_count_pipeline = pipeline_cache
        .get_compute_pipeline(pipelines.update_particles_count.pipeline)
        .unwrap();
    pass.set_pipeline(&update_particles_count_pipeline);
    pass.set_bind_group(0, &bind_groups.update_particles_count_bind_group, &[]);
    pass.dispatch_workgroups(1, 1, 1);

    let add_particles_pipeline = pipeline_cache
        .get_compute_pipeline(pipelines.add_particles.pipeline)
        .unwrap();
    pass.set_pipeline(&add_particles_pipeline);
    pass.set_bind_group(0, &bind_groups.add_particles_bind_group, &[]);
    pass.dispatch_center(size);

    pass.pop_debug_group();
}
