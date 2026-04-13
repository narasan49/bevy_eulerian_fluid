use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::RenderAssets,
        render_resource::{
            binding_types::{texture_storage_2d, uniform_buffer},
            BindGroup, BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries,
            CachedComputePipelineId, ComputePass, ComputePipeline, ComputePipelineDescriptor,
            PipelineCache, ShaderStages, StorageTextureAccess, TextureFormat, UniformBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
        Render, RenderApp, RenderSystems,
    },
};

use crate::{
    fluid_uniform::{uniform_bind_group_layout_desc, SimulationUniformBindGroup},
    pipeline::{is_pipeline_loaded, WORKGROUP_SIZE},
    projection::gauss_seidel::{GaussSeidelConfig, GaussSeidelPipeline},
    texture::NewTexture,
};

pub(crate) struct MultiGridPassPlugin;

impl Plugin for MultiGridPassPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/residual.wgsl");
        embedded_asset!(app, "shaders/restriction.wgsl");
        embedded_asset!(app, "shaders/prolongation.wgsl");
        app.add_plugins((
            ExtractComponentPlugin::<MultiGridResources>::default(),
            ExtractComponentPlugin::<MultiGridNumLevels>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(
            Render,
            prepare_bind_groups.in_set(RenderSystems::PrepareBindGroups),
        );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.init_resource::<MultiGridPipelines>();
    }
}

#[derive(Clone, Debug)]
pub struct MultiGridConfig {
    pub pre_smooth_config: GaussSeidelConfig,
    pub post_smooth_config: GaussSeidelConfig,
    pub coarsest_config: GaussSeidelConfig,
}

impl Default for MultiGridConfig {
    fn default() -> Self {
        Self {
            pre_smooth_config: GaussSeidelConfig { num_iterations: 2 },
            post_smooth_config: GaussSeidelConfig { num_iterations: 2 },
            coarsest_config: GaussSeidelConfig { num_iterations: 3 },
        }
    }
}

#[derive(Component, ExtractComponent, Clone)]
pub(crate) struct MultiGridNumLevels(usize);

#[derive(Component, ExtractComponent, Clone)]
pub(crate) struct MultiGridResources {
    x: Vec<Handle<Image>>,
    b: Vec<Handle<Image>>,
    levelset: Vec<Handle<Image>>,
    area_fraction_solid: Vec<Handle<Image>>,
    r: Vec<Handle<Image>>,
}

pub(crate) fn setup_multigrid_resources(
    commands: &mut Commands,
    entity: Entity,
    grid_size: UVec2,
    div: &Handle<Image>,
    p: &Handle<Image>,
    levelset_air: &Handle<Image>,
    area_fraction_solid: &Handle<Image>,
    images: &mut ResMut<Assets<Image>>,
) {
    let num_levels = ((grid_size.min_element() as f32).log2() as usize).saturating_sub(2);

    let mut x = Vec::<Handle<Image>>::with_capacity(num_levels);
    let mut b = Vec::<Handle<Image>>::with_capacity(num_levels);
    let mut levelset = Vec::<Handle<Image>>::with_capacity(num_levels);
    let mut area_fraction_solids = Vec::<Handle<Image>>::with_capacity(num_levels);
    let mut r = Vec::<Handle<Image>>::with_capacity(num_levels);
    x.push(p.clone());
    b.push(div.clone());
    levelset.push(levelset_air.clone());
    area_fraction_solids.push(area_fraction_solid.clone());
    r.push(images.new_texture_storage(grid_size, TextureFormat::R32Float));
    let mut grid_size = grid_size;
    for _ in 1..num_levels {
        grid_size /= 2;
        x.push(images.new_texture_storage(grid_size, TextureFormat::R32Float));
        b.push(images.new_texture_storage(grid_size, TextureFormat::R32Float));
        levelset.push(images.new_texture_storage(grid_size, TextureFormat::R32Float));
        area_fraction_solids
            .push(images.new_texture_storage(grid_size, TextureFormat::Rgba32Float));
        r.push(images.new_texture_storage(grid_size, TextureFormat::R32Float));
    }

    let resources = MultiGridResources {
        x,
        b,
        levelset,
        area_fraction_solid: area_fraction_solids,
        r,
    };

    commands
        .entity(entity)
        .insert((resources, MultiGridNumLevels(num_levels)));
}

