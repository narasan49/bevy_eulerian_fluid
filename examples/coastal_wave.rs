// extern crate bevy_eulerian_fluid;

use avian2d::{
    math::Vector,
    prelude::{Gravity, IntoCollider, LinearVelocity, RigidBody},
    PhysicsPlugins,
};
use bevy::{camera::ScalingMode, input::common_conditions::input_just_pressed, prelude::*};

use bevy_eulerian_fluid::{
    diagnostics::{component::FluidVolume, FluidDiagnosticsPlugin},
    fluid_source::{
        FluidSource, FluidSourceMode, FluidSourceOneshot, FluidSourceShape, FluidSourceVelocity,
    },
    settings::{FluidSettings, FluidTextures},
    FluidPlugin,
};
use example_utils::{
    material::{BackgroundMaterial, ExampleMaterialsPlugin, LevelsetMaterial},
    mouse_motion,
    overlay::OverlayPlugin,
    scene_helper::spawn_walls,
    ExampleDefaultPlugins,
};

const SIZE: UVec2 = UVec2::new(512, 128);
const LENGTH_UNIT: f32 = 50.0;
const WAVE_SOURCE_V: f32 = 30.0;
const COLOR_SAND: Color = Color::srgb(0.88, 0.83, 0.69);

#[derive(Component)]
struct WaveSource;

#[derive(Component)]
struct FluidLevelAdjustment;

#[derive(Component)]

struct RigidBodyRoot;

