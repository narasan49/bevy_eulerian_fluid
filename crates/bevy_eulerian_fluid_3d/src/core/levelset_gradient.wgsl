#import euler_fluid_3d::workgroup_shape::WG_SIZE

@group(0) @binding(0) var levelset: texture_storage_3d<r32float, read>;
@group(0) @binding(1) var grad_levelset: texture_storage_3d<rgba32float, write>;

@compute @workgroup_size(WG_SIZE.x, WG_SIZE.y, WG_SIZE.z)
fn levelset_gradient(
    @builtin(global_invocation_id) gid: vec3u,
) {
    let dim = textureDimensions(levelset);

    if any(gid == vec3u(0)) || any(dim <= gid) {
        return;
    }

    let grad = vec3f(
        0.5 * (textureLoad(levelset, gid + vec3u(1, 0, 0)).r - textureLoad(levelset, gid - vec3u(1, 0, 0)).r),
        0.5 * (textureLoad(levelset, gid + vec3u(0, 1, 0)).r - textureLoad(levelset, gid - vec3u(0, 1, 0)).r),
        0.5 * (textureLoad(levelset, gid + vec3u(0, 0, 1)).r - textureLoad(levelset, gid - vec3u(0, 0, 1)).r),
    );

    textureStore(grad_levelset, gid, vec4<f32>(grad, 0.0));
}