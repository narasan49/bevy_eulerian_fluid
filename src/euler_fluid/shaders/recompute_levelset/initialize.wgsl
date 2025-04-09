@group(0) @binding(1) var levelset_air1: texture_storage_2d<r32float, read_write>;

@group(1) @binding(0) var seeds_x: texture_storage_2d<r32float, read_write>;
@group(1) @binding(1) var seeds_y: texture_storage_2d<r32float, read_write>;

fn set_seed(x: vec2<i32>, seed: vec2<f32>) {
    textureStore(seeds_x, x, vec4<f32>(seed.x, 0.0, 0.0, 0.0));
    textureStore(seeds_y, x, vec4<f32>(seed.y, 0.0, 0.0, 0.0));
}

@compute @workgroup_size(8, 8, 1)
fn initialize(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    let x = vec2<i32>(i32(global_id.x), i32(global_id.y));
    let level = textureLoad(levelset_air1, x).r;
    var min_distance = 10.0;
    var min_distance_seed = vec2<f32>(-1.0, -1.0);

    // find the point to intersect the zero level set
    let dim = vec2<i32>(textureDimensions(levelset_air1));
    // array can be looped over only with a constant index
    // let neibors = array<vec2<i32>, 4>(
    //     x + vec2<i32>(-1, 0),
    //     x + vec2<i32>(1, 0),
    //     x + vec2<i32>(0, -1),
    //     x + vec2<i32>(0, 1)
    // );

    // ToDo: Condider if the result is better when using 8 neighbors
    for (var k: i32 = 0; k < 4; k++) {
        let i = select(-1, 1, k / 2 == 0);
        let j = select(-1, 1, k % 2 == 1);
        let neighbor = x + vec2<i32>(i, j);
        if (neighbor.x < 0 || neighbor.y < 0 || neighbor.x >= dim.x || neighbor.y >= dim.y) {
            continue;
        }
        let neighbor_level = textureLoad(levelset_air1, neighbor).r;

        // Crucial level == 0 belongs to empty air
        if ((level < 0.0 && neighbor_level < 0.0) || (level >= 0.0 && neighbor_level >= 0.0)) {
            continue;
        }
        let distance_to_level_zero = level / (level - neighbor_level);

        if (abs(distance_to_level_zero) < min_distance) {
            min_distance = abs(distance_to_level_zero);
            min_distance_seed = vec2<f32>(x) + vec2<f32>(distance_to_level_zero * f32(i), distance_to_level_zero * f32(j));
        }
    }
    
    set_seed(x, min_distance_seed);
}