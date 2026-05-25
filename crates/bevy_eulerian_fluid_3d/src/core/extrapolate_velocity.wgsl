#import euler_fluid_3d::workgroup_shape::WG_SIZE

@group(0) @binding(0) var u0: texture_storage_3d<r32float, read_write>;
@group(0) @binding(1) var in_is_u_valid: texture_storage_3d<r32sint, read>;
@group(0) @binding(2) var out_is_u_valid: texture_storage_3d<r32sint, write>;

@compute @workgroup_size(WG_SIZE.x, WG_SIZE.y, WG_SIZE.z)
fn extrapolate_velocity(
    @builtin(global_invocation_id) gid: vec3u,
) {
    let idx = vec3i(gid);
    let dim = vec3i(textureDimensions(in_is_u_valid));
    if any(idx >= dim) {
        return;
    }
    let is_valid = textureLoad(in_is_u_valid, idx).r;
    if (is_valid == 1) {
        textureStore(out_is_u_valid, idx, vec4<i32>(1, 0, 0, 0));
    } else {
        var count = 0;
        var new_u = 0.0;
        let neighbors = array<vec3i, 6>(
            idx + vec3i(-1, 0, 0),
            idx + vec3i(1, 0, 0),
            idx + vec3i(0, 1, 0),
            idx + vec3i(0, -1, 0),
            idx + vec3i(0, 0, -1),
            idx + vec3i(0, 0, 1),
        );
        for (var i = 0; i < 6; i++) {
            let neighbor = neighbors[i];
            if all(neighbor < dim) && all(vec3i(0) <= neighbor) {
                let neighbor_valid = textureLoad(in_is_u_valid, neighbor).r;
                if neighbor_valid == 1 {
                    new_u += textureLoad(u0, neighbor).r;
                    count += 1;
                }
            }
        }

        if count > 0 {
            new_u /= f32(count);
            textureStore(u0, idx, vec4<f32>(new_u, 0, 0, 0));
            textureStore(out_is_u_valid, idx, vec4<i32>(1, 0, 0, 0));
        } else {
            textureStore(out_is_u_valid, idx, vec4<i32>(0, 0, 0, 0));
        }
    }
}