/// GPU implementation of Particle Level Set Method by Enright 2002.
use bevy::{
    ecs::query::QueryData,
    prelude::*,
    render::{
        render_resource::{ComputePass, PipelineCache, TextureFormat},
        storage::ShaderStorageBuffer,
    },
    shader::load_shader_library,
};

use crate::{
    common_pass::prefix_sum::PREFIX_SUM_BLOCK_SIZE,
    fluid_uniform::SimulationUniformBindGroup,
    particle_levelset::debug_draw_particles::DebugDrawLevelsetParticlesPlugin,
    particle_levelset_two_layers::{
        advect_particles::{
            AdvectParticlesBindGroup, AdvectParticlesPass, AdvectParticlesPipeline,
            AdvectParticlesResource,
        },
        initialize_particles::{
            InitializeParticlesBindGroup, InitializeParticlesPass, InitializeParticlesPipeline,
            InitializeParticlesResource,
        },
        levelset_correction::{
            accumulate_phi_correction::{
                AccumulateLevelSetCorrectionMinusResource, AccumulateLevelSetCorrectionPlusResource,
            },
            accumulate_phi_correction_second::{
                AccumulateLevelSetCorrectionMinusSecondResource,
                AccumulateLevelSetCorrectionPlusSecondResource,
            },
            correct_levelset::CorrectLevelSetResource,
            correct_levelset_second::CorrectLevelSetSecondResource,
            mark_escaped_particles::MarkEscapedParticlesResource,
            mark_escaped_particles_second::MarkEscapedParticlesSecondResource,
            reset_levelset_correction::ResetLevelSetCorrectionResource,
            reset_levelset_correction_second::ResetLevelSetCorrectionSecondResource,
            LevelsetCorrectionPlugin,
        },
        particle::{Particle, MAX_PARTICLES_PER_CELL},
        reseed::{
            add_particles::{AddNegativeParticlesResource, AddPositiveParticlesResource},
            count_particles_in_cell::{
                CountNegativeParticlesInCellResource, CountPositiveParticlesInCellResource,
            },
            delete_particles::{DeleteNegativeParticlesResource, DeletePositiveParticlesResource},
            prefix_sum_alive_particles::{
                PrefixSumAliveNegativeParticlesResource, PrefixSumAlivePositiveParticlesResource,
            },
            prefix_sum_particle_counts::{
                PrefixSumNegativeParticlesCountResource, PrefixSumPositiveParticlesCountResource,
            },
            reseed_particles::{ReseedNegativeParticlesResource, ReseedPositiveParticlesResource},
            sort_particles::{SortNegativeParticlesResource, SortPositiveParticlesResource},
            update_particles_count::{
                UpdateNegativeParticlesCountResource, UpdatePositiveParticlesCountResource,
            },
            ReseedPlugin,
        },
        update_interface_band_mask::{
            UpdateInterfaceBandMaskBindGroup, UpdateInterfaceBandMaskPass,
            UpdateInterfaceBandMaskPipeline, UpdateInterfaceBandMaskResource,
        },
    },
    plugin::FluidComputePassPlugin,
    settings::FluidSettings,
    texture::NewTexture,
};

pub(crate) struct ParticleLevelsetTwoLayersPlugin;

impl Plugin for ParticleLevelsetTwoLayersPlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "shaders/particle.wgsl");
        load_shader_library!(app, "shaders/constants.wgsl");
        load_shader_library!(app, "shaders/fixed_point.wgsl");

        app.add_plugins((
            LevelsetCorrectionPlugin,
            ReseedPlugin,
            DebugDrawLevelsetParticlesPlugin,
        ))
        .add_plugins((
            FluidComputePassPlugin::<UpdateInterfaceBandMaskPass>::default(),
            FluidComputePassPlugin::<InitializeParticlesPass>::default(),
            FluidComputePassPlugin::<AdvectParticlesPass>::default(),
        ))
        .add_systems(Update, reset_buffers);
    }
}

#[derive(QueryData)]
pub(crate) struct PLSInitializeBindGroupsQuery {
    pub update_interface_mask_bind_group: &'static UpdateInterfaceBandMaskBindGroup,
    pub initialize_particles_bind_group: &'static InitializeParticlesBindGroup,
}

#[derive(QueryData)]
pub(crate) struct PLSAdvectionBindGroupsQuery {
    pub advect_particles_bind_group: &'static AdvectParticlesBindGroup,
}

