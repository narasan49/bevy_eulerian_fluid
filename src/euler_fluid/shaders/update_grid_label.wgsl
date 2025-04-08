#import bevy_fluid::fluid_uniform::SimulationUniform;

struct Circle {
    radius: f32,
    transform: mat4x4<f32>,
    velocity: vec2<f32>,
}
@group(0) @binding(0) var u0: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var v0: texture_storage_2d<r32float, read_write>;

@group(1) @binding(0) var levelset: texture_storage_2d<r32float, read_write>;
@group(1) @binding(1) var grid_label: texture_storage_2d<r32uint, read_write>;

@group(2) @binding(0) var<storage, read> circles: array<Circle>;

@group(3) @binding(0) var<uniform> simulation_uniform: SimulationUniform;

@compute
@workgroup_size(8, 8, 1)
fn update_grid_label(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = vec2<i32>(i32(global_id.x), i32(global_id.y));
    let dim_grid = textureDimensions(grid_label);
    let xy = to_world(vec2<f32>(x), dim_grid);

    // ToDo: User defined boundary conditions
    if (x.x == 0 || x.x == i32(dim_grid.x) - 1 || x.y == 0 || x.y == i32(dim_grid.y) - 1) {
        textureStore(grid_label, x, vec4<u32>(2, 0, 0, 0));
        textureStore(u0, x, vec4<f32>(0, 0, 0, 0));
        textureStore(v0, x, vec4<f32>(0, 0, 0, 0));
        return;
    }
    
    let total = arrayLength(&circles);
    let level = textureLoad(levelset, x).r;

    var i = 0u;
    var label = 0u;
    if level < 0.0 {
        label = 1u;
    }
    var u = 0.0;
    var v = 0.0;
    loop {
        if (i >= total) {
            break;
        }
        let circle = circles[i];
        let translation = circle.transform[3].xy;
        
        let distance = distance(xy, translation);
        if distance < circle.radius {
            label = 2u;
            u = circle.velocity.x;
            v = circle.velocity.y;
        }

        continuing {
            i = i + 1u;
        }
    }
    textureStore(grid_label, x, vec4<u32>(label, 0, 0, 0));

    if (label == 2u) {
        textureStore(u0, x, vec4<f32>(u, 0.0, 0.0, 0.0));
        textureStore(v0, x, vec4<f32>(v, 0.0, 0.0, 0.0));
    }
}

fn to_world(x: vec2<f32>, dim: vec2<u32>) -> vec2<f32> {
    let uv = x / vec2<f32>(dim);
    // [0, 1] -> [-0.5, 0.5] -> [-0.5 * size, 0.5 * size]
    let xy = vec2<f32>(uv.x - 0.5, -uv.y + 0.5) * simulation_uniform.size;
    return (simulation_uniform.fluid_transform * vec4<f32>(xy, 0.0, 1.0)).xy;
}