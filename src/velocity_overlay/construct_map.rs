use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    prelude::*,
    render::{
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{
            binding_types::uniform_buffer, AsBindGroup, BindGroup, BindGroupEntries,
            BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId, CachedPipelineState,
            ComputePassDescriptor, ComputePipelineDescriptor, PipelineCache, ShaderStages,
        },
        renderer::RenderDevice,
        storage::{GpuShaderStorageBuffer, ShaderStorageBuffer},
        texture::{FallbackImage, GpuImage},
        Render, RenderApp, RenderSystems,
    },
};

use crate::{
    fluid_uniform::{create_uniform_bind_group_layout, SimulationUniformBindGroup},
    pipeline::DispatchFluidPass,
    render_node::FluidLabel,
    settings::FluidSettings,
    velocity_overlay::VelocityOverlay,
};

pub(crate) struct ConstructVelocityArrowsPlugin;

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct ConstructVelocityArrowsResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub u0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadOnly)]
    pub v0: Handle<Image>,
    #[storage(2, visibility(compute))]
    pub arrows: Handle<ShaderStorageBuffer>,
}

#[derive(Resource)]
struct Pipeline {
    pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayout,
    bin_size_bind_group_layout: BindGroupLayout,
}

#[derive(Component)]
struct BindGroups(BindGroup);

#[derive(Component)]
struct BinSizeBindGroup {
    bind_group: BindGroup,
    index: u32,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct ConstructVelocityMapLabel;

#[derive(Debug)]
enum State {
    Loading,
    Update,
}

struct ConstructVelocityMapNode {
    state: State,
    query: QueryState<(
        &'static BindGroups,
        &'static FluidSettings,
        &'static VelocityOverlay,
        &'static BinSizeBindGroup,
        &'static SimulationUniformBindGroup,
    )>,
}

impl Plugin for ConstructVelocityArrowsPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/construct_velocity_map.wgsl");
        app.add_plugins((
            ExtractComponentPlugin::<ConstructVelocityArrowsResource>::default(),
            ExtractComponentPlugin::<VelocityOverlay>::default(),
            UniformComponentPlugin::<VelocityOverlay>::default(),
        ));

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            prepare_bind_group.in_set(RenderSystems::PrepareBindGroups),
        );

        let mut world = render_app.world_mut();
        let node = ConstructVelocityMapNode::new(&mut world);
        let mut render_graph = world.resource_mut::<RenderGraph>();
        render_graph.add_node(ConstructVelocityMapLabel, node);
        render_graph.add_node_edge(ConstructVelocityMapLabel, FluidLabel);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<Pipeline>();
    }
}

impl FromWorld for Pipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let bind_group_layout = ConstructVelocityArrowsResource::bind_group_layout(render_device);
        let bin_size_bind_group_layout = render_device.create_bind_group_layout(
            Some("VelocityMapBinSizeBindGroupLayout"),
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                uniform_buffer::<VelocityOverlay>(true),
            ),
        );
        let simulation_uniform_bind_group_layout = create_uniform_bind_group_layout(render_device);

        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("ConstructVelocityArrowsPipeline".into()),
            layout: vec![
                bind_group_layout.clone(),
                bin_size_bind_group_layout.clone(),
                simulation_uniform_bind_group_layout.clone(),
            ],
            shader: load_embedded_asset!(world, "shaders/construct_velocity_map.wgsl"),
            entry_point: Some("construct_velocity_arrows".into()),
            ..default()
        });

        Pipeline {
            pipeline,
            bind_group_layout,
            bin_size_bind_group_layout,
        }
    }
}

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<Pipeline>,
    query: Query<(
        Entity,
        &ConstructVelocityArrowsResource,
        &DynamicUniformIndex<VelocityOverlay>,
    )>,
    bin_size_uniform: Res<ComponentUniforms<VelocityOverlay>>,
    render_device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    fallback_image: Res<FallbackImage>,
    buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
) {
    let mut param = (gpu_images, fallback_image, buffers);
    for (entity, resource, bin_size_uniform_index) in &query {
        let bind_group = resource
            .as_bind_group(&pipeline.bind_group_layout, &render_device, &mut param)
            .unwrap()
            .bind_group;

        let bin_size_bind_group = render_device.create_bind_group(
            "VelocityMapBinSizeBindGroup",
            &pipeline.bin_size_bind_group_layout,
            &BindGroupEntries::single(bin_size_uniform.uniforms()),
        );

        commands.entity(entity).insert((
            BindGroups(bind_group),
            BinSizeBindGroup {
                bind_group: bin_size_bind_group.clone(),
                index: bin_size_uniform_index.index(),
            },
        ));
    }
}

impl ConstructVelocityMapNode {
    fn new(world: &mut World) -> Self {
        Self {
            state: State::Loading,
            query: world.query_filtered(),
        }
    }
}

impl render_graph::Node for ConstructVelocityMapNode {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
        let pipeline_cache = world.resource::<PipelineCache>();
        match self.state {
            State::Loading => {
                let pipeline = world.resource::<Pipeline>();
                match pipeline_cache.get_compute_pipeline_state(pipeline.pipeline) {
                    CachedPipelineState::Ok(_) => {
                        info!("ConstructVelocityMap is ready");
                        self.state = State::Update;
                    }
                    _ => {}
                }
            }
            State::Update => {}
        }
    }

    fn run<'w>(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        world: &'w World,
    ) -> std::result::Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let velocity_map_pipeline = world.resource::<Pipeline>();
        match self.state {
            State::Loading => {}
            State::Update => {
                // info!("running construct velocity map");
                let pipeline = pipeline_cache
                    .get_compute_pipeline(velocity_map_pipeline.pipeline)
                    .unwrap();
                for (
                    bind_groups,
                    fluid_settings,
                    overlay_settings,
                    bin_size_bind_group,
                    simulation_uniform_bind_group,
                ) in self.query.iter_manual(world)
                {
                    let mut pass = render_context.command_encoder().begin_compute_pass(
                        &ComputePassDescriptor {
                            label: Some("Construct velocity map"),
                            ..default()
                        },
                    );

                    pass.set_pipeline(pipeline);
                    pass.set_bind_group(0, &bind_groups.0, &[]);
                    pass.set_bind_group(
                        1,
                        &bin_size_bind_group.bind_group,
                        &[bin_size_bind_group.index],
                    );
                    pass.set_bind_group(
                        2,
                        &simulation_uniform_bind_group.bind_group,
                        &[simulation_uniform_bind_group.index],
                    );
                    pass.dispatch_center(fluid_settings.size / overlay_settings.bin_size);
                }
            }
        }
        Ok(())
    }
}
