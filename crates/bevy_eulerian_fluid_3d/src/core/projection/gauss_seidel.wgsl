#import euler_fluid_3d::fluid_uniform::FluidUniform;
#import euler_fluid_3d::area_fraction::{load_area_fraction, fully_solid};
#import euler_fluid_3d::workgroup_shape::WG_SIZE

@group(0) @binding(0) var p: texture_storage_3d<r32float, read_write>;
@group(0) @binding(1) var div: texture_storage_3d<r32float, read>;
@group(0) @binding(2) var levelset_air0: texture_storage_3d<r32float, read>;
@group(0) @binding(3) var area_fraction_solid: texture_storage_3d<rgba32float, read>;
@group(0) @binding(4) var<uniform> weight: f32;
@group(0) @binding(5) var<uniform> resolution_scale: f32;

@group(1) @binding(0) var<uniform> constants: FluidUniform;

@compute @workgroup_size(WG_SIZE.x, WG_SIZE.y, WG_SIZE.z)
fn gauss_seidel_red(
    @builtin(global_invocation_id) gid: vec3<u32>,
) {
    let dim = textureDimensions(div);
    if any(gid >= dim) {
        return;
    }

    if ((gid.x + gid.y + gid.z) % 2 == 1) {
        let p_new = update_pressure(vec3i(gid));
        textureStore(p, gid, vec4<f32>(p_new, 0.0, 0.0, 0.0));
    }
}

@compute @workgroup_size(WG_SIZE.x, WG_SIZE.y, WG_SIZE.z)
fn gauss_seidel_black(
    @builtin(global_invocation_id) gid: vec3<u32>,
) {
    let dim = textureDimensions(div);
    if any(gid >= dim) {
        return;
    }

    if ((gid.x + gid.y + gid.z) % 2 == 0) {
        let p_new = update_pressure(vec3i(gid));
        textureStore(p, gid, vec4<f32>(p_new, 0.0, 0.0, 0.0));
    }
}

fn update_pressure(idx: vec3i) -> f32 {
    let level_air_ij = textureLoad(levelset_air0, idx).r;
    if (level_air_ij >= 0.0) {
        return 0.0;
    }
    let f = load_area_fraction(area_fraction_solid, vec3u(idx));
    if (fully_solid(f)) {
        return 0.0;
    }

    var denom = 0.0;
    let dx = resolution_scale * constants.dx;
    var nume = dx * dx * constants.rho / constants.dt * textureLoad(div, idx).r;
    let offsets = array<vec3i, 6>(
        vec3i(-1, 0, 0),
        vec3i(1, 0, 0),
        vec3i(0, -1, 0),
        vec3i(0, 1, 0),
        vec3i(0, 0, -1),
        vec3i(0, 0, 1),
    );

    let dim = vec3i(textureDimensions(levelset_air0));
    for (var i = 0; i < 6; i++) {
        var j = idx + offsets[i];
        if all(vec3i(0) <= j) && all(j < dim) {
            let level = textureLoad(levelset_air0, j).r;
            if level < 0.0 {
                denom += f[i];
                nume += f[i] * textureLoad(p, j).r;
            } else {
                let theta = clamp(level_air_ij / (level_air_ij - level), 0.1, 1.0);
                denom += f[i] / theta;
            }
        }
    }

    if (abs(denom) < 1.0e-6) {
        return 0.0;
    }

    let p_new = nume / denom;
    let p_old = textureLoad(p, idx).r;

    return  weight * p_new + (1.0 - weight) * p_old;
}
