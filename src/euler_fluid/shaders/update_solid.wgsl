#import bevy_fluid::fluid_uniform::SimulationUniform;

struct Circle {
    radius: f32,
    transform: mat4x4<f32>,
    velocity: vec2<f32>,
}

struct Rectangle {
    half_size: vec2<f32>,
    transform: mat4x4<f32>,
    inverse_transform: mat4x4<f32>,
    velocity: vec2<f32>,
    angular_velocity: f32,
}

@group(0) @binding(0) var u_solid: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var v_solid: texture_storage_2d<r32float, read_write>;
@group(0) @binding(2) var solid_id: texture_storage_2d<r32sint, read_write>;

@group(1) @binding(2) var levelset_solid: texture_storage_2d<r32float, read_write>;

@group(2) @binding(0) var<storage, read> circles: array<Circle>;
@group(2) @binding(1) var<storage, read> rectangles: array<Rectangle>;

@group(3) @binding(0) var<uniform> simulation_uniform: SimulationUniform;

@compute @workgroup_size(8, 8, 1)
fn update_solid(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = vec2<i32>(i32(global_id.x), i32(global_id.y));
    let dim_grid = textureDimensions(levelset_solid);
    let xy_center = to_world(vec2<f32>(x), dim_grid);
    let xy_edge_x = to_world(vec2<f32>(x) - vec2<f32>(0.5, 0.0), dim_grid);
    let xy_edge_y = to_world(vec2<f32>(x) - vec2<f32>(0.0, 0.5), dim_grid);

    // ToDo: User defined boundary conditions
    if (x.x == 0 || x.x == i32(dim_grid.x) - 1 || x.y == 0 || x.y == i32(dim_grid.y) - 1) {
        textureStore(levelset_solid, x, vec4<f32>(0));
        textureStore(u_solid, x, vec4<f32>(0, 0, 0, 0));
        textureStore(v_solid, x, vec4<f32>(0, 0, 0, 0));
        return;
    }
    
    var level = 1.0e6; // initialize to a large value
    let num_circles = arrayLength(&circles);

    var i = 0u;
    var u = 0.0;
    var v = 0.0;
    var solid_id_sample = -1;
    loop {
        if (i >= num_circles) {
            break;
        }
        let circle = circles[i];
        let translation = circle.transform[3].xy;

        let distance_center = distance(xy_center, translation);
        let level_center = distance_center - circle.radius;
        if (level_center < level) {
            level = level_center;
            solid_id_sample = i32(i);
        }

        let distance_edge_x = distance(xy_edge_x, translation);
        let level_edge_x = distance_edge_x - circle.radius;
        if (level_edge_x < 0.5) {
            u = circle.velocity.x;
        }

        let distance_edge_y = distance(xy_edge_y, translation);
        let level_edge_y = distance_edge_y - circle.radius;
        if (level_edge_y < 0.5) {
            v = circle.velocity.y;
        }

        continuing {
            i = i + 1u;
        }
    }
    
    let total_rect = arrayLength(&rectangles);
    i = 0u;
    loop {
        if (i >= total_rect) {
            break;
        }

        let rectangle = rectangles[i];
        
        let level_center = level_rectangle(rectangle, xy_center);
        if (level > level_center) {
            level = level_center;
            solid_id_sample = i32(i) + i32(num_circles);
        }

        let level_edge_x = level_rectangle(rectangle, xy_edge_x);
        if (level_edge_x < 0.5) {
            u = rectangle_velocity(rectangle, xy_edge_x).x;
        }

        let level_edge_y = level_rectangle(rectangle, xy_edge_y);
        if (level_edge_y < 0.5) {
            // flip the y velocity
            v = -rectangle_velocity(rectangle, xy_edge_y).y;
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

fn level_rectangle(rectangle: Rectangle, x: vec2<f32>) -> f32 {
    var level = 1000.0;
    if (determinant(rectangle.transform) == 0.0) {
        return level;
    }
    let x0 = (rectangle.inverse_transform * vec4<f32>(x, 0.0, 1.0)).xy;
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

fn rectangle_velocity(rectangle: Rectangle, x: vec2<f32>) -> vec2<f32> {
    let r = x - rectangle.transform[3].xy;
    let omega = rectangle.angular_velocity;
    let v = rectangle.velocity + vec2<f32>(-omega * r.y, omega * r.x);
    return v;
}