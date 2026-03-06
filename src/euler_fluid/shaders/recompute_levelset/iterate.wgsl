@group(0) @binding(0) var seeds_in: texture_storage_2d<rg32float, read>;
@group(1) @binding(0) var seeds_out: texture_storage_2d<rg32float, write>;

@group(2) @binding(0) var<uniform> step: i32;

fn set_seed(x: vec2<i32>, seed: vec2<f32>) {
    textureStore(seeds_out, x, vec4<f32>(seed, 0.0, 0.0));
}

fn get_seed(x: vec2<i32>) -> vec2<f32> {
    return textureLoad(seeds_in, x).rg;
}

fn is_valid_seed(seed: vec2<f32>) -> bool {
    return all(seed != vec2<f32>(-1.0, -1.0));
}

@compute
@workgroup_size(8, 8, 1)
fn iterate(
    @builtin(global_invocation_id) invocation_id: vec3<u32>
) {
    let x = vec2<i32>(invocation_id.xy);
    let size = vec2<i32>(textureDimensions(seeds_in));

    let current_seed = get_seed(x);
    var best_seed = get_seed(x);
    var best_dist = 1e10;

    if (is_valid_seed(current_seed)) {
        best_dist = distance(current_seed, vec2<f32>(x));
    }
    
    for (var i: i32 = -1; i <= 1; i++) {
        for (var j: i32 = -1; j <= 1; j++) {
            if (i == 0 && j == 0) {
                continue;
            }
            let neighbor = vec2<i32>(x.x + i * step, x.y + j * step);
            if (any(neighbor < vec2<i32>(0)) || any(size <= neighbor)) {
                continue;
            }
            let neighbor_seed = get_seed(neighbor);
            if (!is_valid_seed(neighbor_seed)) {
                continue;
            }
            let candidate_dist = distance(neighbor_seed, vec2<f32>(x));
            if (candidate_dist < best_dist) {
                best_seed = neighbor_seed;
                best_dist = candidate_dist;
            }
        }
    }

    set_seed(x, best_seed);
}