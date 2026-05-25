use bevy::render::render_resource::ShaderType;

#[derive(ShaderType, Clone, Copy)]
pub struct Edge {
    pub vertices: [u32; 2],
}

#[derive(ShaderType, Clone, Copy)]
pub struct EdgeTriangle {
    pub edges: [Edge; 3],
}

impl EdgeTriangle {
    const INVALID: Self = EdgeTriangle::new([[0, 0], [0, 0], [0, 0]]);
    const fn new(slice: [[u32; 2]; 3]) -> Self {
        // let edges = slice.map(|edge| Edge { vertices: edge });

        Self {
            edges: [
                Edge { vertices: slice[0] },
                Edge { vertices: slice[1] },
                Edge { vertices: slice[2] },
            ],
        }
    }
}

#[derive(ShaderType, Clone, Copy)]
pub struct EdgeTriangles {
    pub triangles: [EdgeTriangle; 5],
    pub count: u32,
}

impl EdgeTriangles {
    const fn one(triangles: [EdgeTriangle; 1]) -> Self {
        Self {
            triangles: [
                triangles[0],
                EdgeTriangle::INVALID,
                EdgeTriangle::INVALID,
                EdgeTriangle::INVALID,
                EdgeTriangle::INVALID,
            ],
            count: 1,
        }
    }

    const fn two(triangles: [EdgeTriangle; 2]) -> Self {
        Self {
            triangles: [
                triangles[0],
                triangles[1],
                EdgeTriangle::INVALID,
                EdgeTriangle::INVALID,
                EdgeTriangle::INVALID,
            ],
            count: 2,
        }
    }

    const fn three(triangles: [EdgeTriangle; 3]) -> Self {
        Self {
            triangles: [
                triangles[0],
                triangles[1],
                triangles[2],
                EdgeTriangle::INVALID,
                EdgeTriangle::INVALID,
            ],
            count: 3,
        }
    }

    const fn four(triangles: [EdgeTriangle; 4]) -> Self {
        Self {
            triangles: [
                triangles[0],
                triangles[1],
                triangles[2],
                triangles[3],
                EdgeTriangle::INVALID,
            ],
            count: 4,
        }
    }

    const fn five(triangles: [EdgeTriangle; 5]) -> Self {
        Self {
            triangles: [
                triangles[0],
                triangles[1],
                triangles[2],
                triangles[3],
                triangles[4],
            ],
            count: 5,
        }
    }
}

const EDGE_INVALID: Edge = Edge { vertices: [0; 2] };

const TRIANGLE_INVALID: EdgeTriangle = EdgeTriangle {
    edges: [EDGE_INVALID; 3],
};

const TRIANGLES_INVALID: EdgeTriangles = EdgeTriangles {
    triangles: [TRIANGLE_INVALID; 5],
    count: 0,
};

const LUT_PATTERN_1A: [EdgeTriangles; 8] = [
    // 0b00000001: 1
    EdgeTriangles::one([EdgeTriangle::new([[0, 1], [0, 4], [0, 2]])]),
    // 0b00000010: 2
    EdgeTriangles::one([EdgeTriangle::new([[1, 3], [1, 5], [0, 1]])]),
    // 0b00000100: 4
    EdgeTriangles::one([EdgeTriangle::new([[0, 2], [2, 6], [2, 3]])]),
    // 0b00001000: 8
    EdgeTriangles::one([EdgeTriangle::new([[2, 3], [3, 7], [1, 3]])]),
    // 0b00010000: 16
    EdgeTriangles::one([EdgeTriangle::new([[4, 5], [4, 6], [0, 4]])]),
    // 0b00100000: 32
    EdgeTriangles::one([EdgeTriangle::new([[5, 7], [4, 5], [1, 5]])]),
    // 0b01000000: 64
    EdgeTriangles::one([EdgeTriangle::new([[4, 6], [6, 7], [2, 6]])]),
    // 0b10000000: 128
    // TRIANGLES_INVALID,
    EdgeTriangles::one([EdgeTriangle::new([[6, 7], [5, 7], [3, 7]])]),
];

const LUT_PATTERN_1B: [EdgeTriangles; 8] = [
    // ~0b00000001: ~1
    EdgeTriangles::one([EdgeTriangle::new([[0, 1], [0, 2], [0, 4]])]),
    // ~0b00000010: ~2
    EdgeTriangles::one([EdgeTriangle::new([[1, 3], [0, 1], [1, 5]])]),
    // ~0b00000100: ~4
    EdgeTriangles::one([EdgeTriangle::new([[0, 2], [2, 3], [2, 6]])]),
    // ~0b00001000: ~8
    EdgeTriangles::one([EdgeTriangle::new([[2, 3], [1, 3], [3, 7]])]),
    // ~0b00010000: ~16
    EdgeTriangles::one([EdgeTriangle::new([[4, 5], [0, 4], [4, 6]])]),
    // ~0b00100000: ~32
    EdgeTriangles::one([EdgeTriangle::new([[5, 7], [1, 5], [4, 5]])]),
    // ~0b01000000: ~64
    EdgeTriangles::one([EdgeTriangle::new([[4, 6], [2, 6], [6, 7]])]),
    // ~0b10000000: ~128
    EdgeTriangles::one([EdgeTriangle::new([[6, 7], [3, 7], [5, 7]])]),
];

// const LUT_PATTERN_2A: [EdgeTriangles; 12] = [TRIANGLES_INVALID; 12];
const LUT_PATTERN_2A: [EdgeTriangles; 12] = [
    // 0x00000011: 3
    EdgeTriangles::two([
        EdgeTriangle::new([[0, 2], [1, 5], [0, 4]]),
        EdgeTriangle::new([[1, 5], [0, 2], [1, 3]]),
    ]),
    // 0x00000110: 5
    EdgeTriangles::two([
        EdgeTriangle::new([[2, 3], [0, 4], [2, 6]]),
        EdgeTriangle::new([[0, 4], [2, 3], [0, 1]]),
    ]),
    // 0x00001010: 10
    EdgeTriangles::two([
        EdgeTriangle::new([[1, 0], [3, 7], [1, 5]]),
        EdgeTriangle::new([[3, 7], [1, 0], [3, 2]]),
    ]),
    // 0x00001100: 12
    EdgeTriangles::two([
        EdgeTriangle::new([[3, 1], [2, 6], [3, 7]]),
        EdgeTriangle::new([[2, 6], [3, 1], [2, 0]]),
    ]),
    // 0x00010001: 17
    EdgeTriangles::two([
        EdgeTriangle::new([[4, 6], [0, 1], [4, 5]]),
        EdgeTriangle::new([[0, 1], [4, 6], [0, 2]]),
    ]),
    // 0x00100010: 34
    EdgeTriangles::two([
        EdgeTriangle::new([[5, 4], [1, 3], [5, 7]]),
        EdgeTriangle::new([[1, 3], [5, 4], [1, 0]]),
    ]),
    // 0x00110000: 48
    EdgeTriangles::two([
        EdgeTriangle::new([[5, 7], [4, 0], [5, 1]]),
        EdgeTriangle::new([[4, 0], [5, 7], [4, 6]]),
    ]),
    // 0x01000100: 68
    EdgeTriangles::two([
        EdgeTriangle::new([[6, 7], [2, 0], [6, 4]]),
        EdgeTriangle::new([[2, 0], [6, 7], [2, 3]]),
    ]),
    // 0x01010000: 80
    EdgeTriangles::two([
        EdgeTriangle::new([[4, 5], [6, 2], [4, 0]]),
        EdgeTriangle::new([[6, 2], [4, 5], [6, 7]]),
    ]),
    // 0x10001000: 136
    EdgeTriangles::two([
        EdgeTriangle::new([[7, 5], [3, 2], [7, 6]]),
        EdgeTriangle::new([[3, 2], [7, 5], [3, 1]]),
    ]),
    // 0x10010000: 160
    EdgeTriangles::two([
        EdgeTriangle::new([[7, 6], [5, 1], [7, 3]]),
        EdgeTriangle::new([[5, 1], [7, 6], [5, 4]]),
    ]),
    // 0x11000000: 192
    EdgeTriangles::two([
        EdgeTriangle::new([[6, 4], [7, 3], [6, 2]]),
        EdgeTriangle::new([[7, 3], [6, 4], [7, 5]]),
    ]),
];

// const LUT_PATTERN_2B: [EdgeTriangles; 12] = [TRIANGLES_INVALID; 12];
const LUT_PATTERN_2B: [EdgeTriangles; 12] = [
    // ~0x00000011: ~3 = 252
    EdgeTriangles::two([
        EdgeTriangle::new([[0, 2], [0, 4], [1, 5]]),
        EdgeTriangle::new([[1, 5], [1, 3], [0, 2]]),
    ]),
    // ~0x00000110: ~5 = 250
    EdgeTriangles::two([
        EdgeTriangle::new([[2, 3], [2, 6], [0, 4]]),
        EdgeTriangle::new([[0, 4], [0, 1], [2, 3]]),
    ]),
    // ~0x00001010: ~9 = 246
    EdgeTriangles::two([
        EdgeTriangle::new([[1, 0], [1, 5], [3, 7]]),
        EdgeTriangle::new([[3, 7], [3, 2], [1, 0]]),
    ]),
    // ~0x00001100: ~12 = 243
    EdgeTriangles::two([
        EdgeTriangle::new([[3, 1], [3, 7], [2, 6]]),
        EdgeTriangle::new([[2, 6], [2, 0], [3, 1]]),
    ]),
    // ~0x00010001: ~17 = 238
    EdgeTriangles::two([
        EdgeTriangle::new([[4, 6], [4, 5], [0, 1]]),
        EdgeTriangle::new([[0, 1], [0, 2], [4, 6]]),
    ]),
    // ~0x00100010: ~34 = 221
    EdgeTriangles::two([
        EdgeTriangle::new([[5, 4], [5, 7], [1, 3]]),
        EdgeTriangle::new([[1, 3], [1, 0], [5, 4]]),
    ]),
    // ~0x00110000: ~48 = 207
    EdgeTriangles::two([
        EdgeTriangle::new([[5, 7], [5, 1], [4, 0]]),
        EdgeTriangle::new([[4, 0], [4, 6], [5, 7]]),
    ]),
    // ~0x01000100: ~68 = 187
    EdgeTriangles::two([
        EdgeTriangle::new([[6, 7], [6, 4], [2, 0]]),
        EdgeTriangle::new([[2, 0], [2, 3], [6, 7]]),
    ]),
    // ~0x01010000: ~80 = 175
    EdgeTriangles::two([
        EdgeTriangle::new([[4, 5], [4, 0], [6, 2]]),
        EdgeTriangle::new([[6, 2], [6, 7], [4, 5]]),
    ]),
    // ~0x10001000: ~136 = 119
    EdgeTriangles::two([
        EdgeTriangle::new([[7, 5], [7, 6], [3, 2]]),
        EdgeTriangle::new([[3, 2], [3, 1], [7, 5]]),
    ]),
    // ~0x10010000: ~160 = 95
    EdgeTriangles::two([
        EdgeTriangle::new([[7, 6], [7, 3], [5, 1]]),
        EdgeTriangle::new([[5, 1], [5, 4], [7, 6]]),
    ]),
    // ~0x11000000: ~192 = 63
    EdgeTriangles::two([
        EdgeTriangle::new([[6, 4], [6, 2], [7, 3]]),
        EdgeTriangle::new([[7, 3], [7, 5], [6, 4]]),
    ]),
];

