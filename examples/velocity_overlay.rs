use avian2d::prelude::*;
use bevy::{
    asset::{io::web::WebAssetPlugin, AssetMetaCheck},
    input::common_conditions::input_just_pressed,
    prelude::*,
    render::{
        render_resource::AsBindGroup,
        settings::{Backends, WgpuSettings},
        RenderPlugin,
    },
    shader::ShaderRef,
    sprite_render::{Material2d, Material2dPlugin},
};
use bevy_eulerian_fluid::{
    settings::{FluidSettings, FluidTextures},
    velocity_overlay::{VelocityOverlay, VelocityOverlayPlugin},
    FluidPlugin,
};
use example_utils::{fps_counter::FpsCounterPlugin, mouse_motion};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;
const SIZE: UVec2 = UVec2::splat(256);
const LENGTH_UNIT: f32 = 10.0;

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
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (WIDTH, HEIGHT).into(),
                    title: "bevy fluid".to_string(),
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            })
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
    .add_plugins(VelocityOverlayPlugin)
    .add_plugins(FpsCounterPlugin)
    .add_plugins(Material2dPlugin::<CustomMaterial>::default())
    .insert_resource(Gravity(Vec2::NEG_Y * 9.8))
    .add_systems(
        Startup,
        (setup_scene, setup_fluid, setup_walls, setup_rigid_bodies),
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
            scaling_mode: bevy::camera::ScalingMode::FixedHorizontal {
                viewport_width: WIDTH as f32,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    commands.spawn((
        Text::new("R: Reset Scene"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor::WHITE,
    ));
}

fn setup_fluid(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    spawn_fluid(&mut commands, &mut meshes);
}

fn spawn_fluid(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>) {
    let fluid_domain_rectangle = Rectangle::from_size(SIZE.as_vec2());
    commands.spawn((
        FluidSettings {
            rho: 99.70, // water in 2D
            gravity: Vec2::Y * 9.8,
            size: SIZE,
            initial_fluid_level: 0.7,
        },
        Mesh2d(meshes.add(fluid_domain_rectangle)),
        Transform::default(),
        VelocityOverlay {
            max_clamp_speed: 20.0,
            bin_size: UVec2::splat(16),
            color: LinearRgba::GREEN,
        },
    ));
}

fn setup_walls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let wall_thickness = 10.0;
    let wall_rect = Rectangle::new(wall_thickness, SIZE.y as f32);
    let wall_mesh = meshes.add(wall_rect);
    let wall_material = materials.add(Color::srgb(0.5, 0.5, 0.5));

    let floor_rect = Rectangle::new(SIZE.x as f32 + 2.0 * wall_thickness, wall_thickness);
    let floor_mesh = meshes.add(floor_rect);

    commands.spawn((
        Mesh2d(wall_mesh.clone()),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz((SIZE.x as f32 + wall_thickness) * 0.5, 0.0, 0.0),
        RigidBody::Static,
        wall_rect.collider(),
    ));

    commands.spawn((
        Mesh2d(wall_mesh.clone()),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz((SIZE.x as f32 + wall_thickness) * -0.5, 0.0, 0.0),
        RigidBody::Static,
        wall_rect.collider(),
    ));

    commands.spawn((
        Mesh2d(floor_mesh.clone()),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz(0.0, (SIZE.y as f32 + wall_thickness) * -0.5, 0.0),
        RigidBody::Static,
        floor_rect.collider(),
    ));
}

fn setup_rigid_bodies(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    spawn_rigid_bodies(&mut commands, &mut materials, &mut meshes, asset_server);
}

fn spawn_rigid_bodies(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, rigid_body) in &q_rigid_bodies {
        if *rigid_body == RigidBody::Dynamic {
            commands.entity(entity).despawn();
        }
    }
    for entity in &q_fluids {
        commands.entity(entity).despawn();
    }

    spawn_fluid(&mut commands, &mut meshes);
    spawn_rigid_bodies(&mut commands, &mut materials, &mut meshes, asset_server);
}

fn on_fluid_setup(
    mut commands: Commands,
    query: Query<(Entity, &FluidTextures), Added<FluidTextures>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    for (entity, levelset_textures) in &query {
        let material = materials.add(CustomMaterial {
            levelset: levelset_textures.levelset_air.clone(),
            base_color: Vec3::new(0.0, 0.0, 1.0),
            offset: 0.0,
            scale: -100.0,
        });

        commands.entity(entity).insert(MeshMaterial2d(material));
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub levelset: Handle<Image>,
    #[uniform(2)]
    pub base_color: Vec3,
    #[uniform(3)]
    pub offset: f32,
    #[uniform(4)]
    pub scale: f32,
}

impl Material2d for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/visualize/scalar.wgsl".into()
    }
}
