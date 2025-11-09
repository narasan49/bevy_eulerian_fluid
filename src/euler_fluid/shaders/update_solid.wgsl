#import bevy_fluid::fluid_uniform::SimulationUniform;
#import bevy_fluid::solid_obstacle::{SolidObstacle, get_circle, get_rectangle, get_capsule, get_triangle, Rectangle, SHAPE_CIRCLE, SHAPE_RECTANGLE, SHAPE_CAPSULE, SHAPE_TRIANGLE};

const LARGE_FLOAT: f32 = 1.0e6;

@group(0) @binding(0) var u_solid: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var v_solid: texture_storage_2d<r32float, read_write>;

@group(1) @binding(0) var levelset_solid: texture_storage_2d<r32float, read_write>;
@group(1) @binding(1) var solid_id: texture_storage_2d<r32sint, read_write>;

@group(2) @binding(0) var<storage, read> obstacles: array<SolidObstacle>;

@group(3) @binding(0) var<uniform> simulation_uniform: SimulationUniform;

@compute @workgroup_size(8, 8, 1)
fn update_solid(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = vec2<i32>(i32(global_id.x), i32(global_id.y));
    let dim_grid = textureDimensions(levelset_solid);
    let xy_center = to_world(vec2<f32>(x), dim_grid);
    let xy_edge_x = to_world(vec2<f32>(x) - vec2<f32>(0.5, 0.0), dim_grid);
    let xy_edge_y = to_world(vec2<f32>(x) - vec2<f32>(0.0, 0.5), dim_grid);

    // ToDo: User defined boundary conditions
    if (any(x == vec2<i32>(0)) || any(x == vec2<i32>(dim_grid) - 1)) {
        textureStore(levelset_solid, x, vec4<f32>(0));
        textureStore(u_solid, x, vec4<f32>(0, 0, 0, 0));
        textureStore(v_solid, x, vec4<f32>(0, 0, 0, 0));
        textureStore(solid_id, x, vec4<i32>(-1, 0, 0, 0));
        return;
    }

    // Initialize solid level set to the domain boundary in uv space
    let tmp = min(x, abs(vec2<i32>(dim_grid) - 1 - x));
    var level = f32(min(tmp.x, tmp.y));

    let num_obstacles = arrayLength(&obstacles);
    var i = 0u;
    var u = 0.0;
    var v = 0.0;
    var solid_id_sample = -1;
    loop {
        if (i >= num_obstacles) {
            break;
        }
        let obstacle = obstacles[i];
        let level_obstacle = level_obstacle(obstacle, xy_center);
        if (level_obstacle < level) {
            level = level_obstacle;
        }

        if (level_obstacle < 0.5) {
            solid_id_sample = i32(obstacle.entity_id);
        }

        let level_edge_x = level_obstacle(obstacle, xy_edge_x);
        if (level_edge_x < 0.5) {
            u = velocity_at(obstacle, xy_edge_x).x;
        }

        let level_edge_y = level_obstacle(obstacle, xy_edge_y);
        if (level_edge_y < 0.5) {
            v = -velocity_at(obstacle, xy_edge_y).y;
        }

        continuing {
            i = i + 1u;
        }
    }

    if (x.y <= i32(dim_grid.y)) {
        textureStore(u_solid, x, vec4<f32>(u, 0.0, 0.0, 0.0));
    }
    if (x.x <= i32(dim_grid.x)) {
        textureStore(v_solid, x, vec4<f32>(v, 0.0, 0.0, 0.0));
    }
    textureStore(levelset_solid, x, vec4<f32>(level, 0.0, 0.0, 0.0));
    textureStore(solid_id, x, vec4<i32>(solid_id_sample, 0, 0, 0));
}

fn to_world(x: vec2<f32>, dim: vec2<u32>) -> vec2<f32> {
    let uv = x / vec2<f32>(dim);
    // [0, 1] -> [-0.5, 0.5] -> [-0.5 * size, 0.5 * size]
    let xy = vec2<f32>(uv.x - 0.5, -uv.y + 0.5) * simulation_uniform.size;
    return (simulation_uniform.fluid_transform * vec4<f32>(xy, 0.0, 1.0)).xy;
}

