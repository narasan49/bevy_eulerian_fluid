#import bevy_fluid::fluid_uniform::SimulationUniform;

struct Circle {
    radius: f32,
    transform: mat4x4<f32>,
    velocity: vec2<f32>,
}
@group(0) @binding(0) var u_solid: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var v_solid: texture_storage_2d<r32float, read_write>;

@group(1) @binding(2) var levelset_solid: texture_storage_2d<r32float, read_write>;

@group(2) @binding(0) var<storage, read> circles: array<Circle>;

@group(3) @binding(0) var<uniform> simulation_uniform: SimulationUniform;

@compute @workgroup_size(8, 8, 1)
fn update_solid(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = vec2<i32>(i32(global_id.x), i32(global_id.y));
    let dim_grid = textureDimensions(levelset_solid);
    let xy_vertex = to_world(vec2<f32>(x) - vec2<f32>(0.5), dim_grid);
    let xy_edge_x = to_world(vec2<f32>(x) - vec2<f32>(0.5, 0.0), dim_grid);
    let xy_edge_y = to_world(vec2<f32>(x) - vec2<f32>(0.0, 0.5), dim_grid);

    // ToDo: User defined boundary conditions
    if (x.x == 0 || x.x == i32(dim_grid.x) || x.y == 0 || x.y == i32(dim_grid.y)) {
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
    loop {
        if (i >= num_circles) {
            break;
        }
        let circle = circles[i];
        let translation = circle.transform[3].xy;

        let distance = distance(xy_vertex, translation);
        let level_vertex = distance - circle.radius;
        if level_vertex < level {
            level = level_vertex;
        }

        let distance_edge_x = distance(xy_edge_x, translation);
        let level_edge_x = distance_edge_x - circle.radius;
        if (level_edge_x < 0.0) {
            u = circle.velocity.x;
        }

        let distance_edge_y = distance(xy_edge_y, translation);
        let level_edge_y = distance_edge_y - circle.radius;
        if (level_edge_y < 0.0) {
            v = circle.velocity.y;
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

fn to_world(x: vec2<f32>, dim: vec2<u32>) -> vec2<f32> {
    let uv = x / vec2<f32>(dim);
    // [0, 1] -> [-0.5, 0.5] -> [-0.5 * size, 0.5 * size]
    let xy = vec2<f32>(uv.x - 0.5, -uv.y + 0.5) * simulation_uniform.size;
    return (simulation_uniform.fluid_transform * vec4<f32>(xy, 0.0, 1.0)).xy;
}