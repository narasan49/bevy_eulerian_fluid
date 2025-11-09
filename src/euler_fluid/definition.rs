use avian2d::{physics_transform::PhysicsTransformSystems, prelude::Physics};
use bevy::{
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        extract_resource::ExtractResource,
        render_resource::{AsBindGroup, ShaderType, UniformBuffer},
        storage::ShaderStorageBuffer,
    },
};

pub const MAX_SOLIDS: usize = 256;

pub struct FluidParametersPlugin;

impl Plugin for FluidParametersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPostUpdate,
            update_simulation_uniform.after(PhysicsTransformSystems::Propagate),
        );
    }

    fn finish(&self, app: &mut App) {
        app.init_resource::<FluidTimeStep>();
    }
}

/// Setting for fluid simulation. By spawning fluid settings, components required to the simulation will be spawned and the simulation will start.
/// Simulation result can be found on [`VelocityTextures`].
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
///     definition::{FluidSettings, VelocityTextures},
/// };
///
/// // On Startup
/// fn setup_scene(mut commands: Commands) {
///     commands.spawn(FluidSettings {
///         rho: 1.293f32,
///         gravity: Vec2::ZERO,
///         size: (512, 512),
///         initial_fluid_level: 1.0f32, // filled with fluid
///     });
/// }
///
/// // On Update
/// fn on_fluid_setup(
///     mut commands: Commands,
///     query: Query<(Entity, &VelocityTextures), Added<VelocityTextures>>,
///     mut meshes: ResMut<Assets<Mesh>>,
///     mut materials: ResMut<Assets<VelocityMaterial>>,
/// ) {
///     // Spawn a mesh to visualize fluid simulation.
///     for (entity, velocity_texture) in &query {
///         let mesh = meshes.add(Rectangle::default());
///         let material = materials.add(VelocityMaterial {
///             u_range: Vec2::new(-10.0, 10.0),
///             v_range: Vec2::new(-10.0, 10.0),
///             u: velocity_texture.u0.clone(),
///             v: velocity_texture.v0.clone(),
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
#[require(Transform)]
pub struct FluidSettings {
    pub rho: f32,
    pub gravity: Vec2,
    pub size: (u32, u32),
    pub initial_fluid_level: f32,
}

#[derive(Resource, Clone, Copy, ExtractResource)]
pub struct FluidTimeStep(pub f32);

impl FromWorld for FluidTimeStep {
    fn from_world(world: &mut World) -> Self {
        let physics_time = world.resource::<Time<Physics>>();
        info!("physics_time: {physics_time:?}");
        let time = world.resource::<Time>();
        info!("time: {time:?}");
        Self(physics_time.delta_secs())
    }
}

#[derive(Resource, Clone, Copy)]
pub struct FluidGridLength(pub f32);

impl Default for FluidGridLength {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Component, ExtractComponent, ShaderType, Clone, Copy, Default)]
pub struct SimulationUniform {
    pub dx: f32,
    pub dt: f32,
    pub rho: f32,
    pub gravity: Vec2,
    pub initial_fluid_level: f32,
    pub fluid_transform: Mat4,
    pub size: Vec2,
}

fn update_simulation_uniform(
    mut query: Query<(&mut SimulationUniform, &FluidSettings, &Transform)>,
    time_step: Res<FluidTimeStep>,
    grid_length: Res<FluidGridLength>,
) {
    for (mut uniform, settings, transform) in &mut query {
        uniform.dx = grid_length.0;
        uniform.dt = time_step.0;
        uniform.rho = settings.rho;
        uniform.gravity = settings.gravity;
        uniform.initial_fluid_level = settings.initial_fluid_level;
        uniform.fluid_transform = transform.to_matrix();
        uniform.size = Vec2::new(settings.size.0 as f32, settings.size.1 as f32);
    }
}

/// Fluid velocity field.
/// To retreive simulation result, please use u0 and v0.
/// u1, v1 are intermediate velocities used for simulation.
/// * u0: x-ward velocity with size of (size.0 + 1, size.1).
/// * v0: y-ward velocity with size of (size.0, size.1 + 1).
/// * u1: intermediate x-ward velocity with size of (size.0 + 1, size.1).
/// * v1: intermediate y-ward velocity with size of (size.0, size.1 + 1).
#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct VelocityTextures {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub u0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub v0: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadWrite)]
    pub u1: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, access = ReadWrite)]
    pub v1: Handle<Image>,
}

/// Textures for x-ward velocities.
#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct VelocityTexturesU {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub u0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub u1: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadWrite)]
    pub u_solid: Handle<Image>,
}

