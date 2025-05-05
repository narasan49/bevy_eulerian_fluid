#import bevy_fluid::fluid_uniform::SimulationUniform;
#import bevy_fluid::coordinate::{interp2d_center, interp2d_edge_x, interp2d_edge_y, runge_kutta};
#import bevy_fluid::levelset_utils::project_onto_surface;

@group(0) @binding(0) var u0: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var v0: texture_storage_2d<r32float, read_write>;
@group(0) @binding(2) var u1: texture_storage_2d<r32float, read_write>;
@group(0) @binding(3) var v1: texture_storage_2d<r32float, read_write>;

@group(1) @binding(2) var levelset_solid: texture_storage_2d<r32float, read_write>;

@group(2) @binding(0) var<uniform> constants: SimulationUniform;

@compute @workgroup_size(1, 64, 1)
fn advect_u(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let backtraced_x: vec2<f32> = runge_kutta(u0, v0, vec2<f32>(x), constants.dt);
    let dim = vec2<f32>(textureDimensions(u0));
    if (backtraced_x.x < 0.0 || backtraced_x.x > dim.x - 1.0 || backtraced_x.y < 0.0 || backtraced_x.y > dim.y - 1.0) {
        textureStore(u1, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
    } else {
        let level = interp2d_center(levelset_solid, backtraced_x + vec2<f32>(0.0, 0.5));
        if (level < 0.0) {
            let levelset_index = vec2<i32>(backtraced_x + vec2<f32>(0.0, 0.5));
            let corrected_backtraced_x = project_onto_surface(levelset_solid, backtraced_x, levelset_index, 0.0);

            let backtraced_u: f32 = interp2d_edge_x(u0, corrected_backtraced_x);
            if (backtraced_x.x < 0.0 || backtraced_x.x > dim.x - 1.0 || backtraced_x.y < 0.0 || backtraced_x.y > dim.y - 1.0) {
                textureStore(u1, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
            } else {
                textureStore(u1, x, vec4<f32>(backtraced_u, 0.0, 0.0, 0.0));
            }
        } else {
            let backtraced_u: f32 = interp2d_edge_x(u0, backtraced_x);
            textureStore(u1, x, vec4<f32>(backtraced_u, 0.0, 0.0, 0.0));
        }
    }
}

@compute @workgroup_size(64, 1, 1)
fn advect_v(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let backtraced_x: vec2<f32> = runge_kutta(u0, v0, vec2<f32>(x), constants.dt);
    let dim = vec2<f32>(textureDimensions(v0));
    if (backtraced_x.x < 0.0 || backtraced_x.x > dim.x - 1.0 || backtraced_x.y < 0.0 || backtraced_x.y > dim.y - 1.0) {
        textureStore(v1, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
    } else {
        let level = interp2d_center(levelset_solid, backtraced_x + vec2<f32>(0.5, 0.0));
        if (level < 0.0) {
            let levelset_index = vec2<i32>(backtraced_x + vec2<f32>(0.5, 0.0));
            let corrected_backtraced_x = project_onto_surface(levelset_solid, backtraced_x, levelset_index, 0.0);

            let backtraced_v: f32 = interp2d_edge_y(v0, corrected_backtraced_x);
            if (backtraced_x.x < 0.0 || backtraced_x.x > dim.x - 1.0 || backtraced_x.y < 0.0 || backtraced_x.y > dim.y - 1.0) {
                textureStore(v1, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
            } else {
                textureStore(v1, x, vec4<f32>(backtraced_v, 0.0, 0.0, 0.0));
            }
        } else {
            let backtraced_v: f32 = interp2d_edge_y(v0, backtraced_x);
            textureStore(v1, x, vec4<f32>(backtraced_v, 0.0, 0.0, 0.0));
        }
    }
}