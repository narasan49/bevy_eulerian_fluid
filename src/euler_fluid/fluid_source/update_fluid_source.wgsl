struct FluidSourceData {
    center: vec2f,
    data: vec2f,
    velocity: vec2f,
    shape: u32,
    mode: u32,
}

struct FluidSrouceUniform {
    data: array<FluidSourceData, 16>,
    count: u32,
}

const SHAPE_CIRCLE: u32 = 0;
const SHAPE_AABB: u32 = 1;

const MODE_SOURCE: u32 = 0;
const MODE_SINK: u32 = 1;

const LARGE_FLOAT: f32 = 1.0e6;

@group(0) @binding(0) var levelset_air: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var u: texture_storage_2d<r32float, write>;
@group(0) @binding(2) var v: texture_storage_2d<r32float, write>;

@group(1) @binding(0) var<uniform> fluid_source_uniform: FluidSrouceUniform;

@compute @workgroup_size(8, 8, 1)
fn update_fluid_source(
    @builtin(global_invocation_id) global_invocation_id: vec3u,
) {
    let idx = global_invocation_id.xy;

    for (var i: u32 = 0; i < fluid_source_uniform.count; i++) {
        let data = fluid_source_uniform.data[i];
        let source_sdf = level_source(data, idx);

        var new_level = textureLoad(levelset_air, idx).r;
        if data.mode == MODE_SOURCE {
            new_level = min(new_level, source_sdf);
        } else {
            new_level = max(new_level, -source_sdf);
        }
        textureStore(levelset_air, idx, vec4f(new_level, vec3f(0.0)));
        if source_sdf < 0.0 && data.mode == MODE_SOURCE {
            textureStore(u, idx, vec4f(data.velocity.x, vec3f(0.0)));
            textureStore(u, idx + vec2u(1, 0), vec4f(data.velocity.x, vec3f(0.0)));
            textureStore(v, idx, vec4f(data.velocity.y, vec3f(0.0)));
            textureStore(v, idx + vec2u(0, 1), vec4f(data.velocity.y, vec3f(0.0)));
            return;
        }
    }
}

fn level_source(data: FluidSourceData, idx: vec2u) -> f32 {
    switch data.shape {
        case SHAPE_CIRCLE: {
            let radius = data.data.x;
            return distance(data.center, vec2f(idx)) - radius;
            
        }
        case SHAPE_AABB: {
            let half_size = data.data;
            return level_aabb(half_size, data.center, vec2f(idx));
        }
        default: {
            return 0.0;
        }
    }
}

fn level_aabb(half_size: vec2f, center: vec2f, x: vec2f) -> f32 {
    var level = LARGE_FLOAT;
    let d = abs(center - x) - half_size;
    let is_inside_x = d.x < 0;
    let is_inside_y = d.y < 0;
    if (is_inside_x) {
        if (is_inside_y) {
            level = max(d.x, d.y);
        } else {
            level = d.y;
        }
    } else {
        if (is_inside_y) {
            level = d.x;
        } else {
            level = length(d);
        }
    }
    return level;
}

fn mode_to_sdf(mode: u32) -> f32 {
    switch mode {
        case MODE_SOURCE: {
            return -LARGE_FLOAT;
        }
        case MODE_SINK: {
            return LARGE_FLOAT;
        }
        default: {
            return -LARGE_FLOAT;
        }
    }
}