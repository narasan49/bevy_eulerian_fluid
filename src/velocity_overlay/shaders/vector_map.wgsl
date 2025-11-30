#import bevy_sprite::mesh2d_functions

struct Arrow {
    position: vec2<f32>,
    vector: vec2<f32>,
    color: vec4<f32>,
}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) color: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<storage, read> arrows: array<Arrow>;

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let arrow = arrows[vertex.instance_index];
    var world_from_local = mesh2d_functions::get_world_from_local(vertex.instance_index);
    let vertex_position = mesh2d_functions::mesh2d_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0)).xy;

    let cos_theta = cos(arrow.vector[1]);
    let sin_theta = sin(arrow.vector[1]);
    let vertex_pos = 0.5 * arrow.vector[0] * vec2<f32>(
        vertex_position.x * cos_theta + vertex_position.y * sin_theta,
        -vertex_position.x * sin_theta + vertex_position.y * cos_theta,
    );

    out.world_position = vec4<f32>(arrow.position + vertex_pos, 2.0, 1.0);
    out.clip_position = mesh2d_functions::mesh2d_position_world_to_clip(out.world_position);
    // out.clip_position = mesh2d_functions::mesh2d_position_local_to_clip(world_from_local, vec4<f32>(vertex.position, 1.0));
    out.color = arrow.color;
    return out;
}

@fragment
fn fragment(
    mesh: VertexOutput
) -> @location(0) vec4<f32> {
    return mesh.color;
}
