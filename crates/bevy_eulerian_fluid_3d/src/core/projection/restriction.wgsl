
#import euler_fluid_3d::workgroup_shape::WG_SIZE

@group(0) @binding(0) var r: texture_storage_3d<r32float, read>;
@group(0) @binding(1) var levelset_air: texture_storage_3d<r32float, read>;
@group(0) @binding(2) var area_fraction_solid: texture_storage_3d<rgba32float, read>;

@group(0) @binding(3) var b_low: texture_storage_3d<r32float, write>;
@group(0) @binding(4) var levelset_air_low: texture_storage_3d<r32float, write>;
@group(0) @binding(5) var area_fraction_solid_low: texture_storage_3d<rgba32float, write>;
@group(0) @binding(6) var x_low: texture_storage_3d<r32float, write>;

@compute @workgroup_size(WG_SIZE.x, WG_SIZE.y, WG_SIZE.z)
fn restriction(
    @builtin(global_invocation_id) gid: vec3u,
) {
    let dim = textureDimensions(b_low);

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
    if all(gid < dim) {
        let b_r = 0.125 * (textureLoad(r, 2 * gid + offsets[0]).r
            + textureLoad(r, 2 * gid + offsets[1]).r
            + textureLoad(r, 2 * gid + offsets[2]).r
            + textureLoad(r, 2 * gid + offsets[3]).r
            + textureLoad(r, 2 * gid + offsets[4]).r
            + textureLoad(r, 2 * gid + offsets[5]).r
            + textureLoad(r, 2 * gid + offsets[6]).r
            + textureLoad(r, 2 * gid + offsets[7]).r);
        
        let phis = array<f32, 8>(
            textureLoad(levelset_air, 2 * gid + offsets[0]).r,
            textureLoad(levelset_air, 2 * gid + offsets[1]).r,
            textureLoad(levelset_air, 2 * gid + offsets[2]).r,
            textureLoad(levelset_air, 2 * gid + offsets[3]).r,
            textureLoad(levelset_air, 2 * gid + offsets[4]).r,
            textureLoad(levelset_air, 2 * gid + offsets[5]).r,
            textureLoad(levelset_air, 2 * gid + offsets[6]).r,
            textureLoad(levelset_air, 2 * gid + offsets[7]).r,
        );

        var phi_r = phis[0];
        for (var i = 1; i < 8; i++) {
            if abs(phis[i]) < abs(phi_r) {
                phi_r = phis[i];
            }
        }

        textureStore(b_low, gid, vec4f(b_r, vec3f(0)));
        textureStore(levelset_air_low, gid, vec4f(phi_r, vec3f(0)));
        textureStore(x_low, gid, vec4f(0));
    }

    let dim_fraction = textureDimensions(area_fraction_solid_low);
    if all(gid < dim_fraction) {
        let fractions = array<vec4f, 7>(
            textureLoad(area_fraction_solid, 2 * gid + offsets[0]),
            textureLoad(area_fraction_solid, 2 * gid + offsets[1]),
            textureLoad(area_fraction_solid, 2 * gid + offsets[2]),
            textureLoad(area_fraction_solid, 2 * gid + offsets[3]),
            textureLoad(area_fraction_solid, 2 * gid + offsets[4]),
            textureLoad(area_fraction_solid, 2 * gid + offsets[5]),
            textureLoad(area_fraction_solid, 2 * gid + offsets[6]),
        );

        let f_r = vec4f(
            0.25 * (fractions[0].x + fractions[2].x + fractions[4].x + fractions[6].x),
            0.25 * (fractions[0].y + fractions[1].y + fractions[4].y + fractions[5].y),
            0.25 * (fractions[0].z + fractions[1].z + fractions[2].z + fractions[3].z),
            0.0,
        );
        textureStore(area_fraction_solid_low, gid, f_r);
    }
}