#import euler_fluid_3d::fluid_uniform::FluidUniform;
#import euler_fluid_3d::area_fraction::{load_area_fraction, fully_solid};
#import euler_fluid_3d::workgroup_shape::WG_SIZE

@group(0) @binding(0) var x: texture_storage_3d<r32float, read>;
@group(0) @binding(1) var b: texture_storage_3d<r32float, read>;
@group(0) @binding(2) var levelset_air: texture_storage_3d<r32float, read>;
@group(0) @binding(3) var area_fraction_solid: texture_storage_3d<rgba32float, read>;
@group(0) @binding(4) var r: texture_storage_3d<r32float, read_write>;
@group(0) @binding(5) var<uniform> resolution_scale: f32;

@group(1) @binding(0) var<uniform> constants: FluidUniform;

@compute @workgroup_size(WG_SIZE.x, WG_SIZE.y, WG_SIZE.z)
fn residual(
    @builtin(global_invocation_id) gid: vec3u,
) {
    let idx = vec3i(gid);
    let dim = vec3i(textureDimensions(levelset_air));
    if any(idx >= dim) {
        return;
    }
    let phi = textureLoad(levelset_air, idx).r;
    if phi >= 0.0 {
        textureStore(r, idx, vec4f(0));
        return;
    }
    let f = load_area_fraction(area_fraction_solid, gid);
    if fully_solid(f) {
        textureStore(r, idx, vec4f(0));
        return;
    }
    let x_center = textureLoad(x, idx).r;

    let dx = constants.dx * resolution_scale;
    let factor = constants.dt / constants.rho / dx / dx;

    var residual = textureLoad(b, idx).r;
    let offsets = array<vec3i, 6>(
        vec3i(-1, 0, 0),
        vec3i(1, 0, 0),
        vec3i(0, -1, 0),
        vec3i(0, 1, 0),
        vec3i(0, 0, -1),
        vec3i(0, 0, 1),
    );
    for (var i = 0; i < 6; i++) {
        let idx_nb = idx + offsets[i];
        if all(vec3i(0) <= idx_nb) && all(idx_nb < dim) {
            let phi_nb = textureLoad(levelset_air, idx_nb).r;
            if phi_nb < 0.0 {
                residual -= f[i] * (x_center - textureLoad(x, idx_nb).r) * factor;
            } else {
                residual -= f[i] * (1.0 - phi_nb / phi) * x_center * factor;
            }
        }
    }

    textureStore(r, idx, vec4f(residual, vec3f(0)));
}