#import bevy_sprite::mesh2d_functions
#import bevy_sprite::mesh2d_view_bindings::globals

struct Arrow {
    position: vec2<f32>,
    vector: vec2<f32>,
    color: vec4<f32>,
}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv_offset_per_instance: f32,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<storage, read> arrows: array<Arrow>;

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    // workaround: wrong instance_index start location for multiple fluids situation.
    // let arrow = arrows[vertex.instance_index] // does not work..
    let idx = mesh2d_functions::get_tag(vertex.instance_index);
    let arrow = arrows[idx];

    let cos_theta = cos(arrow.vector[1]);
    let sin_theta = sin(arrow.vector[1]);
    let arrow_vertex_local_pos = arrow.vector[0] * vec2<f32>(
        vertex.position.x * cos_theta + vertex.position.y * sin_theta,
        -vertex.position.x * sin_theta + vertex.position.y * cos_theta,
    );

    out.world_position = vec4<f32>(arrow.position + arrow_vertex_local_pos, 2.0, 1.0);
    out.clip_position = mesh2d_functions::mesh2d_position_world_to_clip(out.world_position);
    out.uv = vertex.uv;
    out.color = arrow.color;
    out.uv_offset_per_instance = fract(sin(f32(idx)) * 1000.0);
    return out;
}

@fragment
fn fragment(
    mesh: VertexOutput
) -> @location(0) vec4<f32> {
    let offset = fract(mesh.uv.x - globals.time + mesh.uv_offset_per_instance);

    // sigmoid
    let alpha = 1.0 / (1.0 + exp(-25.0 * (offset - 0.75)));
    return mesh.color * vec4<f32>(vec3<f32>(offset), alpha);
}
