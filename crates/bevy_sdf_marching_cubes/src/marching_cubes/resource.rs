use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{AsBindGroup, BufferUsages, DrawIndirectArgs, ShaderType},
        storage::ShaderStorageBuffer,
    },
};

use crate::marching_cubes::{draw_pipeline::MarchingCubesUniform, lookup_table::LUT};

#[derive(Component, ExtractComponent, Clone)]
#[require(Transform)]
pub struct MarchingCubes {
    pub half_size: Vec3,
    pub sdf: Handle<Image>,
    pub grad_sdf: Handle<Image>,
    pub resolution: UVec3,
}

#[derive(Component, ExtractComponent, Clone, ShaderType)]
pub struct MarchingCubesConfigUniform {
    pub half_size: Vec3,
}

#[derive(ShaderType, Clone, Default)]
pub struct Vertex {
    position: Vec3,
    normal: Vec3,
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub struct MarchingCubesExtractResource {
    #[storage(0, visibility(compute))]
    pub vertices: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(compute))]
    pub indirect_args: Handle<ShaderStorageBuffer>,
    #[storage_texture(2, image_format = R32Float, access = ReadOnly, dimension = "3d")]
    pub sdf: Handle<Image>,
    #[storage_texture(3, image_format = Rgba32Float, access = ReadOnly, dimension = "3d")]
    pub grad_sdf: Handle<Image>,
    #[storage(4, read_only, visibility(compute))]
    pub lookup_table: Handle<ShaderStorageBuffer>,
    #[uniform(5)]
    pub config: MarchingCubesConfigUniform,
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub struct MarchingCubesDrawResource {
    #[storage(0, visibility(vertex, fragment))]
    pub vertices: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(vertex, fragment))]
    pub indirect_args: Handle<ShaderStorageBuffer>,
}

pub fn setup_resources(
    mut commands: Commands,
    query: Query<(Entity, &MarchingCubes), Added<MarchingCubes>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    for (entity, marching_cubes) in &query {
        info!("Setting up MarchingCubes resources.");
        let scale = (marching_cubes.resolution.element_product() as f32).powf(2.0 / 3.0) as u32;
        let num_vertices = scale * 10 * 3 * 5;
        let mut vertices_buuffer =
            ShaderStorageBuffer::from(vec![Vertex::default(); num_vertices as usize]);
        vertices_buuffer.buffer_description.usage |=
            BufferUsages::STORAGE | BufferUsages::VERTEX | BufferUsages::COPY_DST;
        let vertices = buffers.add(vertices_buuffer);

        let indirect_args_value = DrawIndirectArgs {
            vertex_count: 0,
            instance_count: 1,
            first_vertex: 0,
            first_instance: 0,
        };
        let mut indirect_args_buffer =
            ShaderStorageBuffer::new(indirect_args_value.as_bytes(), RenderAssetUsages::default());
        indirect_args_buffer.buffer_description.usage |=
            BufferUsages::STORAGE | BufferUsages::INDIRECT | BufferUsages::COPY_DST;
        let indirect_args = buffers.add(indirect_args_buffer);

        let lookup_table = buffers.add(ShaderStorageBuffer::from(LUT));

        let compute_resource = MarchingCubesExtractResource {
            vertices: vertices.clone(),
            indirect_args: indirect_args.clone(),
            sdf: marching_cubes.sdf.clone(),
            grad_sdf: marching_cubes.grad_sdf.clone(),
            lookup_table,
            config: MarchingCubesConfigUniform {
                half_size: marching_cubes.half_size,
            },
        };

        let draw_resource = MarchingCubesDrawResource {
            vertices,
            indirect_args,
        };

        let uniform = MarchingCubesUniform {
            world_from_local: Mat4::IDENTITY,
        };

        commands
            .entity(entity)
            .insert((compute_resource, draw_resource, uniform));
    }
}