fn level_rectangle(rectangle: Rectangle, transform: mat4x4<f32>, inverse_transform: mat4x4<f32>, x: vec2<f32>) -> f32 {
    var level = LARGE_FLOAT;
    if (determinant(transform) == 0.0) {
        return level;
    }
    let x0 = (inverse_transform * vec4<f32>(x, 0.0, 1.0)).xy;
    let is_inside_x = abs(x0.x) < rectangle.half_size.x;
    let is_inside_y = abs(x0.y) < rectangle.half_size.y;
    if (is_inside_x) {
        if (is_inside_y) {
            level = max(abs(x0.x) - rectangle.half_size.x, abs(x0.y) - rectangle.half_size.y);
        } else {
            level = abs(x0.y) - rectangle.half_size.y;
        }
    } else {
        if (is_inside_y) {
            level = abs(x0.x) - rectangle.half_size.x;
        } else {
            level = length(vec2<f32>(abs(x0.x) - rectangle.half_size.x, abs(x0.y) - rectangle.half_size.y));
        }
    }
    return level;
}

fn velocity_at(
    obstacle: SolidObstacle,
    x: vec2<f32>
) -> vec2<f32> {
    let r = x - obstacle.transform[3].xy;
    let omega = obstacle.angular_velocity;
    let v = obstacle.linear_velocity + vec2<f32>(-omega * r.y, omega * r.x);
    return v;
}

fn level_obstacle(obstacle: SolidObstacle, x: vec2<f32>) -> f32 {
    switch (obstacle.shape.shape) {
        case SHAPE_CIRCLE: {
            let circle = get_circle(obstacle.shape);
            let translation = obstacle.transform[3].xy;
            return distance(x, translation) - circle.radius;
        }
        case SHAPE_RECTANGLE: {
            let rectangle = get_rectangle(obstacle.shape);
            return level_rectangle(rectangle, obstacle.transform, obstacle.inverse_transform, x);
        }
        case SHAPE_CAPSULE: {
            let capsule = get_capsule(obstacle.shape);
            let a = obstacle.transform * vec4<f32>(capsule.a, 0.0, 1.0);
            let b = obstacle.transform * vec4<f32>(capsule.b, 0.0, 1.0);
            let level = distance_to_line_segment(x, a.xy, b.xy) - capsule.radius;
            return level;
        }
        case SHAPE_TRIANGLE: {
            let triangle = get_triangle(obstacle.shape);
            let p0 = obstacle.transform * vec4<f32>(triangle.a, 0.0, 1.0);
            let p1 = obstacle.transform * vec4<f32>(triangle.b, 0.0, 1.0);
            let p2 = obstacle.transform * vec4<f32>(triangle.c, 0.0, 1.0);

            let dist0 = distance_to_line_segment(x, p0.xy, p1.xy);
            let dist1 = distance_to_line_segment(x, p1.xy, p2.xy);
            let dist2 = distance_to_line_segment(x, p2.xy, p0.xy);
            let sign0 = distance_of_sign(x, p0.xy, p1.xy);
            let sign1 = distance_of_sign(x, p1.xy, p2.xy);
            let sign2 = distance_of_sign(x, p2.xy, p0.xy);

            if (sign0 > 0.0 && sign1 > 0.0 && sign2 > 0.0) {
                return -min(min(dist0, dist1), dist2);
            } else {
                return min(min(dist0, dist1), dist2);
            }
        }
        default: {
            return LARGE_FLOAT;
        }
    }
}

fn distance_to_line_segment(x: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let ab = b - a;
    let t = clamp(dot(x - a, ab) / dot(ab, ab), 0.0, 1.0);
    let projection = a + t * ab;
    return length(x - projection);
}

fn distance_of_sign(x: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let x1 = x - a;
    let ab = b - a;
    return sign(x1.y * ab.x - x1.x * ab.y);
}