pub(crate) fn dispatch_initialize(
    world: &World,
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    particle_bind_groups: PLSInitializeBindGroupsQueryItem,
    grid_size: UVec2,
) {
    let num_workgroups_grid = (grid_size / 8).extend(1);

    let update_interface_band_mask_pipeline = world.resource::<UpdateInterfaceBandMaskPipeline>();
    update_interface_band_mask_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &particle_bind_groups
            .update_interface_mask_bind_group
            .bind_group,
        num_workgroups_grid,
    );

    let initialize_particles_pipeline = world.resource::<InitializeParticlesPipeline>();
    initialize_particles_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &particle_bind_groups
            .initialize_particles_bind_group
            .bind_group,
        num_workgroups_grid,
    );
}

pub(crate) fn dispatch_update(
    world: &World,
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    particle_bind_groups: PLSAdvectionBindGroupsQueryItem,
    uniform_bind_group: &SimulationUniformBindGroup,
    grid_size: UVec2,
) {
    let num_workgroups_particle = UVec3::new(
        grid_size.element_product() * MAX_PARTICLES_PER_CELL as u32 / 256,
        1,
        1,
    );

    let advect_particles_pipeline = world.resource::<AdvectParticlesPipeline>();
    advect_particles_pipeline.pipeline.dispatch_with_uniform(
        pipeline_cache,
        pass,
        &particle_bind_groups.advect_particles_bind_group.bind_group,
        uniform_bind_group,
        num_workgroups_particle,
    );
}

#[derive(Component)]
pub(crate) struct PLSResources {
    pub positive_particles: Handle<ShaderStorageBuffer>,
    pub positive_particles_count: Handle<ShaderStorageBuffer>,
    pub negative_particles: Handle<ShaderStorageBuffer>,
    pub negative_particles_count: Handle<ShaderStorageBuffer>,
    pub interface_band_mask: Handle<Image>,
    pub phi_plus: Handle<ShaderStorageBuffer>,
    pub phi_minus: Handle<ShaderStorageBuffer>,
    pub num_positive_particles_in_cell: Handle<ShaderStorageBuffer>,
    pub positive_cell_offsets: Handle<ShaderStorageBuffer>,
    pub num_positive_particles_block_sums: Handle<ShaderStorageBuffer>,
    pub num_negative_particles_in_cell: Handle<ShaderStorageBuffer>,
    pub negative_cell_offsets: Handle<ShaderStorageBuffer>,
    pub num_negative_particles_block_sums: Handle<ShaderStorageBuffer>,
    pub positive_alive_particles_mask: Handle<ShaderStorageBuffer>,
    pub positive_alive_particles_mask_scan: Handle<ShaderStorageBuffer>,
    pub positive_alive_particles_mask_block_sums: Handle<ShaderStorageBuffer>,
    pub sorted_positive_particles: Handle<ShaderStorageBuffer>,
    pub positive_cell_cursor: Handle<ShaderStorageBuffer>,
    pub negative_alive_particles_mask: Handle<ShaderStorageBuffer>,
    pub negative_alive_particles_mask_scan: Handle<ShaderStorageBuffer>,
    pub negative_alive_particles_mask_block_sums: Handle<ShaderStorageBuffer>,
    pub sorted_negative_particles: Handle<ShaderStorageBuffer>,
    pub negative_cell_cursor: Handle<ShaderStorageBuffer>,
    pub positive_particles_to_be_added: Handle<ShaderStorageBuffer>,
    pub negative_particles_to_be_added: Handle<ShaderStorageBuffer>,
}

