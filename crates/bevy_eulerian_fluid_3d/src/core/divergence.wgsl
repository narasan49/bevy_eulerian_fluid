#import euler_fluid_3d::workgroup_shape::WG_SIZE
#import euler_fluid_3d::fluid_uniform::FluidUniform;

@group(0) @binding(0) var u1: texture_storage_3d<r32float, read>;
@group(0) @binding(1) var v1: texture_storage_3d<r32float, read>;
@group(0) @binding(2) var w1: texture_storage_3d<r32float, read>;
@group(0) @binding(3) var u_solid: texture_storage_3d<r32float, read>;
@group(0) @binding(4) var v_solid: texture_storage_3d<r32float, read>;
@group(0) @binding(5) var w_solid: texture_storage_3d<r32float, read>;
@group(0) @binding(6) var area_fraction_solid: texture_storage_3d<rgba32float, read>;
@group(0) @binding(7) var div: texture_storage_3d<r32float, write>;

@group(1) @binding(0) var<uniform> constants: FluidUniform;

@compute @workgroup_size(WG_SIZE.x, WG_SIZE.y, WG_SIZE.z)
fn divergence(
    @builtin(global_invocation_id) gid: vec3u
) {
    if any(gid >= textureDimensions(div)) {
        return;
    }
    let u_minus = textureLoad(u1, gid).r;
    let u_plus = textureLoad(u1, gid + vec3u(1, 0, 0)).r;
    let v_minus = textureLoad(v1, gid).r;
    let v_plus = textureLoad(v1, gid + vec3u(0, 1, 0)).r;
    let w_minus = textureLoad(w1, gid).r;
    let w_plus = textureLoad(w1, gid + vec3u(0, 0, 1)).r;

    let fraction_minus = textureLoad(area_fraction_solid, gid);
    let f_minus_x = fraction_minus.x;
    let f_minus_y = fraction_minus.y;
    let f_minus_z = fraction_minus.z;
    let f_plus_x = textureLoad(area_fraction_solid, gid + vec3u(1, 0, 0)).x;
    let f_plus_y = textureLoad(area_fraction_solid, gid + vec3u(0, 1, 0)).y;
    let f_plus_z = textureLoad(area_fraction_solid, gid + vec3u(0, 0, 1)).z;

    let du_fluid = f_plus_x * u_plus - f_minus_x * u_minus;
    let dv_fluid = f_plus_y * v_plus - f_minus_y * v_minus;
    let dw_fluid = f_plus_z * w_plus - f_minus_z * w_minus;

    let u_solid_minus = textureLoad(u_solid, gid).r;
    let v_solid_minus = textureLoad(v_solid, gid).r;
    let w_solid_minus = textureLoad(w_solid, gid).r;
    let u_solid_plus = textureLoad(u_solid, gid + vec3u(1, 0, 0)).r;
    let v_solid_plus = textureLoad(v_solid, gid + vec3u(0, 1, 0)).r;
    let w_solid_plus = textureLoad(w_solid, gid + vec3u(0, 0, 1)).r;

    let du_solid = (1.0 - f_plus_x) * u_solid_plus - (1.0 - f_minus_x) * u_solid_minus;
    let dv_solid = (1.0 - f_plus_y) * v_solid_plus - (1.0 - f_minus_y) * v_solid_minus;
    let dw_solid = (1.0 - f_plus_z) * w_solid_plus - (1.0 - f_minus_z) * w_solid_minus;

    let rhs = -(du_fluid + dv_fluid + dw_fluid + du_solid + dv_solid + dw_solid) / constants.dx;

    textureStore(div, gid, vec4f(rhs, 0.0, 0.0, 0.0));
}