use bevy::{
    prelude::*,
    render::{
        extract_component::ExtractComponent, render_resource::TextureFormat,
        storage::ShaderStorageBuffer,
    },
};
use bevy_eulerian_fluid_common::texture::new_texture_storage_3d;

use crate::{
    core::{
        advect_levelset::AdvectLevelSetResource,
        advect_velocity::AdvectVelocityResource,
        apply_forces::ApplyForcesResource,
        divergence::DivergenceResource,
        extrapolate_velocity::ExtrapolateVelocityResource,
        fluid_source::{
            fluid_source_uniform::{FluidSourceInitUniform, FluidSourceUniform},
            update_fluid_source::UpdateFluidSourceResource,
        },
        fluid_uniform::FluidUniform,
        initialize_resources::InitializeResourcesResource,
        projection::{
            gauss_seidel::GaussSeidelResource, multigrid::setup_multigrid_resources,
            ProjectionMethod,
        },
        reinitialize_levelset::{self, ReinitializeMethod},
        solid_body::{
            update_area_fraction_solid::UpdateAreaFractionResource,
            update_solid::UpdateSolidResource,
        },
        solve_u::SolveUResource,
        solve_v::SolveVResource,
        solve_w::SolveWResource,
    },
    fluid_status::FluidStatus,
};

#[derive(Component, ExtractComponent, Clone)]
#[require(
    Transform,
    ProjectionMethod,
    ReinitializeMethod,
    FluidSourceUniform,
    FluidSourceInitUniform,
    FluidStatus
)]
pub struct EulerFluid3d {
    pub rho: f32,
    pub gravity: Vec3,
    pub resolution: UVec3,
}

#[derive(Resource, Clone, Copy)]
pub struct FluidGridLength(pub f32);

impl Default for FluidGridLength {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Component)]
pub struct FluidResources {
    pub u0: Handle<Image>,
    pub v0: Handle<Image>,
    pub w0: Handle<Image>,
    pub u1: Handle<Image>,
    pub v1: Handle<Image>,
    pub w1: Handle<Image>,
    pub u_solid: Handle<Image>,
    pub v_solid: Handle<Image>,
    pub w_solid: Handle<Image>,
    pub levelset_air0: Handle<Image>,
    pub levelset_air1: Handle<Image>,
    pub levelset_solid: Handle<Image>,
    pub area_fraction_solid: Handle<Image>,
    pub p: Handle<Image>,
    pub div: Handle<Image>,
    pub in_is_u_valid: Handle<Image>,
    pub in_is_v_valid: Handle<Image>,
    pub in_is_w_valid: Handle<Image>,
    pub out_is_u_valid: Handle<Image>,
    pub out_is_v_valid: Handle<Image>,
    pub out_is_w_valid: Handle<Image>,
}

