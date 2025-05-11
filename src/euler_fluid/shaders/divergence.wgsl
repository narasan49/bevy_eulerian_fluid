#import bevy_fluid::coordinate::{left, right, bottom, top};
#import bevy_fluid::area_fraction::area_fractions;


// The number of texture_storage binding for WebGPU is limited to 8.
// So divergence has only bindings for u1 and v1.
@group(0) @binding(0) var u1: texture_storage_2d<r32float, read>;
@group(0) @binding(1) var v1: texture_storage_2d<r32float, read>;

@group(1) @binding(0) var div: texture_storage_2d<r32float, read_write>;

@group(2) @binding(2) var levelset_solid: texture_storage_2d<r32float, read_write>;

@group(3) @binding(0) var u_solid: texture_storage_2d<r32float, read_write>;
@group(3) @binding(1) var v_solid: texture_storage_2d<r32float, read_write>;

@compute @workgroup_size(8, 8, 1)
fn divergence(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let idx = vec2<i32>(invocation_id.xy);
    let f = area_fractions(levelset_solid, idx);

    let x_top = top(idx);
    let x_right = right(idx);

    let u_ij = textureLoad(u1, idx).r;
    let u_iplusj = textureLoad(u1, x_right).r;
    let v_ij = textureLoad(v1, idx).r;
    let v_ijplus = textureLoad(v1, x_top).r;
    let du_fluid = f.iplusj * u_iplusj - f.iminusj * u_ij;
    let dv_fluid = f.ijplus * v_ijplus - f.ijminus * v_ij;

    let u_solid_ij = textureLoad(u_solid, idx).r;
    let u_solid_iplusj = textureLoad(u_solid, x_right).r;
    let v_solid_ij = textureLoad(v_solid, idx).r;
    let v_solid_ijplus = textureLoad(v_solid, x_top).r;

    let du_solid = (1.0 - f.iplusj) * u_solid_iplusj - (1.0 - f.iminusj) * u_solid_ij;
    let dv_solid = (1.0 - f.ijplus) * v_solid_ijplus - (1.0 - f.ijminus) * v_solid_ij;
    
    let rhs = du_fluid + dv_fluid + du_solid + dv_solid;

    textureStore(div, idx, vec4<f32>(rhs, 0.0, 0.0, 0.0));
}