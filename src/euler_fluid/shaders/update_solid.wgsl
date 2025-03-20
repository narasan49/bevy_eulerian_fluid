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
}
@group(0) @binding(0) var u_solid: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var v_solid: texture_storage_2d<r32float, read_write>;

@group(1) @binding(2) var levelset_solid: texture_storage_2d<r32float, read_write>;

@group(2) @binding(0) var<storage, read> circles: array<Circle>;
@group(2) @binding(1) var<storage, read> rectangles: array<Rectangle>;

@group(3) @binding(0) var<uniform> simulation_uniform: SimulationUniform;

@compute
@workgroup_size(8, 8, 1)
fn update_solid(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = vec2<i32>(i32(global_id.x), i32(global_id.y));
    let dim_grid = textureDimensions(levelset_solid) - vec2<u32>(1, 1);
    let uv_vertex = vec2<f32>((f32(x.x) - 0.5) / f32(dim_grid.x), (f32(x.y) - 0.5) / f32(dim_grid.y));
    let xy_vertex = (vec4<f32>(uv_vertex, 0.0, 1.0) * simulation_uniform.fluid_transform).xy;

    let uv_edge_x = vec2<f32>((f32(x.x) - 0.5) / f32(dim_grid.x), f32(x.y) / f32(dim_grid.y));
    let xy_edge_x = (vec4<f32>(uv_edge_x, 0.0, 1.0) * simulation_uniform.fluid_transform).xy;

    let uv_edge_y = vec2<f32>(f32(x.x) / f32(dim_grid.x), (f32(x.y) - 0.5) / f32(dim_grid.y));
    let xy_edge_y = (vec4<f32>(uv_edge_y, 0.0, 1.0) * simulation_uniform.fluid_transform).xy;

    // ToDo: User defined boundary conditions
    if (x.x == 0 || x.x == i32(dim_grid.x) || x.y == 0 || x.y == i32(dim_grid.y)) {
        textureStore(u_solid, x, vec4<f32>(0, 0, 0, 0));
        textureStore(v_solid, x, vec4<f32>(0, 0, 0, 0));
        textureStore(levelset_solid, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
        return;
    }
    
    var level = 1000.0;
    let total = arrayLength(&circles);

    var i = 0u;
    var u = 0.0;
    var v = 0.0;
    loop {
        if (i >= total) {
            break;
        }
        let circle = circles[i];
        let translation = circle.transform[3].xz;
        let dx = xy_vertex.x - translation.x;
        let dy = xy_vertex.y - translation.y;
        let distance = length(vec2<f32>(dx, dy));
        let level_vertex = distance - circle.radius;
        
        if (level > level_vertex) {
            level = level_vertex;
        }

        let dx_edge_x = xy_edge_x.x - translation.x;
        let dy_edge_x = xy_edge_x.y - translation.y;
        let distance_edge_x = length(vec2<f32>(dx_edge_x, dy_edge_x));
        let level_edge_x = distance_edge_x - circle.radius;
        if (level_edge_x < 0.0) {
            u = circle.velocity.x;
        }

        let dx_edge_y = xy_edge_y.x - translation.x;
        let dy_edge_y = xy_edge_y.y - translation.y;
        let distance_edge_y = length(vec2<f32>(dx_edge_y, dy_edge_y));
        let level_edge_y = distance_edge_y - circle.radius;
        if (level_edge_y < 0.0) {
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
        
        let level_rectangle = level_rectangle(rectangle, vec2<f32>(x) - vec2<f32>(0.5));
        if (level > level_rectangle) {
            level = level_rectangle;
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