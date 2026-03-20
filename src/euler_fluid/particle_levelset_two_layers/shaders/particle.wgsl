#define_import_path bevy_fluid::particle_levelset::particle

#import bevy_fluid::coordinate::{interp2d_center, interp2d_center_rg32float}

struct Particle {
    position: vec2<f32>,
    radius: f32,
    sign: f32,
    escaped: u32,
}

const INVALID_PARTICLE = Particle(vec2<f32>(1e30), 1e30, 0.0, 0);
const MAX_ITER: u32 = 15;

fn is_particle_valid(p: Particle) -> bool {
    return p.sign != INVALID_PARTICLE.sign;
}

fn is_particle_escaped_u32(
    particle: Particle,
    levelset_air: texture_storage_2d<r32float, read>
) -> u32 {
    let phi = interp2d_center(levelset_air, particle.position);
    if particle.sign * phi + particle.radius < 0.0 {
        return 1;
    } else {
        return 0;
    }
}

fn is_particle_escaped(particle: Particle) -> bool {
    return particle.escaped == 1;
}

fn attract(
    x: vec2<f32>,
    lambda: f32,
    phi_goal: f32,
    levelset_air: texture_storage_2d<r32float, read>,
    grad_levelset_air: texture_storage_2d<rg32float, read>,
) -> vec2<f32> {
    let phi = interp2d_center(levelset_air, x);
    let grad_phi = interp2d_center_rg32float(grad_levelset_air, x);
    let grad_phi_norm = normalize(grad_phi);

    return x + lambda * (phi_goal - phi) * grad_phi_norm;
}

fn particle_radius(phi: f32) -> f32 {
    let rmax = 0.5;
    let rmin = 0.1;
    return clamp(abs(phi), rmin, rmax);
}

fn spawn_and_attract_particle(
    x: vec2<f32>,
    phi_goal: f32,
    bmin: f32,
    bmax: f32,
    fdim: vec2<f32>,
    levelset_air: texture_storage_2d<r32float, read>,
    grad_levelset_air: texture_storage_2d<rg32float, read>,
    sign: f32,
) -> Particle {
    var lambda = 1.0;
    var x_new = x;
    var phi_new = 0.0;

    x_new = attract(x_new, lambda, phi_goal, levelset_air, grad_levelset_air);
    var acceptable = false;
    if all(vec2<f32>(0.0) < x_new) || all(x_new < fdim - vec2<f32>(1.0)) {
        phi_new = interp2d_center(levelset_air, x_new);
        if bmin <= phi_new && phi_new <= bmax {
            acceptable = true;
        }
    }
    // for (var j = 0u; j < MAX_ITER; j++) {
    //     while (any(x_new < vec2<f32>(0.0)) || any(fdim - vec2<f32>(1.0) < x_new)) {
    //         lambda *= 0.5;
    //         x_new = attract(x_new, lambda, phi_goal, levelset_air, grad_levelset_air);
    //     }

    //     phi_new = interp2d_center(levelset_air, x_new);
    //     if bmin <= phi_new && phi_new <= bmax {
    //         acceptable = true;
    //         break;
    //     } else {
    //         lambda = 1.0;
    //     }
    // }

    if acceptable {
        return Particle(x_new, particle_radius(phi_new), sign, 0);
    } else {
        return INVALID_PARTICLE;
    }
}