#[derive(Resource)]
pub(crate) struct MultiGridPipelines {
    smoother_pipeline: GaussSeidelPipeline,
    residual_pipeline: CachedComputePipelineId,
    restriction_pipeline: CachedComputePipelineId,
    prolongation_pipeline: CachedComputePipelineId,
    residual_bind_group_layout: BindGroupLayoutDescriptor,
    restriction_bind_group_layout: BindGroupLayoutDescriptor,
    prolongation_bind_group_layout: BindGroupLayoutDescriptor,
}

impl FromWorld for MultiGridPipelines {
    fn from_world(world: &mut World) -> Self {
        let smoother_pipeline = GaussSeidelPipeline::from_world(world);
        let pipeline_cache = world.resource::<PipelineCache>();

        let residual_bind_group_layout = BindGroupLayoutDescriptor::new(
            "ResidualBindGroupLayout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::ReadOnly),
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::ReadOnly),
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::ReadOnly),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadOnly),
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::WriteOnly),
                    uniform_buffer::<f32>(false),
                ),
            ),
        );

        let uniform_bind_group_layout = uniform_bind_group_layout_desc();

        let residual_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("ResidualPipeline".into()),
            layout: vec![
                residual_bind_group_layout.clone(),
                uniform_bind_group_layout,
            ],
            shader: load_embedded_asset!(world, "shaders/residual.wgsl"),
            entry_point: Some("residual".into()),
            ..default()
        });

        let restriction_bind_group_layout = BindGroupLayoutDescriptor::new(
            "RestrictionBindGroupLayout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::ReadOnly),
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::ReadOnly),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadOnly),
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::WriteOnly),
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::WriteOnly),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::WriteOnly),
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::WriteOnly),
                ),
            ),
        );

        let restriction_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("RestrictionPipeline".into()),
                layout: vec![restriction_bind_group_layout.clone()],
                shader: load_embedded_asset!(world, "shaders/restriction.wgsl"),
                entry_point: Some("restriction".into()),
                ..default()
            });

        let prolongation_bind_group_layout = BindGroupLayoutDescriptor::new(
            "ProlongationBindGroupLayout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::ReadWrite),
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::ReadOnly),
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::ReadOnly),
                ),
            ),
        );

        let prolongation_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("ProlongationPipeline".into()),
                layout: vec![prolongation_bind_group_layout.clone()],
                shader: load_embedded_asset!(world, "shaders/prolongation.wgsl"),
                entry_point: Some("prolongation".into()),
                ..default()
            });

        Self {
            smoother_pipeline,
            residual_pipeline,
            restriction_pipeline,
            prolongation_pipeline,
            residual_bind_group_layout,
            restriction_bind_group_layout,
            prolongation_bind_group_layout,
        }
    }
}

impl MultiGridPipelines {
    pub(crate) fn ready(&self, pipeline_cache: &PipelineCache) -> bool {
        self.smoother_pipeline.is_ready(pipeline_cache)
            && is_pipeline_loaded(pipeline_cache, self.residual_pipeline)
            && is_pipeline_loaded(pipeline_cache, self.restriction_pipeline)
            && is_pipeline_loaded(pipeline_cache, self.prolongation_pipeline)
    }
}

#[derive(Component)]
pub(crate) struct MultiGridBindGroups {
    smoother_bind_groups: Box<[BindGroup]>,
    residual_bind_groups: Box<[BindGroup]>,
    restriction_bind_groups: Box<[BindGroup]>,
    prolongation_bind_groups: Box<[BindGroup]>,
}