const LUT_PATTERN_3A: [EdgeTriangles; 12] = [
    // 0x00000110: 6
    EdgeTriangles::four([
        EdgeTriangle::new([[2, 6], [1, 0], [2, 0]]),
        EdgeTriangle::new([[1, 0], [2, 6], [1, 5]]),
        EdgeTriangle::new([[3, 2], [1, 5], [2, 6]]),
        EdgeTriangle::new([[1, 5], [3, 2], [3, 1]]),
    ]),
    // 0x00001001: 9
    EdgeTriangles::four([
        EdgeTriangle::new([[0, 4], [3, 1], [0, 1]]),
        EdgeTriangle::new([[3, 1], [0, 4], [3, 7]]),
        EdgeTriangle::new([[2, 0], [3, 7], [0, 4]]),
        EdgeTriangle::new([[3, 7], [2, 0], [2, 3]]),
    ]),
    // 0x00010010: 18
    EdgeTriangles::four([
        EdgeTriangle::new([[4, 6], [1, 5], [4, 5]]),
        EdgeTriangle::new([[1, 5], [4, 6], [1, 3]]),
        EdgeTriangle::new([[0, 4], [1, 3], [4, 6]]),
        EdgeTriangle::new([[1, 3], [0, 4], [0, 1]]),
    ]),
    // 0x00010100: 20
    EdgeTriangles::four([
        EdgeTriangle::new([[2, 3], [4, 6], [2, 6]]),
        EdgeTriangle::new([[4, 6], [2, 3], [4, 5]]),
        EdgeTriangle::new([[0, 2], [4, 5], [2, 3]]),
        EdgeTriangle::new([[4, 5], [0, 2], [0, 4]]),
    ]),
    // 0x00100001: 33
    EdgeTriangles::four([
        EdgeTriangle::new([[0, 2], [5, 4], [0, 4]]),
        EdgeTriangle::new([[5, 4], [0, 2], [5, 7]]),
        EdgeTriangle::new([[1, 0], [5, 7], [0, 2]]),
        EdgeTriangle::new([[5, 7], [1, 0], [1, 5]]),
    ]),
    // 0x00503000: 40
    EdgeTriangles::four([
        EdgeTriangle::new([[5, 4], [3, 7], [5, 7]]),
        EdgeTriangle::new([[3, 7], [5, 4], [3, 2]]),
        EdgeTriangle::new([[1, 5], [3, 2], [5, 4]]),
        EdgeTriangle::new([[3, 2], [1, 5], [1, 3]]),
    ]),
    // 0x01000001: 65
    EdgeTriangles::four([
        EdgeTriangle::new([[6, 7], [0, 4], [6, 4]]),
        EdgeTriangle::new([[0, 4], [6, 7], [0, 1]]),
        EdgeTriangle::new([[2, 6], [0, 1], [6, 7]]),
        EdgeTriangle::new([[0, 1], [2, 6], [2, 0]]),
    ]),
    // 0x01001000: 72
    EdgeTriangles::four([
        EdgeTriangle::new([[3, 1], [6, 7], [3, 7]]),
        EdgeTriangle::new([[6, 7], [3, 1], [6, 4]]),
        EdgeTriangle::new([[2, 3], [6, 4], [3, 1]]),
        EdgeTriangle::new([[6, 4], [2, 3], [2, 6]]),
    ]),
    // 0x01100000: 96
    EdgeTriangles::four([
        EdgeTriangle::new([[4, 0], [7, 6], [4, 6]]),
        EdgeTriangle::new([[7, 6], [4, 0], [7, 3]]),
        EdgeTriangle::new([[5, 4], [7, 3], [4, 0]]),
        EdgeTriangle::new([[7, 3], [5, 4], [5, 7]]),
    ]),
    // 0x10000010: 130
    EdgeTriangles::four([
        EdgeTriangle::new([[1, 0], [7, 5], [1, 5]]),
        EdgeTriangle::new([[7, 5], [1, 0], [7, 6]]),
        EdgeTriangle::new([[3, 1], [7, 6], [1, 0]]),
        EdgeTriangle::new([[7, 6], [3, 1], [3, 7]]),
    ]),
    // 0x10000100: 132
    EdgeTriangles::four([
        EdgeTriangle::new([[7, 5], [2, 6], [7, 6]]),
        EdgeTriangle::new([[2, 6], [7, 5], [2, 0]]),
        EdgeTriangle::new([[3, 7], [2, 0], [7, 5]]),
        EdgeTriangle::new([[2, 0], [3, 7], [3, 2]]),
    ]),
    // 0x10010000: 144
    EdgeTriangles::four([
        EdgeTriangle::new([[4, 0], [7, 6], [4, 6]]),
        EdgeTriangle::new([[7, 6], [4, 0], [7, 3]]),
        EdgeTriangle::new([[5, 4], [7, 3], [4, 0]]),
        EdgeTriangle::new([[7, 3], [5, 4], [5, 7]]),
    ]),
];

const LUT_PATTERN_3B: [EdgeTriangles; 12] = [
    // 0x00000110: ~6 = 249
    EdgeTriangles::two([
        EdgeTriangle::new([[1, 0], [1, 5], [1, 3]]),
        EdgeTriangle::new([[2, 3], [2, 6], [2, 0]]),
    ]),
    // 0x00001001: ~9 = 246
    EdgeTriangles::two([
        EdgeTriangle::new([[0, 2], [0, 4], [0, 1]]),
        EdgeTriangle::new([[3, 1], [3, 7], [3, 2]]),
    ]),
    // 0x00010010: ~18 = 237
    EdgeTriangles::two([
        EdgeTriangle::new([[1, 0], [1, 5], [1, 3]]),
        EdgeTriangle::new([[4, 6], [4, 5], [4, 0]]),
    ]),
    // 0x00010100: ~20 = 235
    EdgeTriangles::two([
        EdgeTriangle::new([[2, 3], [2, 6], [2, 0]]),
        EdgeTriangle::new([[4, 6], [4, 5], [4, 0]]),
    ]),
    // 0x00100001: ~33 = 222
    EdgeTriangles::two([
        EdgeTriangle::new([[0, 2], [0, 4], [0, 1]]),
        EdgeTriangle::new([[5, 1], [5, 4], [5, 7]]),
    ]),
    // 0x00101000: ~40 = 215
    EdgeTriangles::two([
        EdgeTriangle::new([[3, 1], [3, 7], [3, 2]]),
        EdgeTriangle::new([[5, 1], [5, 4], [5, 7]]),
    ]),
    // 0x01000001: ~65 = 190
    EdgeTriangles::two([
        EdgeTriangle::new([[0, 2], [0, 4], [0, 1]]),
        EdgeTriangle::new([[6, 7], [6, 4], [6, 2]]),
    ]),
    // 0x01001000: ~72 = 183
    EdgeTriangles::two([
        EdgeTriangle::new([[3, 1], [3, 7], [3, 2]]),
        EdgeTriangle::new([[6, 7], [6, 4], [6, 2]]),
    ]),
    // 0x01100000: ~96 = 159
    EdgeTriangles::two([
        EdgeTriangle::new([[5, 1], [5, 4], [5, 7]]),
        EdgeTriangle::new([[6, 7], [6, 4], [6, 2]]),
    ]),
    // 0x10000010: ~130 = 125
    EdgeTriangles::two([
        EdgeTriangle::new([[1, 0], [1, 5], [1, 3]]),
        EdgeTriangle::new([[7, 3], [7, 5], [7, 6]]),
    ]),
    // 0x10000100: ~132 = 123
    EdgeTriangles::two([
        EdgeTriangle::new([[2, 3], [2, 6], [2, 0]]),
        EdgeTriangle::new([[7, 3], [7, 5], [7, 6]]),
    ]),
    // 0x10010000: ~144 = 111
    EdgeTriangles::two([
        EdgeTriangle::new([[4, 6], [4, 5], [4, 0]]),
        EdgeTriangle::new([[7, 3], [7, 5], [7, 6]]),
    ]),
];

const LUT_PATTERN_4A: [EdgeTriangles; 4] = [
    // 0x00011000: 24
    EdgeTriangles::two([
        EdgeTriangle::new([[3, 1], [3, 7], [3, 2]]),
        EdgeTriangle::new([[4, 6], [4, 5], [4, 0]]),
    ]),
    // 0x00100100: 36
    EdgeTriangles::two([
        EdgeTriangle::new([[2, 3], [2, 6], [2, 0]]),
        EdgeTriangle::new([[5, 1], [5, 4], [5, 7]]),
    ]),
    // 0x01000010: 66
    EdgeTriangles::two([
        EdgeTriangle::new([[1, 0], [1, 5], [1, 3]]),
        EdgeTriangle::new([[6, 7], [6, 4], [6, 2]]),
    ]),
    // 0x10000001: 129
    EdgeTriangles::two([
        EdgeTriangle::new([[0, 2], [0, 4], [0, 1]]),
        EdgeTriangle::new([[7, 3], [7, 5], [7, 6]]),
    ]),
];

const LUT_PATTERN_4B: [EdgeTriangles; 4] = [
    // 0x00011000: ~24 = 231
    EdgeTriangles::two([
        EdgeTriangle::new([[3, 1], [3, 2], [3, 7]]),
        EdgeTriangle::new([[4, 6], [4, 0], [4, 5]]),
    ]),
    // 0x00100100: ~36 = 219
    EdgeTriangles::two([
        EdgeTriangle::new([[2, 3], [2, 0], [2, 6]]),
        EdgeTriangle::new([[5, 1], [5, 7], [5, 4]]),
    ]),
    // 0x01000010: ~66 = 189
    EdgeTriangles::two([
        EdgeTriangle::new([[1, 0], [1, 3], [1, 5]]),
        EdgeTriangle::new([[6, 7], [6, 2], [6, 4]]),
    ]),
    // 0x10000001: ~129 = 126
    EdgeTriangles::two([
        EdgeTriangle::new([[0, 2], [0, 1], [0, 4]]),
        EdgeTriangle::new([[7, 3], [7, 6], [7, 5]]),
    ]),
];

const LUT_PATTERN_5A: [EdgeTriangles; 24] = [
    // 0b00000111: 7
    EdgeTriangles::three([
        EdgeTriangle::new([[2, 3], [1, 3], [2, 6]]),
        EdgeTriangle::new([[2, 6], [1, 3], [1, 5]]),
        EdgeTriangle::new([[2, 6], [1, 5], [0, 4]]),
    ]),
    // 0b00001011: 11
    EdgeTriangles::three([
        EdgeTriangle::new([[0, 2], [2, 3], [0, 4]]),
        EdgeTriangle::new([[0, 4], [2, 3], [3, 7]]),
        EdgeTriangle::new([[0, 4], [3, 7], [1, 5]]),
    ]),
    // 0b00001101: 13
    EdgeTriangles::three([
        EdgeTriangle::new([[1, 3], [0, 1], [3, 7]]),
        EdgeTriangle::new([[3, 7], [0, 1], [0, 4]]),
        EdgeTriangle::new([[3, 7], [0, 4], [2, 6]]),
    ]),
    // 0b00001110: 14
    EdgeTriangles::three([
        EdgeTriangle::new([[0, 1], [0, 2], [1, 5]]),
        EdgeTriangle::new([[1, 5], [0, 2], [2, 6]]),
        EdgeTriangle::new([[1, 5], [2, 6], [3, 7]]),
    ]),
    // 0b00010011: 19
    EdgeTriangles::three([
        EdgeTriangle::new([[1, 5], [4, 5], [1, 3]]),
        EdgeTriangle::new([[1, 3], [4, 5], [4, 6]]),
        EdgeTriangle::new([[1, 3], [4, 6], [0, 2]]),
    ]),
    // 0b00010101: 21
    EdgeTriangles::three([
        EdgeTriangle::new([[4, 6], [2, 6], [4, 5]]),
        EdgeTriangle::new([[4, 5], [2, 6], [2, 3]]),
        EdgeTriangle::new([[4, 5], [2, 3], [0, 1]]),
    ]),
    // 0b00100011: 35
    EdgeTriangles::three([
        EdgeTriangle::new([[4, 5], [0, 4], [5, 7]]),
        EdgeTriangle::new([[5, 7], [0, 4], [0, 2]]),
        EdgeTriangle::new([[5, 7], [0, 2], [1, 3]]),
    ]),
    // 0b00101010: 42
    EdgeTriangles::three([
        EdgeTriangle::new([[3, 7], [5, 7], [2, 3]]),
        EdgeTriangle::new([[2, 3], [5, 7], [4, 5]]),
        EdgeTriangle::new([[2, 3], [4, 5], [0, 1]]),
    ]),
    // 0b00110001: 49
    EdgeTriangles::three([
        EdgeTriangle::new([[0, 1], [1, 5], [0, 2]]),
        EdgeTriangle::new([[0, 2], [1, 5], [5, 7]]),
        EdgeTriangle::new([[0, 2], [5, 7], [4, 6]]),
    ]),
    // 0b00110010: 50
    EdgeTriangles::three([
        EdgeTriangle::new([[0, 4], [0, 1], [4, 6]]),
        EdgeTriangle::new([[4, 6], [0, 1], [1, 3]]),
        EdgeTriangle::new([[4, 6], [1, 3], [5, 7]]),
    ]),
    // 0b01000101: 69
    EdgeTriangles::three([
        EdgeTriangle::new([[0, 4], [4, 6], [0, 1]]),
        EdgeTriangle::new([[0, 1], [4, 6], [6, 7]]),
        EdgeTriangle::new([[0, 1], [6, 7], [2, 3]]),
    ]),
    // 0b01001100: 76
    EdgeTriangles::three([
        EdgeTriangle::new([[6, 7], [3, 7], [4, 6]]),
        EdgeTriangle::new([[4, 6], [3, 7], [1, 3]]),
        EdgeTriangle::new([[4, 6], [1, 3], [0, 2]]),
    ]),
    // 0b01010001: 81
    EdgeTriangles::three([
        EdgeTriangle::new([[2, 6], [0, 2], [6, 7]]),
        EdgeTriangle::new([[6, 7], [0, 2], [0, 1]]),
        EdgeTriangle::new([[6, 7], [0, 1], [4, 5]]),
    ]),
    // 0b01010100: 84
    EdgeTriangles::three([
        EdgeTriangle::new([[0, 2], [0, 4], [2, 3]]),
        EdgeTriangle::new([[2, 3], [0, 4], [4, 5]]),
        EdgeTriangle::new([[2, 3], [4, 5], [6, 7]]),
    ]),
    // 0b01110000: 112
    EdgeTriangles::three([
        EdgeTriangle::new([[5, 7], [6, 7], [1, 5]]),
        EdgeTriangle::new([[1, 5], [6, 7], [2, 6]]),
        EdgeTriangle::new([[1, 5], [2, 6], [0, 4]]),
    ]),
    // 0b10001010: 138
    EdgeTriangles::three([
        EdgeTriangle::new([[5, 7], [1, 5], [6, 7]]),
        EdgeTriangle::new([[6, 7], [1, 5], [0, 1]]),
        EdgeTriangle::new([[6, 7], [0, 1], [2, 3]]),
    ]),
    // 0b10001100: 140
    EdgeTriangles::three([
        EdgeTriangle::new([[2, 6], [6, 7], [0, 2]]),
        EdgeTriangle::new([[0, 2], [6, 7], [5, 7]]),
        EdgeTriangle::new([[0, 2], [5, 7], [1, 3]]),
    ]),
    // 0b10100010: 162
    EdgeTriangles::three([
        EdgeTriangle::new([[1, 3], [3, 7], [0, 1]]),
        EdgeTriangle::new([[0, 1], [3, 7], [6, 7]]),
        EdgeTriangle::new([[0, 1], [6, 7], [4, 5]]),
    ]),
    // 0b10101000: 168
    EdgeTriangles::three([
        EdgeTriangle::new([[1, 5], [1, 3], [4, 5]]),
        EdgeTriangle::new([[4, 5], [1, 3], [2, 3]]),
        EdgeTriangle::new([[4, 5], [2, 3], [6, 7]]),
    ]),
    // 0b10110000: 176
    EdgeTriangles::three([
        EdgeTriangle::new([[6, 7], [4, 6], [3, 7]]),
        EdgeTriangle::new([[3, 7], [4, 6], [0, 4]]),
        EdgeTriangle::new([[3, 7], [0, 4], [1, 5]]),
    ]),
    // 0b11000100: 196
    EdgeTriangles::three([
        EdgeTriangle::new([[3, 7], [2, 3], [5, 7]]),
        EdgeTriangle::new([[5, 7], [2, 3], [0, 2]]),
        EdgeTriangle::new([[5, 7], [0, 2], [4, 6]]),
    ]),
    // 0b11001000: 200
    EdgeTriangles::three([
        EdgeTriangle::new([[2, 3], [2, 6], [1, 3]]),
        EdgeTriangle::new([[1, 3], [2, 6], [4, 6]]),
        EdgeTriangle::new([[1, 3], [4, 6], [5, 7]]),
    ]),
    // 0b11010000: 208
    EdgeTriangles::three([
        EdgeTriangle::new([[4, 5], [5, 7], [0, 4]]),
        EdgeTriangle::new([[0, 4], [5, 7], [3, 7]]),
        EdgeTriangle::new([[0, 4], [3, 7], [2, 6]]),
    ]),
    // 0b11100000: 224
    EdgeTriangles::three([
        EdgeTriangle::new([[4, 6], [4, 5], [2, 6]]),
        EdgeTriangle::new([[2, 6], [4, 5], [1, 5]]),
        EdgeTriangle::new([[2, 6], [1, 5], [3, 7]]),
    ]),
];

