use bevy::{
    asset::{embedded_asset, embedded_path},
    ecs::{schedule::ScheduleConfigs, system::ScheduleSystem},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, BindGroup, BindGroupLayout, CachedComputePipelineId, ComputePass,
            PipelineCache,
        },
        renderer::RenderDevice,
        storage::GpuShaderStorageBuffer,
        texture::{FallbackImage, GpuImage},
    },
};

use crate::{
    fluid_uniform::{create_uniform_bind_group_layout, SimulationUniformBindGroup},
    pipeline::{is_pipeline_loaded, queue_compute_pipeline},
    plugin::FluidComputePass,
};

pub(crate) struct GaussSeidelPass;

impl FluidComputePass for GaussSeidelPass {
    type P = GaussSeidelPipeline;

    type Resource = GaussSeidelResource;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/gauss_seidel.wgsl");
    }

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
        prepare_bind_groups.into_configs()
    }
}

#[derive(Clone, Debug)]
pub struct GaussSeidelConfig {
    pub num_iterations: u32,
}

impl Default for GaussSeidelConfig {
    fn default() -> Self {
        Self { num_iterations: 20 }
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct GaussSeidelResource {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    p: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadOnly)]
    div: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadOnly)]
    levelset_air: Handle<Image>,
    #[storage_texture(3, image_format = Rgba32Float, access = ReadOnly)]
    area_fraction_solid: Handle<Image>,
}

impl GaussSeidelResource {
    pub fn new(
        p: &Handle<Image>,
        div: &Handle<Image>,
        levelset_air: &Handle<Image>,
        area_fraction_solid: &Handle<Image>,
    ) -> Self {
        Self {
            p: p.clone(),
            div: div.clone(),
            levelset_air: levelset_air.clone(),
            area_fraction_solid: area_fraction_solid.clone(),
        }
    }
}

#[derive(Resource)]
pub(crate) struct GaussSeidelPipeline {
    pipelines: [CachedComputePipelineId; 2],
    bind_group_layout: BindGroupLayout,
}

impl FromWorld for GaussSeidelPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let bind_group_layout = GaussSeidelResource::bind_group_layout(render_device);
        let uniform_bind_group_layout = create_uniform_bind_group_layout(render_device);

        let pipeline_red = queue_compute_pipeline(
            world,
            "GaussSeidelRedPipeline",
            embedded_path!("shaders/gauss_seidel.wgsl"),
            "gauss_seidel_red",
            vec![bind_group_layout.clone(), uniform_bind_group_layout.clone()],
        );

        let pipeline_black = queue_compute_pipeline(
            world,
            "GaussSeidelBlackPipeline",
            embedded_path!("shaders/gauss_seidel.wgsl"),
            "gauss_seidel_black",
            vec![bind_group_layout.clone(), uniform_bind_group_layout.clone()],
        );

        Self {
            pipelines: [pipeline_red, pipeline_black],
            bind_group_layout,
        }
    }
}

impl GaussSeidelPipeline {
    pub fn is_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        is_pipeline_loaded(pipeline_cache, self.pipelines[0])
            && is_pipeline_loaded(pipeline_cache, self.pipelines[1])
    }

    pub fn dispatch(
        &self,
        pipeline_cache: &PipelineCache,
        pass: &mut ComputePass,
        bind_group: &BindGroup,
        uniform_bind_group: &SimulationUniformBindGroup,
        num_workgroups: UVec3,
        config: &GaussSeidelConfig,
    ) {
        let pipeline_red = pipeline_cache
            .get_compute_pipeline(self.pipelines[0])
            .unwrap();
        let pipeline_black = pipeline_cache
            .get_compute_pipeline(self.pipelines[1])
            .unwrap();

        pass.set_bind_group(0, bind_group, &[]);
        pass.set_bind_group(
            1,
            &uniform_bind_group.bind_group,
            &[uniform_bind_group.index],
        );
        for _ in 0..config.num_iterations {
            pass.set_pipeline(pipeline_red);
            pass.dispatch_workgroups(num_workgroups.x, num_workgroups.y, num_workgroups.z);
            pass.set_pipeline(pipeline_black);
            pass.dispatch_workgroups(num_workgroups.x, num_workgroups.y, num_workgroups.z);
        }
    }
}

#[derive(Component)]
pub(crate) struct GaussSeidelBindGroup {
    pub bind_group: BindGroup,
}

pub(super) fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipeline: Res<GaussSeidelPipeline>,
    query: Query<(Entity, &GaussSeidelResource)>,
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
            .insert(GaussSeidelBindGroup { bind_group });
    }
}