/// Textures for y-ward velocities.
#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct VelocityTexturesV {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub v0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub v1: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadWrite)]
    pub v_solid: Handle<Image>,
}

/// Textures for intermediate velocities.
#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct VelocityTexturesIntermediate {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub u1: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadOnly)]
    pub v1: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct SolidVelocityTextures {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub u_solid: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub v_solid: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct SolidCenterTextures {
    // levelset between solid (<0) vs fluid or empty air (>=0).
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub levelset_solid: Handle<Image>,
    #[storage_texture(1, image_format = R32Sint, access = ReadWrite)]
    pub solid_id: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct PressureTextures {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub p0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub p1: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct DivergenceTextures {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub div: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct LevelsetTextures {
    // levelset between empty air (>=0) vs fluid or solid (<0).
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub levelset_air0: Handle<Image>,
    // intermediate levelset between empty air (>=0) vs fluid or solid (<0).
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub levelset_air1: Handle<Image>,
    // levelset between solid (<0) vs fluid or empty air (>=0).
    #[storage_texture(2, image_format = R32Float, access = ReadWrite)]
    pub levelset_solid: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct LocalForces {
    #[storage(0, read_only, visibility(compute))]
    pub forces: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub positions: Handle<ShaderStorageBuffer>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct SolidForcesBins {
    #[storage(0, visibility(compute))]
    pub bins_force_x: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(compute))]
    pub bins_force_y: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub bins_torque: Handle<ShaderStorageBuffer>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct SampleForcesResource {
    #[storage(0, visibility(compute))]
    pub bins_force_x: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(compute))]
    pub bins_force_y: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub bins_torque: Handle<ShaderStorageBuffer>,
    #[storage_texture(3, image_format = R32Float, access = ReadWrite)]
    pub levelset_solid: Handle<Image>,
    #[storage_texture(4, image_format = R32Sint, access = ReadOnly)]
    pub solid_id: Handle<Image>,
    #[storage_texture(5, image_format = R32Float, access = ReadOnly)]
    pub p1: Handle<Image>,
}

#[derive(Clone, Copy, Default, ShaderType)]
pub struct Force {
    pub force: Vec2,
    pub torque: f32,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct ForcesToSolid {
    #[storage(0, visibility(compute))]
    pub forces: Handle<ShaderStorageBuffer>,
}

#[derive(Resource, Clone, ExtractResource, AsBindGroup)]
pub struct SolidObstaclesBuffer {
    #[storage(0, read_only, visibility(compute))]
    pub obstacles: Handle<ShaderStorageBuffer>,
}

#[derive(Component, Clone, ExtractComponent)]
pub struct SolidEntities {
    pub entities: Vec<Entity>,
}

impl FromWorld for SolidObstaclesBuffer {
    fn from_world(world: &mut World) -> Self {
        let mut buffers = world.resource_mut::<Assets<ShaderStorageBuffer>>();
        let obstacles = buffers.add(ShaderStorageBuffer::from(vec![0; 0]));
        Self { obstacles }
    }
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct JumpFloodingSeedsTextures {
    /// Note: Only R32Float, R32Sint, and R32Uint storage textures can have ReadWrite access on WebGPU.
    /// https://webgpufundamentals.org/webgpu/lessons/webgpu-storage-textures.html
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub jump_flooding_seeds_x: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub jump_flooding_seeds_y: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, ShaderType)]
pub struct JumpFloodingUniform {
    pub step: u32,
}

#[derive(Component)]
pub struct JumpFloodingUniformBuffer {
    pub buffer: Vec<UniformBuffer<JumpFloodingUniform>>,
}

#[derive(Bundle)]
pub(crate) struct FluidSimulationBundle {
    pub velocity_textures: VelocityTextures,
    pub velocity_textures_u: VelocityTexturesU,
    pub velocity_textures_v: VelocityTexturesV,
    pub velocity_textures_intermediate: VelocityTexturesIntermediate,
    pub solid_velocity_textures: SolidVelocityTextures,
    pub pressure_textures: PressureTextures,
    pub divergence_textures: DivergenceTextures,
    pub levelset_textures: LevelsetTextures,
    pub local_forces: LocalForces,
    pub jump_flooding_seeds_textures: JumpFloodingSeedsTextures,
    pub solid_forces_bins: SolidForcesBins,
    pub forces_to_solid: ForcesToSolid,
    pub solid_center_textures: SolidCenterTextures,
    pub sample_forces_resource: SampleForcesResource,
}
