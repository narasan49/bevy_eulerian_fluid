extern crate bevy_eulerian_fluid;

use bevy::{
    asset::AssetMetaCheck,
    math::vec3,
    prelude::*,
    render::{
        settings::{Backends, WgpuSettings},
        RenderPlugin,
    },
};

use bevy_eulerian_fluid::{
    definition::{FluidSettings, VelocityTextures},
    geometry,
    material::VelocityMaterial,
    FluidPlugin,
};
use example_utils::{fps_counter::FpsCounterPlugin, mouse_motion};

const WIDTH: f32 = 640.0;
const HEIGHT: f32 = 360.0;

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
            }),
    )
    .add_plugins(FluidPlugin)
    .add_plugins(FpsCounterPlugin)
    .add_systems(Startup, setup_scene)
    .add_systems(Update, on_fluid_setup)
    .add_systems(Update, mouse_motion);

    app.run();
}

fn setup_scene(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        FluidSettings {
            dx: 1.0f32,
            dt: 0.5f32,
            rho: 1.293f32, // air
            gravity: Vec2::ZERO,
            size: (256, 256),
            initial_fluid_level: 1.0f32,
        },
        Transform::default().with_scale(Vec3::splat(256.0)),
    ));

    commands.spawn((
        geometry::RectangleComponent {
            rectangle: Rectangle {
                half_size: Vec2::splat(16.0),
            },
        },
        Transform::from_translation(vec3(128.0, 128.0, 0.0)),
        geometry::Velocity(Vec2::ZERO),
    ));
}

fn on_fluid_setup(
    mut commands: Commands,
    query: Query<(Entity, &VelocityTextures), Added<VelocityTextures>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VelocityMaterial>>,
) {
    for (entity, velocity_texture) in &query {
        let mesh = meshes.add(Rectangle::default());
        let material = materials.add(VelocityMaterial {
            u_range: Vec2::new(-10.0, 10.0),
            v_range: Vec2::new(-10.0, 10.0),
            u: velocity_texture.u0.clone(),
            v: velocity_texture.v0.clone(),
        });

        commands
            .entity(entity)
            .insert((Mesh2d(mesh), MeshMaterial2d(material)));
    }
}