impl PLSResources {
    pub fn new(
        images: &mut ResMut<Assets<Image>>,
        buffers: &mut ResMut<Assets<ShaderStorageBuffer>>,
        grid_size: UVec2,
    ) -> Self {
        let grid_length = grid_size.element_product() as usize;
        let particle_buffer_length = grid_length * MAX_PARTICLES_PER_CELL;

        let particles_data =
            ShaderStorageBuffer::from(vec![Particle::ZERO; particle_buffer_length]);
        let single_data = ShaderStorageBuffer::from(0u32);
        let grid_data = ShaderStorageBuffer::from(vec![0.0; grid_length]);
        let grid_data_u32 = ShaderStorageBuffer::from(vec![0u32; grid_length]);
        let prefix_sum_intermediate =
            ShaderStorageBuffer::from(vec![0u32; grid_length / PREFIX_SUM_BLOCK_SIZE]);

        let positive_particles = buffers.add(particles_data.clone());
        let positive_particles_count = buffers.add(single_data.clone());
        let negative_particles = buffers.add(particles_data.clone());
        let negative_particles_count = buffers.add(single_data.clone());
        let interface_band_mask = images.new_texture_storage(grid_size, TextureFormat::R8Uint);
        let phi_plus = buffers.add(grid_data.clone());
        let phi_minus = buffers.add(grid_data.clone());
        let num_positive_particles_in_cell = buffers.add(grid_data_u32.clone());
        let positive_cell_offsets = buffers.add(grid_data_u32.clone());
        let num_positive_particles_block_sums = buffers.add(prefix_sum_intermediate.clone());
        let num_negative_particles_in_cell = buffers.add(grid_data_u32.clone());
        let negative_cell_offsets = buffers.add(grid_data_u32.clone());
        let num_negative_particles_block_sums = buffers.add(prefix_sum_intermediate.clone());
        let positive_alive_particles_mask = buffers.add(grid_data_u32.clone());
        let positive_alive_particles_mask_scan = buffers.add(grid_data_u32.clone());
        let positive_alive_particles_mask_block_sums = buffers.add(prefix_sum_intermediate.clone());
        let sorted_positive_particles = buffers.add(particles_data.clone());
        let positive_cell_cursor = buffers.add(grid_data_u32.clone());
        let negative_alive_particles_mask = buffers.add(grid_data_u32.clone());
        let negative_alive_particles_mask_scan = buffers.add(grid_data_u32.clone());
        let negative_alive_particles_mask_block_sums = buffers.add(prefix_sum_intermediate.clone());
        let sorted_negative_particles = buffers.add(particles_data.clone());
        let negative_cell_cursor = buffers.add(grid_data_u32.clone());
        let positive_particles_to_be_added = buffers.add(grid_data_u32.clone());
        let negative_particles_to_be_added = buffers.add(grid_data_u32.clone());

        Self {
            positive_particles,
            positive_particles_count,
            negative_particles,
            negative_particles_count,
            interface_band_mask,
            phi_plus,
            phi_minus,
            num_positive_particles_in_cell,
            positive_cell_offsets,
            num_positive_particles_block_sums,
            num_negative_particles_in_cell,
            negative_cell_offsets,
            num_negative_particles_block_sums,
            positive_alive_particles_mask,
            positive_alive_particles_mask_scan,
            positive_alive_particles_mask_block_sums,
            sorted_positive_particles,
            positive_cell_cursor,
            negative_alive_particles_mask,
            negative_alive_particles_mask_scan,
            negative_alive_particles_mask_block_sums,
            sorted_negative_particles,
            negative_cell_cursor,
            positive_particles_to_be_added,
            negative_particles_to_be_added,
        }
    }
}

