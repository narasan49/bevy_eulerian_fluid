@group(0) @binding(0) var u0: texture_storage_2d<r32float, write>;
@group(0) @binding(1) var v0: texture_storage_2d<r32float, write>;
@group(0) @binding(2) var u1: texture_storage_2d<r32float, write>;
@group(0) @binding(3) var v1: texture_storage_2d<r32float, write>;

@compute @workgroup_size(8, 8, 1)
fn initialize_grid_edge(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let idx = invocation_id.xy;
    let dim_u = textureDimensions(u0);
    let dim_v = textureDimensions(v0);
    if all(idx < dim_u) {
        textureStore(u0, idx, vec4<f32>(0.0));
        textureStore(u1, idx, vec4<f32>(0.0));
    }
    if all(idx < dim_v) {
        textureStore(v0, idx, vec4<f32>(0.0));
        textureStore(v1, idx, vec4<f32>(0.0));
    }
}
