#define_import_path bevy_fluid::coordinate

fn left(x: vec2<i32>) -> vec2<i32> {
    return x - vec2<i32>(1, 0);
}

fn right(x: vec2<i32>) -> vec2<i32> {
    return x + vec2<i32>(1, 0);
}

fn bottom(x: vec2<i32>) -> vec2<i32> {
    return x - vec2<i32>(0, 1);
}

fn top(x: vec2<i32>) -> vec2<i32> {
    return x + vec2<i32>(0, 1);
}