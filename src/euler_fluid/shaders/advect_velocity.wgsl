#import bevy_fluid::fluid_uniform::SimulationUniform;
#import bevy_fluid::coordinate::{interp2d_edge_x, interp2d_edge_y, runge_kutta};
#import bevy_fluid::levelset_utils::project_onto_surface;

@group(0) @binding(0) var u0: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var v0: texture_storage_2d<r32float, read_write>;
@group(0) @binding(2) var u1: texture_storage_2d<r32float, read_write>;
@group(0) @binding(3) var v1: texture_storage_2d<r32float, read_write>;

@group(2) @binding(0) var<uniform> constants: SimulationUniform;

@compute @workgroup_size(1, 64, 1)
fn advect_u(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let idx = vec2<i32>(invocation_id.xy);
    // backtrace the velocity at the point (i - 0.5, j).
    let x = vec2<f32>(idx) + vec2<f32>(-0.5, 0.0);
    let backtraced_x: vec2<f32> = runge_kutta(u0, v0, x, constants.dt);
    let dim = vec2<f32>(textureDimensions(u0));
    if (!is_inside_domain(backtraced_x, dim)) {
        textureStore(u1, idx, vec4<f32>(0.0, 0.0, 0.0, 0.0));
    } else {
        let backtraced_u = interp2d_edge_x(u0, backtraced_x);
        textureStore(u1, idx, vec4<f32>(backtraced_u, 0.0, 0.0, 0.0));
    }
}

@compute @workgroup_size(64, 1, 1)
fn advect_v(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let idx = vec2<i32>(invocation_id.xy);
    // backtrace the velocity at the point (i, j - 0.5).
    let x = vec2<f32>(idx) + vec2<f32>(0.0, -0.5);
    let backtraced_x: vec2<f32> = runge_kutta(u0, v0, x, constants.dt);
    let dim = vec2<f32>(textureDimensions(v0));
    if (!is_inside_domain(backtraced_x, dim)) {
        textureStore(v1, idx, vec4<f32>(0.0, 0.0, 0.0, 0.0));
    } else {
        let backtraced_v = interp2d_edge_y(v0, backtraced_x);
        textureStore(v1, idx, vec4<f32>(backtraced_v, 0.0, 0.0, 0.0));
    }
}

fn is_inside_domain(
    x: vec2<f32>,
    dim: vec2<f32>,
) -> bool {
    return all(x >= vec2<f32>(0.0)) && all(x <= dim - vec2<f32>(1.0));
}