#import bevy_fluid::fluid_uniform::SimulationUniform;
#import bevy_render::view::uv_to_ndc;
#import bevy_render::maths::PI;

struct Arrow {
    position: vec2<f32>,
    vector: vec2<f32>,
    color: vec4<f32>,
}

struct OverlaySettings {
    max_clamp_speed: f32,
    bin_size: vec2<i32>,
    color: vec4<f32>,
}

@group(0) @binding(0) var u0: texture_storage_2d<r32float, read>;
@group(0) @binding(1) var v0: texture_storage_2d<r32float, read>;
@group(0) @binding(2) var<storage, read_write> arrows: array<Arrow>;

@group(1) @binding(0) var<uniform> overlay_settings: OverlaySettings;

@group(2) @binding(0) var<uniform> simulation_uniform: SimulationUniform;

const NUM_THREADS_PER_WORKGROUP: u32 = 8 * 8 * 1;

@compute @workgroup_size(8, 8, 1)
fn construct_velocity_arrows(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id : vec3<u32>,
    @builtin(local_invocation_index) local_invocation_index: u32,
    @builtin(num_workgroups) num_workgroups: vec3<u32>
) {
    let idx = vec2<i32>(global_invocation_id.xy);
    var u_average = 0.0;
    var v_average = 0.0;

    let base_sampling_idx = overlay_settings.bin_size * idx;

    for (var i = 0; i < overlay_settings.bin_size.x + 1; i++) {
        for (var j = 0; j < overlay_settings.bin_size.y; j++) {
            u_average += textureLoad(u0, base_sampling_idx + vec2<i32>(i, j)).r / f32((overlay_settings.bin_size.x + 1) * overlay_settings.bin_size.y);
        }
    }

    for (var i = 0; i < overlay_settings.bin_size.x; i++) {
        for (var j = 0; j < overlay_settings.bin_size.y + 1; j++) {
            v_average += textureLoad(v0, base_sampling_idx + vec2<i32>(i, j)).r / f32(overlay_settings.bin_size.x * (overlay_settings.bin_size.y + 1));
        }
    }

    let r = clamp(sqrt(u_average * u_average + v_average * v_average), 0, overlay_settings.max_clamp_speed);
    var theta = atan(v_average/u_average);
    if (u_average == 0.0) {
        theta = 0.0;
    } else if (u_average < 0.0) {
        theta += PI;
    }

    let position_ij = vec2<f32>(base_sampling_idx) + vec2<f32>(overlay_settings.bin_size) / 2.0;
    let uv = position_ij / vec2<f32>(textureDimensions(u0) - vec2<u32>(1, 0));
    let ndc = uv_to_ndc(uv);
    let half_size = 0.5 * simulation_uniform.size;
    let world_position = simulation_uniform.fluid_transform * vec4<f32>(ndc * half_size, 0.0, 1.0);

    let workgroup_index = workgroup_id.x + workgroup_id.y * num_workgroups.x + workgroup_id.z * num_workgroups.x * num_workgroups.z;
    let bin_idx = workgroup_index * NUM_THREADS_PER_WORKGROUP + local_invocation_index;
    arrows[bin_idx] = Arrow(world_position.xy, vec2<f32>(r, theta), overlay_settings.color);
}
