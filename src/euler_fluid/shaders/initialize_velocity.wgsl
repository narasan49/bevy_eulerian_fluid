@group(0) @binding(0) var u0: texture_storage_2d<r32float, write>;
@group(0) @binding(1) var v0: texture_storage_2d<r32float, write>;
@group(0) @binding(2) var u1: texture_storage_2d<r32float, write>;
@group(0) @binding(3) var v1: texture_storage_2d<r32float, write>;

@compute
@workgroup_size(1, 64, 1)
fn initialize_velocity(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let x_u = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let x_v = vec2<i32>(x_u.y, x_u.x);
    textureStore(u0, x_u, vec4<f32>(0.0, 0.0, 0.0, 0.0));
    textureStore(v0, x_v, vec4<f32>(0.0, 0.0, 0.0, 0.0));
    textureStore(u1, x_u, vec4<f32>(0.0, 0.0, 0.0, 0.0));
    textureStore(v1, x_v, vec4<f32>(0.0, 0.0, 0.0, 0.0));
}