// const LUT_PATTERN_5B: [EdgeTriangles; 24] = [TRIANGLES_INVALID; 24];
const LUT_PATTERN_5B: [EdgeTriangles; 24] = [
    // 0b00000111: ~7 = 248
    EdgeTriangles::three([
        EdgeTriangle::new([[2, 3], [2, 6], [1, 3]]),
        EdgeTriangle::new([[2, 6], [1, 5], [1, 3]]),
        EdgeTriangle::new([[2, 6], [0, 4], [1, 5]]),
    ]),
    // 0b00001011: ~11 = 244
    EdgeTriangles::three([
        EdgeTriangle::new([[0, 2], [0, 4], [2, 3]]),
        EdgeTriangle::new([[0, 4], [3, 7], [2, 3]]),
        EdgeTriangle::new([[0, 4], [1, 5], [3, 7]]),
    ]),
    // 0b00001101: ~13 = 242
    EdgeTriangles::three([
        EdgeTriangle::new([[1, 3], [3, 7], [0, 1]]),
        EdgeTriangle::new([[3, 7], [0, 4], [0, 1]]),
        EdgeTriangle::new([[3, 7], [2, 6], [0, 4]]),
    ]),
    // 0b00001110: ~14 = 241
    EdgeTriangles::three([
        EdgeTriangle::new([[0, 1], [1, 5], [0, 2]]),
        EdgeTriangle::new([[1, 5], [2, 6], [0, 2]]),
        EdgeTriangle::new([[1, 5], [3, 7], [2, 6]]),
    ]),
    // 0b00010011: ~19 = 236
    EdgeTriangles::three([
        EdgeTriangle::new([[1, 5], [1, 3], [4, 5]]),
        EdgeTriangle::new([[1, 3], [4, 6], [4, 5]]),
        EdgeTriangle::new([[1, 3], [0, 2], [4, 6]]),
    ]),
    // 0b00010101: ~21 = 234
    EdgeTriangles::three([
        EdgeTriangle::new([[4, 6], [4, 5], [2, 6]]),
        EdgeTriangle::new([[4, 5], [2, 3], [2, 6]]),
        EdgeTriangle::new([[4, 5], [0, 1], [2, 3]]),
    ]),
    // 0b00100011: ~35 = 220
    EdgeTriangles::three([
        EdgeTriangle::new([[4, 5], [5, 7], [0, 4]]),
        EdgeTriangle::new([[5, 7], [0, 2], [0, 4]]),
        EdgeTriangle::new([[5, 7], [1, 3], [0, 2]]),
    ]),
    // 0b00101010: ~42 = 213
    EdgeTriangles::three([
        EdgeTriangle::new([[3, 7], [2, 3], [5, 7]]),
        EdgeTriangle::new([[2, 3], [4, 5], [5, 7]]),
        EdgeTriangle::new([[2, 3], [0, 1], [4, 5]]),
    ]),
    // 0b00110001: ~49 = 206
    EdgeTriangles::three([
        EdgeTriangle::new([[0, 1], [0, 2], [1, 5]]),
        EdgeTriangle::new([[0, 2], [5, 7], [1, 5]]),
        EdgeTriangle::new([[0, 2], [4, 6], [5, 7]]),
    ]),
    // 0b00110010: ~50 = 205
    EdgeTriangles::three([
        EdgeTriangle::new([[0, 4], [4, 6], [0, 1]]),
        EdgeTriangle::new([[4, 6], [1, 3], [0, 1]]),
        EdgeTriangle::new([[4, 6], [5, 7], [1, 3]]),
    ]),
    // 0b01000101: ~69 = 186
    EdgeTriangles::three([
        EdgeTriangle::new([[0, 4], [0, 1], [4, 6]]),
        EdgeTriangle::new([[0, 1], [6, 7], [4, 6]]),
        EdgeTriangle::new([[0, 1], [2, 3], [6, 7]]),
    ]),
    // 0b01001100: ~76 = 179
    EdgeTriangles::three([
        EdgeTriangle::new([[6, 7], [4, 6], [3, 7]]),
        EdgeTriangle::new([[4, 6], [1, 3], [3, 7]]),
        EdgeTriangle::new([[4, 6], [0, 2], [1, 3]]),
    ]),
    // 0b01010001: ~81 = 174
    EdgeTriangles::three([
        EdgeTriangle::new([[2, 6], [6, 7], [0, 2]]),
        EdgeTriangle::new([[6, 7], [0, 1], [0, 2]]),
        EdgeTriangle::new([[6, 7], [4, 5], [0, 1]]),
    ]),
    // 0b01010100: ~84 = 171
    EdgeTriangles::three([
        EdgeTriangle::new([[0, 2], [2, 3], [0, 4]]),
        EdgeTriangle::new([[2, 3], [4, 5], [0, 4]]),
        EdgeTriangle::new([[2, 3], [6, 7], [4, 5]]),
    ]),
    // 0b01110000: ~112 = 143
    EdgeTriangles::three([
        EdgeTriangle::new([[5, 7], [1, 5], [6, 7]]),
        EdgeTriangle::new([[1, 5], [2, 6], [6, 7]]),
        EdgeTriangle::new([[1, 5], [0, 4], [2, 6]]),
    ]),
    // 0b10001010: ~138 = 117
    EdgeTriangles::three([
        EdgeTriangle::new([[5, 7], [6, 7], [1, 5]]),
        EdgeTriangle::new([[6, 7], [0, 1], [1, 5]]),
        EdgeTriangle::new([[6, 7], [2, 3], [0, 1]]),
    ]),
    // 0b10001100: ~140 = 115
    EdgeTriangles::three([
        EdgeTriangle::new([[2, 6], [0, 2], [6, 7]]),
        EdgeTriangle::new([[0, 2], [5, 7], [6, 7]]),
        EdgeTriangle::new([[0, 2], [1, 3], [5, 7]]),
    ]),
    // 0b10100010: ~162 = 93
    EdgeTriangles::three([
        EdgeTriangle::new([[1, 3], [0, 1], [3, 7]]),
        EdgeTriangle::new([[0, 1], [6, 7], [3, 7]]),
        EdgeTriangle::new([[0, 1], [4, 5], [6, 7]]),
    ]),
    // 0b10101000: ~168 = 87
    EdgeTriangles::three([
        EdgeTriangle::new([[1, 5], [4, 5], [1, 3]]),
        EdgeTriangle::new([[4, 5], [2, 3], [1, 3]]),
        EdgeTriangle::new([[4, 5], [6, 7], [2, 3]]),
    ]),
    // 0b10110000: ~176 = 79
    EdgeTriangles::three([
        EdgeTriangle::new([[6, 7], [3, 7], [4, 6]]),
        EdgeTriangle::new([[3, 7], [0, 4], [4, 6]]),
        EdgeTriangle::new([[3, 7], [1, 5], [0, 4]]),
    ]),
    // 0b11000100: ~196 = 59
    EdgeTriangles::three([
        EdgeTriangle::new([[3, 7], [5, 7], [2, 3]]),
        EdgeTriangle::new([[5, 7], [0, 2], [2, 3]]),
        EdgeTriangle::new([[5, 7], [4, 6], [0, 2]]),
    ]),
    // 0b11001000: ~200 = 55
    EdgeTriangles::three([
        EdgeTriangle::new([[2, 3], [1, 3], [2, 6]]),
        EdgeTriangle::new([[1, 3], [4, 6], [2, 6]]),
        EdgeTriangle::new([[1, 3], [5, 7], [4, 6]]),
    ]),
    // 0b11010000: ~208 = 47
    EdgeTriangles::three([
        EdgeTriangle::new([[4, 5], [0, 4], [5, 7]]),
        EdgeTriangle::new([[0, 4], [3, 7], [5, 7]]),
        EdgeTriangle::new([[0, 4], [2, 6], [3, 7]]),
    ]),
    // 0b11100000: ~224 = 31
    EdgeTriangles::three([
        EdgeTriangle::new([[4, 6], [2, 6], [4, 5]]),
        EdgeTriangle::new([[2, 6], [1, 5], [4, 5]]),
        EdgeTriangle::new([[2, 6], [3, 7], [1, 5]]),
    ]),
];

