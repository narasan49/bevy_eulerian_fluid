use bevy::{
    prelude::*,
    render::{gpu_readback::Readback, storage::ShaderStorageBuffer},
};

use crate::{
    advect_levelset::AdvectLevelSetResource,
    advection::AdvectionResource,
    apply_forces::ApplyForcesResource,
    divergence::DivergenceResource,
    extrapolate_velocity::{
        ExtrapolateUResource, ExtrapolateVResource, InitializeUValid, InitializeVValid,
    },
    fluid_source::update_fluid_source::UpdateFluidSourceResource,
    fluid_to_solid::{forces_to_solid_readback, AccumulateForcesResource, SampleForcesResource},
    fluid_uniform::SimulationUniform,
    initialize::{InitializeGridCenterResource, InitializeGridEdgeResource},
    levelset_gradient::LevelSetGradientResource,
    obstacle::SolidEntities,
    particle_levelset_two_layers,
    projection::{gauss_seidel::GaussSeidelResource, multi_grid},
    reinitialize_levelset::{self, ReinitializeMethod},
    resource_management::{FluidResource, FluidResources},
    settings::{FluidGridLength, FluidSettings, FluidTextures},
    solve_pressure::{JacobiIterationResource, JacobiIterationReverseResource},
    solve_velocity::{SolveUResource, SolveVResource},
    update_area_fraction::UpdateAreaFractionResource,
    update_solid::UpdateSolidResource,
};

pub(crate) fn watch_fluid_component(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &FluidSettings,
            &ReinitializeMethod,
            Option<&Transform>,
        ),
        Added<FluidSettings>,
    >,
    mut images: ResMut<Assets<Image>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    grid_length: Res<FluidGridLength>,
) {
    for (entity, settings, reinit_method, transform) in &query {
        let size = settings.size;

        if size.x % 64 != 0 || size.y % 64 != 0 {
            warn!("the size is recommended to be multiple of 64. {size:?}");
        }

        let resources = FluidResources::new(&mut images, &mut buffers, size);

        let fluid_transform = match transform {
            Some(t) => t.to_matrix(),
            None => Mat4::IDENTITY,
        };

        let uniform = SimulationUniform {
            dx: grid_length.0,
            dt: 0.0,
            rho: settings.rho,
            gravity: settings.gravity,
            fluid_transform,
            size: size.as_vec2(),
        };

        let fluid_textures = FluidTextures::new(&resources);

        let initialize_grid_edge_resource = InitializeGridEdgeResource::new(&resources);
        let initialize_grid_center_resource = InitializeGridCenterResource::new(&resources);

        let update_solid_resource = UpdateSolidResource::new(&resources);
        let update_area_fraction_resource = UpdateAreaFractionResource::new(&resources);

        let advection_resource = AdvectionResource::new(&resources);

        let apply_forces_resource = ApplyForcesResource::new(&resources);

        let divergence_resource = DivergenceResource::new(&resources);

        let jacobi_iter_resource = JacobiIterationResource::new(&resources);
        let jacobi_iter_rev_resource = JacobiIterationReverseResource::new(&resources);
        let gauss_seidel_resource = GaussSeidelResource::new(&resources);

        let solve_u_resource = SolveUResource::new(&resources);
        let solve_v_resource = SolveVResource::new(&resources);

        let init_u_valid = InitializeUValid::new(&resources);
        let extrapolate_u_resource = ExtrapolateUResource::new(&resources);
        let init_v_valid = InitializeVValid::new(&resources);
        let extrapolate_v_resource = ExtrapolateVResource::new(&resources);

        let advect_levelset_resource = AdvectLevelSetResource::new(&resources);
        let levelset_gradient_resource = LevelSetGradientResource::new(&resources);

        let sample_forces_resource = SampleForcesResource::new(&resources);

        let accumulate_forces_resource = AccumulateForcesResource::new(&resources);

        let solid_entites = SolidEntities {
            entities: Vec::new(),
        };

        let update_fluid_source = UpdateFluidSourceResource::new(&resources);

        commands
            .entity(entity)
            .insert((
                fluid_textures,
                initialize_grid_edge_resource,
                initialize_grid_center_resource,
                update_solid_resource,
                update_area_fraction_resource,
                advection_resource,
                apply_forces_resource,
                divergence_resource,
                jacobi_iter_resource,
                jacobi_iter_rev_resource,
                gauss_seidel_resource,
            ))
            .insert((
                solve_u_resource,
                solve_v_resource,
                advect_levelset_resource,
                levelset_gradient_resource,
                sample_forces_resource,
                accumulate_forces_resource,
                update_fluid_source,
            ))
            .insert((
                init_u_valid,
                init_v_valid,
                extrapolate_u_resource,
                extrapolate_v_resource,
            ))
            .insert(uniform)
            .insert(solid_entites)
            .insert(Readback::buffer(resources.forces_to_solid_buffer.clone()))
            .observe(forces_to_solid_readback);

        reinitialize_levelset::setup(
            &mut commands,
            entity,
            &mut images,
            settings.size,
            &resources,
            reinit_method,
        );

        particle_levelset_two_layers::plugin::setup(
            &mut commands,
            entity,
            &mut images,
            &mut buffers,
            settings.size,
            &resources,
        );

        multi_grid::setup_multigrid_resources(
            &mut commands,
            entity,
            settings.size,
            &resources,
            &mut images,
        );
    }
}
