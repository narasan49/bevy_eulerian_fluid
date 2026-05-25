#import euler_fluid_3d::interp::{trilinear, trilinear_x, trilinear_y, trilinear_z}
#import euler_fluid_3d::workgroup_shape::WG_SIZE
#import euler_fluid_3d::fluid_uniform::FluidUniform

@group(0) @binding(0) var u0: texture_storage_3d<r32float, read>;
@group(0) @binding(1) var v0: texture_storage_3d<r32float, read>;
@group(0) @binding(2) var w0: texture_storage_3d<r32float, read>;
@group(0) @binding(3) var levelset_air0: texture_storage_3d<r32float, read>;
@group(0) @binding(4) var levelset_air1: texture_storage_3d<r32float, write>;

@group(1) @binding(0) var<uniform> constants: FluidUniform;

@compute @workgroup_size(WG_SIZE.x, WG_SIZE.y, WG_SIZE.z)
fn advect_levelset(
    @builtin(global_invocation_id) gid: vec3u,
) {
    let dim = textureDimensions(levelset_air0);
    if any(gid >= dim) {
        return;
    }
    let idx = vec3i(gid);
    let x = vec3f(idx);

    let dt = constants.dt;
    var backtraced_x = backtrace(u0, v0, w0, x, dt);
    backtraced_x = clamp(backtraced_x, vec3f(0.0), vec3f(dim) - vec3f(1.0));

    var new_level = trilinear(levelset_air0, backtraced_x, vec3f(0.0));
    if abs(new_level) < 3.0 {
        let base_idx = vec3i(backtraced_x);
        let t = backtraced_x - vec3f(base_idx);
        new_level = cubic3d(base_idx, levelset_air0, t);
    }
    
    if abs(new_level) < 1000.0 {
        textureStore(levelset_air1, idx, vec4f(new_level, 0.0, 0.0, 0.0));
    } 
}

fn backtrace(
    u: texture_storage_3d<r32float, read>,
    v: texture_storage_3d<r32float, read>,
    w: texture_storage_3d<r32float, read>,
    x: vec3f,
    dt: f32,
) -> vec3f {
    let velocity = vec3f(trilinear_x(u, x), trilinear_y(v, x), trilinear_z(w, x));
    let x_mid = x - vec3f(0.5 * dt) * velocity;
    let velocity_mid = vec3f(trilinear_x(u, x_mid), trilinear_y(v, x_mid), trilinear_z(w, x_mid));

    return x - dt * velocity_mid;
}

fn cubic1d_x(
    base_idx: vec3i,
    texture: texture_storage_3d<r32float, read>,
    t: f32
) -> f32 {
    let y0 = textureLoad(texture, base_idx - vec3i(1, 0, 0)).x;
    let y1 = textureLoad(texture, base_idx).x;
    let y2 = textureLoad(texture, base_idx + vec3i(1, 0, 0)).x;
    let y3 = textureLoad(texture, base_idx + vec3i(2, 0, 0)).x;

    return cubic1d(vec4f(y0, y1, y2, y3), t);
}

// y: values at points [-1, 0, 1, 2]
// t: interpolant in range [0, 1]
fn cubic1d(y: vec4f, t: f32) -> f32 {
    let dydx1 = 0.5 * (y.z - y.x);
    let dydx2 = 0.5 * (y.w - y.y);

    let a0 = y.y;
    let a1 = dydx1;
    let a2 = -2.0 * dydx1 - dydx2 + 3.0 * (y.z - y.y);
    let a3 = dydx1 + dydx2 - 2.0 * (y.z - y.y);

    return a3 * t * t * t + a2 * t * t + a1 * t + a0;
}

fn cubic2d(
    base_idx: vec3i,
    texture: texture_storage_3d<r32float, read>,
    t: vec2f,
) -> f32 {
    let y0 = cubic1d_x(base_idx + vec3i(0, -1, 0), texture, t.x);
    let y1 = cubic1d_x(base_idx, texture, t.x);
    let y2 = cubic1d_x(base_idx + vec3i(0, 1, 0), texture, t.x);
    let y3 = cubic1d_x(base_idx + vec3i(0, 2, 0), texture, t.x);

    return cubic1d(vec4f(y0, y1, y2, y3), t.y);
}

fn cubic3d(
    base_idx: vec3i,
    texture: texture_storage_3d<r32float, read>,
    t: vec3f,
) -> f32 {
    let z0 = cubic2d(base_idx + vec3i(0, 0, -1), texture, t.xy);
    let z1 = cubic2d(base_idx, texture, t.xy);
    let z2 = cubic2d(base_idx + vec3i(0, 0, 1), texture, t.xy);
    let z3 = cubic2d(base_idx + vec3i(0, 0, 2), texture, t.xy);

    return cubic1d(vec4f(z0, z1, z2, z3), t.z);
}