const LUT_PATTERN_6A: [EdgeTriangles; 24] = [
    // 0b00011001: 25
    EdgeTriangles::five([
        EdgeTriangle::new([[4, 6], [0, 2], [2, 3]]),
        EdgeTriangle::new([[4, 6], [2, 3], [3, 7]]),
        EdgeTriangle::new([[4, 5], [4, 6], [3, 7]]),
        EdgeTriangle::new([[4, 5], [1, 3], [0, 1]]),
        EdgeTriangle::new([[4, 5], [3, 7], [1, 3]]),
    ]),
    // 0b00011010: 26
    EdgeTriangles::five([
        EdgeTriangle::new([[3, 7], [1, 5], [4, 5]]),
        EdgeTriangle::new([[3, 7], [4, 5], [4, 6]]),
        EdgeTriangle::new([[2, 3], [3, 7], [4, 6]]),
        EdgeTriangle::new([[2, 3], [0, 4], [0, 1]]),
        EdgeTriangle::new([[2, 3], [4, 6], [0, 4]]),
    ]),
    // 0b00011100: 28
    EdgeTriangles::five([
        EdgeTriangle::new([[1, 3], [0, 2], [0, 4]]),
        EdgeTriangle::new([[1, 3], [0, 4], [4, 5]]),
        EdgeTriangle::new([[3, 7], [1, 3], [4, 5]]),
        EdgeTriangle::new([[3, 7], [4, 6], [2, 6]]),
        EdgeTriangle::new([[3, 7], [4, 5], [4, 6]]),
    ]),
    // 0b00100101: 37
    EdgeTriangles::five([
        EdgeTriangle::new([[2, 3], [0, 1], [1, 5]]),
        EdgeTriangle::new([[2, 3], [1, 5], [5, 7]]),
        EdgeTriangle::new([[2, 6], [2, 3], [5, 7]]),
        EdgeTriangle::new([[2, 6], [4, 5], [0, 4]]),
        EdgeTriangle::new([[2, 6], [5, 7], [4, 5]]),
    ]),
    // 0b00100110: 38
    EdgeTriangles::five([
        EdgeTriangle::new([[4, 5], [0, 1], [0, 2]]),
        EdgeTriangle::new([[4, 5], [0, 2], [2, 6]]),
        EdgeTriangle::new([[5, 7], [4, 5], [2, 6]]),
        EdgeTriangle::new([[5, 7], [2, 3], [1, 3]]),
        EdgeTriangle::new([[5, 7], [2, 6], [2, 3]]),
    ]),
    // 0b00101100: 44
    EdgeTriangles::five([
        EdgeTriangle::new([[2, 6], [3, 7], [5, 7]]),
        EdgeTriangle::new([[2, 6], [5, 7], [4, 5]]),
        EdgeTriangle::new([[0, 2], [2, 6], [4, 5]]),
        EdgeTriangle::new([[0, 2], [1, 5], [1, 3]]),
        EdgeTriangle::new([[0, 2], [4, 5], [1, 5]]),
    ]),
    // 0b00110100: 52
    EdgeTriangles::five([
        EdgeTriangle::new([[5, 7], [4, 6], [2, 6]]),
        EdgeTriangle::new([[5, 7], [2, 6], [2, 3]]),
        EdgeTriangle::new([[1, 5], [5, 7], [2, 3]]),
        EdgeTriangle::new([[1, 5], [0, 2], [0, 4]]),
        EdgeTriangle::new([[1, 5], [2, 3], [0, 2]]),
    ]),
    // 0b00111000: 56
    EdgeTriangles::five([
        EdgeTriangle::new([[0, 4], [1, 5], [1, 3]]),
        EdgeTriangle::new([[0, 4], [1, 3], [2, 3]]),
        EdgeTriangle::new([[4, 6], [0, 4], [2, 3]]),
        EdgeTriangle::new([[4, 6], [3, 7], [5, 7]]),
        EdgeTriangle::new([[4, 6], [2, 3], [3, 7]]),
    ]),
    // 0b01000011: 67
    EdgeTriangles::five([
        EdgeTriangle::new([[1, 5], [0, 4], [4, 6]]),
        EdgeTriangle::new([[1, 5], [4, 6], [6, 7]]),
        EdgeTriangle::new([[1, 3], [1, 5], [6, 7]]),
        EdgeTriangle::new([[1, 3], [2, 6], [0, 2]]),
        EdgeTriangle::new([[1, 3], [6, 7], [2, 6]]),
    ]),
    // 0b01000110: 70
    EdgeTriangles::five([
        EdgeTriangle::new([[6, 7], [2, 3], [1, 3]]),
        EdgeTriangle::new([[6, 7], [1, 3], [1, 5]]),
        EdgeTriangle::new([[4, 6], [6, 7], [1, 5]]),
        EdgeTriangle::new([[4, 6], [0, 1], [0, 2]]),
        EdgeTriangle::new([[4, 6], [1, 5], [0, 1]]),
    ]),
    // 0b01001010: 74
    EdgeTriangles::five([
        EdgeTriangle::new([[0, 1], [2, 3], [2, 6]]),
        EdgeTriangle::new([[0, 1], [2, 6], [4, 6]]),
        EdgeTriangle::new([[1, 5], [0, 1], [4, 6]]),
        EdgeTriangle::new([[1, 5], [6, 7], [3, 7]]),
        EdgeTriangle::new([[1, 5], [4, 6], [6, 7]]),
    ]),
    // 0b01010010: 82
    EdgeTriangles::five([
        EdgeTriangle::new([[2, 6], [0, 4], [0, 1]]),
        EdgeTriangle::new([[2, 6], [0, 1], [1, 3]]),
        EdgeTriangle::new([[6, 7], [2, 6], [1, 3]]),
        EdgeTriangle::new([[6, 7], [1, 5], [4, 5]]),
        EdgeTriangle::new([[6, 7], [1, 3], [1, 5]]),
    ]),
    // 0b01011000: 88
    EdgeTriangles::five([
        EdgeTriangle::new([[4, 5], [6, 7], [3, 7]]),
        EdgeTriangle::new([[4, 5], [3, 7], [1, 3]]),
        EdgeTriangle::new([[0, 4], [4, 5], [1, 3]]),
        EdgeTriangle::new([[0, 4], [2, 3], [2, 6]]),
        EdgeTriangle::new([[0, 4], [1, 3], [2, 3]]),
    ]),
    // 0b01100010: 98
    EdgeTriangles::five([
        EdgeTriangle::new([[1, 3], [5, 7], [6, 7]]),
        EdgeTriangle::new([[1, 3], [6, 7], [2, 6]]),
        EdgeTriangle::new([[0, 1], [1, 3], [2, 6]]),
        EdgeTriangle::new([[0, 1], [4, 6], [4, 5]]),
        EdgeTriangle::new([[0, 1], [2, 6], [4, 6]]),
    ]),
    // 0b01100100: 100
    EdgeTriangles::five([
        EdgeTriangle::new([[0, 2], [4, 6], [4, 5]]),
        EdgeTriangle::new([[0, 2], [4, 5], [1, 5]]),
        EdgeTriangle::new([[2, 3], [0, 2], [1, 5]]),
        EdgeTriangle::new([[2, 3], [5, 7], [6, 7]]),
        EdgeTriangle::new([[2, 3], [1, 5], [5, 7]]),
    ]),
    // 0b10000011: 131
    EdgeTriangles::five([
        EdgeTriangle::new([[0, 2], [1, 3], [3, 7]]),
        EdgeTriangle::new([[0, 2], [3, 7], [6, 7]]),
        EdgeTriangle::new([[0, 4], [0, 2], [6, 7]]),
        EdgeTriangle::new([[0, 4], [5, 7], [1, 5]]),
        EdgeTriangle::new([[0, 4], [6, 7], [5, 7]]),
    ]),
    // 0b10000101: 133
    EdgeTriangles::five([
        EdgeTriangle::new([[0, 4], [2, 6], [6, 7]]),
        EdgeTriangle::new([[0, 4], [6, 7], [5, 7]]),
        EdgeTriangle::new([[0, 1], [0, 4], [5, 7]]),
        EdgeTriangle::new([[0, 1], [3, 7], [2, 3]]),
        EdgeTriangle::new([[0, 1], [5, 7], [3, 7]]),
    ]),
    // 0b10001001: 137
    EdgeTriangles::five([
        EdgeTriangle::new([[5, 7], [1, 3], [0, 1]]),
        EdgeTriangle::new([[5, 7], [0, 1], [0, 4]]),
        EdgeTriangle::new([[6, 7], [5, 7], [0, 4]]),
        EdgeTriangle::new([[6, 7], [0, 2], [2, 3]]),
        EdgeTriangle::new([[6, 7], [0, 4], [0, 2]]),
    ]),
    // 0b10010001: 145
    EdgeTriangles::five([
        EdgeTriangle::new([[0, 1], [4, 5], [5, 7]]),
        EdgeTriangle::new([[0, 1], [5, 7], [3, 7]]),
        EdgeTriangle::new([[0, 2], [0, 1], [3, 7]]),
        EdgeTriangle::new([[0, 2], [6, 7], [4, 6]]),
        EdgeTriangle::new([[0, 2], [3, 7], [6, 7]]),
    ]),
    // 0b10011000: 152
    EdgeTriangles::five([
        EdgeTriangle::new([[2, 3], [6, 7], [4, 6]]),
        EdgeTriangle::new([[2, 3], [4, 6], [0, 4]]),
        EdgeTriangle::new([[1, 3], [2, 3], [0, 4]]),
        EdgeTriangle::new([[1, 3], [4, 5], [5, 7]]),
        EdgeTriangle::new([[1, 3], [0, 4], [4, 5]]),
    ]),
    // 0b10100001: 161
    EdgeTriangles::five([
        EdgeTriangle::new([[6, 7], [4, 5], [0, 4]]),
        EdgeTriangle::new([[6, 7], [0, 4], [0, 2]]),
        EdgeTriangle::new([[3, 7], [6, 7], [0, 2]]),
        EdgeTriangle::new([[3, 7], [0, 1], [1, 5]]),
        EdgeTriangle::new([[3, 7], [0, 2], [0, 1]]),
    ]),
    // 0b10100100: 164
    EdgeTriangles::five([
        EdgeTriangle::new([[1, 5], [3, 7], [2, 3]]),
        EdgeTriangle::new([[1, 5], [2, 3], [0, 2]]),
        EdgeTriangle::new([[4, 5], [1, 5], [0, 2]]),
        EdgeTriangle::new([[4, 5], [2, 6], [6, 7]]),
        EdgeTriangle::new([[4, 5], [0, 2], [2, 6]]),
    ]),
    // 0b11000001: 193
    EdgeTriangles::five([
        EdgeTriangle::new([[3, 7], [2, 6], [0, 2]]),
        EdgeTriangle::new([[3, 7], [0, 2], [0, 1]]),
        EdgeTriangle::new([[5, 7], [3, 7], [0, 1]]),
        EdgeTriangle::new([[5, 7], [0, 4], [4, 6]]),
        EdgeTriangle::new([[5, 7], [0, 1], [0, 4]]),
    ]),
    // 0b11000010: 194
    EdgeTriangles::five([
        EdgeTriangle::new([[4, 6], [5, 7], [1, 5]]),
        EdgeTriangle::new([[4, 6], [1, 5], [0, 1]]),
        EdgeTriangle::new([[2, 6], [4, 6], [0, 1]]),
        EdgeTriangle::new([[2, 6], [1, 3], [3, 7]]),
        EdgeTriangle::new([[2, 6], [0, 1], [1, 3]]),
    ]),
];

