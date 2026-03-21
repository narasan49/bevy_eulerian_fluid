#import bevy_fluid::coordinate::{interp2d_center, interp2d_center_rg32float}
#import bevy_fluid::particle_levelset::particle::{Particle, is_particle_escaped};
#import bevy_fluid::particle_levelset::constants::{MAX_PARTICLES_PER_CELL, BAND_WIDTH};

@group(0) @binding(0) var<storage, read> sorted_particles: array<Particle>;
@group(0) @binding(1) var<storage, read> num_particles_in_cell: array<u32>;
@group(0) @binding(2) var<storage, read> cell_offsets: array<u32>;
@group(0) @binding(3) var<storage, read_write> alive_particles_mask: array<u32>;
@group(0) @binding(4) var<storage, read_write> particles_to_be_added: array<u32>;
@group(0) @binding(5) var levelset_air: texture_storage_2d<r32float, read>;
@group(0) @binding(6) var<uniform> grid_size: vec2<u32>;

@compute @workgroup_size(8, 8, 1)
fn reseed_particles(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let cell_id = global_invocation_id.xy;
    let cell_id_1d = cell_id.x + grid_size.x * cell_id.y;
    let n = num_particles_in_cell[cell_id_1d];
    let cell_offset = cell_offsets[cell_id_1d];

    let cell_center = vec2<f32>(cell_id) + vec2<f32>(0.5);
    let is_near_interface = abs(interp2d_center(levelset_air, cell_center)) < BAND_WIDTH;
    var heap: array<Node, MAX_PARTICLES_PER_CELL>;
    var heap_capacity = MAX_PARTICLES_PER_CELL;
    var heap_idx = 0u;
    for (var i = 0u; i < n; i++) {
        let p_idx = cell_offset + i;
        var p = sorted_particles[p_idx];

        if !is_near_interface {
            // mark as delete
            alive_particles_mask[p_idx] = 0u;
            continue;
        }

        if is_particle_escaped(p) {
            alive_particles_mask[p_idx] = 1u;

            if heap_capacity != 0u {
                heap_capacity -= 1u;
                if heap_idx > heap_capacity {
                    // delete the particle on top of the heap.
                    alive_particles_mask[heap[0].index] = 0u;
                    // swap the first element with the last one and reconstruct heap.
                    heap[0] = heap[heap_idx - 1];
                    heap_idx -=1u;
                    construct_heap(&heap, heap_capacity);
                }
            }
            continue;
        }

        if heap_idx < heap_capacity {
            // insert a particle to heap.
            alive_particles_mask[p_idx] = 1u;
            heap[heap_idx] = Node(p_idx, eval_priority(p, levelset_air));
            heap_idx += 1u;
            sift_up(&heap, heap_idx);
        } else {
            let incoming = eval_priority(p, levelset_air);
            if incoming < heap[0].value {
                alive_particles_mask[p_idx] = 1u;
                // delete the particle on top of the heap.
                alive_particles_mask[heap[0].index] = 0u;
                // swap with the first element and reconstruct heap.
                heap[0] = Node(p_idx, incoming);
                construct_heap(&heap, heap_capacity);
            } else {
                alive_particles_mask[p_idx] = 0u;
            }
        }
    }

    if !is_near_interface {
        particles_to_be_added[cell_id_1d] = 0u;
    } else {
        particles_to_be_added[cell_id_1d] = heap_capacity - heap_idx;
    }
}

fn eval_priority(p: Particle, levelset_air: texture_storage_2d<r32float, read>) -> f32 {
    return p.sign * interp2d_center(levelset_air, p.position) - p.radius;
}

struct Node {
    index: u32,
    value: f32,
}

fn left(i: u32) -> u32 {
    return 2 * i + 1;
}

fn right(i: u32) -> u32 {
    return 2 * i + 2;
}

fn correct_heap(heap: ptr<function, array<Node, MAX_PARTICLES_PER_CELL>>, capacity: u32, start: u32) {
    var i = start;
    loop {
        let l = left(i);
        let r = right(i);
        var largest = i;
        if ((l < capacity) && (*heap)[l].value > (*heap)[largest].value) {
            largest = l;
        }

        if ((r < capacity) && (*heap)[r].value > (*heap)[largest].value) {
            largest = r;
        }

        if largest == i {
            return;
        }
        
        let tmp = (*heap)[largest];
        (*heap)[largest] = (*heap)[i];
        (*heap)[i] = tmp;

        i = largest;
    }
}

fn construct_heap(heap: ptr<function, array<Node, MAX_PARTICLES_PER_CELL>>, capacity: u32) {
    correct_heap(heap, capacity, 0);
}

fn sift_up(heap: ptr<function, array<Node, MAX_PARTICLES_PER_CELL>>, capacity: u32) {
    var i = capacity - 1;
    loop {
        let p = (i - 1) / 2;
        var largest = i;
        if ((*heap)[p].value > (*heap)[i].value) {
            return;
        }
        
        let tmp = (*heap)[p];
        (*heap)[p] = (*heap)[i];
        (*heap)[i] = tmp;

        i = p;

        if i == 0 {
            return;
        }
    }
}