pub mod fast_iterative_method;

use std::fmt::Display;

use bevy::{
    ecs::query::QueryData,
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_resource::{ComputePass, PipelineCache, TextureFormat},
    },
};
use bevy_eulerian_fluid_common::{
    fluid_compute_pass::FluidComputePassPlugin, texture::new_texture_storage_3d,
};

use crate::{
    core::{
        reinitialize_levelset::fast_iterative_method::{
            FastIterativeInitializeActiveLabelBindGroup, FastIterativeInitializeActiveLabelPass,
            FastIterativeInitializeActiveLabelPipeline, FastIterativeInitializeActiveLabelResource,
            FastIterativeInitializeBindGroup, FastIterativeInitializePass,
            FastIterativeInitializePipeline, FastIterativeInitializeResource,
            FastIterativeMethodConfig, FastIterativeUpdateBindGroup, FastIterativeUpdatePass,
            FastIterativeUpdatePipeline, FastIterativeUpdateResource,
        },
        workgroup::{workgroup_size_center, WorkgroupShape},
    },
    resource::FluidResources,
};

#[derive(Component, ExtractComponent, Clone, Debug)]
pub enum ReinitializeMethod {
    FastIterative(FastIterativeMethodConfig),
}

impl Default for ReinitializeMethod {
    fn default() -> Self {
        ReinitializeMethod::FastIterative(FastIterativeMethodConfig::default())
    }
}

impl Display for ReinitializeMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReinitializeMethod::FastIterative(fast_iterative_method_config) => {
                write!(
                    f,
                    "Fast Iterative({})",
                    fast_iterative_method_config.num_iterations
                )
            }
        }
    }
}

#[derive(QueryData)]
pub(crate) struct ReinitializeLevelSetBindGroupQuery {
    pub fast_iterative_bind_groups: Option<(
        &'static FastIterativeInitializeBindGroup,
        &'static FastIterativeInitializeActiveLabelBindGroup,
        &'static FastIterativeUpdateBindGroup,
    )>,
}

pub(crate) struct ReinitializeLevelSetPlugin;

impl Plugin for ReinitializeLevelSetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<ReinitializeMethod>::default(),
            FluidComputePassPlugin::<FastIterativeInitializePass>::default(),
            FluidComputePassPlugin::<FastIterativeInitializeActiveLabelPass>::default(),
            FluidComputePassPlugin::<FastIterativeUpdatePass>::default(),
        ));
    }
}

pub(crate) fn is_pipeline_ready(world: &World, pipeline_cache: &PipelineCache) -> bool {
    let fim_init_pipeline = world.resource::<FastIterativeInitializePipeline>();
    let fim_init_active_label_pipeline =
        world.resource::<FastIterativeInitializeActiveLabelPipeline>();
    let fim_update_pipeline = world.resource::<FastIterativeUpdatePipeline>();

    fim_init_pipeline.pipeline.is_ready(pipeline_cache)
        && fim_init_active_label_pipeline
            .pipeline
            .is_ready(pipeline_cache)
        && fim_update_pipeline.pipeline.is_ready(pipeline_cache)
}

pub(crate) fn setup(
    commands: &mut Commands,
    entity: Entity,
    images: &mut ResMut<Assets<Image>>,
    grid_size: UVec3,
    resources: &FluidResources,
    method: &ReinitializeMethod,
) {
    match method {
        ReinitializeMethod::FastIterative(_config) => {
            let labels0 = new_texture_storage_3d(images, grid_size, TextureFormat::R32Uint);
            let labels = new_texture_storage_3d(images, grid_size, TextureFormat::R32Uint);

            let init_textures = FastIterativeInitializeResource::new(
                &resources.levelset_air1,
                &resources.levelset_air0,
                &labels0,
            );

            let init_active_label_textures =
                FastIterativeInitializeActiveLabelResource::new(&labels0, &labels);

            let update_textures =
                FastIterativeUpdateResource::new(&resources.levelset_air0, &labels);

            commands.entity(entity).insert((
                init_textures,
                init_active_label_textures,
                update_textures,
            ));
        }
    }
}

pub(crate) fn dispatch(
    world: &World,
    method: &ReinitializeMethod,
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    bind_groups: ReinitializeLevelSetBindGroupQueryItem,
    workgroup_shape: &WorkgroupShape,
    size: UVec3,
) {
    match method {
        ReinitializeMethod::FastIterative(config) => {
            pass.push_debug_group("reinitialize_levelset (FIM)");
            let num_workgroups = workgroup_size_center(size, workgroup_shape);
            let initialize_pipeline = world.resource::<FastIterativeInitializePipeline>();
            initialize_pipeline.pipeline.dispatch(
                pipeline_cache,
                pass,
                &bind_groups.fast_iterative_bind_groups.unwrap().0.bind_group,
                num_workgroups,
            );

            let initialize_active_label =
                world.resource::<FastIterativeInitializeActiveLabelPipeline>();
            initialize_active_label.pipeline.dispatch(
                pipeline_cache,
                pass,
                &bind_groups.fast_iterative_bind_groups.unwrap().1.bind_group,
                num_workgroups,
            );

            let update_pipeline = world.resource::<FastIterativeUpdatePipeline>();
            for _ in 0..config.num_iterations {
                update_pipeline.pipeline.dispatch(
                    pipeline_cache,
                    pass,
                    &bind_groups.fast_iterative_bind_groups.unwrap().2.bind_group,
                    num_workgroups,
                );
            }
            pass.pop_debug_group();
        }
    }
}
