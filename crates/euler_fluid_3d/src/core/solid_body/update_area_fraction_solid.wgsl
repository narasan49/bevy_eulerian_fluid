#import euler_fluid_3d::area_fraction::area_fraction
#import euler_fluid_3d::workgroup_shape::WG_SIZE

@group(0) @binding(0) var levelset_solid: texture_storage_3d<r32float, read>;
@group(0) @binding(1) var area_fraction_solid: texture_storage_3d<rgba32float, write>;

@compute @workgroup_size(WG_SIZE.x, WG_SIZE.y, WG_SIZE.z)
fn update_area_fraction_solid(
    @builtin(global_invocation_id) gid: vec3<u32>,
) {
    let idx = vec3i(gid);
    let f = area_fraction(levelset_solid, idx);

    textureStore(area_fraction_solid, gid, vec4<f32>(f, 0.0));
}