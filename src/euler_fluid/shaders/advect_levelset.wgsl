#import bevy_fluid::fluid_uniform::SimulationUniform;

@group(0) @binding(0) var u0: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var v0: texture_storage_2d<r32float, read_write>;

@group(1) @binding(0) var levelset_air0: texture_storage_2d<r32float, read_write>;
@group(1) @binding(1) var levelset_air1: texture_storage_2d<r32float, read_write>;

@group(2) @binding(0) var<uniform> constants: SimulationUniform;

@compute @workgroup_size(8, 8, 1)
fn advect_levelset(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let dt = constants.dt;
    let size = textureDimensions(levelset_air0);
    var x_new = runge_kutta(u0, v0, vec2<f32>(x), dt);
    if (x_new.x > f32(size.x) - 1.0) {
        x_new.x = f32(size.x) - 1.0;
    }
    if (x_new.y > f32(size.y) - 1.0) {
        x_new.y = f32(size.y) - 1.0;
    }
    if (x_new.x < 0.0) {
        x_new.x = 0.0;
    }
    if (x_new.y < 0.0) {
        x_new.y = 0.0;
    }
    let new_level = interpolate2d_grid_center(levelset_air0, x_new);
    textureStore(levelset_air1, x, vec4<f32>(new_level, 0.0, 0.0, 0.0));
}

fn runge_kutta(
    u: texture_storage_2d<r32float, read_write>,
    v: texture_storage_2d<r32float, read_write>,
    x: vec2<f32>,
    dt: f32,
) -> vec2<f32> {
    let velocity = vec2<f32>(u_at(u, x), v_at(v, x));
    let x_mid = x - vec2<f32>(0.5 * dt) * velocity;
    let velocity_mid = vec2<f32>(u_at(u, x_mid), v_at(v, x_mid));

    return x - dt * velocity_mid;
}

fn u_at(
    u: texture_storage_2d<r32float, read_write>,
    x: vec2<f32>,
) -> f32 {
    let i = i32(round(x.x));
    let j = i32(floor(x.y));
    let fract_i = x.x + 0.5 - f32(i);
    let fract_j = x.y - f32(j);
    let u00 = textureLoad(u, vec2<i32>(i, j)).r;
    let u10 = textureLoad(u, vec2<i32>(i + 1, j)).r;
    let u01 = textureLoad(u, vec2<i32>(i, j + 1)).r;
    let u11 = textureLoad(u, vec2<i32>(i + 1, j + 1)).r;

    return mix(mix(u00, u10, fract_i), mix(u01, u11, fract_i), fract_j);
}

// interpolate by y-ward edges of the grid
fn v_at(
    v: texture_storage_2d<r32float, read_write>,
    x: vec2<f32>,
) -> f32 {
    let i = i32(floor(x.x));
    let j = i32(round(x.y));
    let fract_i = x.x - f32(i);
    let fract_j = x.y + 0.5 - f32(j);
    let v00 = textureLoad(v, vec2<i32>(i, j)).r;
    let v10 = textureLoad(v, vec2<i32>(i + 1, j)).r;
    let v01 = textureLoad(v, vec2<i32>(i, j + 1)).r;
    let v11 = textureLoad(v, vec2<i32>(i + 1, j + 1)).r;

    return mix(mix(v00, v10, fract_i), mix(v01, v11, fract_i), fract_j);
}

fn interpolate2d_grid_center(
    grid: texture_storage_2d<r32float, read_write>,
    x: vec2<f32>,
) -> f32 {
    let i = i32(floor(x.x));
    let j = i32(floor(x.y));
    let fract_i = x.x - f32(i);
    let fract_j = x.y - f32(j);

    let dim = vec2<i32>(textureDimensions(grid));

    let u00 = textureLoad(grid, vec2<i32>(i, j)).r;
    var u10 = 0.0;
    if i + 1 < dim.x {
        u10 = textureLoad(grid, vec2<i32>(i + 1, j)).r;
    }
    var u01 = 0.0;
    if j + 1 < dim.y {
        u01 = textureLoad(grid, vec2<i32>(i, j + 1)).r;
    }
    var u11 = 0.0;
    if i + 1 < dim.x && j + 1 < dim.y {
        u11 = textureLoad(grid, vec2<i32>(i + 1, j + 1)).r;
    }

    return mix(mix(u00, u10, fract_i), mix(u01, u11, fract_i), fract_j);
}