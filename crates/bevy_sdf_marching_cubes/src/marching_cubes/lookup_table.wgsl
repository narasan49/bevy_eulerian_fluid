#define_import_path marching_cubes::lut

// cube vertex indices with range 0-7.
struct Edge {
    // vertices: array<u32, 2>,
    a: u32,
    b: u32,
}

struct EdgeTriangle {
    edges: array<Edge, 3>,
}

struct EdgeTriangles {
    triangles: array<EdgeTriangle, 5>,
    count: u32,
}