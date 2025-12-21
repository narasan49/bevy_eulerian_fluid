#import bevy_fluid::fluid_uniform::SimulationUniform;
#import bevy_fluid::coordinate::{interp2d_center, runge_kutta};

@group(0) @binding(0) var u0: texture_storage_2d<r32float, read>;
@group(0) @binding(1) var v0: texture_storage_2d<r32float, read>;
@group(0) @binding(2) var levelset_air0: texture_storage_2d<r32float, read>;
@group(0) @binding(3) var levelset_air1: texture_storage_2d<r32float, write>;

@group(1) @binding(0) var<uniform> constants: SimulationUniform;

@compute @workgroup_size(8, 8, 1)
fn advect_levelset(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let idx = vec2<i32>(invocation_id.xy);
    let x = vec2<f32>(idx);

    let dt = constants.dt;
    let size = textureDimensions(levelset_air0);
    var x_new = runge_kutta(u0, v0, x, dt);
    x_new = clamp(x_new, vec2<f32>(0.0), vec2<f32>(size) - vec2<f32>(1.0));

#ifdef CUBIC
    let base_idx = vec2<i32>(x_new);
    let t = x_new - vec2<f32>(base_idx);
    let new_level = cubic2d(base_idx, levelset_air0, t);
#else
    let new_level = interp2d_center(levelset_air0, x_new);
#endif
    
    textureStore(levelset_air1, idx, vec4<f32>(new_level, 0.0, 0.0, 0.0));
}

fn cubuc1d_x(
    base_idx: vec2<i32>,
    texture: texture_storage_2d<r32float, read>,
    t: f32
) -> f32 {
    let y0 = textureLoad(texture, base_idx - vec2<i32>(1, 0)).x;
    let y1 = textureLoad(texture, base_idx).x;
    let y2 = textureLoad(texture, base_idx + vec2<i32>(1, 0)).x;
    let y3 = textureLoad(texture, base_idx + vec2<i32>(2, 0)).x;

    return cubic1d(vec4<f32>(y0, y1, y2, y3), t);
}

// y: values at points [-1, 0, 1, 2]
// t: interpolant in range [0, 1]
fn cubic1d(y: vec4<f32>, t: f32) -> f32 {
    let dydx1 = 0.5 * (y.z - y.x);
    let dydx2 = 0.5 * (y.w - y.y);

    let a0 = y.y;
    let a1 = dydx1;
    let a2 = -2.0 * dydx1 - dydx2 + 3.0 * (y.z - y.y);
    let a3 = dydx1 + dydx2 - 2.0 * (y.z - y.y);

    return a3 * t * t * t + a2 * t * t + a1 * t + a0;
}

fn cubic2d(
    base_idx: vec2<i32>,
    texture: texture_storage_2d<r32float, read>,
    t: vec2<f32>,
) -> f32 {
    let y0 = cubuc1d_x(base_idx + vec2<i32>(0, -1), texture, t.x);
    let y1 = cubuc1d_x(base_idx, texture, t.x);
    let y2 = cubuc1d_x(base_idx + vec2<i32>(0, 1), texture, t.x);
    let y3 = cubuc1d_x(base_idx + vec2<i32>(0, 2), texture, t.x);

    return cubic1d(vec4<f32>(y0, y1, y2, y3), t.y);
}