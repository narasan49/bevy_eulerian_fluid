#import bevy_fluid::fluid_uniform::SimulationUniform;
#import bevy_fluid::area_fraction::area_fraction;

// The number of texture_storage binding for WebGPU is limited to 8.
// So solve_velocity_u and solve_velocity_v have different bindings for u0, u1 and v0, v1.
@group(0) @binding(0) var v0: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var v1: texture_storage_2d<r32float, read_write>;

@group(1) @binding(0) var<uniform> constants: SimulationUniform;

@group(2) @binding(1) var p1: texture_storage_2d<r32float, read_write>;

@group(3) @binding(0) var levelset_air0: texture_storage_2d<r32float, read_write>;
@group(3) @binding(2) var levelset_solid: texture_storage_2d<r32float, read_write>;

@compute @workgroup_size(64, 1, 1)
fn solve_velocity_v(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let factor = constants.dt / (constants.dx * constants.rho);
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let level_minus = textureLoad(levelset_air0, x - vec2<i32>(0, 1)).r;
    let level_plus = textureLoad(levelset_air0, x).r;
    if (level_minus >= 0.0 && level_plus >= 0.0) {
        textureStore(v0, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
    } else {
        var p_ij = textureLoad(p1, x).r;
        var p_ijminus = textureLoad(p1, x - vec2<i32>(0, 1)).r;

        if (level_minus > 0.0) {
            p_ijminus = level_minus / level_plus * p_ij;
        }
        if (level_plus > 0.0) {
            p_ij = level_plus / level_minus * p_ijminus;
        }
        let v = textureLoad(v1, x);

        let level_solid = textureLoad(levelset_solid, x).r;
        let level_solid_right = textureLoad(levelset_solid, x + vec2<i32>(1, 0)).r;
        let fraction = area_fraction(level_solid, level_solid_right);

        let dv = vec4<f32>(factor * (p_ij - p_ijminus) * fraction, 0.0, 0.0, 0.0);
        textureStore(v0, x, v - dv);
    }
}