pub mod fast_iterative_method;
pub mod jump_flooding;

use bevy::{
    ecs::query::QueryData,
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_resource::{ComputePass, PipelineCache, TextureFormat},
    },
};

use crate::{
    pipeline::{is_pipeline_loaded, DispatchFluidPass},
    plugin::FluidComputePassPlugin,
    reinitialize_levelset::{
        fast_iterative_method::{
            FastIterativeInitializeActiveLabelBindGroup, FastIterativeInitializeActiveLabelPass,
            FastIterativeInitializeActiveLabelPipeline, FastIterativeInitializeActiveLabelResource,
            FastIterativeInitializeBindGroup, FastIterativeInitializePass,
            FastIterativeInitializePipeline, FastIterativeInitializeResource,
            FastIterativeMethodConfig, FastIterativeUpdateBindGroup, FastIterativeUpdatePass,
            FastIterativeUpdatePipeline, FastIterativeUpdateResource,
        },
        jump_flooding::{
            JumpFloodingBindGroups, JumpFloodingCalculateSdfResource,
            JumpFloodingInitializeSeedsResource, JumpFloodingPipeline, JumpFloodingPlugin,
            JumpFloodingSeedsTextures,
        },
    },
    texture::NewTexture,
};

#[derive(Component, ExtractComponent, Clone, Debug)]
pub enum ReinitializeMethod {
    JumpFlooding,
    FastIterative(FastIterativeMethodConfig),
}

impl Default for ReinitializeMethod {
    fn default() -> Self {
        ReinitializeMethod::FastIterative(FastIterativeMethodConfig::default())
    }
}

#[derive(QueryData)]
pub(crate) struct ReinitializeLevelSetBindGroupQuery {
    pub jump_flooding_bind_groups: Option<&'static JumpFloodingBindGroups>,
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
            JumpFloodingPlugin,
            ExtractComponentPlugin::<ReinitializeMethod>::default(),
            FluidComputePassPlugin::<FastIterativeInitializePass>::default(),
            FluidComputePassPlugin::<FastIterativeInitializeActiveLabelPass>::default(),
            FluidComputePassPlugin::<FastIterativeUpdatePass>::default(),
        ));
    }
}

pub(crate) fn is_pipeline_ready(world: &World, pipeline_cache: &PipelineCache) -> bool {
    let pipeline = world.resource::<JumpFloodingPipeline>();
    is_pipeline_loaded(pipeline_cache, pipeline.init_seeds_pipeline)
        && is_pipeline_loaded(pipeline_cache, pipeline.iterate_pipeline)
        && is_pipeline_loaded(pipeline_cache, pipeline.sdf_pipeline)
}

pub(crate) fn setup(
    commands: &mut Commands,
    entity: Entity,
    images: &mut ResMut<Assets<Image>>,
    grid_size: UVec2,
    levelset_air0: &Handle<Image>,
    levelset_air1: &Handle<Image>,
    method: &ReinitializeMethod,
) {
    match method {
        ReinitializeMethod::JumpFlooding => {
            let jump_flooding_seeds0 =
                images.new_texture_storage(grid_size, TextureFormat::Rg32Float);
            let jump_flooding_seeds1 =
                images.new_texture_storage(grid_size, TextureFormat::Rg32Float);

            let reinit_levelset_initialize_seeds_resource = JumpFloodingInitializeSeedsResource {
                levelset_air1: levelset_air1.clone(),
            };

            let reinit_levelset_calculate_sdf_resource = JumpFloodingCalculateSdfResource {
                levelset_air0: levelset_air0.clone(),
                levelset_air1: levelset_air1.clone(),
            };

            let reinit_levelset_seeds_textures =
                JumpFloodingSeedsTextures([jump_flooding_seeds0, jump_flooding_seeds1]);

            commands.entity(entity).insert((
                reinit_levelset_initialize_seeds_resource,
                reinit_levelset_calculate_sdf_resource,
                reinit_levelset_seeds_textures,
            ));
        }
        ReinitializeMethod::FastIterative(_config) => {
            let labels0 = images.new_texture_storage(grid_size, TextureFormat::R32Uint);
            let labels = images.new_texture_storage(grid_size, TextureFormat::R32Uint);

            let init_textures =
                FastIterativeInitializeResource::new(levelset_air1, levelset_air0, &labels0);

            let init_active_label_textures =
                FastIterativeInitializeActiveLabelResource::new(&labels0, &labels);

            let update_textures = FastIterativeUpdateResource::new(levelset_air0, &labels);

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
    size: UVec2,
) {
    match method {
        ReinitializeMethod::JumpFlooding => {
            pass.push_debug_group("Reinitialize levelset (JFA)");

            let pipeline = world.resource::<JumpFloodingPipeline>();
            let bind_groups = bind_groups.jump_flooding_bind_groups.unwrap();
            let init_seeds_pipeline = pipeline_cache
                .get_compute_pipeline(pipeline.init_seeds_pipeline)
                .unwrap();
            let iterate_pipeline = pipeline_cache
                .get_compute_pipeline(pipeline.iterate_pipeline)
                .unwrap();
            let sdf_pipeline = pipeline_cache
                .get_compute_pipeline(pipeline.sdf_pipeline)
                .unwrap();

            pass.set_pipeline(init_seeds_pipeline);
            pass.set_bind_group(0, &bind_groups.init_seeds_bind_group, &[]);
            pass.set_bind_group(1, &bind_groups.write_only_seeds_bind_groups[0], &[]);
            pass.dispatch_center(size);

            pass.set_pipeline(&iterate_pipeline);
            let mut src_idx = 0;
            let mut dst_idx = 1;

            for bind_group in &bind_groups.jump_flooding_step_bind_groups {
                pass.set_bind_group(0, &bind_groups.read_only_seeds_bind_groups[src_idx], &[]);
                pass.set_bind_group(1, &bind_groups.write_only_seeds_bind_groups[dst_idx], &[]);
                pass.set_bind_group(2, bind_group, &[]);
                pass.dispatch_center(size);

                std::mem::swap(&mut src_idx, &mut dst_idx);
            }

            pass.set_pipeline(&sdf_pipeline);
            pass.set_bind_group(0, &bind_groups.sdf_bind_group, &[]);
            pass.set_bind_group(1, &bind_groups.read_only_seeds_bind_groups[src_idx], &[]);

            pass.dispatch_center(size);
            pass.pop_debug_group();
        }
        ReinitializeMethod::FastIterative(config) => {
            pass.push_debug_group("Reinitialize levelset (FIM)");
            let num_workgroups_grid = (size / 8).extend(1);
            let initialize_pipeline = world.resource::<FastIterativeInitializePipeline>();
            initialize_pipeline.pipeline.dispatch(
                pipeline_cache,
                pass,
                &bind_groups.fast_iterative_bind_groups.unwrap().0.bind_group,
                num_workgroups_grid,
            );

            let initialize_active_label =
                world.resource::<FastIterativeInitializeActiveLabelPipeline>();
            initialize_active_label.pipeline.dispatch(
                pipeline_cache,
                pass,
                &bind_groups.fast_iterative_bind_groups.unwrap().1.bind_group,
                num_workgroups_grid,
            );

            let update_pipeline = world.resource::<FastIterativeUpdatePipeline>();
            for _ in 0..config.num_iterations {
                update_pipeline.pipeline.dispatch(
                    pipeline_cache,
                    pass,
                    &bind_groups.fast_iterative_bind_groups.unwrap().2.bind_group,
                    num_workgroups_grid,
                );
            }
            pass.pop_debug_group();
        }
    }
}
