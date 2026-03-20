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
    extrapolate_velocity::{
        ExtrapolateUResource, ExtrapolateVResource, InitializeUValid, InitializeVValid,
    },
    fluid_to_solid::{
        forces_to_solid_readback, AccumulateForcesResource, FluidToSolidForce,
        SampleForcesResource, MAX_SOLIDS,
    },
    fluid_uniform::SimulationUniform,
    initialize::{InitializeGridCenterResource, InitializeVelocityResource},
    levelset_gradient::LevelSetGradientResource,
    obstacle::SolidEntities,
    particle_levelset::{
        advect_particles::AdvectParticlesResource,
        distribute_particles_to_grid,
        initialize_interface_indices::InitializeInterfaceIndicesResource,
        initialize_particles::InitializeParticlesResource,
        reseed_particles::{self, ReseedParticlesBundle},
        Particle,
    },
    particle_levelset_two_layers,
    reinitialize_levelset::{
        ReinitLevelsetCalculateSdfResource, ReinitLevelsetInitializeSeedsResource,
        ReinitLevelsetSeedsTextures,
    },
    settings::{FluidGridLength, FluidSettings, FluidTextures},
    solve_pressure::{JacobiIterationResource, JacobiIterationReverseResource},
    solve_velocity::{SolveUResource, SolveVResource},
    texture::NewTexture,
    update_solid::UpdateSolidResource,
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

        let in_is_u_valid = images.new_texture_storage(size_u, TextureFormat::R32Sint);
        let out_is_u_valid = images.new_texture_storage(size_u, TextureFormat::R32Sint);
        let in_is_v_valid = images.new_texture_storage(size_v, TextureFormat::R32Sint);
        let out_is_v_valid = images.new_texture_storage(size_v, TextureFormat::R32Sint);

        let div = images.new_texture_storage(size, TextureFormat::R32Float);

        let p0 = images.new_texture_storage(size, TextureFormat::R32Float);
        let p1 = images.new_texture_storage(size, TextureFormat::R32Float);

        let levelset_air0 = images.new_texture_storage(size, TextureFormat::R32Float);
        let levelset_air1 = images.new_texture_storage(size, TextureFormat::R32Float);
        let grad_levelset_air = images.new_texture_storage(size, TextureFormat::Rg32Float);
        let levelset_solid = images.new_texture_storage(size, TextureFormat::R32Float);

        let jump_flooding_seeds0 = images.new_texture_storage(size, TextureFormat::Rg32Float);
        let jump_flooding_seeds1 = images.new_texture_storage(size, TextureFormat::Rg32Float);

        let forces_to_fluid =
            buffers.add(ShaderStorageBuffer::from(vec![ForceToFluid::default(); 0]));

        let bins_force_x = buffers.add(ShaderStorageBuffer::from(vec![0u32; MAX_SOLIDS]));
        let bins_force_y = buffers.add(ShaderStorageBuffer::from(vec![0u32; MAX_SOLIDS]));
        let bins_torque = buffers.add(ShaderStorageBuffer::from(vec![0u32; MAX_SOLIDS]));

        let mut forces_to_solid_buffer =
            ShaderStorageBuffer::from(vec![FluidToSolidForce::default(); MAX_SOLIDS]);
        forces_to_solid_buffer.buffer_description.usage |= BufferUsages::COPY_SRC;
        let forces_to_solid_buffer = buffers.add(forces_to_solid_buffer);

        let levelset_particles = buffers.add(ShaderStorageBuffer::from(vec![
            Particle::default();
            4 * size.element_product()
                as usize
        ]));
        let near_interface = images.new_texture_storage(size, TextureFormat::R8Uint);

        let (
            cell_particle_counts,
            cell_offsets,
            sorted_particles,
            block_scan_sums,
            cell_cursor,
            levelset_correction,
            weight,
        ) = distribute_particles_to_grid::create_buffers(&mut buffers, size);

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
            levelset_particles: levelset_particles.clone(),
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
            grad_levelset_air: grad_levelset_air.clone(),
        };

        let initialize_interface_indices_resource = InitializeInterfaceIndicesResource {
            levelset: levelset_air0.clone(),
            near_interface: near_interface.clone(),
        };

        let count = buffers.add(ShaderStorageBuffer::from(0u32));

        let initialize_particles_resource = InitializeParticlesResource {
            count: count.clone(),
            levelset_particles: levelset_particles.clone(),
            levelset_air: levelset_air0.clone(),
            grad_levelset_air: grad_levelset_air.clone(),
            near_interface: near_interface.clone(),
        };

        let update_solid_resource = UpdateSolidResource {
            u_solid: u_solid.clone(),
            v_solid: v_solid.clone(),
            levelset_solid: levelset_solid.clone(),
            solid_id: solid_id.clone(),
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

        let init_u_valid = InitializeUValid {
            is_u_valid: in_is_u_valid.clone(),
            levelset_air: levelset_air0.clone(),
        };

        let extrapolate_u_resource = ExtrapolateUResource {
            u0: u0.clone(),
            in_is_u_valid: in_is_u_valid.clone(),
            out_is_u_valid: out_is_u_valid.clone(),
        };

        let init_v_valid = InitializeVValid {
            is_v_valid: in_is_v_valid.clone(),
            levelset_air: levelset_air0.clone(),
        };

        let extrapolate_v_resource = ExtrapolateVResource {
            v0: v0.clone(),
            in_is_v_valid: in_is_v_valid.clone(),
            out_is_v_valid: out_is_v_valid.clone(),
        };

        let advect_levelset_resource = AdvectLevelsetResource {
            u0: u0.clone(),
            v0: v0.clone(),
            levelset_air0: levelset_air0.clone(),
            levelset_air1: levelset_air1.clone(),
        };

        let advect_levelset_particles_resource = AdvectParticlesResource {
            count: count.clone(),
            levelset_particles: levelset_particles.clone(),
            u0: u0.clone(),
            v0: v0.clone(),
            levelset_air: levelset_air1.clone(),
        };

        let reinit_levelset_initialize_seeds_resource = ReinitLevelsetInitializeSeedsResource {
            levelset_air1: levelset_air1.clone(),
        };

        let reinit_levelset_calculate_sdf_resource = ReinitLevelsetCalculateSdfResource {
            levelset_air0: levelset_air0.clone(),
            levelset_air1: levelset_air1.clone(),
        };

        let reinit_levelset_seeds_textures =
            ReinitLevelsetSeedsTextures([jump_flooding_seeds0, jump_flooding_seeds1]);

        let levelset_gradient_resource =
            LevelSetGradientResource::new(&levelset_air0, &grad_levelset_air);

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

        let (alive_particles_mask, alive_particles_mask_scan, sums) =
            reseed_particles::create_buffers(&mut buffers, size);

        let reseed_particles_bundle = ReseedParticlesBundle::new(
            &sorted_particles,
            &alive_particles_mask,
            &alive_particles_mask_scan,
            &sums,
            &levelset_particles,
            &count,
            &cell_particle_counts,
            &cell_offsets,
            &near_interface,
            &levelset_air1,
            &grad_levelset_air,
        );

        commands
            .entity(entity)
            .insert((
                fluid_textures,
                initialize_resource,
                initialize_grid_center_resource,
                initialize_interface_indices_resource,
                initialize_particles_resource,
                update_solid_resource,
                advection_resource,
                apply_forces_resource,
                divergence_resource,
                jacobi_iter_resource,
                jacobi_iter_rev_resource,
            ))
            .insert((
                solve_u_resource,
                solve_v_resource,
                advect_levelset_resource,
                advect_levelset_particles_resource,
                reinit_levelset_initialize_seeds_resource,
                reinit_levelset_calculate_sdf_resource,
                reinit_levelset_seeds_textures,
                levelset_gradient_resource,
                sample_forces_resource,
                accumulate_forces_resource,
            ))
            .insert((
                init_u_valid,
                init_v_valid,
                extrapolate_u_resource,
                extrapolate_v_resource,
            ))
            .insert(reseed_particles_bundle)
            .insert(uniform)
            .insert(solid_entites)
            .insert(Readback::buffer(forces_to_solid_buffer.clone()))
            .observe(forces_to_solid_readback);

        distribute_particles_to_grid::insert_distribute_particles_resources(
            &mut commands,
            entity,
            levelset_particles,
            count,
            cell_particle_counts,
            cell_offsets,
            block_scan_sums,
            sorted_particles,
            cell_cursor,
            levelset_correction,
            weight,
            &levelset_air1,
            settings.size,
        );

        particle_levelset_two_layers::plugin::setup(
            &mut commands,
            entity,
            &mut images,
            &mut buffers,
            settings.size,
            &u0,
            &v0,
            &levelset_air0,
            &levelset_air1,
            &grad_levelset_air,
        );
    }
}
