extern crate bevy_eulerian_fluid;

use avian2d::PhysicsPlugins;
use bevy::{camera::ScalingMode, input::common_conditions::input_just_pressed, prelude::*};

use bevy_eulerian_fluid::{
    diagnostics::FluidDiagnosticsPlugin,
    fluid_source::{FluidSource, FluidSourceMode, FluidSourceOneshot, FluidSourceShape},
    projection::{multi_grid::MultiGridConfig, ProjectionMethod},
    settings::{FluidSettings, FluidTextures},
    FluidPlugin,
};
use example_utils::{
    material::{BackgroundMaterial, ExampleMaterialsPlugin, LevelsetMaterial},
    mouse_motion, ExampleDefaultPlugins,
};

const SIZE: UVec2 = UVec2::splat(256);
const LENGTH_UNIT: f32 = 10.0;

fn main() {
    let mut app = App::new();

    app.add_plugins(ExampleDefaultPlugins)
        .add_plugins(FluidPlugin::new(LENGTH_UNIT))
        .add_plugins(PhysicsPlugins::default().with_length_unit(LENGTH_UNIT))
        .add_plugins((FluidDiagnosticsPlugin, ExampleMaterialsPlugin))
        .add_systems(Startup, (setup_scene, setup_fluid))
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
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 2.4 * SIZE.x as f32,
                min_height: 1.2 * SIZE.y as f32,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn setup_fluid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
) {
    let fluid_domain_rectangle = Rectangle::from_size(SIZE.as_vec2());
    let mesh = meshes.add(fluid_domain_rectangle);
    commands
        .spawn((
            FluidSettings {
                rho: 99.7, // water density in 2D
                gravity: Vec2::Y * 9.8,
                size: SIZE,
            },
            ProjectionMethod::MultiGrid(MultiGridConfig::default()),
            Mesh2d(mesh),
            Transform::default()
                .with_translation((SIZE.as_vec2() * Vec2::new(-0.5, 0.0)).extend(0.0)),
        ))
        .with_children(|commands| {
            commands.spawn((
                FluidSource {
                    active: true,
                    mode: FluidSourceMode::Source,
                },
                Transform::from_translation((Vec2::new(0.0, -0.1) * SIZE.as_vec2()).extend(0.0)),
                FluidSourceShape::Circle {
                    radius: SIZE.as_vec2().min_element() * 0.1,
                },
                FluidSourceOneshot,
            ));

            commands.spawn((
                Mesh2d(meshes.add(fluid_domain_rectangle)),
                Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
                MeshMaterial2d(materials.add(BackgroundMaterial {})),
            ));
        });
}

fn on_fluid_setup(
    mut commands: Commands,
    query: Query<(Entity, &FluidTextures), Added<FluidTextures>>,
    mut materials: ResMut<Assets<LevelsetMaterial>>,
) {
    for (entity, fluid_textures) in &query {
        let material = materials.add(LevelsetMaterial {
            levelset: fluid_textures.levelset_air.clone(),
            base_color: Vec3::new(0.5, 0.78, 0.83),
        });

        commands.entity(entity).insert(MeshMaterial2d(material));
    }
}

fn reset_scene(mut commands: Commands, q_fluids: Query<Entity, With<FluidSettings>>) {
    for entity in &q_fluids {
        commands.entity(entity).despawn();
    }
    commands.run_system_cached(setup_fluid);
}
