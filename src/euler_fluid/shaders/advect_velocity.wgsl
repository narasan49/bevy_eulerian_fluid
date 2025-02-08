#import bevy_fluid::fluid_uniform::SimulationUniform;

@group(0) @binding(0) var u0: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var v0: texture_storage_2d<r32float, read_write>;
@group(0) @binding(2) var u1: texture_storage_2d<r32float, read_write>;
@group(0) @binding(3) var v1: texture_storage_2d<r32float, read_write>;

@group(1) @binding(1) var grid_label: texture_storage_2d<r32uint, read_write>;

@group(2) @binding(0) var<uniform> constants: SimulationUniform;

@compute @workgroup_size(1, 64, 1)
fn advect_u(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let x_u = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let label_u = textureLoad(grid_label, x_u - vec2<i32>(0, 1)).r;
    let label_uplus = textureLoad(grid_label, x_u).r;
    // At this point, we don't update the solid velocity. Solid velocity is taken into account in the divergence and pressure-update steps.
    if (label_u == 0 || label_uplus == 0) {
        textureStore(u1, x_u, vec4<f32>(0.0, 0.0, 0.0, 0.0));
    } else {
        let backtraced_x_u: vec2<f32> = runge_kutta(u0, v0, x_u, constants.dt);
        let dim_u = vec2<f32>(textureDimensions(u0));
        if (backtraced_x_u.x < 0.0 || backtraced_x_u.x > dim_u.x - 1.0 || backtraced_x_u.y < 0.0 || backtraced_x_u.y > dim_u.y - 1.0) {
            textureStore(u1, x_u, vec4<f32>(0.0, 0.0, 0.0, 0.0));
        } else {
            let backtraced_u: f32 = u_at(u0, backtraced_x_u);
            textureStore(u1, x_u, vec4<f32>(backtraced_u, 0.0, 0.0, 0.0));
        }
    }
}

@compute @workgroup_size(64, 1, 1)
fn advect_v(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let label_v = textureLoad(grid_label, x - vec2<i32>(0, 1)).r;
    let label_vplus = textureLoad(grid_label, x).r;
    if (label_v == 0 || label_vplus == 0) {
        textureStore(v1, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
    } else {
        let backtraced_x: vec2<f32> = runge_kutta(u0, v0, x, constants.dt);
        let dim_v = vec2<f32>(textureDimensions(v0));
        if (backtraced_x.x < 0.0 || backtraced_x.x > dim_v.x - 1.0 || backtraced_x.y < 0.0 || backtraced_x.y > dim_v.y - 1.0) {
            textureStore(v1, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
        } else {
            let backtraced_v: f32 = v_at(v0, backtraced_x);
            textureStore(v1, x, vec4<f32>(backtraced_v, 0.0, 0.0, 0.0));
        }
    }
}

fn runge_kutta(
    u: texture_storage_2d<r32float, read_write>,
    v: texture_storage_2d<r32float, read_write>,
    x: vec2<i32>,
    dt: f32,
) -> vec2<f32> {
    let velocity = vec2<f32>(u_at(u, vec2<f32>(x)), v_at(v, vec2<f32>(x)));
    let x_mid = vec2<f32>(x) - vec2<f32>(0.5 * dt) * velocity;
    let velocity_mid = vec2<f32>(u_at(u, x_mid), v_at(v, x_mid));

    return vec2<f32>(x) - dt * velocity_mid;
}

fn u_at(
    u: texture_storage_2d<r32float, read_write>,
    x: vec2<f32>,
) -> f32 {
    let i = i32(round(x.x));
    let j = i32(floor(x.y));
    let fract_i = f32(i) - round(x.x);
    let fract_j = f32(j) - floor(x.y);
    let u00 = textureLoad(u, vec2<i32>(i, j)).r;
    let u10 = textureLoad(u, vec2<i32>(i + 1, j)).r;
    let u01 = textureLoad(u, vec2<i32>(i, j + 1)).r;
    let u11 = textureLoad(u, vec2<i32>(i + 1, j + 1)).r;

    return mix(mix(u00, u10, fract_i), mix(u01, u11, fract_i), fract_j);
}

fn v_at(
    v: texture_storage_2d<r32float, read_write>,
    x: vec2<f32>,
) -> f32 {
    let i = i32(floor(x.x));
    let j = i32(round(x.y));
    let fract_i = f32(i) - floor(x.x);
    let fract_j = f32(j) - round(x.y);
    let v00 = textureLoad(v, vec2<i32>(i, j)).r;
    let v10 = textureLoad(v, vec2<i32>(i + 1, j)).r;
    let v01 = textureLoad(v, vec2<i32>(i, j + 1)).r;
    let v11 = textureLoad(v, vec2<i32>(i + 1, j + 1)).r;

    return mix(mix(v00, v10, fract_i), mix(v01, v11, fract_i), fract_j);
}