#import bevy_fluid::fluid_uniform::SimulationUniform;

@group(0) @binding(0) var x: texture_storage_2d<r32float, read>;
@group(0) @binding(1) var b: texture_storage_2d<r32float, read>;
@group(0) @binding(2) var levelset_air: texture_storage_2d<r32float, read>;
@group(0) @binding(3) var area_fraction_solid: texture_storage_2d<rgba32float, read>;
@group(0) @binding(4) var r: texture_storage_2d<r32float, read_write>;
@group(0) @binding(5) var<uniform> resolution_scale: f32;

@group(1) @binding(0) var<uniform> constants: SimulationUniform;

@compute @workgroup_size(8, 8, 1)
fn residual(
    @builtin(global_invocation_id) global_invocation_id: vec3u,
) {
    let idx = vec2i(global_invocation_id.xy);
    let dim = vec2i(textureDimensions(levelset_air));
    if any(idx >= dim) {
        return;
    }
    let phi = textureLoad(levelset_air, idx).r;
    if phi >= 0.0 {
        textureStore(r, idx, vec4f(0));
        return;
    }
    let f = textureLoad(area_fraction_solid, idx);
    if all(f == vec4f(0.0)) {
        textureStore(r, idx, vec4f(0));
        return;
    }
    let x_center = textureLoad(x, idx).r;

    let dx = constants.dx * resolution_scale;
    let factor = constants.dt / constants.rho / dx / dx;

    var residual = textureLoad(b, idx).r;
    let offsets = array<vec2<i32>, 4>(
        vec2<i32>(-1, 0),
        vec2<i32>(1, 0),
        vec2<i32>(0, -1),
        vec2<i32>(0, 1),
    );
    let f_vec = array<f32, 4>(f.x, f.y, f.z, f.w);
    for (var i = 0; i < 4; i++) {
        let idx_nb = idx + offsets[i];
        if all(vec2i(0) <= idx_nb) && all(idx_nb < dim) {
            let phi_nb = textureLoad(levelset_air, idx_nb).r;
            if phi_nb < 0.0 {
                residual -= f_vec[i] * (x_center - textureLoad(x, idx_nb).r) * factor;
            } else {
                residual -= f_vec[i] * (1.0 - phi_nb / phi) * x_center * factor;
            }
        }
    }

    textureStore(r, idx, vec4f(residual, vec3f(0)));
}