#import bevy_fluid::levelset_utils::{project_onto_surface, levelset_solid_grid_center};

@group(0) @binding(0) var p0: texture_storage_2d<r32float, read_write>;

@group(1) @binding(2) var levelset_solid: texture_storage_2d<r32float, read_write>;

@compute @workgroup_size(8, 8, 1)
fn update_solid_pressure(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let x = vec2<i32>(i32(global_invocation_id.x), i32(global_invocation_id.y));
    let levelset_solid_x = textureLoad(levelset_solid, x).r;

    if (levelset_solid_x >= 0.0) {
        return;
    }

    let surface = project_onto_surface(levelset_solid, vec2<f32>(x), x, -0.5);
    // sample positive pressure from the solid surface
    let surface_floor = vec2<i32>(i32(surface.x), i32(surface.y));
    var p = 0.0;
    for (var i = 0; i <= 2; i += 1) {
        for (var j = 0; j <= 2; j += 1) {
            let offset = vec2<i32>((i + 3) % 3 - 3, (j + 3) % 3 - 3);
            let levelset_solid_surface = textureLoad(levelset_solid, surface_floor + offset).r;
            if (levelset_solid_surface >= 1.0) {
                p = textureLoad(p0, surface_floor + offset).r;
                break;
            }
        }
    }

    textureStore(p0, x, vec4<f32>(p, 0.0, 0.0, 0.0));
}