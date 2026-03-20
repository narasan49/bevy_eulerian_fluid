#import bevy_fluid::particle_levelset::fixed_point::i32_to_f32;

@group(0) @binding(0) var levelset_air: texture_storage_2d<r32float, write>;
@group(0) @binding(1) var<storage, read> phi_plus: array<i32>;
@group(0) @binding(2) var<storage, read> phi_minus: array<i32>;

@compute @workgroup_size(8, 8, 1)
fn correct_levelset(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let idx = global_invocation_id.xy;
    let dim = textureDimensions(levelset_air);
    let idx_1d = idx.x + dim.x * idx.y;

    let phi_plus_value = i32_to_f32(phi_plus[idx_1d]);
    let phi_minus_value = i32_to_f32(phi_minus[idx_1d]);
    var phi_new = 0.0;
    if abs(phi_plus_value) <= abs(phi_minus_value) {
        phi_new = phi_plus_value;
    } else {
        phi_new = phi_minus_value;
    }

    textureStore(levelset_air, idx, vec4<f32>(phi_new, 0.0, 0.0, 0.0));
}