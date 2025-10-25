#import bevy_fluid::fluid_uniform::SimulationUniform;

const LARGE_FLOAT: f32 = 1.0e6;

const SHAPE_CIRCLE: u32 = 0;
const SHAPE_RECTANGLE: u32 = 1;
const SHAPE_TRIANGLE: u32 = 4;

struct Circle {
    radius: f32,
}

struct Rectangle {
    half_size: vec2<f32>,
}

struct ShapeVariant {
    shape: u32,
    values: array<f32, 6>,
}

fn get_circle(variant: ShapeVariant) -> Circle {
    return Circle(variant.values[0]);
}

fn get_rectangle(variant: ShapeVariant) -> Rectangle {
    return Rectangle(vec2<f32>(variant.values[0], variant.values[1]));
}

struct SolidObstacle {
    entity_id: u32,
    shape: ShapeVariant,
    transform: mat4x4<f32>,
    inverse_transform: mat4x4<f32>,
    linear_velocity: vec2<f32>,
    angular_velocity: f32,
}

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
        default: {
            return LARGE_FLOAT;
        }
    }
}