const LUT_PATTERN_6B: [EdgeTriangles; 24] = [
    // 0b00111101: 61
    EdgeTriangles::three([
        EdgeTriangle::new([[4, 6], [2, 6], [3, 7]]),
        EdgeTriangle::new([[4, 6], [3, 7], [5, 7]]),
        EdgeTriangle::new([[1, 3], [0, 1], [1, 5]]),
    ]),
    // 0b00111110: 62
    EdgeTriangles::three([
        EdgeTriangle::new([[3, 7], [5, 7], [4, 6]]),
        EdgeTriangle::new([[3, 7], [4, 6], [2, 6]]),
        EdgeTriangle::new([[0, 4], [0, 1], [0, 2]]),
    ]),
    // 0b01011011: 91
    EdgeTriangles::three([
        EdgeTriangle::new([[1, 5], [4, 5], [6, 7]]),
        EdgeTriangle::new([[1, 5], [6, 7], [3, 7]]),
        EdgeTriangle::new([[2, 6], [0, 2], [2, 3]]),
    ]),
    // 0b01011110: 94
    EdgeTriangles::three([
        EdgeTriangle::new([[6, 7], [3, 7], [1, 5]]),
        EdgeTriangle::new([[6, 7], [1, 5], [4, 5]]),
        EdgeTriangle::new([[0, 1], [0, 2], [0, 4]]),
    ]),
    // 0b01100111: 103
    EdgeTriangles::three([
        EdgeTriangle::new([[2, 3], [1, 3], [5, 7]]),
        EdgeTriangle::new([[2, 3], [5, 7], [6, 7]]),
        EdgeTriangle::new([[4, 5], [0, 4], [4, 6]]),
    ]),
    // 0b01101110: 110
    EdgeTriangles::three([
        EdgeTriangle::new([[0, 1], [0, 2], [4, 6]]),
        EdgeTriangle::new([[0, 1], [4, 6], [4, 5]]),
        EdgeTriangle::new([[6, 7], [3, 7], [5, 7]]),
    ]),
    // 0b01110110: 118
    EdgeTriangles::three([
        EdgeTriangle::new([[5, 7], [6, 7], [2, 3]]),
        EdgeTriangle::new([[5, 7], [2, 3], [1, 3]]),
        EdgeTriangle::new([[0, 2], [0, 4], [0, 1]]),
    ]),
    // 0b01111010: 122
    EdgeTriangles::three([
        EdgeTriangle::new([[0, 4], [0, 1], [2, 3]]),
        EdgeTriangle::new([[0, 4], [2, 3], [2, 6]]),
        EdgeTriangle::new([[3, 7], [5, 7], [6, 7]]),
    ]),
    // 0b01111100: 124
    EdgeTriangles::three([
        EdgeTriangle::new([[0, 2], [0, 4], [1, 5]]),
        EdgeTriangle::new([[0, 2], [1, 5], [1, 3]]),
        EdgeTriangle::new([[5, 7], [6, 7], [3, 7]]),
    ]),
    // 0b10011011: 155
    EdgeTriangles::three([
        EdgeTriangle::new([[0, 2], [2, 3], [6, 7]]),
        EdgeTriangle::new([[0, 2], [6, 7], [4, 6]]),
        EdgeTriangle::new([[5, 7], [1, 5], [4, 5]]),
    ]),
    // 0b10011101: 157
    EdgeTriangles::three([
        EdgeTriangle::new([[1, 3], [0, 1], [4, 5]]),
        EdgeTriangle::new([[1, 3], [4, 5], [5, 7]]),
        EdgeTriangle::new([[4, 6], [2, 6], [6, 7]]),
    ]),
    // 0b10100111: 167
    EdgeTriangles::three([
        EdgeTriangle::new([[4, 5], [0, 4], [2, 6]]),
        EdgeTriangle::new([[4, 5], [2, 6], [6, 7]]),
        EdgeTriangle::new([[2, 3], [1, 3], [3, 7]]),
    ]),
    // 0b10101101: 173
    EdgeTriangles::three([
        EdgeTriangle::new([[2, 6], [6, 7], [4, 5]]),
        EdgeTriangle::new([[2, 6], [4, 5], [0, 4]]),
        EdgeTriangle::new([[1, 5], [1, 3], [0, 1]]),
    ]),
    // 0b10110101: 181
    EdgeTriangles::three([
        EdgeTriangle::new([[0, 1], [1, 5], [3, 7]]),
        EdgeTriangle::new([[0, 1], [3, 7], [2, 3]]),
        EdgeTriangle::new([[6, 7], [4, 6], [2, 6]]),
    ]),
    // 0b10111001: 185
    EdgeTriangles::three([
        EdgeTriangle::new([[6, 7], [4, 6], [0, 2]]),
        EdgeTriangle::new([[6, 7], [0, 2], [2, 3]]),
        EdgeTriangle::new([[0, 1], [1, 5], [1, 3]]),
    ]),
    // 0b10111100: 188
    EdgeTriangles::three([
        EdgeTriangle::new([[1, 5], [1, 3], [0, 2]]),
        EdgeTriangle::new([[1, 5], [0, 2], [0, 4]]),
        EdgeTriangle::new([[2, 6], [6, 7], [4, 6]]),
    ]),
    // 0b11000111: 199
    EdgeTriangles::three([
        EdgeTriangle::new([[0, 4], [4, 6], [5, 7]]),
        EdgeTriangle::new([[0, 4], [5, 7], [1, 5]]),
        EdgeTriangle::new([[3, 7], [2, 3], [1, 3]]),
    ]),
    // 0b11001011: 203
    EdgeTriangles::three([
        EdgeTriangle::new([[5, 7], [1, 5], [0, 4]]),
        EdgeTriangle::new([[5, 7], [0, 4], [4, 6]]),
        EdgeTriangle::new([[0, 2], [2, 3], [2, 6]]),
    ]),
    // 0b11010011: 211
    EdgeTriangles::three([
        EdgeTriangle::new([[2, 6], [0, 2], [1, 3]]),
        EdgeTriangle::new([[2, 6], [1, 3], [3, 7]]),
        EdgeTriangle::new([[1, 5], [4, 5], [5, 7]]),
    ]),
    // 0b11011001: 217
    EdgeTriangles::three([
        EdgeTriangle::new([[4, 5], [5, 7], [1, 3]]),
        EdgeTriangle::new([[4, 5], [1, 3], [0, 1]]),
        EdgeTriangle::new([[2, 3], [2, 6], [0, 2]]),
    ]),
    // 0b11011010: 218
    EdgeTriangles::three([
        EdgeTriangle::new([[2, 3], [2, 6], [0, 4]]),
        EdgeTriangle::new([[2, 3], [0, 4], [0, 1]]),
        EdgeTriangle::new([[4, 5], [5, 7], [1, 5]]),
    ]),
    // 0b11100011: 227
    EdgeTriangles::three([
        EdgeTriangle::new([[1, 3], [3, 7], [2, 6]]),
        EdgeTriangle::new([[1, 3], [2, 6], [0, 2]]),
        EdgeTriangle::new([[4, 6], [4, 5], [0, 4]]),
    ]),
    // 0b11100101: 229
    EdgeTriangles::three([
        EdgeTriangle::new([[3, 7], [2, 3], [0, 1]]),
        EdgeTriangle::new([[3, 7], [0, 1], [1, 5]]),
        EdgeTriangle::new([[0, 4], [4, 6], [4, 5]]),
    ]),
    // 0b11100110: 230
    EdgeTriangles::three([
        EdgeTriangle::new([[4, 6], [4, 5], [0, 1]]),
        EdgeTriangle::new([[4, 6], [0, 1], [0, 2]]),
        EdgeTriangle::new([[1, 3], [3, 7], [2, 3]]),
    ]),
];

const LUT_PATTERN_7A: [EdgeTriangles; 8] = [
    // 0b00010110: 22
    EdgeTriangles::five([
        EdgeTriangle::new([[4, 6], [1, 5], [4, 5]]),
        EdgeTriangle::new([[4, 6], [2, 6], [1, 5]]),
        EdgeTriangle::new([[1, 3], [1, 5], [2, 6]]),
        EdgeTriangle::new([[1, 3], [2, 6], [2, 3]]),
        EdgeTriangle::new([[0, 2], [0, 4], [0, 1]]),
    ]),
    // 0b00101001: 41
    EdgeTriangles::five([
        EdgeTriangle::new([[4, 5], [3, 7], [5, 7]]),
        EdgeTriangle::new([[4, 5], [0, 4], [3, 7]]),
        EdgeTriangle::new([[2, 3], [3, 7], [0, 4]]),
        EdgeTriangle::new([[2, 3], [0, 4], [0, 2]]),
        EdgeTriangle::new([[0, 1], [1, 5], [1, 3]]),
    ]),
    // 0b01001001: 73
    EdgeTriangles::five([
        EdgeTriangle::new([[6, 7], [0, 4], [4, 6]]),
        EdgeTriangle::new([[6, 7], [3, 7], [0, 4]]),
        EdgeTriangle::new([[0, 1], [0, 4], [3, 7]]),
        EdgeTriangle::new([[0, 1], [3, 7], [1, 3]]),
        EdgeTriangle::new([[2, 3], [2, 6], [0, 2]]),
    ]),
    // 0b01100001: 97
    EdgeTriangles::five([
        EdgeTriangle::new([[2, 6], [5, 7], [6, 7]]),
        EdgeTriangle::new([[2, 6], [0, 2], [5, 7]]),
        EdgeTriangle::new([[1, 5], [5, 7], [0, 2]]),
        EdgeTriangle::new([[1, 5], [0, 2], [0, 1]]),
        EdgeTriangle::new([[0, 4], [4, 6], [4, 5]]),
    ]),
    // 0b01101000: 104
    EdgeTriangles::five([
        EdgeTriangle::new([[1, 5], [4, 6], [4, 5]]),
        EdgeTriangle::new([[1, 5], [1, 3], [4, 6]]),
        EdgeTriangle::new([[2, 6], [4, 6], [1, 3]]),
        EdgeTriangle::new([[2, 6], [1, 3], [2, 3]]),
        EdgeTriangle::new([[3, 7], [5, 7], [6, 7]]),
    ]),
    // 0b10000110: 134
    EdgeTriangles::five([
        EdgeTriangle::new([[5, 7], [2, 6], [6, 7]]),
        EdgeTriangle::new([[5, 7], [1, 5], [2, 6]]),
        EdgeTriangle::new([[0, 2], [2, 6], [1, 5]]),
        EdgeTriangle::new([[0, 2], [1, 5], [0, 1]]),
        EdgeTriangle::new([[1, 3], [3, 7], [2, 3]]),
    ]),
    // 0b10010010: 146
    EdgeTriangles::five([
        EdgeTriangle::new([[0, 4], [6, 7], [4, 6]]),
        EdgeTriangle::new([[0, 4], [0, 1], [6, 7]]),
        EdgeTriangle::new([[3, 7], [6, 7], [0, 1]]),
        EdgeTriangle::new([[3, 7], [0, 1], [1, 3]]),
        EdgeTriangle::new([[1, 5], [4, 5], [5, 7]]),
    ]),
    // 0b10010100: 148
    EdgeTriangles::five([
        EdgeTriangle::new([[3, 7], [4, 5], [5, 7]]),
        EdgeTriangle::new([[3, 7], [2, 3], [4, 5]]),
        EdgeTriangle::new([[0, 4], [4, 5], [2, 3]]),
        EdgeTriangle::new([[0, 4], [2, 3], [0, 2]]),
        EdgeTriangle::new([[2, 6], [6, 7], [4, 6]]),
    ]),
];

const LUT_PATTERN_7B: [EdgeTriangles; 8] = [
    // 0b01101011: 107
    EdgeTriangles::three([
        EdgeTriangle::new([[2, 3], [2, 6], [0, 2]]),
        EdgeTriangle::new([[3, 7], [5, 7], [6, 7]]),
        EdgeTriangle::new([[0, 4], [4, 6], [4, 5]]),
    ]),
    // 0b01101101: 109
    EdgeTriangles::three([
        EdgeTriangle::new([[0, 1], [1, 5], [1, 3]]),
        EdgeTriangle::new([[0, 4], [4, 6], [4, 5]]),
        EdgeTriangle::new([[3, 7], [5, 7], [6, 7]]),
    ]),
    // 0b01111001: 121
    EdgeTriangles::three([
        EdgeTriangle::new([[1, 5], [1, 3], [0, 1]]),
        EdgeTriangle::new([[5, 7], [6, 7], [3, 7]]),
        EdgeTriangle::new([[0, 2], [2, 3], [2, 6]]),
    ]),
    // 0b10010111: 151
    EdgeTriangles::three([
        EdgeTriangle::new([[1, 3], [3, 7], [2, 3]]),
        EdgeTriangle::new([[1, 5], [4, 5], [5, 7]]),
        EdgeTriangle::new([[2, 6], [6, 7], [4, 6]]),
    ]),
    // 0b10011110: 158
    EdgeTriangles::three([
        EdgeTriangle::new([[0, 2], [0, 4], [0, 1]]),
        EdgeTriangle::new([[2, 6], [6, 7], [4, 6]]),
        EdgeTriangle::new([[1, 5], [4, 5], [5, 7]]),
    ]),
    // 0b10110110: 182
    EdgeTriangles::three([
        EdgeTriangle::new([[3, 7], [2, 3], [1, 3]]),
        EdgeTriangle::new([[6, 7], [4, 6], [2, 6]]),
        EdgeTriangle::new([[0, 1], [0, 2], [0, 4]]),
    ]),
    // 0b11010110: 214
    EdgeTriangles::three([
        EdgeTriangle::new([[0, 4], [0, 1], [0, 2]]),
        EdgeTriangle::new([[4, 5], [5, 7], [1, 5]]),
        EdgeTriangle::new([[2, 3], [1, 3], [3, 7]]),
    ]),
    // 0b11101001: 233
    EdgeTriangles::three([
        EdgeTriangle::new([[2, 6], [0, 2], [2, 3]]),
        EdgeTriangle::new([[4, 6], [4, 5], [0, 4]]),
        EdgeTriangle::new([[1, 3], [0, 1], [1, 5]]),
    ]),
];