impl FluidResources {
    fn new(images: &mut Assets<Image>, resolution: UVec3) -> Self {
        let resolution_u = resolution + UVec3::X;
        let resolution_v = resolution + UVec3::Y;
        let resolution_w = resolution + UVec3::Z;
        let resolution_uvw = resolution + UVec3::ONE;

        let u0 = new_texture_storage_3d(images, resolution_u, TextureFormat::R32Float);
        let v0 = new_texture_storage_3d(images, resolution_v, TextureFormat::R32Float);
        let w0 = new_texture_storage_3d(images, resolution_w, TextureFormat::R32Float);

        let u1 = new_texture_storage_3d(images, resolution_u, TextureFormat::R32Float);
        let v1 = new_texture_storage_3d(images, resolution_v, TextureFormat::R32Float);
        let w1 = new_texture_storage_3d(images, resolution_w, TextureFormat::R32Float);

        let u_solid = new_texture_storage_3d(images, resolution_u, TextureFormat::R32Float);
        let v_solid = new_texture_storage_3d(images, resolution_v, TextureFormat::R32Float);
        let w_solid = new_texture_storage_3d(images, resolution_w, TextureFormat::R32Float);

        let levelset_air0 = new_texture_storage_3d(images, resolution, TextureFormat::R32Float);
        let levelset_air1 = new_texture_storage_3d(images, resolution, TextureFormat::R32Float);
        let levelset_solid = new_texture_storage_3d(images, resolution, TextureFormat::R32Float);
        let area_fraction_solid =
            new_texture_storage_3d(images, resolution_uvw, TextureFormat::Rgba32Float);

        let p = new_texture_storage_3d(images, resolution, TextureFormat::R32Float);
        let div = new_texture_storage_3d(images, resolution, TextureFormat::R32Float);

        let in_is_u_valid = new_texture_storage_3d(images, resolution_u, TextureFormat::R32Sint);
        let in_is_v_valid = new_texture_storage_3d(images, resolution_v, TextureFormat::R32Sint);
        let in_is_w_valid = new_texture_storage_3d(images, resolution_w, TextureFormat::R32Sint);
        let out_is_u_valid = new_texture_storage_3d(images, resolution_u, TextureFormat::R32Sint);
        let out_is_v_valid = new_texture_storage_3d(images, resolution_v, TextureFormat::R32Sint);
        let out_is_w_valid = new_texture_storage_3d(images, resolution_w, TextureFormat::R32Sint);

        Self {
            u0,
            v0,
            w0,
            u1,
            v1,
            w1,
            u_solid,
            v_solid,
            w_solid,
            levelset_air0,
            levelset_air1,
            levelset_solid,
            area_fraction_solid,
            p,
            div,
            in_is_u_valid,
            in_is_v_valid,
            in_is_w_valid,
            out_is_u_valid,
            out_is_v_valid,
            out_is_w_valid,
        }
    }
}

pub fn setup_fluid_resources(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    grid_length: Res<FluidGridLength>,
    fluids: Query<(Entity, &EulerFluid3d, &Transform, &ReinitializeMethod), Added<EulerFluid3d>>,
) {
    for (entity, fluid3d, transform, reinitialize_method) in &fluids {
        let resolution = fluid3d.resolution;
        let fluid_uniform = FluidUniform {
            dx: grid_length.0,
            dt: 0.0,
            rho: fluid3d.rho,
            gravity: fluid3d.gravity,
            transform: transform.to_matrix(),
            resolution: fluid3d.resolution,
        };

        let resources = FluidResources::new(&mut images, resolution);

        let initialize = InitializeResourcesResource::new(&resources);
        let update_solid = UpdateSolidResource::new(&resources);
        let update_area_fraction_solid = UpdateAreaFractionResource::new(&resources);
        let advect_velocity = AdvectVelocityResource::new(&resources);
        let apply_forces = ApplyForcesResource::new(&resources, &mut buffers);
        let divergence = DivergenceResource::new(&resources);
        let gauss_seidel = GaussSeidelResource::new(&resources);
        setup_multigrid_resources(&mut commands, entity, resolution, &resources, &mut images);
        let solve_u = SolveUResource::new(&resources);
        let solve_v = SolveVResource::new(&resources);
        let solve_w = SolveWResource::new(&resources);
        ExtrapolateVelocityResource::setup(
            &mut commands,
            entity,
            resolution,
            &resources,
            &mut images,
        );
        let advect_levelset = AdvectLevelSetResource::new(&resources);
        reinitialize_levelset::setup(
            &mut commands,
            entity,
            &mut images,
            resolution,
            &resources,
            reinitialize_method,
        );
        let update_fluid_source = UpdateFluidSourceResource::new(&resources);

        commands.entity(entity).insert((
            fluid_uniform,
            resources,
            initialize,
            update_solid,
            update_area_fraction_solid,
            advect_velocity,
            apply_forces,
            divergence,
            gauss_seidel,
            solve_u,
            solve_v,
            solve_w,
            advect_levelset,
            update_fluid_source,
        ));
    }
}