fn prepare_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    query: Query<(Entity, &MultiGridResources, &MultiGridNumLevels)>,
    pipelines: Res<MultiGridPipelines>,
) {
    for (entity, resources, num_levels) in &query {
        let mut smoother_bind_groups = Vec::with_capacity(num_levels.0);
        let mut residual_bind_groups = Vec::with_capacity(num_levels.0 - 1);
        let mut restriction_bind_groups = Vec::with_capacity(num_levels.0 - 1);
        let mut prolongation_bind_groups = Vec::with_capacity(num_levels.0 - 1);
        for i in 0..num_levels.0 {
            let mut resolution_scale_buffer = UniformBuffer::from((1 << i) as f32);
            resolution_scale_buffer.write_buffer(&render_device, &render_queue);
            let mut gs_weight_buffer = UniformBuffer::from(1.0);
            gs_weight_buffer.write_buffer(&render_device, &render_queue);

            let x = gpu_images.get(&resources.x[i]).unwrap();
            let b = gpu_images.get(&resources.b[i]).unwrap();
            let levelset = gpu_images.get(&resources.levelset[i]).unwrap();
            let area_fraction_solid = gpu_images.get(&resources.area_fraction_solid[i]).unwrap();
            let r = gpu_images.get(&resources.r[i]).unwrap();

            smoother_bind_groups.push(
                render_device.create_bind_group(
                    format!("SmootherBindGroup_Level{}", i).as_str(),
                    &pipeline_cache
                        .get_bind_group_layout(&pipelines.smoother_pipeline.bind_group_layout),
                    &BindGroupEntries::sequential((
                        &x.texture_view,
                        &b.texture_view,
                        &levelset.texture_view,
                        &area_fraction_solid.texture_view,
                        gs_weight_buffer.binding().unwrap(),
                        resolution_scale_buffer.binding().unwrap(),
                    )),
                ),
            );

            if i == num_levels.0 - 1 {
                continue;
            }

            residual_bind_groups.push(render_device.create_bind_group(
                format!("ResidualBindGroup_Level{i}").as_str(),
                &pipeline_cache.get_bind_group_layout(&pipelines.residual_bind_group_layout),
                &BindGroupEntries::sequential((
                    &x.texture_view,
                    &b.texture_view,
                    &levelset.texture_view,
                    &area_fraction_solid.texture_view,
                    &r.texture_view,
                    resolution_scale_buffer.binding().unwrap(),
                )),
            ));

            let x_plus = gpu_images.get(&resources.x[i + 1]).unwrap();
            let b_plus = gpu_images.get(&resources.b[i + 1]).unwrap();
            let levelset_plus = gpu_images.get(&resources.levelset[i + 1]).unwrap();
            let area_fraction_solid_plus = gpu_images
                .get(&resources.area_fraction_solid[i + 1])
                .unwrap();

            restriction_bind_groups.push(render_device.create_bind_group(
                format!("Restriction_Level{}_to_Level{}", i, i + 1).as_str(),
                &pipeline_cache.get_bind_group_layout(&pipelines.restriction_bind_group_layout),
                &BindGroupEntries::sequential((
                    &r.texture_view,
                    &levelset.texture_view,
                    &area_fraction_solid.texture_view,
                    &b_plus.texture_view,
                    &levelset_plus.texture_view,
                    &area_fraction_solid_plus.texture_view,
                    &x_plus.texture_view,
                )),
            ));

            prolongation_bind_groups.push(render_device.create_bind_group(
                format!("Prolongation_Level{}_to_Level{}", i + 1, i).as_str(),
                &pipeline_cache.get_bind_group_layout(&pipelines.prolongation_bind_group_layout),
                &BindGroupEntries::sequential((
                    &x.texture_view,
                    &x_plus.texture_view,
                    &levelset.texture_view,
                )),
            ));
        }

        commands.entity(entity).insert(MultiGridBindGroups {
            smoother_bind_groups: smoother_bind_groups.into_boxed_slice(),
            residual_bind_groups: residual_bind_groups.into_boxed_slice(),
            restriction_bind_groups: restriction_bind_groups.into_boxed_slice(),
            prolongation_bind_groups: prolongation_bind_groups.into_boxed_slice(),
        });
    }
}

