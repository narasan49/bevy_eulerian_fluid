use bevy::{
    prelude::*,
    render::{
        gpu_readback::Readback,
        render_resource::{BufferUsages, TextureFormat},
        storage::ShaderStorageBuffer,
    },
};

use crate::{
    advect_scalar::AdvectLevelsetResource,
    advection::AdvectionResource,
    apply_forces::{ApplyForcesResource, ForceToFluid},
    divergence::DivergenceResource,
    extrapolate_velocity::ExtrapolateVelocityResource,
    fluid_to_solid::{
        forces_to_solid_readback, AccumulateForcesResource, FluidToSolidForce,
        SampleForcesResource, MAX_SOLIDS,
    },
    fluid_uniform::SimulationUniform,
    initialize::{InitializeGridCenterResource, InitializeVelocityResource},
    obstacle::SolidEntities,
    reinitialize_levelset::{
        ReinitLevelsetCalculateSdfResource, ReinitLevelsetInitializeSeedsResource,
        ReinitLevelsetIterateResource,
    },
    settings::{FluidGridLength, FluidSettings, FluidTextures},
    solve_pressure::{JacobiIterationResource, JacobiIterationReverseResource},
    solve_velocity::{SolveUResource, SolveVResource},
    texture::NewTexture,
    update_solid::{UpdateSolidPressureResource, UpdateSolidResource},
};

