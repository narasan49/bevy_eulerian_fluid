#import euler_fluid_3d::area_fraction::area_fraction
#import euler_fluid_3d::workgroup_shape::WG_SIZE

@group(0) @binding(0) var is_u_valid: texture_storage_3d<r32sint, write>;
@group(0) @binding(1) var is_v_valid: texture_storage_3d<r32sint, write>;
@group(0) @binding(2) var is_w_valid: texture_storage_3d<r32sint, write>;
@group(0) @binding(3) var levelset_air: texture_storage_3d<r32float, read>;

@compute @workgroup_size(WG_SIZE.x, WG_SIZE.y, WG_SIZE.z)
fn extrapolate_velocity_initialize(
    @builtin(global_invocation_id) gid: vec3u,
) {
    let idx = vec3i(gid);
    let dim_u = vec3i(textureDimensions(is_u_valid));
    if any(idx >= (dim_u + vec3i(0, 1, 1))) {
        return;
    }

    let f = area_fraction(levelset_air, idx);
    
    if all(idx < dim_u) {
        if f.x > 0.0 {
            textureStore(is_u_valid, idx, vec4<i32>(0, 0, 0, 0));
        } else {
            textureStore(is_u_valid, idx, vec4<i32>(1, 0, 0, 0));
        }
    }

    if all(idx < vec3i(textureDimensions(is_v_valid))) {
        if f.y > 0.0 {
            textureStore(is_v_valid, idx, vec4<i32>(0, 0, 0, 0));
        } else {
            textureStore(is_v_valid, idx, vec4<i32>(1, 0, 0, 0));
        }
    }

    if all(idx < vec3i(textureDimensions(is_w_valid))) {
        if f.z > 0.0 {
            textureStore(is_w_valid, idx, vec4<i32>(0, 0, 0, 0));
        } else {
            textureStore(is_w_valid, idx, vec4<i32>(1, 0, 0, 0));
        }
    }
}