impl MultiGridPipelines {
    pub(crate) fn dispatch(
        &self,
        pipeline_cache: &PipelineCache,
        pass: &mut ComputePass,
        bind_groups: &MultiGridBindGroups,
        uniform_bind_group: &SimulationUniformBindGroup,
        grid_size: UVec2,
        config: &MultiGridConfig,
        levels: &MultiGridNumLevels,
    ) {
        let residual_pipeline = pipeline_cache
            .get_compute_pipeline(self.residual_pipeline)
            .unwrap();
        let restriction_pipeline = pipeline_cache
            .get_compute_pipeline(self.restriction_pipeline)
            .unwrap();
        let prolongation_pipeline = pipeline_cache
            .get_compute_pipeline(self.prolongation_pipeline)
            .unwrap();
        v_cycle(
            0,
            pipeline_cache,
            &self.smoother_pipeline,
            residual_pipeline,
            restriction_pipeline,
            prolongation_pipeline,
            pass,
            bind_groups,
            uniform_bind_group,
            grid_size,
            config,
            levels,
        );
    }
}

fn v_cycle(
    i: usize,
    pipeline_cache: &PipelineCache,
    smoother_pipeline: &GaussSeidelPipeline,
    residual_pipeline: &ComputePipeline,
    restriction_pipeline: &ComputePipeline,
    prolongation_pipeline: &ComputePipeline,
    pass: &mut ComputePass,
    bind_groups: &MultiGridBindGroups,
    uniform_bind_group: &SimulationUniformBindGroup,
    grid_size: UVec2,
    config: &MultiGridConfig,
    levels: &MultiGridNumLevels,
) {
    pass.push_debug_group(format!("V-Cycle (Level {i})").as_str());
    let num_workgroups = ((grid_size + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE).extend(1);

    if i == levels.0 - 1 {
        pass.push_debug_group("Solve");
        smoother_pipeline.dispatch(
            pipeline_cache,
            pass,
            &bind_groups.smoother_bind_groups[i],
            uniform_bind_group,
            num_workgroups,
            &config.coarsest_config,
        );
        pass.pop_debug_group();
        pass.pop_debug_group();
        return;
    }
    pass.push_debug_group("Pre smooth");
    smoother_pipeline.dispatch(
        pipeline_cache,
        pass,
        &bind_groups.smoother_bind_groups[i],
        uniform_bind_group,
        num_workgroups,
        &config.pre_smooth_config,
    );
    pass.pop_debug_group();

    pass.set_pipeline(residual_pipeline);
    pass.set_bind_group(0, &bind_groups.residual_bind_groups[i], &[]);
    pass.set_bind_group(
        1,
        &uniform_bind_group.bind_group,
        &[uniform_bind_group.index],
    );
    pass.dispatch_workgroups(num_workgroups.x, num_workgroups.y, num_workgroups.z);

    pass.set_pipeline(restriction_pipeline);
    pass.set_bind_group(0, &bind_groups.restriction_bind_groups[i], &[]);
    pass.dispatch_workgroups(num_workgroups.x, num_workgroups.y, num_workgroups.z);

    v_cycle(
        i + 1,
        pipeline_cache,
        smoother_pipeline,
        residual_pipeline,
        restriction_pipeline,
        prolongation_pipeline,
        pass,
        bind_groups,
        uniform_bind_group,
        grid_size / 2,
        config,
        levels,
    );

    pass.set_pipeline(prolongation_pipeline);
    pass.set_bind_group(0, &bind_groups.prolongation_bind_groups[i], &[]);
    pass.dispatch_workgroups(num_workgroups.x, num_workgroups.y, num_workgroups.z);

    pass.push_debug_group("Post smooth");
    smoother_pipeline.dispatch(
        pipeline_cache,
        pass,
        &bind_groups.smoother_bind_groups[i],
        uniform_bind_group,
        num_workgroups,
        &config.post_smooth_config,
    );
    pass.pop_debug_group();
    pass.pop_debug_group();
}
