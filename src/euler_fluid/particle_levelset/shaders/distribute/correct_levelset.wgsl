@group(0) @binding(0) var levelset_air: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var<storage, read> levelset_correction: array<i32>;
@group(0) @binding(2) var<storage, read> weight: array<i32>;

@compute @workgroup_size(8, 8, 1)
fn correct_levelset(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let idx = global_invocation_id.xy;
    let dim = textureDimensions(levelset_air);
    let array_idx = idx.x + dim.x * idx.y;

    let level = textureLoad(levelset_air, idx).r;

    let weight = i32_to_f32(weight[array_idx]);
    if weight != 0.0 {
        let correction = i32_to_f32(levelset_correction[array_idx]);
        textureStore(levelset_air, idx, vec4<f32>(level - correction / weight, 0.0, 0.0, 0.0));
    }
}

const SCALE = 1000.0;
fn i32_to_f32(value: i32) -> f32 {
    return f32(value) / SCALE;
}

fn f32_to_i32(value: f32) -> i32 {
    return i32(value * SCALE);
}