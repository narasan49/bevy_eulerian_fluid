#import bevy_fluid::coordinate::{interp2d_center, interp2d_center_rg32float}

@group(0) @binding(0) var<storage, read_write> count: atomic<u32>;
@group(0) @binding(1) var<storage, read_write> particles: array<vec2<f32>>;
@group(0) @binding(2) var levelset_air: texture_storage_2d<r32float, read>;
@group(0) @binding(3) var grad_levelset_air: texture_storage_2d<rg32float, read>;
@group(0) @binding(4) var near_interface: texture_storage_2d<r8uint, read>;

const offsets = array<vec2<f32>, 4>(
    vec2<f32>(1.0 / 3.0, 1.0 / 3.0),
    vec2<f32>(2.0 / 3.0, 1.0 / 3.0),
    vec2<f32>(1.0 / 3.0, 2.0 / 3.0),
    vec2<f32>(2.0 / 3.0, 2.0 / 3.0),
);

@compute @workgroup_size(8, 8, 1)
fn initialize_particles(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let tex_idx = vec2<i32>(global_invocation_id.xy);
    let is_near_interface = textureLoad(near_interface, tex_idx).r;

    if (is_near_interface == 1) {
        for (var i = 0; i < 4; i++) {
            let pos = attraction(levelset_air, grad_levelset_air, vec2<f32>(tex_idx) + offsets[i]);
            let idx = atomicAdd(&count, 1);
            particles[idx] = pos;
        }
    }
}

fn attraction(
    levelset_air: texture_storage_2d<r32float, read>,
    grad_levelset_air: texture_storage_2d<rg32float, read>,
    x: vec2<f32>,
) -> vec2<f32> {
    let phi = interp2d_center(levelset_air, x);
    let grad_phi = interp2d_center_rg32float(grad_levelset_air, x);
    let grad_phi_norm = normalize(grad_phi);

    return x - phi * grad_phi_norm;
}