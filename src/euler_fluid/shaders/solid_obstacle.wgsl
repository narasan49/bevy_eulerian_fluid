#define_import_path bevy_fluid::solid_obstacle

const SHAPE_CIRCLE: u32 = 0;
const SHAPE_RECTANGLE: u32 = 1;
const SHAPE_TRIANGLE: u32 = 4;

struct Circle {
    radius: f32,
}

struct Rectangle {
    half_size: vec2<f32>,
}

struct Triangle {
    a: vec2<f32>,
    b: vec2<f32>,
    c: vec2<f32>,
}

struct ShapeVariant {
    shape: u32,
    values: array<f32, 6>,
}

struct SolidObstacle {
    entity_id: u32,
    shape: ShapeVariant,
    transform: mat4x4<f32>,
    inverse_transform: mat4x4<f32>,
    linear_velocity: vec2<f32>,
    angular_velocity: f32,
}

fn get_circle(variant: ShapeVariant) -> Circle {
    return Circle(variant.values[0]);
}

fn get_rectangle(variant: ShapeVariant) -> Rectangle {
    return Rectangle(vec2<f32>(variant.values[0], variant.values[1]));
}

fn get_triangle(variant: ShapeVariant) -> Triangle {
    return Triangle(
        vec2<f32>(variant.values[0], variant.values[1]),
        vec2<f32>(variant.values[2], variant.values[3]),
        vec2<f32>(variant.values[4], variant.values[5]),
    );
}

fn center_of_mass(solid_obstacle: SolidObstacle) -> vec2<f32> {
    switch (solid_obstacle.shape.shape) {
        case SHAPE_CIRCLE, SHAPE_RECTANGLE:
        {
            return solid_obstacle.transform[3].xy;
        }
        case SHAPE_TRIANGLE:
        {
            let triangle = get_triangle(solid_obstacle.shape);
            let triangle_center_local = vec4<f32>(
                (triangle.a.x + triangle.b.x + triangle.c.x) / 3.0,
                (triangle.a.y + triangle.b.y + triangle.c.y) / 3.0,
                0.0,
                1.0,
            );
            return (solid_obstacle.transform * triangle_center_local).xy;
        }
        default:
        {
            return vec2<f32>(0.0);
        }
    }
}