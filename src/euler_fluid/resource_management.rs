use bevy::{
    prelude::*,
    render::{
        render_resource::{BufferUsages, TextureFormat},
        storage::ShaderStorageBuffer,
    },
};
use bevy_eulerian_fluid_common::texture::new_texture_storage_2d;

use crate::{
    apply_forces::ForceToFluid,
    fluid_to_solid::{FluidToSolidForce, MAX_SOLIDS},
};

pub(crate) struct FluidResources {
    pub u0: Handle<Image>,
    pub u1: Handle<Image>,
    pub v0: Handle<Image>,
    pub v1: Handle<Image>,
    pub u_solid: Handle<Image>,
    pub v_solid: Handle<Image>,
    pub solid_id: Handle<Image>,
    pub in_is_u_valid: Handle<Image>,
    pub out_is_u_valid: Handle<Image>,
    pub in_is_v_valid: Handle<Image>,
    pub out_is_v_valid: Handle<Image>,
    pub div: Handle<Image>,
    pub p0: Handle<Image>,
    pub p1: Handle<Image>,
    pub levelset_air0: Handle<Image>,
    pub levelset_air1: Handle<Image>,
    pub grad_levelset_air: Handle<Image>,
    pub levelset_solid: Handle<Image>,
    pub area_fraction_solid: Handle<Image>,
    pub forces_to_fluid: Handle<ShaderStorageBuffer>,
    pub bins_force_x: Handle<ShaderStorageBuffer>,
    pub bins_force_y: Handle<ShaderStorageBuffer>,
    pub bins_torque: Handle<ShaderStorageBuffer>,
    pub forces_to_solid_buffer: Handle<ShaderStorageBuffer>,
}

impl FluidResources {
    pub fn new(
        images: &mut Assets<Image>,
        buffers: &mut Assets<ShaderStorageBuffer>,
        size: UVec2,
    ) -> Self {
        let size_u = size + UVec2::X;
        let size_v = size + UVec2::Y;

        let u0 = new_texture_storage_2d(images, size_u, TextureFormat::R32Float);
        let u1 = new_texture_storage_2d(images, size_u, TextureFormat::R32Float);

        let v0 = new_texture_storage_2d(images, size_v, TextureFormat::R32Float);
        let v1 = new_texture_storage_2d(images, size_v, TextureFormat::R32Float);

        let u_solid = new_texture_storage_2d(images, size_u, TextureFormat::R32Float);
        let v_solid = new_texture_storage_2d(images, size_v, TextureFormat::R32Float);
        let solid_id = new_texture_storage_2d(images, size, TextureFormat::R32Sint);

        let in_is_u_valid = new_texture_storage_2d(images, size_u, TextureFormat::R32Sint);
        let out_is_u_valid = new_texture_storage_2d(images, size_u, TextureFormat::R32Sint);
        let in_is_v_valid = new_texture_storage_2d(images, size_v, TextureFormat::R32Sint);
        let out_is_v_valid = new_texture_storage_2d(images, size_v, TextureFormat::R32Sint);

        let div = new_texture_storage_2d(images, size, TextureFormat::R32Float);

        let p0 = new_texture_storage_2d(images, size, TextureFormat::R32Float);
        let p1 = new_texture_storage_2d(images, size, TextureFormat::R32Float);

        let levelset_air0 = new_texture_storage_2d(images, size, TextureFormat::R32Float);
        let levelset_air1 = new_texture_storage_2d(images, size, TextureFormat::R32Float);
        let grad_levelset_air = new_texture_storage_2d(images, size, TextureFormat::Rg32Float);
        let levelset_solid = new_texture_storage_2d(images, size, TextureFormat::R32Float);

        let area_fraction_solid = new_texture_storage_2d(images, size, TextureFormat::Rgba32Float);

        let forces_to_fluid =
            buffers.add(ShaderStorageBuffer::from(vec![ForceToFluid::default(); 0]));

        let bins_force_x = buffers.add(ShaderStorageBuffer::from(vec![0u32; MAX_SOLIDS]));
        let bins_force_y = buffers.add(ShaderStorageBuffer::from(vec![0u32; MAX_SOLIDS]));
        let bins_torque = buffers.add(ShaderStorageBuffer::from(vec![0u32; MAX_SOLIDS]));

        let mut forces_to_solid_buffer =
            ShaderStorageBuffer::from(vec![FluidToSolidForce::default(); MAX_SOLIDS]);
        forces_to_solid_buffer.buffer_description.usage |= BufferUsages::COPY_SRC;
        let forces_to_solid_buffer = buffers.add(forces_to_solid_buffer);

        Self {
            u0,
            u1,
            v0,
            v1,
            u_solid,
            v_solid,
            solid_id,
            in_is_u_valid,
            out_is_u_valid,
            in_is_v_valid,
            out_is_v_valid,
            div,
            p0,
            p1,
            levelset_air0,
            levelset_air1,
            grad_levelset_air,
            levelset_solid,
            area_fraction_solid,
            forces_to_fluid,
            bins_force_x,
            bins_force_y,
            bins_torque,
            forces_to_solid_buffer,
        }
    }
}

pub(crate) trait FluidResource {
    fn new(resources: &FluidResources) -> Self;
}
