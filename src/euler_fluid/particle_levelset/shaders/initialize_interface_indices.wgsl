@group(0) @binding(0) var levelset_air: texture_storage_2d<r32float, read>;
@group(0) @binding(1) var near_interface: texture_storage_2d<r8uint, write>;

@compute @workgroup_size(8, 8, 1)
fn initialize_interface_indices(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let idx = vec2<i32>(invocation_id.xy);

    let level = textureLoad(levelset_air, idx).r;
    if (abs(level) < 1.0) {
        textureStore(near_interface, idx, vec4<u32>(1, 0, 0, 0));
    } else {
        textureStore(near_interface, idx, vec4<u32>(0, 0, 0, 0));
    }
}