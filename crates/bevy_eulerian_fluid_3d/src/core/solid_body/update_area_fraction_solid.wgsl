#import euler_fluid_3d::area_fraction::area_fraction
#import euler_fluid_3d::workgroup_shape::WG_SIZE

@group(0) @binding(0) var levelset_solid: texture_storage_3d<r32float, read>;
@group(0) @binding(1) var area_fraction_solid: texture_storage_3d<rgba32float, write>;

@compute @workgroup_size(WG_SIZE.x, WG_SIZE.y, WG_SIZE.z)
fn update_area_fraction_solid(
    @builtin(global_invocation_id) gid: vec3<u32>,
) {
    let dim = textureDimensions(area_fraction_solid);
    if any(gid == vec3u(0)) || any(gid == (dim - vec3u(1))) {
        var f_edge = vec3f(0.5, 0.5, 0.5);
        if gid.x == 0 || gid.x == (dim.x - 1) {
            f_edge.x = 0.0;
        }
        if gid.y == 0 || gid.y == (dim.y - 1) {
            f_edge.y = 0.0;
        }
        if gid.z == 0 || gid.z == (dim.z - 1) {
            f_edge.z = 0.0;
        }
        textureStore(area_fraction_solid, gid, vec4f(f_edge, 0.0));
        return;
    }
    if any(gid == (dim - vec3u(2))) {
        textureStore(area_fraction_solid, gid, vec4f(1.0, 1.0, 1.0, 0.0));
        return;
    }
    let idx = vec3i(gid);
    let f = area_fraction(levelset_solid, idx);

    textureStore(area_fraction_solid, gid, vec4<f32>(f, 0.0));
}