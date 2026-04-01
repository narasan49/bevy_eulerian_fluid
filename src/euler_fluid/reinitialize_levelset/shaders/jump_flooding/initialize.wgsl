@group(0) @binding(0) var levelset_air1: texture_storage_2d<r32float, read>;
@group(1) @binding(0) var seeds: texture_storage_2d<rg32float, write>;

fn set_seed(x: vec2<i32>, seed: vec2<f32>) {
    textureStore(seeds, x, vec4<f32>(seed, 0.0, 0.0));
}

@compute @workgroup_size(8, 8, 1)
fn initialize(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    let x = vec2<i32>(i32(global_id.x), i32(global_id.y));
    var min_distance = 10.0;
    var min_distance_seed = vec2<f32>(-1.0, -1.0);
    let level = textureLoad(levelset_air1, x).r;

    // find the point to intersect the zero level set
    let dim = vec2<i32>(textureDimensions(levelset_air1));
    for (var i = -1; i <= 1; i++) {
        for (var j = -1; j <= 1; j++) {
            if (i == 0 && j == 0) {
                continue;
            }
            let neighbor = x + vec2<i32>(i, j);
            if (any(neighbor < vec2<i32>(0)) || any(neighbor >= dim)) {
                continue;
            }
            let neighbor_level = textureLoad(levelset_air1, neighbor).r;
            if ((is_air(neighbor_level) && !is_air(level))) {
                let distance_to_level_zero = level / (level - neighbor_level);

                if (abs(distance_to_level_zero) < min_distance) {
                    min_distance = abs(distance_to_level_zero);
                    min_distance_seed = vec2<f32>(x) + vec2<f32>(distance_to_level_zero * f32(i), distance_to_level_zero * f32(j));
                }
            }
        }
    }

    set_seed(x, min_distance_seed);
}

// level == 0 belongs to empty air
fn is_air(level_air: f32) -> bool {
    return level_air >= 0.0;
}
