#import bevy_fluid::fluid_uniform::SimulationUniform;

@group(0) @binding(0) var u0: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var v0: texture_storage_2d<r32float, read_write>;
@group(0) @binding(2) var u1: texture_storage_2d<r32float, read_write>;
@group(0) @binding(3) var v1: texture_storage_2d<r32float, read_write>;

@group(1) @binding(0) var<uniform> constants: SimulationUniform;

@group(2) @binding(1) var p1: texture_storage_2d<r32float, read_write>;

@group(3) @binding(1) var grid_label: texture_storage_2d<r32uint, read_write>;

@compute @workgroup_size(1, 64, 1)
fn solve_velocity_u(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let factor = constants.dt / (constants.dx * constants.rho);
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let grid_label_u0 = textureLoad(grid_label, x - vec2<i32>(1, 0)).r;
    let grid_label_u1 = textureLoad(grid_label, x).r;
    if (grid_label_u0 == 2) {
        let u_solid = textureLoad(u1, x - vec2<i32>(1, 0)).r;
        textureStore(u0, x, vec4<f32>(u_solid, 0.0, 0.0, 0.0));
    } else if (grid_label_u1 == 2) {
        let u_solid = textureLoad(u1, x).r;
        textureStore(u0, x, vec4<f32>(u_solid, 0.0, 0.0, 0.0));
    } else {
        let p1_u = textureLoad(p1, x).r;
        var p0_u = 0.0;
        if x.x != 0 {
            p0_u = textureLoad(p1, x - vec2<i32>(1, 0)).r;
        }
        let u = textureLoad(u1, x);
        let du = vec4<f32>(factor * (p1_u - p0_u), 0.0, 0.0, 0.0);
        textureStore(u0, x, u - du);
    }
}

@compute @workgroup_size(64, 1, 1)
fn solve_velocity_v(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let factor = constants.dt / (constants.dx * constants.rho);
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let grid_label_v0 = textureLoad(grid_label, x - vec2<i32>(0, 1)).r;
    let grid_label_v1 = textureLoad(grid_label, x).r;
    if (grid_label_v0 == 2) {
        let v_solid = textureLoad(v1, x - vec2<i32>(0, 1)).r;
        textureStore(v0, x, vec4<f32>(v_solid, 0.0, 0.0, 0.0));
    } else if (grid_label_v1 == 2) {
        let v_solid = textureLoad(v1, x).r;
        textureStore(v0, x, vec4<f32>(v_solid, 0.0, 0.0, 0.0));
    } else {
        let p1_v = textureLoad(p1, x).r;
        var p0_v = 0.0;
        if x.y != 0 {
            p0_v = textureLoad(p1, x - vec2<i32>(0, 1)).r;
        }
        let v = textureLoad(v1, x);
        let dv = vec4<f32>(factor * (p1_v - p0_v), 0.0, 0.0, 0.0);
        textureStore(v0, x, v - dv);
    }
}

fn is_solid(label: texture_storage_2d<r32uint, read_write>, x: vec2<i32>) -> f32 {
    if (textureLoad(label, x).r == 2) {
        return 1.0;
    } else {
        return 0.0;
    }
}