pub mod fps_counter;
pub mod material;
pub mod overlay;
pub mod scene_helper;

use bevy::{
    asset::{io::web::WebAssetPlugin, AssetMetaCheck},
    camera::Projection,
    input::mouse::MouseMotion,
    prelude::*,
    render::{
        settings::{Backends, WgpuSettings},
        RenderPlugin,
    },
    window::PrimaryWindow,
};
use bevy_eulerian_fluid::{
    apply_forces::{ForceToFluid, ForcesToFluid},
    settings::FluidSettings,
};

pub struct ExampleDefaultPlugins;

impl Plugin for ExampleDefaultPlugins {
    fn build(&self, app: &mut App) {
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
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: bevy::render::settings::RenderCreation::Automatic(
                        WgpuSettings {
                            backends: Some(Backends::DX12 | Backends::BROWSER_WEBGPU),
                            ..default()
                        },
                    ),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check,
                    ..default()
                })
                .set(WebAssetPlugin {
                    silence_startup_warning: true,
                }),
        );
    }
}

pub fn mouse_motion(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    touches: Res<Touches>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<&Projection, With<Camera2d>>,
    mut q_fluid: Query<(&mut ForcesToFluid, &FluidSettings, &Transform)>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        let window = q_window.single().unwrap();
        if let Some(cursor_position) = window.cursor_position() {
            let forces = mouse_motion
                .read()
                .map(|mouse| 5.0 * mouse.delta)
                .collect::<Vec<_>>();

            for (mut forces_to_fluid, settings, transform) in &mut q_fluid {
                let position = screen_to_mesh_coordinate(
                    cursor_position,
                    window,
                    q_camera.single().unwrap(),
                    settings.size.as_vec2(),
                    transform,
                );
                let positions = vec![position; forces.len()];

                forces_to_fluid.forces = forces
                    .iter()
                    .zip(positions.iter())
                    .map(|(&force, &position)| ForceToFluid { force, position })
                    .collect();
            }
            return;
        }
    } else {
        let touch_forces = touches
            .iter()
            .map(|touch| 5.0 * touch.delta())
            .collect::<Vec<_>>();
        for (mut forces_to_fluid, settings, transform) in &mut q_fluid {
            let touch_positions = touches
                .iter()
                .map(|touch| {
                    screen_to_mesh_coordinate(
                        touch.position(),
                        q_window.single().unwrap(),
                        q_camera.single().unwrap(),
                        settings.size.as_vec2(),
                        transform,
                    )
                })
                .collect::<Vec<_>>();

            forces_to_fluid.forces = touch_forces
                .iter()
                .zip(touch_positions.iter())
                .map(|(&force, &position)| ForceToFluid { force, position })
                .collect();
        }
    }
}

fn screen_to_mesh_coordinate(
    position: Vec2,
    window: &Window,
    projection: &Projection,
    scale: Vec2,
    transform: &Transform,
) -> Vec2 {
    let window_size = window.size();
    let normalized_position = 2.0 * (position - window_size) / window_size + 1.0;
    let inv_proj = projection.get_clip_from_view().inverse();

    let position_on_mesh = inv_proj.mul_vec4(Vec4::new(
        normalized_position.x,
        normalized_position.y,
        0.0,
        1.0,
    ));

    position_on_mesh.xy() + 0.5 * scale - transform.translation.xy() * Vec2::new(1.0, -1.0)
}
