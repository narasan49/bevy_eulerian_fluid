#import euler_fluid_3d::fluid_uniform::FluidUniform;
#import euler_fluid_3d::area_fraction::{load_area_fraction, fully_solid};
#import euler_fluid_3d::workgroup_shape::WG_SIZE

@group(0) @binding(0) var u0: texture_storage_3d<r32float, write>;
@group(0) @binding(1) var u1: texture_storage_3d<r32float, read>;
@group(0) @binding(2) var u_solid: texture_storage_3d<r32float, read>;
@group(0) @binding(3) var p0: texture_storage_3d<r32float, read>;
@group(0) @binding(4) var levelset_air0: texture_storage_3d<r32float, read>;
@group(0) @binding(5) var area_fraction_solid: texture_storage_3d<rgba32float, read>;

@group(1) @binding(0) var<uniform> constants: FluidUniform;

@compute @workgroup_size(WG_SIZE.x, WG_SIZE.y, WG_SIZE.z)
fn solve_u(@builtin(global_invocation_id) gid: vec3u) {
    let factor = constants.dt / (constants.dx * constants.rho);
    let x = vec3i(gid);
    let dim = vec3i(textureDimensions(u0));
    if any(x >= dim) {
        return;
    }

    let f = textureLoad(area_fraction_solid, gid);
    if (f.x == 0.0) {
        textureStore(u0, x, textureLoad(u_solid, x));
        return;
    }

    var p_plus = textureLoad(p0, x).r;
    var p_minus = textureLoad(p0, x - vec3i(1, 0, 0)).r;

    let level_plus = textureLoad(levelset_air0, x).r;
    let level_minus = textureLoad(levelset_air0, x - vec3i(1, 0, 0)).r;
    if (level_minus >= 0.0 && level_plus < 0.0) {
        p_minus = level_minus / level_plus * p_plus;
    } else if (level_minus < 0.0 && level_plus >= 0.0) {
        p_plus = level_plus / level_minus * p_minus;
    } else if (level_minus >= 0.0 && level_plus >= 0.0) {
        textureStore(u0, x, vec4f(0.0));
        return;
    }

    let u = textureLoad(u1, x);
    let du = vec4f(factor * (p_plus - p_minus), 0.0, 0.0, 0.0);
    textureStore(u0, x, u - du);
}