// const LUT_PATTERN_8: [EdgeTriangles; 6] = [TRIANGLES_INVALID; 6];
const LUT_PATTERN_8: [EdgeTriangles; 6] = [
    // 0b00001111: 15
    EdgeTriangles::two([
        EdgeTriangle::new([[0, 4], [2, 6], [1, 5]]),
        EdgeTriangle::new([[1, 5], [2, 6], [3, 7]]),
    ]),
    // 0b00110011: 51
    EdgeTriangles::two([
        EdgeTriangle::new([[4, 6], [0, 2], [5, 7]]),
        EdgeTriangle::new([[5, 7], [0, 2], [1, 3]]),
    ]),
    // 0b01010101: 85
    EdgeTriangles::two([
        EdgeTriangle::new([[6, 7], [2, 3], [4, 5]]),
        EdgeTriangle::new([[4, 5], [2, 3], [0, 1]]),
    ]),
    // 0b10101010: 170
    EdgeTriangles::two([
        EdgeTriangle::new([[4, 5], [0, 1], [6, 7]]),
        EdgeTriangle::new([[6, 7], [0, 1], [2, 3]]),
    ]),
    // 0b11001100: 204
    EdgeTriangles::two([
        EdgeTriangle::new([[5, 7], [1, 3], [4, 6]]),
        EdgeTriangle::new([[4, 6], [1, 3], [0, 2]]),
    ]),
    // 0b11110000: 240
    EdgeTriangles::two([
        EdgeTriangle::new([[2, 6], [0, 4], [3, 7]]),
        EdgeTriangle::new([[3, 7], [0, 4], [1, 5]]),
    ]),
];

// const LUT_PATTERN_9: [EdgeTriangles; 8] = [TRIANGLES_INVALID; 8];
const LUT_PATTERN_9: [EdgeTriangles; 8] = [
    // 0b00010111: 23
    EdgeTriangles::four([
        EdgeTriangle::new([[1, 3], [1, 5], [4, 5]]),
        EdgeTriangle::new([[1, 3], [4, 5], [2, 3]]),
        EdgeTriangle::new([[2, 3], [4, 5], [4, 6]]),
        EdgeTriangle::new([[2, 3], [4, 6], [2, 6]]),
    ]),
    // 0b00101011: 43
    EdgeTriangles::four([
        EdgeTriangle::new([[2, 3], [3, 7], [5, 7]]),
        EdgeTriangle::new([[2, 3], [5, 7], [0, 2]]),
        EdgeTriangle::new([[0, 2], [5, 7], [4, 5]]),
        EdgeTriangle::new([[0, 2], [4, 5], [0, 4]]),
    ]),
    // 0b01001101: 77
    EdgeTriangles::four([
        EdgeTriangle::new([[0, 1], [0, 4], [4, 6]]),
        EdgeTriangle::new([[0, 1], [4, 6], [1, 3]]),
        EdgeTriangle::new([[1, 3], [4, 6], [6, 7]]),
        EdgeTriangle::new([[1, 3], [6, 7], [3, 7]]),
    ]),
    // 0b01110001: 113
    EdgeTriangles::four([
        EdgeTriangle::new([[6, 7], [2, 6], [0, 2]]),
        EdgeTriangle::new([[6, 7], [0, 2], [5, 7]]),
        EdgeTriangle::new([[5, 7], [0, 2], [0, 1]]),
        EdgeTriangle::new([[5, 7], [0, 1], [1, 5]]),
    ]),
    // 0b10001110: 142
    EdgeTriangles::four([
        EdgeTriangle::new([[0, 2], [2, 6], [6, 7]]),
        EdgeTriangle::new([[0, 2], [6, 7], [0, 1]]),
        EdgeTriangle::new([[0, 1], [6, 7], [5, 7]]),
        EdgeTriangle::new([[0, 1], [5, 7], [1, 5]]),
    ]),
    // 0b10110010: 178
    EdgeTriangles::four([
        EdgeTriangle::new([[4, 6], [0, 4], [0, 1]]),
        EdgeTriangle::new([[4, 6], [0, 1], [6, 7]]),
        EdgeTriangle::new([[6, 7], [0, 1], [1, 3]]),
        EdgeTriangle::new([[6, 7], [1, 3], [3, 7]]),
    ]),
    // 0b11010100: 212
    EdgeTriangles::four([
        EdgeTriangle::new([[5, 7], [3, 7], [2, 3]]),
        EdgeTriangle::new([[5, 7], [2, 3], [4, 5]]),
        EdgeTriangle::new([[4, 5], [2, 3], [0, 2]]),
        EdgeTriangle::new([[4, 5], [0, 2], [0, 4]]),
    ]),
    // 0b11101000: 232
    EdgeTriangles::four([
        EdgeTriangle::new([[4, 5], [1, 5], [1, 3]]),
        EdgeTriangle::new([[4, 5], [1, 3], [4, 6]]),
        EdgeTriangle::new([[4, 6], [1, 3], [2, 3]]),
        EdgeTriangle::new([[4, 6], [2, 3], [2, 6]]),
    ]),
];
const LUT_PATTERN_10: [EdgeTriangles; 6] = [
    // 0b00111100: 60
    EdgeTriangles::four([
        EdgeTriangle::new([[5, 7], [4, 6], [2, 6]]),
        EdgeTriangle::new([[5, 7], [2, 6], [3, 7]]),
        EdgeTriangle::new([[1, 5], [0, 2], [0, 4]]),
        EdgeTriangle::new([[1, 5], [1, 3], [0, 2]]),
    ]),
    // 0b01011010: 90
    EdgeTriangles::four([
        EdgeTriangle::new([[4, 5], [6, 7], [3, 7]]),
        EdgeTriangle::new([[4, 5], [3, 7], [1, 5]]),
        EdgeTriangle::new([[0, 4], [2, 3], [2, 6]]),
        EdgeTriangle::new([[0, 4], [0, 1], [2, 3]]),
    ]),
    // 0b01100110: 102
    EdgeTriangles::four([
        EdgeTriangle::new([[6, 7], [2, 3], [1, 3]]),
        EdgeTriangle::new([[6, 7], [1, 3], [5, 7]]),
        EdgeTriangle::new([[4, 6], [0, 1], [0, 2]]),
        EdgeTriangle::new([[4, 6], [4, 5], [0, 1]]),
    ]),
    // 0b10011001: 153
    EdgeTriangles::four([
        EdgeTriangle::new([[0, 1], [4, 5], [5, 7]]),
        EdgeTriangle::new([[0, 1], [5, 7], [1, 3]]),
        EdgeTriangle::new([[0, 2], [6, 7], [4, 6]]),
        EdgeTriangle::new([[0, 2], [2, 3], [6, 7]]),
    ]),
    // 0b10100101: 165
    EdgeTriangles::four([
        EdgeTriangle::new([[6, 7], [4, 5], [0, 4]]),
        EdgeTriangle::new([[6, 7], [0, 4], [2, 6]]),
        EdgeTriangle::new([[3, 7], [0, 1], [1, 5]]),
        EdgeTriangle::new([[3, 7], [2, 3], [0, 1]]),
    ]),
    // 0b11000011: 195
    EdgeTriangles::four([
        EdgeTriangle::new([[4, 6], [5, 7], [1, 5]]),
        EdgeTriangle::new([[4, 6], [1, 5], [0, 4]]),
        EdgeTriangle::new([[2, 6], [1, 3], [3, 7]]),
        EdgeTriangle::new([[2, 6], [0, 2], [1, 3]]),
    ]),
];
const LUT_PATTERN_11: [EdgeTriangles; 12] = [
    // 0b00011011: 27
    EdgeTriangles::four([
        EdgeTriangle::new([[2, 3], [3, 7], [1, 5]]),
        EdgeTriangle::new([[2, 3], [1, 5], [0, 2]]),
        EdgeTriangle::new([[0, 2], [1, 5], [4, 5]]),
        EdgeTriangle::new([[0, 2], [4, 5], [0, 4]]),
    ]),
    // 0b00101110: 46
    EdgeTriangles::four([
        EdgeTriangle::new([[0, 2], [2, 6], [3, 7]]),
        EdgeTriangle::new([[0, 2], [3, 7], [0, 1]]),
        EdgeTriangle::new([[0, 1], [3, 7], [5, 7]]),
        EdgeTriangle::new([[0, 1], [5, 7], [1, 5]]),
    ]),
    // 0b00110101: 53
    EdgeTriangles::four([
        EdgeTriangle::new([[1, 5], [5, 7], [4, 6]]),
        EdgeTriangle::new([[1, 5], [4, 6], [0, 1]]),
        EdgeTriangle::new([[0, 1], [4, 6], [2, 6]]),
        EdgeTriangle::new([[0, 1], [2, 6], [0, 2]]),
    ]),
    // 0b01000111: 71
    EdgeTriangles::four([
        EdgeTriangle::new([[1, 3], [1, 5], [0, 4]]),
        EdgeTriangle::new([[1, 3], [0, 4], [2, 3]]),
        EdgeTriangle::new([[2, 3], [0, 4], [4, 6]]),
        EdgeTriangle::new([[2, 3], [4, 6], [2, 6]]),
    ]),
    // 0b01011100: 92
    EdgeTriangles::four([
        EdgeTriangle::new([[0, 4], [4, 5], [6, 7]]),
        EdgeTriangle::new([[0, 4], [6, 7], [0, 2]]),
        EdgeTriangle::new([[0, 2], [6, 7], [3, 7]]),
        EdgeTriangle::new([[0, 2], [3, 7], [2, 3]]),
    ]),
    // 0b01110010: 114
    EdgeTriangles::four([
        EdgeTriangle::new([[6, 7], [2, 6], [0, 4]]),
        EdgeTriangle::new([[6, 7], [0, 4], [5, 7]]),
        EdgeTriangle::new([[5, 7], [0, 4], [0, 1]]),
        EdgeTriangle::new([[5, 7], [0, 1], [1, 5]]),
    ]),
    // 0b10001101: 141
    EdgeTriangles::four([
        EdgeTriangle::new([[0, 1], [0, 4], [2, 6]]),
        EdgeTriangle::new([[0, 1], [2, 6], [1, 3]]),
        EdgeTriangle::new([[1, 3], [2, 6], [6, 7]]),
        EdgeTriangle::new([[1, 3], [6, 7], [3, 7]]),
    ]),
    // 0b10100011: 163
    EdgeTriangles::four([
        EdgeTriangle::new([[3, 7], [6, 7], [4, 5]]),
        EdgeTriangle::new([[3, 7], [4, 5], [1, 3]]),
        EdgeTriangle::new([[1, 3], [4, 5], [0, 4]]),
        EdgeTriangle::new([[1, 3], [0, 4], [0, 1]]),
    ]),
    // 0b10111000: 184
    EdgeTriangles::four([
        EdgeTriangle::new([[4, 6], [0, 4], [1, 5]]),
        EdgeTriangle::new([[4, 6], [1, 5], [6, 7]]),
        EdgeTriangle::new([[6, 7], [1, 5], [1, 3]]),
        EdgeTriangle::new([[6, 7], [1, 3], [3, 7]]),
    ]),
    // 0b11001010: 202
    EdgeTriangles::four([
        EdgeTriangle::new([[2, 6], [4, 6], [5, 7]]),
        EdgeTriangle::new([[2, 6], [5, 7], [2, 3]]),
        EdgeTriangle::new([[2, 3], [5, 7], [1, 5]]),
        EdgeTriangle::new([[2, 3], [1, 5], [1, 3]]),
    ]),
    // 0b11010001: 209
    EdgeTriangles::four([
        EdgeTriangle::new([[5, 7], [3, 7], [2, 6]]),
        EdgeTriangle::new([[5, 7], [2, 6], [4, 5]]),
        EdgeTriangle::new([[4, 5], [2, 6], [0, 2]]),
        EdgeTriangle::new([[4, 5], [0, 2], [0, 4]]),
    ]),
    // 0b11100100: 228
    EdgeTriangles::four([
        EdgeTriangle::new([[4, 5], [1, 5], [3, 7]]),
        EdgeTriangle::new([[4, 5], [3, 7], [4, 6]]),
        EdgeTriangle::new([[4, 6], [3, 7], [2, 3]]),
        EdgeTriangle::new([[4, 6], [2, 3], [2, 6]]),
    ]),
];