fn main() {
    let mut app = App::new();

    app.add_plugins(ExampleDefaultPlugins)
        .add_plugins(FluidPlugin::new(LENGTH_UNIT))
        .add_plugins(PhysicsPlugins::default().with_length_unit(LENGTH_UNIT))
        .add_plugins((
            FluidDiagnosticsPlugin,
            ExampleMaterialsPlugin,
            OverlayPlugin::<16>,
        ))
        .insert_resource(Gravity(Vector::NEG_Y * 9.8))
        .add_systems(
            Startup,
            (
                setup_scene,
                setup_fluid,
                spawn_walls::<{ SIZE.x }, { SIZE.y }>,
                setup_rigid_bodies,
            ),
        )
        .add_systems(Update, on_fluid_setup)
        .add_systems(Update, mouse_motion)
        .add_systems(
            Update,
            reset_scene.run_if(input_just_pressed(KeyCode::KeyR)),
        )
        .add_systems(Update, (update_wave_source, keep_fluid_volume));

    app.run();
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 1.2 * SIZE.x as f32,
                min_height: 1.2 * SIZE.y as f32,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    commands.spawn((
        Text::new("R: Reset Scene\nV: Toggle Velocity Overlay"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor::WHITE,
    ));
}

fn setup_fluid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
) {
    let fluid_domain_rectangle = Rectangle::from_size(SIZE.as_vec2());
    commands
        .spawn((
            FluidSettings {
                rho: 99.7, // water density in 2D
                gravity: Vec2::Y * 9.8,
                size: SIZE,
            },
            Mesh2d(meshes.add(fluid_domain_rectangle.clone())),
            Transform::default(),
        ))
        .with_children(|commands| {
            commands.spawn((
                FluidSource {
                    active: false,
                    mode: FluidSourceMode::Source,
                },
                Transform::from_translation((Vec2::new(-0.4, 0.3) * SIZE.as_vec2()).extend(0.0)),
                FluidSourceShape::Aabb {
                    half_size: Vec2::splat(10.0),
                },
                FluidSourceVelocity(Vec2::new(0.0, 30.0)),
                FluidLevelAdjustment,
            ));

            let dam_half_extent = Vec2::new(SIZE.x as f32 * 0.5, SIZE.y as f32 * 0.2);
            commands.spawn((
                FluidSource {
                    active: true,
                    mode: FluidSourceMode::Source,
                },
                Transform::from_translation((dam_half_extent - 0.5 * SIZE.as_vec2()).extend(0.0)),
                FluidSourceShape::Aabb {
                    half_size: dam_half_extent,
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

fn setup_rigid_bodies(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands
        .spawn((RigidBodyRoot, Transform::default(), Visibility::default()))
        .with_children(|commands| {
            // Wave source
            let rectangle = Rectangle::new(10.0, 50.0);
            let rectangle_mesh = meshes.add(rectangle);
            let material = materials.add(Color::srgb(0.0, 1.0, 0.0));
            commands.spawn((
                Mesh2d(rectangle_mesh.clone()),
                MeshMaterial2d(material.clone()),
                Transform::from_translation((SIZE.as_vec2() * Vec2::new(-0.4, -0.3)).extend(1.0)),
                rectangle.collider(),
                RigidBody::Kinematic,
                LinearVelocity(WAVE_SOURCE_V * Vec2::X),
                WaveSource,
            ));

            // Beach
            let triangle = Triangle2d::new(
                Vec2::new(0.0, 0.0),
                Vec2::new(SIZE.x as f32 * 0.6, 0.0),
                Vec2::new(SIZE.x as f32 * 0.6, SIZE.y as f32 * 0.5),
            );
            let triangle_mesh = meshes.add(triangle);
            let triangle_material = materials.add(COLOR_SAND);
            commands.spawn((
                Mesh2d(triangle_mesh.clone()),
                MeshMaterial2d(triangle_material.clone()),
                Transform::from_translation((SIZE.as_vec2() * Vec2::new(-0.1, -0.5)).extend(1.0)),
                triangle.collider(),
                RigidBody::Static,
            ));
        });
}

fn reset_scene(
    mut commands: Commands,
    q_rigid_bodies: Query<Entity, With<RigidBodyRoot>>,
    q_fluids: Query<Entity, With<FluidSettings>>,
) {
    for entity in &q_rigid_bodies {
        commands.entity(entity).despawn();
    }
    for entity in &q_fluids {
        commands.entity(entity).despawn();
    }

    commands.run_system_cached(setup_fluid);
    commands.run_system_cached(setup_rigid_bodies);
}

fn on_fluid_setup(
    mut commands: Commands,
    query: Query<(Entity, &FluidTextures), Added<FluidTextures>>,
    mut materials: ResMut<Assets<LevelsetMaterial>>,
) {
    for (entity, levelset_textures) in &query {
        let material = materials.add(LevelsetMaterial {
            levelset: levelset_textures.levelset_air.clone(),
            base_color: Vec3::new(0.5, 0.78, 0.83),
        });

        commands.entity(entity).insert(MeshMaterial2d(material));
    }
}

fn update_wave_source(mut query: Query<(&mut LinearVelocity, &Transform), With<WaveSource>>) {
    for (mut velocity, transform) in &mut query {
        // if transform.translation.y >= -0.1 * SIZE.y as f32 {
        //     velocity.0 = -Vec2::Y * WAVE_SOURCE_V;
        // } else if transform.translation.y < -0.2 * SIZE.y as f32 {
        //     velocity.0 = Vec2::Y * WAVE_SOURCE_V;
        // }
        if transform.translation.x >= -0.3 * SIZE.x as f32 {
            velocity.0 = -Vec2::X * WAVE_SOURCE_V;
        } else if transform.translation.x < -0.4 * SIZE.x as f32 {
            velocity.0 = Vec2::X * WAVE_SOURCE_V;
        }
    }
}

fn keep_fluid_volume(
    mut query: Query<(&mut FluidSource, &ChildOf), With<FluidLevelAdjustment>>,
    f_query: Query<(&FluidVolume, &FluidSettings)>,
) {
    for (mut fluid_source, child) in &mut query {
        if let Ok((volume, settings)) = f_query.get(child.parent()) {
            let volume_lower_limit = settings.size.element_product() as f32 * 0.4;
            let volume_upper_limit = settings.size.element_product() as f32 * 0.5;
            if volume.0 < volume_lower_limit {
                fluid_source.active = true;
            } else if volume.0 > volume_upper_limit {
                fluid_source.active = false;
            }
        }
    }
}
