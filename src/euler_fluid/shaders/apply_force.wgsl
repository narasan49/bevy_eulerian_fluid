#import bevy_fluid::fluid_uniform::SimulationUniform;

@group(0) @binding(2) var u1: texture_storage_2d<r32float, read_write>;
@group(0) @binding(3) var v1: texture_storage_2d<r32float, read_write>;

@group(1) @binding(0) var<uniform> constants: SimulationUniform;

@group(2) @binding(0) var<storage, read> force: array<vec2<f32>>;
@group(2) @binding(1) var<storage, read> position: array<vec2<f32>>;

@group(3) @binding(0) var levelset: texture_storage_2d<r32float, read_write>;

@compute @workgroup_size(1, 64, 1)
fn apply_force_u(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    var n = arrayLength(&force);
    var net_force = vec2<f32>(0.0, 0.0);
    let levelset = textureLoad(levelset, x).r;
    if (levelset < 0.0) {
        net_force.x = constants.gravity.x;
    }

    loop {
        if (n == 0) {
            break;
        }
        n = n - 1u;
        let f = force[n];
        let p = position[n];
        net_force = net_force + f.x * gaussian_2d(vec2<f32>(x), p, 10.0);
    }

    let u_val = textureLoad(u1, x).r;
    textureStore(u1, x, vec4<f32>(u_val + net_force.x * constants.dt, 0.0, 0.0, 0.0));
}

@compute @workgroup_size(64, 1, 1)
fn apply_force_v(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    var net_force = 0.0;
    var n = arrayLength(&force);
    let levelset = textureLoad(levelset, x).r;

    if (levelset < 0.0) {
        net_force = constants.gravity.y;
    }

    loop {
        if (n == 0) {
            break;
        }
        n = n - 1u;
        let f = force[n];
        let p = position[n];
        net_force = net_force + f.y * gaussian_2d(vec2<f32>(x), p, 10.0);
    }

    let v_val = textureLoad(v1, x).r;
    textureStore(v1, x, vec4<f32>(v_val + net_force * constants.dt, 0.0, 0.0, 0.0));
}

fn gaussian_2d(x: vec2<f32>, x0: vec2<f32>, sigma: f32) -> f32 {
    let b = -1.0 / (2.0 * sigma * sigma);
    return exp(b * dot(x - x0, x - x0));
}