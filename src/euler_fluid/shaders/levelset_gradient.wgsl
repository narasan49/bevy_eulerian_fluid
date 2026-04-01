@group(0) @binding(0) var levelset: texture_storage_2d<r32float, read>;
@group(0) @binding(1) var grad_levelset: texture_storage_2d<rg32float, write>;

@compute @workgroup_size(8, 8, 1)
fn levelset_gradient(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let idx = global_invocation_id.xy;
    let dim = textureDimensions(levelset);

    if any(idx == vec2<u32>(0)) || any(dim <= idx) {
        return;
    }

    let grad = vec2<f32>(
        0.5 * (textureLoad(levelset, idx + vec2<u32>(1, 0)).r - textureLoad(levelset, idx - vec2<u32>(1, 0)).r),
        0.5 * (textureLoad(levelset, idx + vec2<u32>(0, 1)).r - textureLoad(levelset, idx - vec2<u32>(0, 1)).r),
    );

    textureStore(grad_levelset, idx, vec4<f32>(grad, 0.0, 0.0));
}