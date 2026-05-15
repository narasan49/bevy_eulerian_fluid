#import euler_fluid_3d::interp::{trilinear_x, trilinear_y, trilinear_z}
#import euler_fluid_3d::workgroup_shape::WG_SIZE
#import euler_fluid_3d::fluid_uniform::FluidUniform;

@group(0) @binding(0) var u0: texture_storage_3d<r32float, read>;
@group(0) @binding(1) var v0: texture_storage_3d<r32float, read>;
@group(0) @binding(2) var w0: texture_storage_3d<r32float, read>;
@group(0) @binding(3) var u1: texture_storage_3d<r32float, write>;
@group(0) @binding(4) var v1: texture_storage_3d<r32float, write>;
@group(0) @binding(5) var w1: texture_storage_3d<r32float, write>;

@group(1) @binding(0) var<uniform> constants: FluidUniform;

@compute @workgroup_size(WG_SIZE.x, WG_SIZE.y, WG_SIZE.z)
fn advect_u(
    @builtin(global_invocation_id) gid: vec3u,
) {
    let dim = textureDimensions(u0);
    if any(gid >= dim) {
        return;
    }
    let x = vec3f(gid) + vec3f(-0.5, 0.0, 0.0);
    let backtraced_x = backtrace(u0, v0, w0, x, constants.dt);
    let dimf = vec3f(dim);
    if inside(backtraced_x, dimf) {
        let backtraced_u = trilinear_x(u0, backtraced_x);
        textureStore(u1, gid, vec4f(backtraced_u, 0.0, 0.0, 0.0));
    } else {
        textureStore(u1, gid, textureLoad(u0, gid));
    }
}

@compute @workgroup_size(WG_SIZE.x, WG_SIZE.y, WG_SIZE.z)
fn advect_v(
    @builtin(global_invocation_id) gid: vec3u,
) {
    let dim = textureDimensions(v0);
    if any(gid >= dim) {
        return;
    }
    let x = vec3f(gid) + vec3f(0.0, -0.5, 0.0);
    let backtraced_x = backtrace(u0, v0, w0, x, constants.dt);
    let dimf = vec3f(dim);
    if inside(backtraced_x, dimf) {
        let backtraced_v = trilinear_x(v0, backtraced_x);
        textureStore(v1, gid, vec4f(backtraced_v, 0.0, 0.0, 0.0));
    } else {
        textureStore(v1, gid, textureLoad(v0, gid));
    }
}

@compute @workgroup_size(WG_SIZE.x, WG_SIZE.y, WG_SIZE.z)
fn advect_w(
    @builtin(global_invocation_id) gid: vec3u,
) {
    let dim = textureDimensions(w0);
    if any(gid >= dim) {
        return;
    }
    let x = vec3f(gid) + vec3f(0.0, 0.0, -0.5);
    let backtraced_x = backtrace(u0, v0, w0, x, constants.dt);
    let dimf = vec3f(dim);
    if inside(backtraced_x, dimf) {
        let backtraced_w = trilinear_x(w0, backtraced_x);
        textureStore(w1, gid, vec4f(backtraced_w, 0.0, 0.0, 0.0));
    } else {
        textureStore(w1, gid, textureLoad(w0, gid));
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

fn inside(x: vec3f, dimf: vec3f) -> bool {
    return all(x >= vec3f(0.0)) && all(x <= dimf - vec3f(1.0));
}