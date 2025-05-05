@group(0) @binding(1) var levelset_air1: texture_storage_2d<r32float, read_write>;
@group(0) @binding(2) var levelset_solid: texture_storage_2d<r32float, read_write>;

@group(1) @binding(0) var seeds_x: texture_storage_2d<r32float, read_write>;
@group(1) @binding(1) var seeds_y: texture_storage_2d<r32float, read_write>;

@group(2) @binding(0) var p0: texture_storage_2d<r32float, read_write>;

fn set_seed(x: vec2<i32>, seed: vec2<f32>) {
    textureStore(seeds_x, x, vec4<f32>(seed.x, 0.0, 0.0, 0.0));
    textureStore(seeds_y, x, vec4<f32>(seed.y, 0.0, 0.0, 0.0));
}

@compute @workgroup_size(8, 8, 1)
fn initialize(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    let x = vec2<i32>(i32(global_id.x), i32(global_id.y));
    var min_distance = 10.0;
    var min_distance_seed = vec2<f32>(-1.0, -1.0);
    let level = textureLoad(levelset_air1, x).r;
    let p = textureLoad(p0, x).r;
    let level_solid = levelset_solid_grid_center(levelset_solid, x);

    // find the point to intersect the zero level set
    let dim = vec2<i32>(textureDimensions(levelset_air1));
    // array can be accessed only via a constant index
    // let neibors = array<vec2<i32>, 4>(
    //     x + vec2<i32>(-1, 0),
    //     x + vec2<i32>(1, 0),
    //     x + vec2<i32>(0, -1),
    //     x + vec2<i32>(0, 1)
    // );

    // ToDo: Condider if the result is better when using 8 neighbors
    for (var k: i32 = 0; k < 4; k++) {
        let i = select(-1, 1, k % 2 == 0) * select(1, 0, k / 2 == 0);
        let j = select(-1, 1, k % 2 == 0) * select(0, 1, k / 2 == 0);
        let neighbor = x + vec2<i32>(i, j);
        if (neighbor.x < 0 || neighbor.y < 0 || neighbor.x >= dim.x || neighbor.y >= dim.y) {
            continue;
        }
        let p_neighbor = textureLoad(p0, neighbor).r;
        let neighbor_level_solid = levelset_solid_grid_center(levelset_solid, neighbor);
        
        var idx_solid0 = x;
        var idx_solid1 = x;
        var x_solid0 = vec2<f32>(x) + vec2<f32>(-0.5);
        if (i == -1) {
            idx_solid1 = x + vec2<i32>(0, 1);
        } else if (i == 1) {
            idx_solid0 = x + vec2<i32>(1, 0);
            idx_solid1 = x + vec2<i32>(1, 1);
            x_solid0 = vec2<f32>(x) + vec2<f32>(0.5, -0.5);
        } else if (j == -1) {
            idx_solid1 = x + vec2<i32>(1, 0);
        } else if (j == 1) {
            idx_solid0 = x + vec2<i32>(0, 1);
            idx_solid1 = x + vec2<i32>(1, 1);
            x_solid0 = vec2<f32>(x) + vec2<f32>(-0.5, 0.5);
        }
        let level_solid0 = textureLoad(levelset_solid, idx_solid0).r;
        let level_solid1 = textureLoad(levelset_solid, idx_solid1).r;
        if ((is_solid(level_solid0) && !is_solid(level_solid1)) || (!is_solid(level_solid0) && is_solid(level_solid1))) {
            if (!is_air(level)) {
                let distance_to_level_zero = -level_solid0 / (level_solid1 - level_solid0);
                if (abs(distance_to_level_zero) < min_distance) {
                    min_distance = abs(distance_to_level_zero);
                    if (i == -1 || i == 1) {
                        min_distance_seed = vec2<f32>(x_solid0) + vec2<f32>(0.0, distance_to_level_zero);
                    } else if (j == -1 || j == 1) {
                        min_distance_seed = vec2<f32>(x_solid0) + vec2<f32>(distance_to_level_zero, 0.0);
                    }
                }
            }
        }

        let neighbor_level = textureLoad(levelset_air1, neighbor).r;
        if ((is_air(neighbor_level) && !is_air(level))) {
        // if ((is_air(neighbor_level) && !is_air(level)) || (!is_air(neighbor_level) && is_air(level))) {
            let distance_to_level_zero = level / (level - neighbor_level);

            if (abs(distance_to_level_zero) < min_distance) {
                min_distance = abs(distance_to_level_zero);
                min_distance_seed = vec2<f32>(x) + vec2<f32>(distance_to_level_zero * f32(i), distance_to_level_zero * f32(j));
            }
        }
    }
    
    set_seed(x, min_distance_seed);
}

fn levelset_solid_grid_center(
    levelset_solid: texture_storage_2d<r32float, read_write>,
    x: vec2<i32>,
) -> f32 {
    let levelset_solid_iminusjminus = textureLoad(levelset_solid, x).r;
    let levelset_solid_iplusjminus = textureLoad(levelset_solid, x + vec2<i32>(1, 0)).r;
    let levelset_solid_iminusjplus = textureLoad(levelset_solid, x + vec2<i32>(0, 1)).r;
    let levelset_solid_iplusjplus = textureLoad(levelset_solid, x + vec2<i32>(1, 1)).r;
    return 
        (levelset_solid_iminusjminus + levelset_solid_iplusjminus +
        levelset_solid_iminusjplus + levelset_solid_iplusjplus) / 4.0;
}

// level == 0 belongs to empty air
fn is_air(level_air: f32) -> bool {
    return level_air >= 0.0;
}

fn is_solid(level_solid: f32) -> bool {
    return level_solid < 0.0;
}