pub(crate) fn watch_fluid_component(
    mut commands: Commands,
    query: Query<(Entity, &FluidSettings, Option<&Transform>), Added<FluidSettings>>,
    mut images: ResMut<Assets<Image>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    grid_length: Res<FluidGridLength>,
) {
    for (entity, settings, transform) in &query {
        let size = settings.size;

        if size.x % 64 != 0 || size.y % 64 != 0 {
            warn!("the size is recommended to be multiple of 64. {size:?}");
        }
        let size_u = size + UVec2::new(1, 0);
        let size_v = size + UVec2::new(0, 1);

        let u0 = images.new_texture_storage(size_u, TextureFormat::R32Float);
        let u1 = images.new_texture_storage(size_u, TextureFormat::R32Float);

        let v0 = images.new_texture_storage(size_v, TextureFormat::R32Float);
        let v1 = images.new_texture_storage(size_v, TextureFormat::R32Float);

        let u_solid = images.new_texture_storage(size_u, TextureFormat::R32Float);
        let v_solid = images.new_texture_storage(size_v, TextureFormat::R32Float);
        let solid_id = images.new_texture_storage(size, TextureFormat::R32Sint);

        let div = images.new_texture_storage(size, TextureFormat::R32Float);

        let p0 = images.new_texture_storage(size, TextureFormat::R32Float);
        let p1 = images.new_texture_storage(size, TextureFormat::R32Float);

        let levelset_air0 = images.new_texture_storage(size, TextureFormat::R32Float);
        let levelset_air1 = images.new_texture_storage(size, TextureFormat::R32Float);
        let levelset_solid = images.new_texture_storage(size, TextureFormat::R32Float);

        let jump_flooding_seeds_x = images.new_texture_storage(size, TextureFormat::R32Float);
        let jump_flooding_seeds_y = images.new_texture_storage(size, TextureFormat::R32Float);

        let forces_to_fluid =
            buffers.add(ShaderStorageBuffer::from(vec![ForceToFluid::default(); 0]));

        let bins_force_x = buffers.add(ShaderStorageBuffer::from(vec![0u32; MAX_SOLIDS]));
        let bins_force_y = buffers.add(ShaderStorageBuffer::from(vec![0u32; MAX_SOLIDS]));
        let bins_torque = buffers.add(ShaderStorageBuffer::from(vec![0u32; MAX_SOLIDS]));

        let mut forces_to_solid_buffer =
            ShaderStorageBuffer::from(vec![FluidToSolidForce::default(); MAX_SOLIDS]);
        forces_to_solid_buffer.buffer_description.usage |= BufferUsages::COPY_SRC;
        let forces_to_solid_buffer = buffers.add(forces_to_solid_buffer);

        let fluid_transform = match transform {
            Some(t) => t.to_matrix(),
            None => Mat4::IDENTITY,
        };

        let uniform = SimulationUniform {
            dx: grid_length.0,
            dt: 0.0,
            rho: settings.rho,
            gravity: settings.gravity,
            initial_fluid_level: settings.initial_fluid_level,
            fluid_transform,
            size: size.as_vec2(),
        };

        let fluid_textures = FluidTextures {
            u: u0.clone(),
            v: v0.clone(),
            u_solid: u_solid.clone(),
            v_solid: v_solid.clone(),
            levelset_air: levelset_air0.clone(),
            levelset_solid: levelset_solid.clone(),
        };

        let initialize_resource = InitializeVelocityResource {
            u0: u0.clone(),
            u1: u1.clone(),
            v0: v0.clone(),
            v1: v1.clone(),
        };

        let initialize_grid_center_resource = InitializeGridCenterResource {
            levelset_air0: levelset_air0.clone(),
            levelset_air1: levelset_air1.clone(),
        };

        let update_solid_resource = UpdateSolidResource {
            u_solid: u_solid.clone(),
            v_solid: v_solid.clone(),
            levelset_solid: levelset_solid.clone(),
            solid_id: solid_id.clone(),
        };

        let update_solid_pressure = UpdateSolidPressureResource {
            p0: p0.clone(),
            levelset_solid: levelset_solid.clone(),
        };

        let advection_resource = AdvectionResource {
            u0: u0.clone(),
            v0: v0.clone(),
            u1: u1.clone(),
            v1: v1.clone(),
        };

        let apply_forces_resource = ApplyForcesResource {
            u1: u1.clone(),
            v1: v1.clone(),
            levelset_air0: levelset_air0.clone(),
            forces_to_fluid: forces_to_fluid.clone(),
        };

        let divergence_resource = DivergenceResource {
            u1: u1.clone(),
            v1: v1.clone(),
            u_solid: u_solid.clone(),
            v_solid: v_solid.clone(),
            levelset_solid: levelset_solid.clone(),
            div: div.clone(),
        };

        let jacobi_iter_resource = JacobiIterationResource {
            p0: p0.clone(),
            p1: p1.clone(),
            div: div.clone(),
            levelset_air0: levelset_air0.clone(),
            levelset_solid: levelset_solid.clone(),
        };

        let jacobi_iter_rev_resource = JacobiIterationReverseResource {
            p0: p0.clone(),
            p1: p1.clone(),
            div: div.clone(),
            levelset_air0: levelset_air0.clone(),
            levelset_solid: levelset_solid.clone(),
        };

        let solve_u_resource = SolveUResource {
            u0: u0.clone(),
            u1: u1.clone(),
            u_solid: u_solid.clone(),
            p1: p1.clone(),
            levelset_air0: levelset_air0.clone(),
            levelset_solid: levelset_solid.clone(),
        };

        let solve_v_resource = SolveVResource {
            v0: v0.clone(),
            v1: v1.clone(),
            v_solid: v_solid.clone(),
            p1: p1.clone(),
            levelset_air0: levelset_air0.clone(),
            levelset_solid: levelset_solid.clone(),
        };

        let extrapolate_velocity_resource = ExtrapolateVelocityResource {
            u0: u0.clone(),
            v0: v0.clone(),
            levelset_air0: levelset_air0.clone(),
            levelset_solid: levelset_solid.clone(),
        };

        let advect_levelset_resource = AdvectLevelsetResource {
            u0: u0.clone(),
            v0: v0.clone(),
            levelset_air0: levelset_air0.clone(),
            levelset_air1: levelset_air1.clone(),
        };

        let reinit_levelset_initialize_seeds_resource = ReinitLevelsetInitializeSeedsResource {
            levelset_air1: levelset_air1.clone(),
            jump_flooding_seeds_x: jump_flooding_seeds_x.clone(),
            jump_flooding_seeds_y: jump_flooding_seeds_y.clone(),
        };

        let reinit_levelset_iterate_resource = ReinitLevelsetIterateResource {
            jump_flooding_seeds_x: jump_flooding_seeds_x.clone(),
            jump_flooding_seeds_y: jump_flooding_seeds_y.clone(),
        };

        let reinit_levelset_calculate_sdf_resource = ReinitLevelsetCalculateSdfResource {
            levelset_air0: levelset_air0.clone(),
            levelset_air1: levelset_air1.clone(),
            jump_flooding_seeds_x: jump_flooding_seeds_x.clone(),
            jump_flooding_seeds_y: jump_flooding_seeds_y.clone(),
        };

        let sample_forces_resource = SampleForcesResource {
            bins_force_x: bins_force_x.clone(),
            bins_force_y: bins_force_y.clone(),
            bins_torque: bins_torque.clone(),
            levelset_solid: levelset_solid.clone(),
            solid_id: solid_id.clone(),
            p1: p1.clone(),
        };

        let accumulate_forces_resource = AccumulateForcesResource {
            bins_force_x: bins_force_x.clone(),
            bins_force_y: bins_force_y.clone(),
            bins_torque: bins_torque.clone(),
            forces: forces_to_solid_buffer.clone(),
        };

        let solid_entites = SolidEntities {
            entities: Vec::new(),
        };

        commands
            .entity(entity)
            .insert((
                fluid_textures,
                initialize_resource,
                initialize_grid_center_resource,
                update_solid_resource,
                update_solid_pressure,
                advection_resource,
                apply_forces_resource,
                divergence_resource,
                jacobi_iter_resource,
                jacobi_iter_rev_resource,
            ))
            .insert((
                solve_u_resource,
                solve_v_resource,
                extrapolate_velocity_resource,
                advect_levelset_resource,
                reinit_levelset_initialize_seeds_resource,
                reinit_levelset_iterate_resource,
                reinit_levelset_calculate_sdf_resource,
                sample_forces_resource,
                accumulate_forces_resource,
            ))
            .insert(uniform)
            .insert(solid_entites)
            .insert(Readback::buffer(forces_to_solid_buffer.clone()))
            .observe(forces_to_solid_readback);
    }
}
