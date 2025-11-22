use bevy::{prelude::*, render::extract_component::ExtractComponent};

use crate::{apply_forces::ForcesToFluid, fluid_status::FluidStatus};

/// Setting for fluid simulation. By spawning fluid settings, components required to the simulation will be spawned and the simulation will start.
/// Simulation result can be found on [`FluidTextures`].
/// # Arguments
/// * `size`: The size of 2D simulation domain in pixels. The size is recommended to be same between each dimension and to be multiple of 64 pixels.
/// * `rho`: The density of fluid in unit of [kg/m^3]. Currently, uniform density is supported only.
/// * `initial_fluid_level`: Initialize fluid level with specified value. the value is valid between 0.0 - 1.0. 0.0 indicates empty and 1.0 indicates the simulation domain is filled with fluid.
/// * `gravity`: Uniform force enforced uniformly to the simulation domain in unit of [m/s^2].
///
/// # Examples
/// ```rust
/// use bevy::{
///     prelude::*,
/// };
/// use bevy_eulerian_fluid::{
///     material::VelocityMaterial,
///     settings::{FluidSettings, FluidTextures},
/// };
///
/// // On Startup
/// fn setup_scene(mut commands: Commands) {
///     commands.spawn(FluidSettings {
///         rho: 1.293f32,
///         gravity: Vec2::ZERO,
///         size: UVec2::splat(512),
///         initial_fluid_level: 1.0f32, // filled with fluid
///     });
/// }
///
/// // On Update
/// fn on_fluid_setup(
///     mut commands: Commands,
///     query: Query<(Entity, &FluidTextures), Added<FluidTextures>>,
///     mut meshes: ResMut<Assets<Mesh>>,
///     mut materials: ResMut<Assets<VelocityMaterial>>,
/// ) {
///     // Spawn a mesh to visualize fluid simulation.
///     for (entity, fluid_texture) in &query {
///         let mesh = meshes.add(Rectangle::default());
///         let material = materials.add(VelocityMaterial {
///             u_range: Vec2::new(-10.0, 10.0),
///             v_range: Vec2::new(-10.0, 10.0),
///             u: fluid_texture.u.clone(),
///             v: fluid_texture.v.clone(),
///         });
///         commands.entity(entity).insert((
///             Mesh2d(mesh),
///             MeshMaterial2d(material),
///             Transform::default().with_scale(Vec3::splat(512.0)),
///         ));
///     }
/// }
/// ```
#[derive(Component, Clone, ExtractComponent)]
#[require(Transform, FluidStatus, ForcesToFluid)]
pub struct FluidSettings {
    pub rho: f32,
    pub gravity: Vec2,
    pub size: UVec2,
    pub initial_fluid_level: f32,
}

#[derive(Resource, Clone, Copy)]
pub struct FluidGridLength(pub f32);

impl Default for FluidGridLength {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Component)]
pub struct FluidTextures {
    pub u: Handle<Image>,
    pub v: Handle<Image>,
    pub u_solid: Handle<Image>,
    pub v_solid: Handle<Image>,
    pub levelset_air: Handle<Image>,
    pub levelset_solid: Handle<Image>,
}
