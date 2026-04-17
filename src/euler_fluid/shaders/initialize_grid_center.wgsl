@group(0) @binding(0) var levelset_air0: texture_storage_2d<r32float, write>;
@group(0) @binding(1) var levelset_air1: texture_storage_2d<r32float, write>;
@group(0) @binding(2) var grad_levelset_air: texture_storage_2d<rg32float, write>;

@compute @workgroup_size(8, 8, 1)
fn initialize_grid_center(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let x = global_invocation_id.xy;
    let dim = textureDimensions(levelset_air0);

    let level = f32(dim.y - x.y);
    let dphi_dx = vec2f(0.0, -1.0);
    textureStore(levelset_air0, x, vec4<f32>(level, 0.0, 0.0, 0.0));
    textureStore(levelset_air1, x, vec4<f32>(level, 0.0, 0.0, 0.0));
    textureStore(grad_levelset_air, x, vec4<f32>(dphi_dx, 0.0, 0.0));
}
