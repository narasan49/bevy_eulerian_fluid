#import euler_fluid_3d::workgroup_shape::WG_SIZE

@group(0) @binding(0) var x: texture_storage_3d<r32float, read_write>;
@group(0) @binding(1) var x_low: texture_storage_3d<r32float, read>;
@group(0) @binding(2) var levelset_air: texture_storage_3d<r32float, read>;

@compute @workgroup_size(WG_SIZE.x, WG_SIZE.y, WG_SIZE.z)
fn prolongation(
    @builtin(global_invocation_id) gid: vec3u,
) {
    let dim = textureDimensions(x_low);
    if any(gid >= dim) {
        return;
    }

    let correction = textureLoad(x_low, gid);
    let offsets = array<vec3u, 8>(
        vec3u(0, 0, 0),
        vec3u(1, 0, 0),
        vec3u(0, 1, 0),
        vec3u(1, 1, 0),
        vec3u(0, 0, 1),
        vec3u(1, 0, 1),
        vec3u(0, 1, 1),
        vec3u(1, 1, 1),
    );

    for (var i = 0u; i < 8; i++) {
        let fine_idx = 2 * gid + offsets[i];
        let level = textureLoad(levelset_air, fine_idx).r;
        if level < 0.0 {
            textureStore(x, fine_idx, correction + textureLoad(x, fine_idx));
        }
    }
}