extern crate bevy_eulerian_fluid;

use avian2d::PhysicsPlugins;
use bevy::{
    asset::AssetMetaCheck,
    prelude::*,
    render::{
        settings::{Backends, RenderCreation, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_eulerian_fluid::{
    material::VelocityMaterial,
    settings::{FluidSettings, FluidTextures},
    FluidPlugin,
};
use example_utils::{fps_counter::FpsCounterPlugin, mouse_motion, overlay::OverlayPlugin};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;
const LENGTH_UNIT: f32 = 10.0;

fn main() {
    // [workaround] Asset meta files cannot be found on browser.
    // see also: https://github.com/bevyengine/bevy/issues/10157
    let meta_check = if cfg!(target_arch = "wasm32") {
        AssetMetaCheck::Never
    } else {
        AssetMetaCheck::Always
    };

    let _app = App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (WIDTH, HEIGHT).into(),
                        title: "fluid component".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
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
        .add_plugins(FluidPlugin::new(LENGTH_UNIT))
        .add_plugins(PhysicsPlugins::default().with_length_unit(LENGTH_UNIT))
        .add_plugins((FpsCounterPlugin, OverlayPlugin::<16>))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (mouse_motion, on_fluid_setup))
        .run();
}

fn setup_scene(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    info!("initialize scene.");
    commands.spawn(Camera2d);

    let size = 128u32;
    for i in 0..4 {
        for j in 0..2 {
            let mesh = meshes.add(Rectangle::from_size(Vec2::splat(size as f32)));
            let translation = Vec3::new(
                (i * size) as f32 * 1.1 - size as f32 * 1.6,
                (j * size) as f32 * 1.1 - size as f32 * 0.8,
                0.0,
            );
            commands.spawn((
                FluidSettings {
                    rho: 99.7, // water density in 2D
                    gravity: Vec2::ZERO,
                    size: UVec2::splat(size),
                    initial_fluid_level: 1.0,
                },
                Transform::default().with_translation(translation),
                Mesh2d(mesh),
            ));
        }
    }

    commands.spawn((
        Text::new("V: Toggle Velocity Overlay"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor::WHITE,
    ));
}

fn on_fluid_setup(
    mut commands: Commands,
    query: Query<(Entity, &FluidTextures), Added<FluidTextures>>,
    mut materials: ResMut<Assets<VelocityMaterial>>,
) {
    for (entity, fluid_texture) in &query {
        let material = materials.add(VelocityMaterial {
            u_range: Vec2::new(-10.0, 10.0),
            v_range: Vec2::new(-10.0, 10.0),
            u: fluid_texture.u.clone(),
            v: fluid_texture.v.clone(),
        });

        commands.entity(entity).insert(MeshMaterial2d(material));
    }
}
