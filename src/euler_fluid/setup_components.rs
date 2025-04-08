use bevy::{
    prelude::*,
    render::{render_resource::TextureFormat, storage::ShaderStorageBuffer},
};

use crate::{
    euler_fluid::definition::{
        FluidSimulationBundle, LocalForces, PressureTextures, SimulationUniform, VelocityTextures,
    },
    texture::NewTexture,
};

use super::definition::{
    DivergenceTextures, FluidSettings, JumpFloodingSeedsTextures, LevelsetTextures,
    SolidVelocityTextures,
};

pub(crate) fn watch_fluid_component(
    mut commands: Commands,
    query: Query<(Entity, &FluidSettings, Option<&Transform>), Added<FluidSettings>>,
    mut images: ResMut<Assets<Image>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    for (entity, settings, transform) in &query {
        let size = settings.size;

        if size.0 % 64 != 0 || size.1 % 64 != 0 {
            warn!("the size is recommended to be multiple of 64. {size:?}");
        }
        let size_u = (size.0 + 1, size.1);
        let size_v = (size.0, size.1 + 1);
        let size_vertex = (size.0 + 1, size.1 + 1);

        let u0 = images.new_texture_storage(size_u, TextureFormat::R32Float);
        let u1 = images.new_texture_storage(size_u, TextureFormat::R32Float);

        let v0 = images.new_texture_storage(size_v, TextureFormat::R32Float);
        let v1 = images.new_texture_storage(size_v, TextureFormat::R32Float);

        let u_solid = images.new_texture_storage(size_u, TextureFormat::R32Float);
        let v_solid = images.new_texture_storage(size_v, TextureFormat::R32Float);

        let div = images.new_texture_storage(size, TextureFormat::R32Float);

        let p0 = images.new_texture_storage(size, TextureFormat::R32Float);
        let p1 = images.new_texture_storage(size, TextureFormat::R32Float);

        let levelset_air0 = images.new_texture_storage(size, TextureFormat::R32Float);
        let levelset_air1 = images.new_texture_storage(size, TextureFormat::R32Float);
        let levelset_solid = images.new_texture_storage(size_vertex, TextureFormat::R32Float);

        let jump_flooding_seeds_x = images.new_texture_storage(size, TextureFormat::R32Float);
        let jump_flooding_seeds_y = images.new_texture_storage(size, TextureFormat::R32Float);

        let force = buffers.add(ShaderStorageBuffer::from(vec![Vec2::ZERO; 0]));
        let position = buffers.add(ShaderStorageBuffer::from(vec![Vec2::ZERO; 0]));

        let velocity_textures = VelocityTextures { u0, v0, u1, v1 };

        let solid_velocity_textures = SolidVelocityTextures { u_solid, v_solid };

        let pressure_textures = PressureTextures { p0, p1 };

        let divergence_textures = DivergenceTextures { div };

        let levelset_textures = LevelsetTextures {
            levelset_air0,
            levelset_air1,
            levelset_solid,
        };

        let fluid_transform = match transform {
            Some(t) => t.compute_matrix(),
            None => Mat4::IDENTITY,
        };

        let uniform = SimulationUniform {
            dx: settings.dx,
            dt: settings.dt,
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

        commands
            .entity(entity)
            .insert(FluidSimulationBundle {
                velocity_textures,
                solid_velocity_textures,
                pressure_textures,
                divergence_textures,
                local_forces,
                levelset_textures,
                jump_flooding_seeds_textures,
            })
            .insert(uniform);
    }
}
