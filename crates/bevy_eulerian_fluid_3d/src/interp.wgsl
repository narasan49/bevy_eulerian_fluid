#define_import_path euler_fluid_3d::interp

fn trilinear(
    tex: texture_storage_3d<r32float, read>,
    x: vec3f,
    offset: vec3f,
) -> f32 {
    let shifted = x + offset;
    let base = floor(shifted);
    let fract = shifted - base;
    let idx = vec3u(base);

    let y = array<f32, 8>(
        textureLoad(tex, idx + vec3u(0, 0, 0)).r,
        textureLoad(tex, idx + vec3u(1, 0, 0)).r,
        textureLoad(tex, idx + vec3u(0, 1, 0)).r,
        textureLoad(tex, idx + vec3u(1, 1, 0)).r,
        textureLoad(tex, idx + vec3u(0, 0, 1)).r,
        textureLoad(tex, idx + vec3u(1, 0, 1)).r,
        textureLoad(tex, idx + vec3u(0, 1, 1)).r,
        textureLoad(tex, idx + vec3u(1, 1, 1)).r,
    );

    return mix(
        mix(mix(y[0], y[1], fract.x), mix(y[2], y[3], fract.x), fract.y),
        mix(mix(y[4], y[5], fract.x), mix(y[6], y[7], fract.x), fract.y),
        fract.z
    );
}

fn trilinear_x(
    tex: texture_storage_3d<r32float, read>,
    x: vec3f,
) -> f32 {
    return trilinear(tex, x, vec3f(0.5, 0.0, 0.0));
}

fn trilinear_y(
    tex: texture_storage_3d<r32float, read>,
    x: vec3f,
) -> f32 {
    return trilinear(tex, x, vec3f(0.0, 0.5, 0.0));
}

fn trilinear_z(
    tex: texture_storage_3d<r32float, read>,
    x: vec3f,
) -> f32 {
    return trilinear(tex, x, vec3f(0.0, 0.0, 0.5));
}