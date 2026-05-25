#import euler_fluid_3d::workgroup_shape::WG_SIZE

const LARGE_FLOAT: f32 = 1.0e6;

@group(0) @binding(0) var u_solid: texture_storage_3d<r32float, write>;
@group(0) @binding(1) var v_solid: texture_storage_3d<r32float, write>;
@group(0) @binding(2) var w_solid: texture_storage_3d<r32float, write>;
@group(0) @binding(3) var levelset_solid: texture_storage_3d<r32float, write>;

@compute @workgroup_size(WG_SIZE.x, WG_SIZE.y, WG_SIZE.z)
fn update_solid(
    @builtin(global_invocation_id) gid: vec3u,
) {
    let dim = textureDimensions(levelset_solid);
    var level_solid = LARGE_FLOAT;
    if all(gid < dim + vec3u(1, 0, 0)) {
        textureStore(u_solid, gid, vec4f(0.0));  
    }
    if all(gid < dim + vec3u(0, 1, 0)) {
        textureStore(v_solid, gid, vec4f(0.0));  
    }
    if all(gid < dim + vec3u(0, 0, 1)) {
        textureStore(w_solid, gid, vec4f(0.0));  
    }

    if all(gid < dim) {
        // x-wall boundary
        level_solid = min(level_solid, f32(gid.x));
        level_solid = min(level_solid, f32(dim.x - gid.x) - 1.0);
        // y-wall boundary
        level_solid = min(level_solid, f32(gid.y));
        level_solid = min(level_solid, f32(dim.y - gid.y) - 1.0);
        // z-wall boundary
        level_solid = min(level_solid, f32(gid.z));
        level_solid = min(level_solid, f32(dim.z - gid.z) - 1.0);
    
        textureStore(levelset_solid, gid, vec4f(level_solid, vec3f(0.0)));
    }
}