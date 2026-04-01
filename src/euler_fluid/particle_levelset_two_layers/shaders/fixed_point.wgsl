#define_import_path bevy_fluid::particle_levelset::fixed_point

const SCALE = 10000.0;
fn i32_to_f32(value: i32) -> f32 {
    return f32(value) / SCALE;
}

fn f32_to_i32(value: f32) -> i32 {
    return i32(value * SCALE);
}