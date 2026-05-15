#define_import_path euler_fluid_3d::area_fraction

fn load_area_fraction(
    area_fraction: texture_storage_3d<rgba32float, read>,
    idx: vec3u,
) -> array<f32, 6> {
    let dim = textureDimensions(area_fraction);

    let f_minus = textureLoad(area_fraction, idx);
    let f_plus_x = textureLoad(area_fraction, idx + vec3u(1, 0, 0)).r;
    let f_plus_y = textureLoad(area_fraction, idx + vec3u(0, 1, 0)).r;
    let f_plus_z = textureLoad(area_fraction, idx + vec3u(0, 0, 1)).r;

    return array<f32, 6>(
        f_minus[0],
        f_plus_x,
        f_minus[1],
        f_plus_y,
        f_minus[2],
        f_plus_z,
    );
}

fn fully_solid(f: array<f32, 6>) -> bool {
    return f[0] == 0.0 && f[1] == 0.0 && f[2] == 0.0 && f[3] == 0.0 && f[4] == 0.0 && f[5] == 0.0;
}

fn area_fraction(
    levelset: texture_storage_3d<r32float, read>,
    idx: vec3i
) -> vec3f {
    let dimi = vec3i(textureDimensions(levelset));
    let offsets = array<vec3i, 27>(
        vec3i(-1, -1, -1), vec3i(0, -1, -1), vec3i(1, -1, -1),
        vec3i(-1, 0, -1), vec3i(0, 0, -1), vec3i(1, 0, -1),
        vec3i(-1, 1, -1), vec3i(0, 1, -1), vec3i(1, 1, -1),
        vec3i(-1, -1, 0), vec3i(0, -1, 0), vec3i(1, -1, 0),
        vec3i(-1, 0, 0), vec3i(0, 0, 0), vec3i(1, 0, 0),
        vec3i(-1, 1, 0), vec3i(0, 1, 0), vec3i(1, 1, 0),
        vec3i(-1, -1, 1), vec3i(0, -1, 1), vec3i(1, -1, 1),
        vec3i(-1, 0, 1), vec3i(0, 0, 1), vec3i(1, 0, 1),
        vec3i(-1, 1, 1), vec3i(0, 1, 1), vec3i(1, 1, 1),
    );
    let levels = array<f32, 27>(
        load_levelset(levelset, idx + offsets[0], dimi),
        load_levelset(levelset, idx + offsets[1], dimi),
        load_levelset(levelset, idx + offsets[2], dimi),
        load_levelset(levelset, idx + offsets[3], dimi),
        load_levelset(levelset, idx + offsets[4], dimi),
        load_levelset(levelset, idx + offsets[5], dimi),
        load_levelset(levelset, idx + offsets[6], dimi),
        load_levelset(levelset, idx + offsets[7], dimi),
        load_levelset(levelset, idx + offsets[8], dimi),
        load_levelset(levelset, idx + offsets[9], dimi),
        load_levelset(levelset, idx + offsets[10], dimi),
        load_levelset(levelset, idx + offsets[11], dimi),
        load_levelset(levelset, idx + offsets[12], dimi),
        load_levelset(levelset, idx + offsets[13], dimi),
        load_levelset(levelset, idx + offsets[14], dimi),
        load_levelset(levelset, idx + offsets[15], dimi),
        load_levelset(levelset, idx + offsets[16], dimi),
        load_levelset(levelset, idx + offsets[17], dimi),
        load_levelset(levelset, idx + offsets[18], dimi),
        load_levelset(levelset, idx + offsets[19], dimi),
        load_levelset(levelset, idx + offsets[20], dimi),
        load_levelset(levelset, idx + offsets[21], dimi),
        load_levelset(levelset, idx + offsets[22], dimi),
        load_levelset(levelset, idx + offsets[23], dimi),
        load_levelset(levelset, idx + offsets[24], dimi),
        load_levelset(levelset, idx + offsets[25], dimi),
        load_levelset(levelset, idx + offsets[26], dimi),
    );
    let level_vertices = array<f32, 7>(
        // [i - 0.5, j - 0.5, k - 0.5]
        level_vertex(levels, 0),
        // [i + 0.5, j - 0.5, k - 0.5]
        level_vertex(levels, 1),
        // [i - 0.5, j + 0.5, k - 0.5]
        level_vertex(levels, 3),
        // [i + 0.5, j + 0.5, k - 0.5]
        level_vertex(levels, 4),
        // [i - 0.5, j - 0.5, k + 0.5]
        level_vertex(levels, 9),
        // [i + 0.5, j - 0.5, k + 0.5]
        level_vertex(levels, 10),
        // [i - 0.5, j + 0.5, k + 0.5]
        level_vertex(levels, 12),
    );
    var area_fraction_x = 0.0;
    var area_fraction_y = 0.0;
    var area_fraction_z = 0.0;
    // X-face
    if idx.x != 0 {
        let level_vertices_x_face = array<f32, 4>(
            level_vertices[0],
            level_vertices[2],
            level_vertices[4],
            level_vertices[6],
        );
        let level_center_x = 0.5 * (levels[12] + levels[13]);
        area_fraction_x = area_fraction_face(level_vertices_x_face, level_center_x);
    }
    // Y-face
    if idx.y != 0 {
        let level_vertices_y_face = array<f32, 4>(
            level_vertices[0],
            level_vertices[1],
            level_vertices[4],
            level_vertices[5],
        );
        let level_center_y = 0.5 * (levels[10] + levels[13]);
        area_fraction_y = area_fraction_face(level_vertices_y_face, level_center_y);
    }
    // Z-face
    if idx.z != 0 {
        let level_vertices_z_face = array<f32, 4>(
            level_vertices[0],
            level_vertices[1],
            level_vertices[2],
            level_vertices[3],
        );
        let level_center_z = 0.5 * (levels[4] + levels[13]);
        area_fraction_z = area_fraction_face(level_vertices_z_face, level_center_z);
    }

    return vec3f(area_fraction_x, area_fraction_y, area_fraction_z);
}

