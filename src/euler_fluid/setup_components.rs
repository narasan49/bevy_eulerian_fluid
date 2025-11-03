use bevy::{
    prelude::*,
    render::{
        gpu_readback::Readback,
        render_resource::{BufferUsages, TextureFormat},
        storage::ShaderStorageBuffer,
    },
};

use super::definition::{
    DivergenceTextures, FluidSettings, JumpFloodingSeedsTextures, LevelsetTextures,
    SolidVelocityTextures, VelocityTexturesIntermediate, VelocityTexturesU, VelocityTexturesV,
};
use crate::definition::{FluidGridLength, Force, SampleForcesResource, SolidCenterTextures};
use crate::{
    definition::{ForcesToSolid, SolidEntities, SolidForcesBins, MAX_SOLIDS},
    euler_fluid::definition::{
        FluidSimulationBundle, LocalForces, PressureTextures, SimulationUniform, VelocityTextures,
    },
    fluid_to_solid::forces_to_solid_readback,
    texture::NewTexture,
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

        if size.0 % 64 != 0 || size.1 % 64 != 0 {
            warn!("the size is recommended to be multiple of 64. {size:?}");
        }
        let size_u = (size.0 + 1, size.1);
        let size_v = (size.0, size.1 + 1);

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

        let force = buffers.add(ShaderStorageBuffer::from(vec![Vec2::ZERO; 0]));
        let position = buffers.add(ShaderStorageBuffer::from(vec![Vec2::ZERO; 0]));

        let bins_force_x = buffers.add(ShaderStorageBuffer::from(vec![0u32; MAX_SOLIDS]));
        let bins_force_y = buffers.add(ShaderStorageBuffer::from(vec![0u32; MAX_SOLIDS]));
        let bins_torque = buffers.add(ShaderStorageBuffer::from(vec![0u32; MAX_SOLIDS]));

        let mut forces_to_solid_buffer =
            ShaderStorageBuffer::from(vec![Force::default(); MAX_SOLIDS]);
        forces_to_solid_buffer.buffer_description.usage |= BufferUsages::COPY_SRC;
        let forces_to_solid_buffer = buffers.add(forces_to_solid_buffer);

        let velocity_textures = VelocityTextures {
            u0: u0.clone(),
            v0: v0.clone(),
            u1: u1.clone(),
            v1: v1.clone(),
        };

        let velocity_textures_u = VelocityTexturesU {
            u0: u0.clone(),
            u1: u1.clone(),
            u_solid: u_solid.clone(),
        };

        let velocity_textures_v = VelocityTexturesV {
            v0: v0.clone(),
            v1: v1.clone(),
            v_solid: v_solid.clone(),
        };

        let velocity_textures_intermediate = VelocityTexturesIntermediate {
            v1: v1.clone(),
            u1: u1.clone(),
        };

        let solid_velocity_textures = SolidVelocityTextures { u_solid, v_solid };

        let solid_center_textures = SolidCenterTextures {
            levelset_solid: levelset_solid.clone(),
            solid_id: solid_id.clone(),
        };

        let pressure_textures = PressureTextures {
            p0: p0.clone(),
            p1: p1.clone(),
        };

        let divergence_textures = DivergenceTextures { div };

        let levelset_textures = LevelsetTextures {
            levelset_air0,
            levelset_air1,
            levelset_solid: levelset_solid.clone(),
        };

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
            size: Vec2::new(size.0 as f32, size.1 as f32),
        };

        let local_forces = LocalForces {
            forces: force,
            positions: position,
        };

        let jump_flooding_seeds_textures = JumpFloodingSeedsTextures {
            jump_flooding_seeds_x,
            jump_flooding_seeds_y,
        };

        let solid_forces_bins = SolidForcesBins {
            bins_force_x: bins_force_x.clone(),
            bins_force_y: bins_force_y.clone(),
            bins_torque: bins_torque.clone(),
        };

        let sample_forces_resource = SampleForcesResource {
            bins_force_x: bins_force_x.clone(),
            bins_force_y: bins_force_y.clone(),
            bins_torque: bins_torque.clone(),
            levelset_solid: levelset_solid.clone(),
            solid_id: solid_id.clone(),
            p1: p1.clone(),
        };

        let forces_to_solid = ForcesToSolid {
            forces: forces_to_solid_buffer.clone(),
        };

        let solid_entites = SolidEntities {
            entities: Vec::new(),
        };

        commands
            .entity(entity)
            .insert(FluidSimulationBundle {
                velocity_textures,
                velocity_textures_u,
                velocity_textures_v,
                velocity_textures_intermediate,
                solid_velocity_textures,
                pressure_textures,
                divergence_textures,
                local_forces,
                levelset_textures,
                jump_flooding_seeds_textures,
                solid_forces_bins,
                forces_to_solid,
                solid_center_textures,
                sample_forces_resource,
            })
            .insert(uniform)
            .insert(solid_entites)
            .insert(Readback::buffer(forces_to_solid_buffer.clone()))
            .observe(forces_to_solid_readback);
    }
}
