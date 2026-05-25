extern crate bevy_eulerian_fluid;

use bevy::{
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    input::common_conditions::input_just_pressed,
    prelude::*,
};

use bevy_eulerian_fluid_3d::{
    core::{
        fluid_source::{FluidSource, FluidSourceMode, FluidSourceOneshot, FluidSourceShape},
        projection::{multigrid::MultiGridConfig, ProjectionMethod},
        Fluid3dCorePlugin,
    },
    resource::{EulerFluid3d, FluidResources},
};
use bevy_sdf_marching_cubes::marching_cubes::{resource::MarchingCubes, MarchingCubesPlugin};
use example_utils::{mouse_motion, ExampleDefaultPlugins};

const SIZE: UVec3 = UVec3::splat(64);
const LENGTH_UNIT: f32 = 10.0;

fn main() {
    let mut app = App::new();

    app.add_plugins(ExampleDefaultPlugins)
        .add_plugins(Fluid3dCorePlugin::new(LENGTH_UNIT))
        .add_plugins(MarchingCubesPlugin)
        .add_plugins(FreeCameraPlugin)
        .add_systems(Startup, (setup_scene, setup_fluid, setup_reference_mesh))
        .add_systems(Update, on_fluid_setup)
        .add_systems(Update, mouse_motion)
        .add_systems(
            Update,
            reset_scene.run_if(input_just_pressed(KeyCode::KeyR)),
        );

    app.run();
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        FreeCamera {
            sensitivity: 0.2,
            friction: 25.0,
            walk_speed: 3.0,
            run_speed: 9.0,
            ..default()
        },
        Transform::from_xyz(5.0, 3.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn setup_fluid(mut commands: Commands) {
    commands
        .spawn((
            EulerFluid3d {
                rho: 997.0, // water density
                gravity: -Vec3::Y * 9.8,
                resolution: SIZE,
            },
            ProjectionMethod::MultiGrid(MultiGridConfig::default()),
        ))
        .with_child((
            FluidSource {
                active: true,
                mode: FluidSourceMode::Source,
            },
            Transform::from_translation(Vec3::new(0.0, -0.5, 0.0) * SIZE.as_vec3()),
            FluidSourceShape::Aabb {
                half_size: 0.5 * Vec3::new(0.95, 0.4, 0.95) * SIZE.as_vec3(),
            },
            FluidSourceOneshot,
        ));
}

fn setup_reference_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_length(1.0))),
        Transform::from_translation(Vec3::new(1.0, 0.0, -10.0)),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ));
}

fn on_fluid_setup(
    mut commands: Commands,
    query: Query<(Entity, &EulerFluid3d, &FluidResources), Added<FluidResources>>,
) {
    for (entity, fluid, fluid_textures) in &query {
        commands.entity(entity).insert(MarchingCubes {
            half_size: Vec3::ONE,
            sdf: fluid_textures.levelset_air0.clone(),
            grad_sdf: fluid_textures.grad_levelset_air.clone(),
            resolution: fluid.resolution,
        });
    }
}

fn reset_scene(mut commands: Commands, q_fluids: Query<Entity, With<EulerFluid3d>>) {
    for entity in &q_fluids {
        commands.entity(entity).despawn();
    }
    commands.run_system_cached(setup_fluid);
}