fn load_levelset(
    levelset: texture_storage_3d<r32float, read>,
    idx: vec3i,
    dim: vec3i,
) -> f32 {
    if all(vec3i(0) <= idx) && all(idx < dim) {
        return textureLoad(levelset, idx).r;
    } else {
        return 0.0;
    }
}

fn level_vertex(level_table: array<f32, 27>, base_idx: u32) -> f32{
    return 0.125 * (
        level_table[base_idx] + 
        level_table[base_idx + 1] + 
        level_table[base_idx + 3] + 
        level_table[base_idx + 4] + 
        level_table[base_idx + 9] + 
        level_table[base_idx + 10] + 
        level_table[base_idx + 12] + 
        level_table[base_idx + 13]
    );
}

fn area_fraction_face(level_vertices: array<f32, 4>, level_center: f32) -> f32 {
    let triangles = array<vec3f, 4>(
        vec3f(level_center, level_vertices[0], level_vertices[1]),
        vec3f(level_center, level_vertices[1], level_vertices[3]),
        vec3f(level_center, level_vertices[3], level_vertices[2]),
        vec3f(level_center, level_vertices[2], level_vertices[0]),
    );

    var fraction = 0.0;
    for (var i = 0; i < 4; i++) {
        let fraction_triangle = area_fraction_triangle(triangles[i]);
        fraction += 0.25 * fraction_triangle;
    }

    return fraction;
}

fn area_fraction_triangle(triangle: vec3f) -> f32 {
    var ordered = triangle;
    if ordered.x > ordered.y {
        let tmp = ordered.x;
        ordered.x = ordered.y;
        ordered.y = tmp;
    }
    if ordered.x > ordered.z {
        let tmp = ordered.x;
        ordered.x = ordered.z;
        ordered.z = tmp;
    }
    if ordered.y > ordered.z {
        let tmp = ordered.y;
        ordered.y = ordered.z;
        ordered.z = tmp;
    }

    if 0.0 < ordered.x {
        return 0.0;
    } else if ordered.x < 0.0 && 0.0 <= ordered.y {
        let theta10 = ordered.x / (ordered.x - ordered.y);
        let theta20 = ordered.x / (ordered.x - ordered.z);
        return 1.0 -  theta10 * theta20;
    } else if ordered.y < 0.0 && 0.0 <= ordered.z {
        let theta02 = ordered.z / (ordered.z - ordered.x);
        let theta12 = ordered.z / (ordered.z - ordered.y);
        return theta02 * theta12;
    } else {
        return 1.0;
    }
}