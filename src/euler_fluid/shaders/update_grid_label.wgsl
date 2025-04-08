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
    let uv = vec2<f32>(f32(x.x) / f32(dim_grid.x), f32(x.y) / f32(dim_grid.y));
    let xy = (vec4<f32>(uv, 0.0, 1.0) * simulation_uniform.fluid_transform).xy;

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
        let translation = circle.transform[3].xz;
        let dx = xy.x - translation.x;
        let dy = xy.y - translation.y;
        let squared_distance = dx * dx + dy * dy;
        let squared_radius = circle.radius * circle.radius;

        if squared_distance < squared_radius {
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