#import euler_fluid_3d::fluid_uniform::FluidUniform;
#import euler_fluid_3d::area_fraction::{load_area_fraction, fully_solid};
#import euler_fluid_3d::workgroup_shape::WG_SIZE

@group(0) @binding(0) var v0: texture_storage_3d<r32float, write>;
@group(0) @binding(1) var v1: texture_storage_3d<r32float, read>;
@group(0) @binding(2) var v_solid: texture_storage_3d<r32float, read>;
@group(0) @binding(3) var p0: texture_storage_3d<r32float, read>;
@group(0) @binding(4) var levelset_air0: texture_storage_3d<r32float, read>;
@group(0) @binding(5) var area_fraction_solid: texture_storage_3d<rgba32float, read>;

@group(1) @binding(0) var<uniform> constants: FluidUniform;

@compute @workgroup_size(WG_SIZE.x, WG_SIZE.y, WG_SIZE.z)
fn solve_v(@builtin(global_invocation_id) gid: vec3u) {
    let factor = constants.dt / (constants.dx * constants.rho);
    let x = vec3i(gid);
    let dim = vec3i(textureDimensions(v0));
    if any(x >= dim) {
        return;
    }

    let f = textureLoad(area_fraction_solid, gid);
    if (f.y == 0.0) {
        textureStore(v0, x, textureLoad(v_solid, x));
        return;
    }

    var p_plus = textureLoad(p0, x).r;
    var p_minus = textureLoad(p0, x - vec3i(0, 1, 0)).r;

    let level_plus = textureLoad(levelset_air0, x).r;
    let level_minus = textureLoad(levelset_air0, x - vec3i(0, 1, 0)).r;
    if (level_minus >= 0.0 && level_plus < 0.0) {
        p_minus = level_minus / level_plus * p_plus;
    } else if (level_minus < 0.0 && level_plus >= 0.0) {
        p_plus = level_plus / level_minus * p_minus;
    } else if (level_minus >= 0.0 && level_plus >= 0.0) {
        textureStore(v0, x, vec4f(0.0));
        return;
    }

    let v = textureLoad(v1, x);
    let dv = vec4f(factor * (p_plus - p_minus), 0.0, 0.0, 0.0);
    textureStore(v0, x, v - dv);
}