const LUT_PATTERN_12: [EdgeTriangles; 24] = [
    // 0b00011110: 30
    EdgeTriangles::four([
        EdgeTriangle::new([[0, 2], [0, 4], [0, 1]]),
        EdgeTriangle::new([[3, 7], [4, 6], [2, 6]]),
        EdgeTriangle::new([[3, 7], [4, 5], [4, 6]]),
        EdgeTriangle::new([[3, 7], [1, 5], [4, 5]]),
    ]),
    // 0b00101101: 45
    EdgeTriangles::four([
        EdgeTriangle::new([[0, 1], [1, 5], [1, 3]]),
        EdgeTriangle::new([[2, 6], [4, 5], [0, 4]]),
        EdgeTriangle::new([[2, 6], [5, 7], [4, 5]]),
        EdgeTriangle::new([[2, 6], [3, 7], [5, 7]]),
    ]),
    // 0b00110110: 54
    EdgeTriangles::four([
        EdgeTriangle::new([[0, 1], [0, 2], [0, 4]]),
        EdgeTriangle::new([[5, 7], [2, 3], [1, 3]]),
        EdgeTriangle::new([[5, 7], [2, 6], [2, 3]]),
        EdgeTriangle::new([[5, 7], [4, 6], [2, 6]]),
    ]),
    // 0b00111001: 57
    EdgeTriangles::four([
        EdgeTriangle::new([[1, 5], [1, 3], [0, 1]]),
        EdgeTriangle::new([[4, 6], [3, 7], [5, 7]]),
        EdgeTriangle::new([[4, 6], [2, 3], [3, 7]]),
        EdgeTriangle::new([[4, 6], [0, 2], [2, 3]]),
    ]),
    // 0b01001011: 75
    EdgeTriangles::four([
        EdgeTriangle::new([[2, 3], [2, 6], [0, 2]]),
        EdgeTriangle::new([[1, 5], [6, 7], [3, 7]]),
        EdgeTriangle::new([[1, 5], [4, 6], [6, 7]]),
        EdgeTriangle::new([[1, 5], [0, 4], [4, 6]]),
    ]),
    // 0b01010110: 86
    EdgeTriangles::four([
        EdgeTriangle::new([[0, 4], [0, 1], [0, 2]]),
        EdgeTriangle::new([[6, 7], [1, 5], [4, 5]]),
        EdgeTriangle::new([[6, 7], [1, 3], [1, 5]]),
        EdgeTriangle::new([[6, 7], [2, 3], [1, 3]]),
    ]),
    // 0b01011001: 89
    EdgeTriangles::four([
        EdgeTriangle::new([[0, 2], [2, 3], [2, 6]]),
        EdgeTriangle::new([[4, 5], [1, 3], [0, 1]]),
        EdgeTriangle::new([[4, 5], [3, 7], [1, 3]]),
        EdgeTriangle::new([[4, 5], [6, 7], [3, 7]]),
    ]),
    // 0b01100011: 99
    EdgeTriangles::four([
        EdgeTriangle::new([[0, 4], [4, 6], [4, 5]]),
        EdgeTriangle::new([[1, 3], [2, 6], [0, 2]]),
        EdgeTriangle::new([[1, 3], [6, 7], [2, 6]]),
        EdgeTriangle::new([[1, 3], [5, 7], [6, 7]]),
    ]),
    // 0b01100101: 101
    EdgeTriangles::four([
        EdgeTriangle::new([[4, 6], [4, 5], [0, 4]]),
        EdgeTriangle::new([[2, 3], [5, 7], [6, 7]]),
        EdgeTriangle::new([[2, 3], [1, 5], [5, 7]]),
        EdgeTriangle::new([[2, 3], [0, 1], [1, 5]]),
    ]),
    // 0b01101010: 106
    EdgeTriangles::four([
        EdgeTriangle::new([[5, 7], [6, 7], [3, 7]]),
        EdgeTriangle::new([[0, 1], [4, 6], [4, 5]]),
        EdgeTriangle::new([[0, 1], [2, 6], [4, 6]]),
        EdgeTriangle::new([[0, 1], [2, 3], [2, 6]]),
    ]),
    // 0b01101100: 108
    EdgeTriangles::four([
        EdgeTriangle::new([[3, 7], [5, 7], [6, 7]]),
        EdgeTriangle::new([[0, 2], [1, 5], [1, 3]]),
        EdgeTriangle::new([[0, 2], [4, 5], [1, 5]]),
        EdgeTriangle::new([[0, 2], [4, 6], [4, 5]]),
    ]),
    // 0b01111000: 120
    EdgeTriangles::four([
        EdgeTriangle::new([[6, 7], [3, 7], [5, 7]]),
        EdgeTriangle::new([[0, 4], [2, 3], [2, 6]]),
        EdgeTriangle::new([[0, 4], [1, 3], [2, 3]]),
        EdgeTriangle::new([[0, 4], [1, 5], [1, 3]]),
    ]),
    // 0b10000111: 135
    EdgeTriangles::four([
        EdgeTriangle::new([[1, 3], [3, 7], [2, 3]]),
        EdgeTriangle::new([[0, 4], [5, 7], [1, 5]]),
        EdgeTriangle::new([[0, 4], [6, 7], [5, 7]]),
        EdgeTriangle::new([[0, 4], [2, 6], [6, 7]]),
    ]),
    // 0b10010011: 147
    EdgeTriangles::four([
        EdgeTriangle::new([[4, 5], [5, 7], [1, 5]]),
        EdgeTriangle::new([[0, 2], [6, 7], [4, 6]]),
        EdgeTriangle::new([[0, 2], [3, 7], [6, 7]]),
        EdgeTriangle::new([[0, 2], [1, 3], [3, 7]]),
    ]),
    // 0b10010101: 149
    EdgeTriangles::four([
        EdgeTriangle::new([[2, 6], [6, 7], [4, 6]]),
        EdgeTriangle::new([[0, 1], [3, 7], [2, 3]]),
        EdgeTriangle::new([[0, 1], [5, 7], [3, 7]]),
        EdgeTriangle::new([[0, 1], [4, 5], [5, 7]]),
    ]),
    // 0b10011010: 154
    EdgeTriangles::four([
        EdgeTriangle::new([[1, 5], [4, 5], [5, 7]]),
        EdgeTriangle::new([[2, 3], [0, 4], [0, 1]]),
        EdgeTriangle::new([[2, 3], [4, 6], [0, 4]]),
        EdgeTriangle::new([[2, 3], [6, 7], [4, 6]]),
    ]),
    // 0b10011100: 156
    EdgeTriangles::four([
        EdgeTriangle::new([[6, 7], [4, 6], [2, 6]]),
        EdgeTriangle::new([[1, 3], [4, 5], [5, 7]]),
        EdgeTriangle::new([[1, 3], [0, 4], [4, 5]]),
        EdgeTriangle::new([[1, 3], [0, 2], [0, 4]]),
    ]),
    // 0b10100110: 166
    EdgeTriangles::four([
        EdgeTriangle::new([[3, 7], [2, 3], [1, 3]]),
        EdgeTriangle::new([[4, 5], [2, 6], [6, 7]]),
        EdgeTriangle::new([[4, 5], [0, 2], [2, 6]]),
        EdgeTriangle::new([[4, 5], [0, 1], [0, 2]]),
    ]),
    // 0b10101001: 169
    EdgeTriangles::four([
        EdgeTriangle::new([[1, 3], [0, 1], [1, 5]]),
        EdgeTriangle::new([[6, 7], [0, 2], [2, 3]]),
        EdgeTriangle::new([[6, 7], [0, 4], [0, 2]]),
        EdgeTriangle::new([[6, 7], [4, 5], [0, 4]]),
    ]),
    // 0b10110100: 180
    EdgeTriangles::four([
        EdgeTriangle::new([[4, 6], [2, 6], [6, 7]]),
        EdgeTriangle::new([[1, 5], [0, 2], [0, 4]]),
        EdgeTriangle::new([[1, 5], [2, 3], [0, 2]]),
        EdgeTriangle::new([[1, 5], [3, 7], [2, 3]]),
    ]),
    // 0b11000110: 198
    EdgeTriangles::four([
        EdgeTriangle::new([[2, 3], [1, 3], [3, 7]]),
        EdgeTriangle::new([[4, 6], [0, 1], [0, 2]]),
        EdgeTriangle::new([[4, 6], [1, 5], [0, 1]]),
        EdgeTriangle::new([[4, 6], [5, 7], [1, 5]]),
    ]),
    // 0b11001001: 201
    EdgeTriangles::four([
        EdgeTriangle::new([[2, 6], [0, 2], [2, 3]]),
        EdgeTriangle::new([[5, 7], [0, 4], [4, 6]]),
        EdgeTriangle::new([[5, 7], [0, 1], [0, 4]]),
        EdgeTriangle::new([[5, 7], [1, 3], [0, 1]]),
    ]),
    // 0b11010010: 210
    EdgeTriangles::four([
        EdgeTriangle::new([[5, 7], [1, 5], [4, 5]]),
        EdgeTriangle::new([[2, 6], [1, 3], [3, 7]]),
        EdgeTriangle::new([[2, 6], [0, 1], [1, 3]]),
        EdgeTriangle::new([[2, 6], [0, 4], [0, 1]]),
    ]),
    // 0b11100001: 225
    EdgeTriangles::four([
        EdgeTriangle::new([[4, 5], [0, 4], [4, 6]]),
        EdgeTriangle::new([[3, 7], [0, 1], [1, 5]]),
        EdgeTriangle::new([[3, 7], [0, 2], [0, 1]]),
        EdgeTriangle::new([[3, 7], [2, 6], [0, 2]]),
    ]),
];

const LUT_PATTERN_13: [EdgeTriangles; 2] = [
    // 0b01101001: 105
    EdgeTriangles::four([
        EdgeTriangle::new([[0, 1], [1, 5], [1, 3]]),
        EdgeTriangle::new([[0, 2], [2, 3], [2, 6]]),
        EdgeTriangle::new([[0, 4], [4, 6], [4, 5]]),
        EdgeTriangle::new([[3, 7], [5, 7], [6, 7]]),
    ]),
    // 0b10010110: 150
    EdgeTriangles::four([
        EdgeTriangle::new([[1, 3], [3, 7], [2, 3]]),
        EdgeTriangle::new([[0, 1], [0, 2], [0, 4]]),
        EdgeTriangle::new([[1, 5], [4, 5], [5, 7]]),
        EdgeTriangle::new([[2, 6], [6, 7], [4, 6]]),
    ]),
];

const LUT_PATTERN_14: [EdgeTriangles; 12] = [
    // 0b00011101: 29
    EdgeTriangles::four([
        EdgeTriangle::new([[0, 1], [4, 5], [4, 6]]),
        EdgeTriangle::new([[0, 1], [4, 6], [2, 6]]),
        EdgeTriangle::new([[0, 1], [2, 6], [1, 3]]),
        EdgeTriangle::new([[1, 3], [2, 6], [3, 7]]),
    ]),
    // 0b00100111: 39
    EdgeTriangles::four([
        EdgeTriangle::new([[1, 3], [5, 7], [4, 5]]),
        EdgeTriangle::new([[1, 3], [4, 5], [0, 4]]),
        EdgeTriangle::new([[1, 3], [0, 4], [2, 3]]),
        EdgeTriangle::new([[2, 3], [0, 4], [2, 6]]),
    ]),
    // 0b00111010: 58
    EdgeTriangles::four([
        EdgeTriangle::new([[5, 7], [4, 6], [0, 4]]),
        EdgeTriangle::new([[5, 7], [0, 4], [0, 1]]),
        EdgeTriangle::new([[5, 7], [0, 1], [3, 7]]),
        EdgeTriangle::new([[3, 7], [0, 1], [2, 3]]),
    ]),
    // 0b01001110: 78
    EdgeTriangles::four([
        EdgeTriangle::new([[0, 2], [4, 6], [6, 7]]),
        EdgeTriangle::new([[0, 2], [6, 7], [3, 7]]),
        EdgeTriangle::new([[0, 2], [3, 7], [0, 1]]),
        EdgeTriangle::new([[0, 1], [3, 7], [1, 5]]),
    ]),
    // 0b01010011: 83
    EdgeTriangles::four([
        EdgeTriangle::new([[4, 5], [6, 7], [2, 6]]),
        EdgeTriangle::new([[4, 5], [2, 6], [0, 2]]),
        EdgeTriangle::new([[4, 5], [0, 2], [1, 5]]),
        EdgeTriangle::new([[1, 5], [0, 2], [1, 3]]),
    ]),
    // 0b01110100: 116
    EdgeTriangles::four([
        EdgeTriangle::new([[6, 7], [2, 3], [0, 2]]),
        EdgeTriangle::new([[6, 7], [0, 2], [0, 4]]),
        EdgeTriangle::new([[6, 7], [0, 4], [5, 7]]),
        EdgeTriangle::new([[5, 7], [0, 4], [1, 5]]),
    ]),
    // 0b10001011: 139
    EdgeTriangles::four([
        EdgeTriangle::new([[2, 3], [6, 7], [5, 7]]),
        EdgeTriangle::new([[2, 3], [5, 7], [1, 5]]),
        EdgeTriangle::new([[2, 3], [1, 5], [0, 2]]),
        EdgeTriangle::new([[0, 2], [1, 5], [0, 4]]),
    ]),
    // 0b10101100: 172
    EdgeTriangles::four([
        EdgeTriangle::new([[6, 7], [4, 5], [1, 5]]),
        EdgeTriangle::new([[6, 7], [1, 5], [1, 3]]),
        EdgeTriangle::new([[6, 7], [1, 3], [2, 6]]),
        EdgeTriangle::new([[2, 6], [1, 3], [0, 2]]),
    ]),
    // 0b10110001: 177
    EdgeTriangles::four([
        EdgeTriangle::new([[4, 6], [0, 2], [0, 1]]),
        EdgeTriangle::new([[4, 6], [0, 1], [1, 5]]),
        EdgeTriangle::new([[4, 6], [1, 5], [6, 7]]),
        EdgeTriangle::new([[6, 7], [1, 5], [3, 7]]),
    ]),
    // 0b11000101: 197
    EdgeTriangles::four([
        EdgeTriangle::new([[4, 6], [5, 7], [3, 7]]),
        EdgeTriangle::new([[4, 6], [3, 7], [2, 3]]),
        EdgeTriangle::new([[4, 6], [2, 3], [0, 4]]),
        EdgeTriangle::new([[0, 4], [2, 3], [0, 1]]),
    ]),
    // 0b11011000: 216
    EdgeTriangles::four([
        EdgeTriangle::new([[5, 7], [1, 3], [2, 3]]),
        EdgeTriangle::new([[5, 7], [2, 3], [2, 6]]),
        EdgeTriangle::new([[5, 7], [2, 6], [4, 5]]),
        EdgeTriangle::new([[4, 5], [2, 6], [0, 4]]),
    ]),
    // 0b11100010: 226
    EdgeTriangles::four([
        EdgeTriangle::new([[4, 5], [0, 1], [1, 3]]),
        EdgeTriangle::new([[4, 5], [1, 3], [3, 7]]),
        EdgeTriangle::new([[4, 5], [3, 7], [4, 6]]),
        EdgeTriangle::new([[4, 6], [3, 7], [2, 6]]),
    ]),
];

