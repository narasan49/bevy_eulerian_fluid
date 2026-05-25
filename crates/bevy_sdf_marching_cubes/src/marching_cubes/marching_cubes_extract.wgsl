#import marching_cubes::lut::EdgeTriangles;

struct Vertex {
    position: vec4f,
    normal: vec4f,
}

struct DrawIndirectArgs {
    vertex_count: atomic<u32>,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
}

struct MarchingCubesConfig {
    half_size: vec3f,
}

@group(0) @binding(0) var<storage, read_write> vertices: array<Vertex>;
@group(0) @binding(1) var<storage, read_write> indirect_args: DrawIndirectArgs;
@group(0) @binding(2) var sdf: texture_storage_3d<r32float, read>;
@group(0) @binding(3) var grad_sdf: texture_storage_3d<rgba32float, read>;
@group(0) @binding(4) var<storage, read> lookup_table: array<EdgeTriangles, 256>;
@group(0) @binding(5) var<uniform> config: MarchingCubesConfig;

@compute @workgroup_size(8, 8, 8)
fn extract(
    @builtin(global_invocation_id) gid: vec3u,
) {
    let offsets_unit = array<vec3u, 8>(
        vec3u(0, 0, 0),
        vec3u(1, 0, 0),
        vec3u(0, 1, 0),
        vec3u(1, 1, 0),
        vec3u(0, 0, 1),
        vec3u(1, 0, 1),
        vec3u(0, 1, 1),
        vec3u(1, 1, 1),
    );
    let dim = textureDimensions(sdf);
    if (any(gid >= dim)) {
        return;
    }
    let dimf = vec3f(dim);
    
    let x = (vec3f(gid) / dimf - vec3f(0.5)) * config.half_size;
    let offsets = array<vec3f, 8>(
        vec3f(offsets_unit[0]) / dimf,
        vec3f(offsets_unit[1]) / dimf,
        vec3f(offsets_unit[2]) / dimf,
        vec3f(offsets_unit[3]) / dimf,
        vec3f(offsets_unit[4]) / dimf,
        vec3f(offsets_unit[5]) / dimf,
        vec3f(offsets_unit[6]) / dimf,
        vec3f(offsets_unit[7]) / dimf,
    );
    
    let cube_levels = array<f32, 8>(
        textureLoad(sdf, gid + offsets_unit[0]).r,
        textureLoad(sdf, gid + offsets_unit[1]).r,
        textureLoad(sdf, gid + offsets_unit[2]).r,
        textureLoad(sdf, gid + offsets_unit[3]).r,
        textureLoad(sdf, gid + offsets_unit[4]).r,
        textureLoad(sdf, gid + offsets_unit[5]).r,
        textureLoad(sdf, gid + offsets_unit[6]).r,
        textureLoad(sdf, gid + offsets_unit[7]).r,
    );

    let cube_normals = array<vec3f, 8>(
        textureLoad(grad_sdf, gid + offsets_unit[0]).xyz,
        textureLoad(grad_sdf, gid + offsets_unit[1]).xyz,
        textureLoad(grad_sdf, gid + offsets_unit[2]).xyz,
        textureLoad(grad_sdf, gid + offsets_unit[3]).xyz,
        textureLoad(grad_sdf, gid + offsets_unit[4]).xyz,
        textureLoad(grad_sdf, gid + offsets_unit[5]).xyz,
        textureLoad(grad_sdf, gid + offsets_unit[6]).xyz,
        textureLoad(grad_sdf, gid + offsets_unit[7]).xyz,
    );

    // let center = vec3f(0.0);
    // let center2 = vec3f(0.3, 0.0, 0.0);
    // let radius = 0.2;
    // let cube_levels = array<f32, 8>(
    //     level_two_spheres(x + offsets[0], center, radius, center2, radius),
    //     level_two_spheres(x + offsets[1], center, radius, center2, radius),
    //     level_two_spheres(x + offsets[2], center, radius, center2, radius),
    //     level_two_spheres(x + offsets[3], center, radius, center2, radius),
    //     level_two_spheres(x + offsets[4], center, radius, center2, radius),
    //     level_two_spheres(x + offsets[5], center, radius, center2, radius),
    //     level_two_spheres(x + offsets[6], center, radius, center2, radius),
    //     level_two_spheres(x + offsets[7], center, radius, center2, radius),
    // );

    // let cube_normals = array<vec3f, 8>(
    //     normal_sphere(x + offsets[0], center),
    //     normal_sphere(x + offsets[1], center),
    //     normal_sphere(x + offsets[2], center),
    //     normal_sphere(x + offsets[3], center),
    //     normal_sphere(x + offsets[4], center),
    //     normal_sphere(x + offsets[5], center),
    //     normal_sphere(x + offsets[6], center),
    //     normal_sphere(x + offsets[7], center),
    // );

    let cube = array<u32, 8>(
        sdf_bit(cube_levels[0]),
        sdf_bit(cube_levels[1]),
        sdf_bit(cube_levels[2]),
        sdf_bit(cube_levels[3]),
        sdf_bit(cube_levels[4]),
        sdf_bit(cube_levels[5]),
        sdf_bit(cube_levels[6]),
        sdf_bit(cube_levels[7]),
    );

    let lut_idx = cube_to_idx(cube);
    let triangles = lookup_table[lut_idx];
    for (var i = 0u; i < triangles.count; i++) {
        let triangle = triangles.triangles[i];
        let base_idx = atomicAdd(&indirect_args.vertex_count, 3u);
        for (var j = 0u; j < 3u; j++) {
            let edge = triangle.edges[j];

            let phi0 = cube_levels[edge.a];
            let phi1 = cube_levels[edge.b];
            let t = phi0 / (phi0 - phi1);
            let vertex_offset = mix(vec3f(offsets[edge.a]), vec3f(offsets[edge.b]), t);
            let position = vec4f(x + vertex_offset, 1.0);
            let normal = mix(cube_normals[edge.a], cube_normals[edge.b], t);

            vertices[base_idx + j] = Vertex(position, vec4f(normal, 0.0));
        }
    }
}

fn sdf_bit(value: f32) -> u32 {
    if value < 0.0 {
        // inside
        return 0;
    } else {
        // outside
        return 1;
    }
}

fn cube_to_idx(cube: array<u32, 8>) -> u32 {
    var lut_idx = 0u;
    for (var i = 0u; i < 8; i++) {
        lut_idx += cube[i] << i;
    }

    return lut_idx;
}

fn level_sphere(x: vec3f, center: vec3f, radius: f32) -> f32 {
    return length(x - center) - radius;
}

fn normal_sphere(x: vec3f, center: vec3f) -> vec3f {
    return (x - center) / length(x - center);
}

fn level_two_spheres(
    x: vec3f,
    center1: vec3f,
    radius1: f32,
    center2: vec3f,
    radius2: f32
) -> f32 {
    let level1 = level_sphere(x, center1, radius1);
    let level2 = level_sphere(x, center2, radius2);
    return min(level1, level2);
}