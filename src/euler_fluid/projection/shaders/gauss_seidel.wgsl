#import bevy_fluid::fluid_uniform::SimulationUniform;

@group(0) @binding(0) var p: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var div: texture_storage_2d<r32float, read>;
@group(0) @binding(2) var levelset_air0: texture_storage_2d<r32float, read>;
@group(0) @binding(3) var area_fraction_solid: texture_storage_2d<rgba32float, read>;

@group(1) @binding(0) var<uniform> constants: SimulationUniform;

const WEIGHT = 1.9;

@compute @workgroup_size(8, 8, 1)
fn gauss_seidel_red(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let idx = vec2<i32>(global_invocation_id.xy);

    if ((idx.x + idx.y) % 2 == 1) {
        let p_new = update_pressure(idx);
        textureStore(p, idx, vec4<f32>(p_new, 0.0, 0.0, 0.0));
    }
}

@compute @workgroup_size(8, 8, 1)
fn gauss_seidel_black(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let idx = vec2<i32>(global_invocation_id.xy);

    if ((idx.x + idx.y) % 2 == 0) {
        let p_new = update_pressure(idx);
        textureStore(p, idx, vec4<f32>(p_new, 0.0, 0.0, 0.0));
    }
}

fn update_pressure(idx: vec2<i32>) -> f32 {
    let level_air_ij = textureLoad(levelset_air0, idx).r;
    if (level_air_ij >= 0.0) {
        return 0.0;
    }
    let f = textureLoad(area_fraction_solid, idx);
    let fully_solid = all(f == vec4<f32>(0.0));
    let p_old = textureLoad(p, idx).r;
    if (fully_solid) {
        return 0.0;
    }

    var denom = 0.0;
    var nume = -constants.dx * constants.rho / constants.dt * textureLoad(div, idx).r;
    let offsets = array<vec2<i32>, 4>(
        vec2<i32>(-1, 0),
        vec2<i32>(1, 0),
        vec2<i32>(0, -1),
        vec2<i32>(0, 1),
    );
    let f_vec = array<f32, 4>(f.x, f.y, f.z, f.w);

    let dim = vec2<i32>(textureDimensions(levelset_air0));
    for (var i = 0; i < 4; i++) {
        var j = idx + offsets[i];
        // j = clamp(j, vec2<i32>(0), dim - vec2<i32>(1));
        if all(vec2<i32>(0) <= j) && all(j < dim) {
            let level = textureLoad(levelset_air0, j).r;
            if level < 0.0 {
                denom += f_vec[i];
                nume += f_vec[i] * textureLoad(p, j).r;
            } else {
                denom += f_vec[i] * (1.0 - level / level_air_ij);
            }
        }
    }

    if (abs(denom) < 1.0e-6) {
        return 0.0;
    }

    let p_new = nume / denom;

    return  WEIGHT * p_new + (1.0 - WEIGHT) * p_old;
}