pub(crate) fn setup(
    commands: &mut Commands,
    entity: Entity,
    images: &mut ResMut<Assets<Image>>,
    buffers: &mut ResMut<Assets<ShaderStorageBuffer>>,
    grid_size: UVec2,
    u0: &Handle<Image>,
    v0: &Handle<Image>,
    levelset_air0: &Handle<Image>,
    levelset_air1: &Handle<Image>,
    grad_levelset_air: &Handle<Image>,
) {
    let pls_resources = PLSResources::new(images, buffers, grid_size);

    let update_interface_band_mask =
        UpdateInterfaceBandMaskResource::new(&pls_resources, levelset_air0);
    let initialize_particles =
        InitializeParticlesResource::new(&pls_resources, levelset_air0, grad_levelset_air);

    let advect_particles = AdvectParticlesResource::new(&pls_resources, u0, v0, levelset_air0);

    // level set correction
    let mark_escaped_particles = MarkEscapedParticlesResource::new(&pls_resources, levelset_air1);
    let reset_levelset_correction =
        ResetLevelSetCorrectionResource::new(&pls_resources, levelset_air1);
    let accumulate_levelset_correction_plus =
        AccumulateLevelSetCorrectionPlusResource::new(&pls_resources, levelset_air1);
    let accumulate_levelset_correction_minus =
        AccumulateLevelSetCorrectionMinusResource::new(&pls_resources, levelset_air1);
    let correct_levelset = CorrectLevelSetResource::new(&pls_resources, levelset_air1);

    // level set correction second
    let mark_escaped_particles_second =
        MarkEscapedParticlesSecondResource::new(&pls_resources, levelset_air0);
    let reset_levelset_correction_second =
        ResetLevelSetCorrectionSecondResource::new(&pls_resources, levelset_air0);
    let accumulate_levelset_correction_plus_second =
        AccumulateLevelSetCorrectionPlusSecondResource::new(&pls_resources, levelset_air0);
    let accumulate_levelset_correction_minus_second =
        AccumulateLevelSetCorrectionMinusSecondResource::new(&pls_resources, levelset_air0);
    let correct_levelset_second = CorrectLevelSetSecondResource::new(&pls_resources, levelset_air0);

    // reseed particles
    let count_positive_particles_in_cell =
        CountPositiveParticlesInCellResource::new(&pls_resources, grid_size);
    let count_negative_particles_in_cell =
        CountNegativeParticlesInCellResource::new(&pls_resources, grid_size);
    let prefix_sum_positive_particles_count =
        PrefixSumPositiveParticlesCountResource::new(&pls_resources);
    let prefix_sum_negative_particles_count =
        PrefixSumNegativeParticlesCountResource::new(&pls_resources);
    let sort_positive_particles = SortPositiveParticlesResource::new(&pls_resources, grid_size);
    let sort_negative_particles = SortNegativeParticlesResource::new(&pls_resources, grid_size);

    let reseed_positive_particles =
        ReseedPositiveParticlesResource::new(&pls_resources, levelset_air0, grid_size);
    let reseed_negative_particles =
        ReseedNegativeParticlesResource::new(&pls_resources, levelset_air0, grid_size);
    let prefix_sum_alive_positive_particles =
        PrefixSumAlivePositiveParticlesResource::new(&pls_resources);
    let prefix_sum_alive_negative_particles =
        PrefixSumAliveNegativeParticlesResource::new(&pls_resources);
    let delete_positive_particles = DeletePositiveParticlesResource::new(&pls_resources);
    let delete_negative_particles = DeleteNegativeParticlesResource::new(&pls_resources);
    let update_positive_particles_count = UpdatePositiveParticlesCountResource::new(&pls_resources);
    let update_negative_particles_count = UpdateNegativeParticlesCountResource::new(&pls_resources);
    let add_positive_particles =
        AddPositiveParticlesResource::new(&pls_resources, levelset_air0, grad_levelset_air);
    let add_negative_particles =
        AddNegativeParticlesResource::new(&pls_resources, levelset_air0, grad_levelset_air);

    commands
        .entity(entity)
        .insert(pls_resources)
        .insert((update_interface_band_mask, initialize_particles))
        .insert(advect_particles)
        .insert((
            mark_escaped_particles,
            reset_levelset_correction,
            accumulate_levelset_correction_plus,
            accumulate_levelset_correction_minus,
            correct_levelset,
        ))
        .insert((
            mark_escaped_particles_second,
            reset_levelset_correction_second,
            accumulate_levelset_correction_plus_second,
            accumulate_levelset_correction_minus_second,
            correct_levelset_second,
        ))
        .insert((
            count_positive_particles_in_cell,
            count_negative_particles_in_cell,
            prefix_sum_positive_particles_count,
            prefix_sum_negative_particles_count,
            sort_positive_particles,
            sort_negative_particles,
        ))
        .insert((
            reseed_positive_particles,
            reseed_negative_particles,
            prefix_sum_alive_positive_particles,
            prefix_sum_alive_negative_particles,
            delete_positive_particles,
            delete_negative_particles,
            update_positive_particles_count,
            update_negative_particles_count,
            add_positive_particles,
            add_negative_particles,
        ));
}

fn reset_buffers(
    query: Query<(&PLSResources, &FluidSettings)>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    for (pls_resources, settings) in &query {
        let grid_size = settings.size.element_product() as usize;
        let grid_data = vec![0u32; grid_size];
        let num_positive_particles_in_cell = buffers
            .get_mut(&pls_resources.num_positive_particles_in_cell)
            .unwrap();
        num_positive_particles_in_cell.set_data(grid_data.clone());

        let num_negative_particles_in_cell = buffers
            .get_mut(&pls_resources.num_negative_particles_in_cell)
            .unwrap();
        num_negative_particles_in_cell.set_data(grid_data.clone());

        let positive_cell_cursor = buffers
            .get_mut(&pls_resources.positive_cell_cursor)
            .unwrap();
        positive_cell_cursor.set_data(grid_data.clone());

        let negative_cell_cursor = buffers
            .get_mut(&pls_resources.negative_cell_cursor)
            .unwrap();
        negative_cell_cursor.set_data(grid_data.clone());
    }
}
