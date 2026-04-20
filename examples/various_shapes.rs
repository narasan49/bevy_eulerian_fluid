use avian2d::{
    math::Vector,
    prelude::{ColliderDensity, Gravity, IntoCollider, RigidBody},
    PhysicsPlugins,
};
use bevy::{
    asset::{io::web::WebAssetPlugin, AssetMetaCheck},
    camera::ScalingMode,
    input::common_conditions::input_just_pressed,
    prelude::*,
    render::{
        settings::{Backends, WgpuSettings},
        RenderPlugin,
    },
};

use bevy_eulerian_fluid::{
    fluid_source::{FluidSource, FluidSourceMode, FluidSourceOneshot, FluidSourceShape},
    settings::{FluidSettings, FluidTextures},
    FluidPlugin,
};
use example_utils::{
    fps_counter::FpsCounterPlugin,
    material::{BackGroundMaterial, ExampleMaterialsPlugin, LevelsetMaterial},
    mouse_motion,
    overlay::OverlayPlugin,
    scene_helper::spawn_walls,
};

const SIZE: UVec2 = UVec2::new(512, 256);
const LENGTH_UNIT: f32 = 50.0;

fn main() {
    let mut app = App::new();
    // [workaround] Asset meta files cannot be found on browser.
    // see also: https://github.com/bevyengine/bevy/issues/10157
    let meta_check = if cfg!(target_arch = "wasm32") {
        AssetMetaCheck::Never
    } else {
        AssetMetaCheck::Always
    };

    app.add_plugins(
        DefaultPlugins
            .set(RenderPlugin {
                render_creation: bevy::render::settings::RenderCreation::Automatic(WgpuSettings {
                    backends: Some(Backends::DX12 | Backends::BROWSER_WEBGPU),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                meta_check,
                ..default()
            })
            .set(WebAssetPlugin {
                silence_startup_warning: true,
            }),
    )
    .add_plugins(FluidPlugin::new(LENGTH_UNIT))
    .add_plugins(PhysicsPlugins::default().with_length_unit(LENGTH_UNIT))
    .add_plugins((
        FpsCounterPlugin,
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
    );

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
    mut materials: ResMut<Assets<BackGroundMaterial>>,
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
                    active: true,
                    mode: FluidSourceMode::Source,
                },
                Transform::from_translation((Vec2::new(0.0, -0.15) * SIZE.as_vec2()).extend(0.0)),
                FluidSourceShape::Aabb {
                    half_size: 0.5 * Vec2::new(1.0, 0.7) * SIZE.as_vec2(),
                },
                FluidSourceOneshot,
            ));
        });

    commands.spawn((
        Mesh2d(meshes.add(fluid_domain_rectangle)),
        Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
        MeshMaterial2d(materials.add(BackGroundMaterial {})),
    ));
}

fn setup_rigid_bodies(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    let circle = Circle::new(50.0);
    let mesh = meshes.add(circle);
    let material = materials.add(ColorMaterial {
        color: Color::WHITE,
        texture: Some(asset_server.load("https://raw.githubusercontent.com/bevyengine/bevy/release-0.17.2/assets/branding/icon.png")),
        ..default()
    });

    commands.spawn((
        Mesh2d(mesh.clone()),
        MeshMaterial2d(material.clone()),
        Transform::from_translation((SIZE.as_vec2() * Vec2::new(-0.2, 0.3)).extend(1.0)),
        circle.collider(),
        RigidBody::Dynamic,
        ColliderDensity(0.2),
    ));

    let rectangle = Rectangle::new(100.0, 30.0);
    let rectangle_mesh = meshes.add(rectangle);
    let material = materials.add(Color::srgb(0.0, 1.0, 0.0));
    commands.spawn((
        Mesh2d(rectangle_mesh.clone()),
        MeshMaterial2d(material.clone()),
        Transform::from_translation((SIZE.as_vec2() * Vec2::new(0.2, 0.1)).extend(1.0)),
        rectangle.collider(),
        RigidBody::Dynamic,
        ColliderDensity(0.5),
    ));

    let rectangle = Rectangle::new(10.0, 10.0);
    let rectangle_mesh = meshes.add(rectangle);
    commands.spawn((
        Mesh2d(rectangle_mesh.clone()),
        MeshMaterial2d(material.clone()),
        Transform::from_translation((SIZE.as_vec2() * Vec2::new(0.0, 0.1)).extend(1.0)),
        rectangle.collider(),
        RigidBody::Dynamic,
        ColliderDensity(0.9),
    ));

    let rectangle = Rectangle::new(5.0, 20.0);
    let rectangle_mesh = meshes.add(rectangle);
    commands.spawn((
        Mesh2d(rectangle_mesh.clone()),
        MeshMaterial2d(material.clone()),
        Transform::from_translation((SIZE.as_vec2() * Vec2::new(-0.4, 0.1)).extend(1.0)),
        rectangle.collider(),
        RigidBody::Dynamic,
        ColliderDensity(2.0),
    ));

    let sqrt3 = 3f32.sqrt();
    let triangle = Triangle2d::new(
        Vec2::new(0.0, sqrt3 * 0.25) * 50.0,
        Vec2::new(-0.5, -sqrt3 * 0.125) * 50.0,
        Vec2::new(0.5, -sqrt3 * 0.125) * 50.0,
    );
    let triangle_mesh = meshes.add(triangle);
    let triangle_material = materials.add(Color::srgb(1.0, 0.0, 0.0));
    commands.spawn((
        Mesh2d(triangle_mesh.clone()),
        MeshMaterial2d(triangle_material.clone()),
        Transform::from_translation((SIZE.as_vec2() * Vec2::new(0.4, 0.3)).extend(1.0)),
        triangle.collider(),
        RigidBody::Dynamic,
        ColliderDensity(1.0),
    ));

    commands.spawn((
        Mesh2d(triangle_mesh.clone()),
        MeshMaterial2d(triangle_material.clone()),
        Transform::from_translation((SIZE.as_vec2() * Vec2::new(0.2, 0.1)).extend(1.0)),
        triangle.collider(),
        RigidBody::Dynamic,
        ColliderDensity(1.9),
    ));

    let capsule = Capsule2d::new(10.0, 30.0);
    let capsule_mesh = meshes.add(capsule);
    let capsule_material = materials.add(Color::srgb(1.0, 1.0, 0.0));
    commands.spawn((
        Mesh2d(capsule_mesh.clone()),
        MeshMaterial2d(capsule_material.clone()),
        Transform::from_translation((SIZE.as_vec2() * Vec2::new(-0.2, 0.1)).extend(1.0)),
        capsule.collider(),
        RigidBody::Dynamic,
        ColliderDensity(8.0),
    ));
}

fn reset_scene(
    mut commands: Commands,
    q_rigid_bodies: Query<(Entity, &RigidBody), With<RigidBody>>,
    q_fluids: Query<Entity, With<FluidSettings>>,
) {
    for (entity, rigid_body) in &q_rigid_bodies {
        if *rigid_body == RigidBody::Dynamic {
            commands.entity(entity).despawn();
        }
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
