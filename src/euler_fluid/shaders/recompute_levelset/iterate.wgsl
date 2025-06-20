@group(0) @binding(0) var seeds_x: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var seeds_y: texture_storage_2d<r32float, read_write>;

@group(1) @binding(0) var<uniform> step: i32;

fn set_seed(x: vec2<i32>, seed: vec2<f32>) {
    textureStore(seeds_x, x, vec4<f32>(seed.x, 0.0, 0.0, 0.0));
    textureStore(seeds_y, x, vec4<f32>(seed.y, 0.0, 0.0, 0.0));
}

fn get_seed(x: vec2<i32>) -> vec2<f32> {
    return vec2<f32>(textureLoad(seeds_x, x).r, textureLoad(seeds_y, x).r);
}

@compute
@workgroup_size(8, 8, 1)
fn iterate(
    @builtin(global_invocation_id) invocation_id: vec3<u32>
) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let size = vec2<i32>(textureDimensions(seeds_x));

    let current_seed = get_seed(x);
    if (all(current_seed == vec2<f32>(-1.0))) {
        return;
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
            if (all(neighbor_seed == vec2<f32>(-1.0))) {
                set_seed(neighbor, current_seed);
            } else {
                let distance_to_seed = distance(vec2<f32>(current_seed.xy), vec2<f32>(neighbor));
                let distance_to_neighbor = distance(vec2<f32>(neighbor_seed.xy), vec2<f32>(neighbor));
                if (distance_to_seed < distance_to_neighbor) {
                    set_seed(neighbor, current_seed);
                }
            }
        }
    }
}