pub const LUT: [EdgeTriangles; 256] = [
    // 0
    TRIANGLES_INVALID,
    LUT_PATTERN_1A[0],
    LUT_PATTERN_1A[1],
    LUT_PATTERN_2A[0],
    LUT_PATTERN_1A[2],
    LUT_PATTERN_2A[1],
    LUT_PATTERN_3A[0],
    LUT_PATTERN_5A[0],
    //
    LUT_PATTERN_1A[3],
    LUT_PATTERN_3A[1],
    LUT_PATTERN_2A[2],
    LUT_PATTERN_5A[1],
    LUT_PATTERN_2A[3],
    LUT_PATTERN_5A[2],
    LUT_PATTERN_5A[3],
    LUT_PATTERN_8[0],
    // 16
    LUT_PATTERN_1A[4],
    LUT_PATTERN_2A[4],
    LUT_PATTERN_3A[2],
    LUT_PATTERN_5A[4],
    LUT_PATTERN_3A[3],
    LUT_PATTERN_5A[5],
    LUT_PATTERN_7A[0],
    LUT_PATTERN_9[0],
    //
    LUT_PATTERN_4A[0],
    LUT_PATTERN_6A[0],
    LUT_PATTERN_6A[1],
    LUT_PATTERN_11[0],
    LUT_PATTERN_6A[2],
    LUT_PATTERN_14[0],
    LUT_PATTERN_12[0],
    LUT_PATTERN_5B[23],
    // 32
    LUT_PATTERN_1A[5],
    LUT_PATTERN_3A[4],
    LUT_PATTERN_2A[5],
    LUT_PATTERN_5A[6],
    LUT_PATTERN_4A[1],
    LUT_PATTERN_6A[3],
    LUT_PATTERN_6A[4],
    LUT_PATTERN_14[1],
    //
    LUT_PATTERN_3A[5],
    LUT_PATTERN_7A[1],
    LUT_PATTERN_5A[7],
    LUT_PATTERN_9[1],
    LUT_PATTERN_6A[5],
    LUT_PATTERN_12[1],
    LUT_PATTERN_11[1],
    LUT_PATTERN_5B[22],
    // 48
    LUT_PATTERN_2A[6],
    LUT_PATTERN_5A[8],
    LUT_PATTERN_5A[9],
    LUT_PATTERN_8[1],
    LUT_PATTERN_6A[6],
    LUT_PATTERN_11[2],
    LUT_PATTERN_12[2],
    LUT_PATTERN_5B[21],
    //
    LUT_PATTERN_6A[7],
    LUT_PATTERN_12[3],
    LUT_PATTERN_14[2],
    LUT_PATTERN_5B[20],
    LUT_PATTERN_10[0],
    LUT_PATTERN_6B[0],
    LUT_PATTERN_6B[1],
    LUT_PATTERN_2B[11],
    // 64
    LUT_PATTERN_1A[6],
    LUT_PATTERN_3A[6],
    LUT_PATTERN_4A[2],
    LUT_PATTERN_6A[8],
    LUT_PATTERN_2A[7],
    LUT_PATTERN_5A[10],
    LUT_PATTERN_6A[9],
    LUT_PATTERN_11[3],
    //
    LUT_PATTERN_3A[7],
    LUT_PATTERN_7A[2],
    LUT_PATTERN_6A[10],
    LUT_PATTERN_12[4],
    LUT_PATTERN_5A[11],
    LUT_PATTERN_9[2],
    LUT_PATTERN_14[3],
    LUT_PATTERN_5B[19],
    // 80
    LUT_PATTERN_2A[8],
    LUT_PATTERN_5A[12],
    LUT_PATTERN_6A[11],
    LUT_PATTERN_14[4],
    LUT_PATTERN_5A[13],
    LUT_PATTERN_8[2],
    LUT_PATTERN_12[5],
    LUT_PATTERN_5B[18],
    //
    LUT_PATTERN_6A[12],
    LUT_PATTERN_12[6],
    LUT_PATTERN_10[1],
    LUT_PATTERN_6B[2],
    LUT_PATTERN_11[4],
    LUT_PATTERN_5B[17],
    LUT_PATTERN_6B[3],
    LUT_PATTERN_2B[10],
    // 96
    LUT_PATTERN_3A[8],
    LUT_PATTERN_7A[3],
    LUT_PATTERN_6A[13],
    LUT_PATTERN_12[7],
    LUT_PATTERN_6A[14],
    LUT_PATTERN_12[8],
    LUT_PATTERN_10[2],
    LUT_PATTERN_6B[4],
    //
    LUT_PATTERN_7A[4],
    LUT_PATTERN_13[0],
    LUT_PATTERN_12[9],
    LUT_PATTERN_7B[0],
    LUT_PATTERN_12[10],
    LUT_PATTERN_7B[1],
    LUT_PATTERN_6B[5],
    LUT_PATTERN_3B[11],
    // 112
    LUT_PATTERN_5A[14],
    LUT_PATTERN_9[3],
    LUT_PATTERN_11[5],
    LUT_PATTERN_5B[16],
    LUT_PATTERN_14[5],
    LUT_PATTERN_5B[15],
    LUT_PATTERN_6B[6],
    LUT_PATTERN_2B[9],
    //
    LUT_PATTERN_12[11],
    LUT_PATTERN_7B[2],
    LUT_PATTERN_6B[7],
    LUT_PATTERN_3B[10],
    LUT_PATTERN_6B[8],
    LUT_PATTERN_3B[9],
    LUT_PATTERN_4B[3],
    LUT_PATTERN_1B[7],
    // 128
    LUT_PATTERN_1A[7],
    LUT_PATTERN_4A[3],
    LUT_PATTERN_3A[9],
    LUT_PATTERN_6A[15],
    LUT_PATTERN_3A[10],
    LUT_PATTERN_6A[16],
    LUT_PATTERN_7A[5],
    LUT_PATTERN_12[12],
    //
    LUT_PATTERN_2A[9],
    LUT_PATTERN_6A[17],
    LUT_PATTERN_5A[15],
    LUT_PATTERN_14[6],
    LUT_PATTERN_5A[16],
    LUT_PATTERN_11[6],
    LUT_PATTERN_9[4],
    LUT_PATTERN_5B[14],
    // 144
    LUT_PATTERN_3A[11],
    LUT_PATTERN_6A[18],
    LUT_PATTERN_7A[6],
    LUT_PATTERN_12[13],
    LUT_PATTERN_7A[7],
    LUT_PATTERN_12[14],
    LUT_PATTERN_13[1],
    LUT_PATTERN_7B[3],
    //
    LUT_PATTERN_6A[19],
    LUT_PATTERN_10[3],
    LUT_PATTERN_12[15],
    LUT_PATTERN_6B[9],
    LUT_PATTERN_12[16],
    LUT_PATTERN_6B[10],
    LUT_PATTERN_7B[4],
    LUT_PATTERN_3B[8],
    // 160
    LUT_PATTERN_2A[10],
    LUT_PATTERN_6A[20],
    LUT_PATTERN_5A[17],
    LUT_PATTERN_11[7],
    LUT_PATTERN_6A[21],
    LUT_PATTERN_10[4],
    LUT_PATTERN_12[17],
    LUT_PATTERN_6B[11],
    //
    LUT_PATTERN_5A[18],
    LUT_PATTERN_12[18],
    LUT_PATTERN_8[3],
    LUT_PATTERN_5B[13],
    LUT_PATTERN_14[7],
    LUT_PATTERN_6B[12],
    LUT_PATTERN_5B[12],
    LUT_PATTERN_2B[8],
    // 176
    LUT_PATTERN_5A[19],
    LUT_PATTERN_14[8],
    LUT_PATTERN_9[5],
    LUT_PATTERN_5B[11],
    LUT_PATTERN_12[19],
    LUT_PATTERN_6B[13],
    LUT_PATTERN_7B[5],
    LUT_PATTERN_3B[7],
    //
    LUT_PATTERN_11[8],
    LUT_PATTERN_6B[14],
    LUT_PATTERN_5B[10],
    LUT_PATTERN_2B[7],
    LUT_PATTERN_6B[15],
    LUT_PATTERN_4B[2],
    LUT_PATTERN_3B[6],
    LUT_PATTERN_1B[6],
    // 192
    LUT_PATTERN_2A[11],
    LUT_PATTERN_6A[22],
    LUT_PATTERN_6A[23],
    LUT_PATTERN_10[5],
    LUT_PATTERN_5A[20],
    LUT_PATTERN_14[9],
    LUT_PATTERN_12[20],
    LUT_PATTERN_6B[16],
    //
    LUT_PATTERN_5A[21],
    LUT_PATTERN_12[21],
    LUT_PATTERN_11[9],
    LUT_PATTERN_6B[17],
    LUT_PATTERN_8[4],
    LUT_PATTERN_5B[9],
    LUT_PATTERN_5B[8],
    LUT_PATTERN_2B[6],
    // 208
    LUT_PATTERN_5A[22],
    LUT_PATTERN_11[10],
    LUT_PATTERN_12[22],
    LUT_PATTERN_6B[18],
    LUT_PATTERN_9[6],
    LUT_PATTERN_5B[7],
    LUT_PATTERN_7B[6],
    LUT_PATTERN_3B[5],
    //
    LUT_PATTERN_14[10],
    LUT_PATTERN_6B[19],
    LUT_PATTERN_6B[20],
    LUT_PATTERN_4B[1],
    LUT_PATTERN_5B[6],
    LUT_PATTERN_2B[5],
    LUT_PATTERN_3B[4],
    LUT_PATTERN_1B[5],
    // 224
    LUT_PATTERN_5A[23],
    LUT_PATTERN_12[23],
    LUT_PATTERN_14[11],
    LUT_PATTERN_6B[21],
    LUT_PATTERN_11[11],
    LUT_PATTERN_6B[22],
    LUT_PATTERN_6B[23],
    LUT_PATTERN_4B[0],
    //
    LUT_PATTERN_9[7],
    LUT_PATTERN_7B[7],
    LUT_PATTERN_5B[5],
    LUT_PATTERN_3B[3],
    LUT_PATTERN_5B[4],
    LUT_PATTERN_3B[2],
    LUT_PATTERN_2B[4],
    LUT_PATTERN_1B[4],
    // 240
    LUT_PATTERN_8[5],
    LUT_PATTERN_5B[3],
    LUT_PATTERN_5B[2],
    LUT_PATTERN_2B[3],
    LUT_PATTERN_5B[1],
    LUT_PATTERN_2B[2],
    LUT_PATTERN_3B[1],
    LUT_PATTERN_1B[3],
    //
    LUT_PATTERN_5B[0],
    LUT_PATTERN_3B[0],
    LUT_PATTERN_2B[1],
    LUT_PATTERN_1B[2],
    LUT_PATTERN_2B[0],
    LUT_PATTERN_1B[1],
    LUT_PATTERN_1B[0],
    TRIANGLES_INVALID,
];
