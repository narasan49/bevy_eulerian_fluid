extern crate bevy_eulerian_fluid;
mod ui;

use bevy::{
    asset::AssetMetaCheck,
    core::FrameCount,
    math::vec3,
    prelude::*,
    render::{
        settings::{Backends, WgpuSettings},
        RenderPlugin,
    },
};

use bevy_eulerian_fluid::{
    definition::{FluidSettings, SimulationUniform, VelocityTextures},
    material::VelocityMaterial,
    obstacle::{self},
    FluidPlugin,
};

use example_utils::fps_counter::FpsCounterPlugin;
use ui::{AddButton, GameUiPlugin, ResetButton};

use rand::Rng;

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
    .add_plugins(GameUiPlugin)
    .add_systems(Startup, setup_scene)
    .add_systems(Update, (on_fluid_setup, update, update_geometry))
    .add_systems(Update, (button_update, add_object));

    app.run();
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands
        .spawn((
            PointLight {
                intensity: 1500.0,
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(4.0, 8.0, 4.0),
        ))
        .insert(Name::new("Light"));

    commands.spawn((
        FluidSettings {
            dx: 1.0f32,
            dt: 0.5f32,
            rho: 1.293f32,
            gravity: Vec2::ZERO,
            size: (512, 512),
            initial_fluid_level: 1.0f32,
        },
        Transform::from_translation(Vec3::new(0.0, 1.0, 1.0)),
    ));
}

fn on_fluid_setup(
    mut commands: Commands,
    query: Query<(Entity, &VelocityTextures), Added<VelocityTextures>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VelocityMaterial>>,
) {
    for (entity, velocity_texture) in &query {
        let mesh = meshes.add(Mesh::from(Plane3d::default()));
        let material = materials.add(VelocityMaterial {
            u_range: Vec2::new(-0.01, 0.01),
            v_range: Vec2::new(-0.01, 0.01),
            u: velocity_texture.u0.clone(),
            v: velocity_texture.v0.clone(),
        });
        commands
            .entity(entity)
            .insert((Mesh3d(mesh), MeshMaterial3d(material)));
    }
}

fn update(mut query: Query<&mut SimulationUniform>, _time: Res<Time>) {
    for mut uniform in &mut query {
        uniform.dt = 0.5;
    }
}

fn update_geometry(
    frame: Res<FrameCount>,
    mut object_query: Query<(&obstacle::SolidCircle, &mut Transform, &mut obstacle::Velocity)>,
) {
    let dt = 0.5;
    let t = frame.0 as f32 * dt;
    let freq = 0.1;
    for (_circle, mut transform, mut velocity) in &mut object_query {
        let u = 0.2 * freq * f32::cos(t * freq);
        velocity.x = u;
        transform.translation.x += u * dt;
    }
}

fn button_update(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<ResetButton>)>,
    object_query: Query<Entity, With<obstacle::SolidCircle>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            for entity in object_query.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn add_object(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<AddButton>)>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            let mut rng = rand::thread_rng();
            commands.spawn((
                obstacle::SolidCircle {
                    radius: rng.gen_range(0.02..0.1),
                },
                Transform::from_translation(vec3(
                    rng.gen_range(0.0..1.0),
                    0.0,
                    rng.gen_range(0.0..1.0),
                )),
                obstacle::Velocity(Vec2::ZERO),
            ));
        }
    }
}
