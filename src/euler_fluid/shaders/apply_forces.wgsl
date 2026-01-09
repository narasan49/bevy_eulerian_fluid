#import bevy_fluid::fluid_uniform::SimulationUniform;
#import bevy_fluid::area_fraction::area_fraction;

struct Force {
    force: vec2<f32>,
    position: vec2<f32>,
}

@group(0) @binding(0) var u1: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var v1: texture_storage_2d<r32float, read_write>;
@group(0) @binding(2) var levelset_air0: texture_storage_2d<r32float, read>;
@group(0) @binding(3) var<storage, read> forces: array<Force>;

@group(1) @binding(0) var<uniform> constants: SimulationUniform;

@compute @workgroup_size(1, 64, 1)
fn apply_forces_u(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let idx = vec2<i32>(invocation_id.xy);
    
    let level_centers = array<f32, 6>(
        textureLoad(levelset_air0, idx + vec2<i32>(-1, -1)).r,
        textureLoad(levelset_air0, idx + vec2<i32>(0, -1)).r,
        textureLoad(levelset_air0, idx + vec2<i32>(-1, 0)).r,
        textureLoad(levelset_air0, idx + vec2<i32>(0, 0)).r,
        textureLoad(levelset_air0, idx + vec2<i32>(-1, 1)).r,
        textureLoad(levelset_air0, idx + vec2<i32>(0, 1)).r,
    );

    let level_air_vertex_minus = 0.25 * (level_centers[0] + level_centers[1] + level_centers[2] + level_centers[3]);
    let level_air_vertex_plus = 0.25 * (level_centers[2] + level_centers[3] + level_centers[4] + level_centers[5]);
    let area_fraction = area_fraction(level_air_vertex_minus, level_air_vertex_plus);
    if area_fraction == 1.0 {
        textureStore(u1, idx, vec4<f32>(0.0));
        return;
    }

    var net_force = constants.gravity.x;

    var n = arrayLength(&forces);
    loop {
        if (n == 0) {
            break;
        }
        n = n - 1u;
        let f = forces[n];
        net_force = net_force + f.force.x * gaussian_2d(vec2<f32>(idx), f.position, 10.0);
    }

    let u_val = textureLoad(u1, idx).r;
    textureStore(u1, idx, vec4<f32>(u_val + net_force * constants.dt / constants.dx, 0.0, 0.0, 0.0));
}

@compute @workgroup_size(64, 1, 1)
fn apply_forces_v(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let idx = vec2<i32>(invocation_id.xy);
    let level_centers = array<f32, 6>(
        textureLoad(levelset_air0, idx + vec2<i32>(-1, -1)).r,
        textureLoad(levelset_air0, idx + vec2<i32>(-1, 0)).r,
        textureLoad(levelset_air0, idx + vec2<i32>(0, -1)).r,
        textureLoad(levelset_air0, idx + vec2<i32>(0, 0)).r,
        textureLoad(levelset_air0, idx + vec2<i32>(1, -1)).r,
        textureLoad(levelset_air0, idx + vec2<i32>(1, 0)).r,
    );

    let level_air_vertex_minus = 0.25 * (level_centers[0] + level_centers[1] + level_centers[2] + level_centers[3]);
    let level_air_vertex_plus = 0.25 * (level_centers[2] + level_centers[3] + level_centers[4] + level_centers[5]);
    let area_fraction = area_fraction(level_air_vertex_minus, level_air_vertex_plus);
    if area_fraction == 1.0 {
        textureStore(v1, idx, vec4<f32>(0.0));
        return;
    }

    var net_force = constants.gravity.y;

    var n = arrayLength(&forces);
    loop {
        if (n == 0) {
            break;
        }
        n = n - 1u;
        let f = forces[n];
        net_force = net_force + f.force.y * gaussian_2d(vec2<f32>(idx), f.position, 10.0);
    }

    let v_val = textureLoad(v1, idx).r;
    textureStore(v1, idx, vec4<f32>(v_val + net_force * constants.dt / constants.dx, 0.0, 0.0, 0.0));
}

fn gaussian_2d(x: vec2<f32>, x0: vec2<f32>, sigma: f32) -> f32 {
    let b = -1.0 / (2.0 * sigma * sigma);
    return exp(b * dot(x - x0, x - x0));
}
