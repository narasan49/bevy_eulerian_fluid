#import euler_fluid_3d::workgroup_shape::WG_SIZE

struct FluidSourceData {
    center: vec3f,
    data: vec3f,
    velocity: vec3f,
    shape: u32,
    mode: u32,
}

struct FluidSourceUniform {
    data: array<FluidSourceData, 16>,
    count: u32,
}

const SHAPE_CIRCLE: u32 = 0;
const SHAPE_AABB: u32 = 1;

const MODE_SOURCE: u32 = 0;
const MODE_SINK: u32 = 1;

const LARGE_FLOAT: f32 = 1.0e6;

@group(0) @binding(0) var levelset_air: texture_storage_3d<r32float, read_write>;
@group(0) @binding(1) var u: texture_storage_3d<r32float, write>;
@group(0) @binding(2) var v: texture_storage_3d<r32float, write>;

@group(1) @binding(0) var<uniform> fluid_source_uniform: FluidSourceUniform;

@compute @workgroup_size(WG_SIZE.x, WG_SIZE.y, WG_SIZE.z)
fn update_fluid_source(
    @builtin(global_invocation_id) gid: vec3u,
) {
    let dim = textureDimensions(levelset_air);
    if any(gid >= dim) {
        return;
    }
    let dimf = vec3f(dim);
    let position = vec3f(gid) - 0.5 * dimf;

    var new_level = textureLoad(levelset_air, gid).r;
    var velocity = vec3f(0.0);
    var has_source = false;
    var velocity_updated = false;
    for (var i: u32 = 0; i < fluid_source_uniform.count; i++) {
        let data = fluid_source_uniform.data[i];
        let source_sdf = level_source(data, position);

        if data.mode == MODE_SOURCE {
            new_level = min(new_level, source_sdf);
            has_source = true;
            if source_sdf < 0.0 {
                velocity = velocity + data.velocity;
                velocity_updated = true;
            }
        } else {
            if !has_source {
                new_level = max(new_level, -source_sdf);
            }
        }
    }

    textureStore(levelset_air, gid, vec4f(new_level, vec3f(0.0)));
    
    if velocity_updated {
        textureStore(u, gid, vec4f(velocity.x, vec3f(0.0)));
        textureStore(u, gid + vec3u(1, 0, 0), vec4f(velocity.x, vec3f(0.0)));
        textureStore(v, gid, vec4f(velocity.y, vec3f(0.0)));
        textureStore(v, gid + vec3u(0, 1, 0), vec4f(velocity.y, vec3f(0.0)));
    }
}

fn level_source(data: FluidSourceData, position: vec3f) -> f32 {
    switch data.shape {
        case SHAPE_CIRCLE: {
            let radius = data.data.x;
            return distance(data.center, position) - radius;
            
        }
        case SHAPE_AABB: {
            let half_size = data.data;
            return level_aabb(half_size, data.center, position);
        }
        default: {
            return 0.0;
        }
    }
}

fn level_aabb(half_size: vec3f, center: vec3f, x: vec3f) -> f32 {
    var level = LARGE_FLOAT;
    let d = abs(center - x) - half_size;
    let is_inside_x = d.x < 0;
    let is_inside_y = d.y < 0;
    let is_inside_z = d.z < 0;
    if (is_inside_x) {
        if (is_inside_y) {
            if (is_inside_z) {
                level = max(d.x, max(d.y,  d.z));
            } else {
                level = d.z;
            }
        } else {
            if (is_inside_z) {
                level = d.y;
            } else {
                level  = length(d.yz);
            }
        }
    } else {
        if (is_inside_y) {
            if (is_inside_z) {
                level = d.x;
            } else {
                level = length(d.xz);
            }
        } else {
            if (is_inside_z) {
                level = length(d.xy);
            } else {
                level = length(d);
            }
        }
    }
    return level;
}