use bevy::{prelude::*, render::extract_component::ExtractComponent};

use crate::{apply_forces::ForcesToFluid, fluid_status::FluidStatus};

/// Setting for fluid simulation. By spawning fluid settings, components required to the simulation will be spawned and the simulation will start.
/// Simulation result can be found on [`FluidTextures`].
/// # Arguments
/// * `size`: The size of 2D simulation domain in pixels. The size is recommended to be same between each dimension and to be multiple of 64 pixels.
/// * `rho`: The density of fluid in unit of [kg/m^2]. Currently, uniform density is supported only.
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
/// fn setup_scene(
///     mut commands: Commands,
///     mut meshes: ResMut<Assets<Mesh>>,
/// ) {
///     let mesh = meshes.add(Rectangle::from_size(Vec2::splat(512.0)));
///     commands.spawn((
///         FluidSettings {
///             rho: 99.7, // water density in 2D
///             gravity: Vec2::ZERO,
///             size: UVec2::splat(512),
///             initial_fluid_level: 1.0, // filled with fluid
///         },
///         Mesh2d(mesh),
///     ));
/// }
///
/// // On Update
/// fn on_fluid_setup(
///     mut commands: Commands,
///     query: Query<(Entity, &FluidTextures), Added<FluidTextures>>,
///     mut materials: ResMut<Assets<VelocityMaterial>>,
/// ) {
///     // Spawn a mesh to visualize fluid simulation.
///     for (entity, fluid_texture) in &query {
///         let material = materials.add(VelocityMaterial {
///             u_range: Vec2::new(-10.0, 10.0),
///             v_range: Vec2::new(-10.0, 10.0),
///             u: fluid_texture.u.clone(),
///             v: fluid_texture.v.clone(),
///         });
///         commands.entity(entity).insert((
///             MeshMaterial2d(material),
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

/// Resultant textures from fluid simulation. These textures are automatically created when [`FluidSettings`] is spawned. See [`FluidSettings`] for the usage.
/// # Textures
/// * **`u`**: x-component velocity with size [`FluidSettings::size`] + (1, 0). Format: R32Float.
/// * **`v`**: y-component velocity with size [`FluidSettings::size`] + (0, 1). Format: R32Float.
/// * **`u_solid`**: x-component velocity of solid boundary with size [`FluidSettings::size`] + (1, 0). Format: R32Float.
/// * **`v_solid`**: y-component velocity of solid boundary with size [`FluidSettings::size`] + (0, 1). Format: R32Float.
/// * **`levelset_air`**: levelset between empty air (>=0) vs fluid or solid (<0) with size [`FluidSettings::size`]. Format: R32Float.
/// * **`levelset_solid`**: levelset between solid (<0) vs fluid or empty air (>=0) with size [`FluidSettings::size`]. Format: R32Float.
///
/// # Notes
/// A staggered (MAC) grid is used for the simulation.
/// * X-component velocity `u` is located on the x-faces of each cell
/// * Y-component velocity `v` is located on the y-faces of each cell.
/// * Levelset values are located on cell centers.
#[derive(Component)]
pub struct FluidTextures {
    pub u: Handle<Image>,
    pub v: Handle<Image>,
    pub u_solid: Handle<Image>,
    pub v_solid: Handle<Image>,
    pub levelset_air: Handle<Image>,
    pub levelset_solid: Handle<Image>,
}
