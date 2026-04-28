extern crate bevy_eulerian_fluid;

use avian2d::PhysicsPlugins;
use bevy::{camera::ScalingMode, input::common_conditions::input_just_pressed, prelude::*};

use bevy_eulerian_fluid::{
    diagnostics::FluidDiagnosticsPlugin,
    fluid_source::{FluidSource, FluidSourceMode, FluidSourceOneshot, FluidSourceShape},
    material::VelocityMaterial,
    settings::{FluidSettings, FluidTextures},
    FluidPlugin,
};
use example_utils::{
    material::{ExampleMaterialsPlugin, LevelsetMaterial},
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

fn setup_fluid(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(Rectangle::from_size(SIZE.as_vec2()));
    commands
        .spawn((
            FluidSettings {
                rho: 99.7, // water density in 2D
                gravity: Vec2::Y * 9.8,
                size: SIZE,
            },
            Mesh2d(mesh),
            Transform::default()
                .with_translation((SIZE.as_vec2() * Vec2::new(-0.5, 0.0)).extend(0.0)),
        ))
        .with_child((
            FluidSource {
                active: true,
                mode: FluidSourceMode::Source,
            },
            Transform::from_translation((Vec2::new(0.0, -0.2) * SIZE.as_vec2()).extend(0.0)),
            FluidSourceShape::Aabb {
                half_size: 0.5 * Vec2::new(1.0, 0.6) * SIZE.as_vec2(),
            },
            FluidSourceOneshot,
        ));
}

fn on_fluid_setup(
    mut commands: Commands,
    query: Query<(Entity, &FluidTextures), Added<FluidTextures>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LevelsetMaterial>>,
    mut velocity_materials: ResMut<Assets<VelocityMaterial>>,
) {
    for (entity, fluid_textures) in &query {
        let material = materials.add(LevelsetMaterial {
            levelset: fluid_textures.levelset_air.clone(),
            base_color: Vec3::new(0.5, 0.78, 0.83),
        });

        commands.entity(entity).insert(MeshMaterial2d(material));

        let material_velocity = velocity_materials.add(VelocityMaterial {
            u_range: Vec2::new(-20.0, 20.0),
            v_range: Vec2::new(-20.0, 20.0),
            u: fluid_textures.u.clone(),
            v: fluid_textures.v.clone(),
        });

        let mesh = meshes.add(Rectangle::from_size(SIZE.as_vec2()));
        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(material_velocity),
            Transform::default()
                .with_translation((SIZE.as_vec2() * Vec2::new(0.5, 0.0)).extend(0.0)),
        ));

        // Draw labels for each panel
        commands.spawn((
            Text::new("Left: Surface, Right: Velocity"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor::WHITE,
        ));
    }
}

fn reset_scene(mut commands: Commands, q_fluids: Query<Entity, With<FluidSettings>>) {
    for entity in &q_fluids {
        commands.entity(entity).despawn();
    }
    commands.run_system_cached(setup_fluid);
}
