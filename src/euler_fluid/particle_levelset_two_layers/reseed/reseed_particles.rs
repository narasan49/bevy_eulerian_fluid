use crate::{
    particle_levelset_two_layers::plugin::PLSResources,
    pipeline::{HasBindGroupLayout, SingleComputePipeline},
    plugin::FluidComputePass,
};
use bevy::{
    asset::{embedded_asset, embedded_path},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{AsBindGroup, BindGroup},
        storage::ShaderStorageBuffer,
    },
};

pub(crate) struct ReseedPositiveParticlesPass;

impl FluidComputePass for ReseedPositiveParticlesPass {
    type Pipeline = ReseedParticlesPipeline;
    type Resource = ReseedPositiveParticlesResource;
    type BG = ReseedPositiveParticlesBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/reseed_particles.wgsl");
    }
}

pub(crate) struct ReseedNegativeParticlesPass;

impl FluidComputePass for ReseedNegativeParticlesPass {
    type Pipeline = ReseedParticlesPipeline;
    type Resource = ReseedNegativeParticlesResource;
    type BG = ReseedNegativeParticlesBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/reseed_particles.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct ReseedPositiveParticlesResource {
    #[storage(0, read_only, visibility(compute))]
    pub sorted_particles: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub num_perticles_in_cell: Handle<ShaderStorageBuffer>,
    #[storage(2, read_only, visibility(compute))]
    pub cell_offsets: Handle<ShaderStorageBuffer>,
    #[storage(3, visibility(compute))]
    pub alive_particles_mask: Handle<ShaderStorageBuffer>,
    #[storage(4, visibility(compute))]
    pub particles_to_be_added: Handle<ShaderStorageBuffer>,
    #[storage_texture(5, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[uniform(6)]
    pub grid_size: UVec2,
}

impl ReseedPositiveParticlesResource {
    pub fn new(
        pls_rersources: &PLSResources,
        levelset_air: &Handle<Image>,
        grid_size: UVec2,
    ) -> Self {
        let sorted_particles = pls_rersources.sorted_positive_particles.clone();
        let alive_particles_mask = pls_rersources.positive_alive_particles_mask.clone();
        let num_perticles_in_cell = pls_rersources.num_positive_particles_in_cell.clone();
        let particles_to_be_added = pls_rersources.positive_particles_to_be_added.clone();
        let cell_offsets = pls_rersources.positive_cell_offsets.clone();

        Self {
            sorted_particles,
            alive_particles_mask,
            num_perticles_in_cell,
            particles_to_be_added,
            cell_offsets,
            levelset_air: levelset_air.clone(),
            grid_size,
        }
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct ReseedNegativeParticlesResource {
    #[storage(0, read_only, visibility(compute))]
    pub sorted_particles: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only, visibility(compute))]
    pub num_perticles_in_cell: Handle<ShaderStorageBuffer>,
    #[storage(2, read_only, visibility(compute))]
    pub cell_offsets: Handle<ShaderStorageBuffer>,
    #[storage(3, visibility(compute))]
    pub alive_particles_mask: Handle<ShaderStorageBuffer>,
    #[storage(4, visibility(compute))]
    pub particles_to_be_added: Handle<ShaderStorageBuffer>,
    #[storage_texture(5, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
    #[uniform(6)]
    pub grid_size: UVec2,
}

impl ReseedNegativeParticlesResource {
    pub fn new(
        pls_resources: &PLSResources,
        levelset_air: &Handle<Image>,
        grid_size: UVec2,
    ) -> Self {
        let sorted_particles = pls_resources.sorted_negative_particles.clone();
        let alive_particles_mask = pls_resources.negative_alive_particles_mask.clone();
        let num_perticles_in_cell = pls_resources.num_negative_particles_in_cell.clone();
        let particles_to_be_added = pls_resources.negative_particles_to_be_added.clone();
        let cell_offsets = pls_resources.negative_cell_offsets.clone();

        Self {
            sorted_particles,
            alive_particles_mask,
            num_perticles_in_cell,
            particles_to_be_added,
            cell_offsets,
            levelset_air: levelset_air.clone(),
            grid_size,
        }
    }
}

#[derive(Resource)]
pub(crate) struct ReseedParticlesPipeline {
    pub pipeline: SingleComputePipeline,
}

impl FromWorld for ReseedParticlesPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline = SingleComputePipeline::new_with_uniform::<ReseedPositiveParticlesResource>(
            world,
            "ReseedParticlesPipeline",
            embedded_path!("shaders/reseed_particles.wgsl"),
            "reseed_particles",
        );

        Self { pipeline }
    }
}

#[derive(Component)]
pub(crate) struct ReseedPositiveParticlesBindGroup {
    pub bind_group: BindGroup,
}

#[derive(Component)]
pub(crate) struct ReseedNegativeParticlesBindGroup {
    pub bind_group: BindGroup,
}

impl HasBindGroupLayout for ReseedParticlesPipeline {
    fn bind_group_layout(&self) -> &bevy::render::render_resource::BindGroupLayoutDescriptor {
        &self.pipeline.bind_group_layout
    }
}

impl From<BindGroup> for ReseedPositiveParticlesBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}

impl From<BindGroup> for ReseedNegativeParticlesBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
