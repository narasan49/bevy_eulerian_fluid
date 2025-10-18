pub mod fps_counter;

use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    render::{camera::CameraProjection, storage::ShaderStorageBuffer},
    window::PrimaryWindow,
};
use bevy_eulerian_fluid::euler_fluid::definition::{FluidSettings, LocalForces};

pub fn mouse_motion(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    touches: Res<Touches>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<&OrthographicProjection, With<Camera2d>>,
    q_fluid: Query<(&LocalForces, &FluidSettings, &Transform)>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        let window = q_window.single();
        if let Some(cursor_position) = window.cursor_position() {
            let forces = mouse_motion
                .read()
                .map(|mouse| 5.0 * mouse.delta)
                .collect::<Vec<_>>();

            for (local_forces, settings, transform) in &q_fluid {
                let position = screen_to_mesh_coordinate(
                    cursor_position - transform.translation.xy() * Vec2::new(1.0, -1.0),
                    window,
                    q_camera.single(),
                    Vec2::new(settings.size.0 as f32, settings.size.1 as f32),
                );
                let positions = vec![position; forces.len()];

                let forces_buffer = buffers.get_mut(&local_forces.forces).unwrap();
                forces_buffer.set_data(forces.clone());
                let positions_buffer = buffers.get_mut(&local_forces.positions).unwrap();
                positions_buffer.set_data(positions);
            }
            return;
        }
    }

    let touch_forces = touches
        .iter()
        .map(|touch| 5.0 * touch.delta())
        .collect::<Vec<_>>();
    for (local_forces, settings, transform) in &q_fluid {
        let touch_positions = touches
            .iter()
            .map(|touch| {
                screen_to_mesh_coordinate(
                    touch.position() - transform.translation.xy() * Vec2::new(1.0, -1.0),
                    q_window.single(),
                    q_camera.single(),
                    Vec2::new(settings.size.0 as f32, settings.size.1 as f32),
                )
            })
            .collect::<Vec<_>>();

        let forces_buffer = buffers.get_mut(&local_forces.forces).unwrap();
        forces_buffer.set_data(touch_forces.clone());
        let positions_buffer = buffers.get_mut(&local_forces.positions).unwrap();
        positions_buffer.set_data(touch_positions);
    }
}

fn screen_to_mesh_coordinate(
    position: Vec2,
    window: &Window,
    projection: &OrthographicProjection,
    scale: Vec2,
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

    position_on_mesh.xy() + 0.5 * scale
}
