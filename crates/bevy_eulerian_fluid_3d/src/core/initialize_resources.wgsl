#import euler_fluid_3d::workgroup_shape::WG_SIZE

@group(0) @binding(0) var levelset_air0: texture_storage_3d<r32float, write>;
@group(0) @binding(1) var levelset_air1: texture_storage_3d<r32float, write>;
@group(0) @binding(2) var u0: texture_storage_3d<r32float, write>;
@group(0) @binding(3) var v0: texture_storage_3d<r32float, write>;
@group(0) @binding(4) var w0: texture_storage_3d<r32float, write>;

@compute @workgroup_size(WG_SIZE.x, WG_SIZE.y, WG_SIZE.z)
fn initialize_resources(
    @builtin(global_invocation_id) gid: vec3u,
) {
    if all(gid < textureDimensions(levelset_air0)) {
        textureStore(levelset_air0, gid, vec4f(1e30, 0.0, 0.0, 0.0));
        textureStore(levelset_air1, gid, vec4f(1e30, 0.0, 0.0, 0.0));
    }
    if all(gid < textureDimensions(u0)) {
        textureStore(u0, gid, vec4f(0.0));
    }
    if all(gid < textureDimensions(v0)) {
        textureStore(v0, gid, vec4f(0.0));
    }
    if all(gid < textureDimensions(w0)) {
        textureStore(w0, gid, vec4f(0.0));
    }
}