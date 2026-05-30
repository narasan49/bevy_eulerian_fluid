#define_import_path bevy_fluid::coordinate

fn left(x: vec2<i32>) -> vec2<i32> {
    return x - vec2<i32>(1, 0);
}

fn right(x: vec2<i32>) -> vec2<i32> {
    return x + vec2<i32>(1, 0);
}

fn bottom(x: vec2<i32>) -> vec2<i32> {
    return x - vec2<i32>(0, 1);
}

fn top(x: vec2<i32>) -> vec2<i32> {
    return x + vec2<i32>(0, 1);
}

fn interp2d_center(
    levelset: texture_storage_2d<r32float, read>,
    x: vec2<f32>,
) -> f32 {
    let base = floor(x);
    let fract = x - base;
    let ij = vec2<i32>(base);

    let levelset00 = textureLoad(levelset, ij + vec2<i32>(0, 0)).r;
    let levelset10 = textureLoad(levelset, ij + vec2<i32>(1, 0)).r;
    let levelset01 = textureLoad(levelset, ij + vec2<i32>(0, 1)).r;
    let levelset11 = textureLoad(levelset, ij + vec2<i32>(1, 1)).r;

    return mix(mix(levelset00, levelset10, fract.x), mix(levelset01, levelset11, fract.x), fract.y);
}

fn interp2d_center_rg32float(
    levelset: texture_storage_2d<rg32float, read>,
    x: vec2<f32>,
) -> vec2<f32> {
    let base = floor(x);
    let fract = x - base;
    let ij = vec2<i32>(base);

    let levelset00 = textureLoad(levelset, ij + vec2<i32>(0, 0)).rg;
    let levelset10 = textureLoad(levelset, ij + vec2<i32>(1, 0)).rg;
    let levelset01 = textureLoad(levelset, ij + vec2<i32>(0, 1)).rg;
    let levelset11 = textureLoad(levelset, ij + vec2<i32>(1, 1)).rg;

    return mix(mix(levelset00, levelset10, fract.x), mix(levelset01, levelset11, fract.x), fract.y);
}

fn interp2d_edge_x(
    u: texture_storage_2d<r32float, read>,
    x: vec2<f32>,
) -> f32 {
    let shifted = x + vec2<f32>(0.5, 0.0);
    let base = floor(shifted);
    let fract = shifted - base;
    let ij = vec2<i32>(base);

    let u00 = textureLoad(u, ij + vec2<i32>(0, 0)).r;
    let u10 = textureLoad(u, ij + vec2<i32>(1, 0)).r;
    let u01 = textureLoad(u, ij + vec2<i32>(0, 1)).r;
    let u11 = textureLoad(u, ij + vec2<i32>(1, 1)).r;

    return mix(mix(u00, u10, fract.x), mix(u01, u11, fract.x), fract.y);
}

fn interp2d_edge_y(
    v: texture_storage_2d<r32float, read>,
    x: vec2<f32>,
) -> f32 {
    let shifted = x + vec2<f32>(0.0, 0.5);
    let base = floor(shifted);
    let fract = shifted - base;
    let ij = vec2<i32>(base);

    let v00 = textureLoad(v, ij + vec2<i32>(0, 0)).r;
    let v10 = textureLoad(v, ij + vec2<i32>(1, 0)).r;
    let v01 = textureLoad(v, ij + vec2<i32>(0, 1)).r;
    let v11 = textureLoad(v, ij + vec2<i32>(1, 1)).r;

    return mix(mix(v00, v10, fract.x), mix(v01, v11, fract.x), fract.y);
}

fn runge_kutta(
    u: texture_storage_2d<r32float, read>,
    v: texture_storage_2d<r32float, read>,
    x: vec2<f32>,
    dt: f32,
) -> vec2<f32> {
    let velocity = vec2<f32>(interp2d_edge_x(u, x), interp2d_edge_y(v, x));
    let x_mid = x - vec2<f32>(0.5 * dt) * velocity;
    let velocity_mid = vec2<f32>(interp2d_edge_x(u, x_mid), interp2d_edge_y(v, x_mid));

    return x - dt * velocity_mid;
}

fn tvd_rk3(
    u: texture_storage_2d<r32float, read>,
    v: texture_storage_2d<r32float, read>,
    x: vec2<f32>,
    dt: f32,
) -> vec2<f32> {
    let u0 = vec2<f32>(interp2d_edge_x(u, x), interp2d_edge_y(v, x));
    let x1 = x + dt * u0;
    let u1 = vec2<f32>(interp2d_edge_x(u, x1), interp2d_edge_y(v, x1));
    let x2 = 0.75 * x + 0.25 * (x1 + dt * u1);
    let u2 = vec2<f32>(interp2d_edge_x(u, x2), interp2d_edge_y(v, x2));

    return (1.0 / 3.0) * x + (2.0 / 3.0) * (x2 + dt * u2);
}