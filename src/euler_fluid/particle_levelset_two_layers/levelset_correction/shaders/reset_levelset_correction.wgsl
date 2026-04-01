#import bevy_fluid::particle_levelset::fixed_point::f32_to_i32;

@group(0) @binding(0) var levelset_air: texture_storage_2d<r32float, read>;
@group(0) @binding(1) var<storage, read_write> phi_plus: array<i32>;
@group(0) @binding(2) var<storage, read_write> phi_minus: array<i32>;

@compute @workgroup_size(8, 8, 1)
fn reset_levelset_correction(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let idx = global_invocation_id.xy;
    let dim = textureDimensions(levelset_air);
    let idx_1d = idx.x + dim.x * idx.y;

    phi_plus[idx_1d] = f32_to_i32(textureLoad(levelset_air, idx).r);
    phi_minus[idx_1d] = f32_to_i32(textureLoad(levelset_air, idx).r);
}