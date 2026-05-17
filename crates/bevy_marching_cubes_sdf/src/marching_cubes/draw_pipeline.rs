use bevy::{
    asset::load_embedded_asset,
    core_pipeline::core_3d::CORE_3D_DEPTH_FORMAT,
    mesh::{VertexBufferLayout, VertexFormat},
    prelude::*,
    render::{
        extract_component::{ComponentUniforms, DynamicUniformIndex, ExtractComponent},
        render_asset::RenderAssets,
        render_resource::{
            BindGroup, BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries, Buffer,
            CachedRenderPipelineId, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState,
            DepthStencilState, Face, FragmentState, MultisampleState, PipelineCache,
            PrimitiveState, RenderPipelineDescriptor, ShaderStages, ShaderType,
            SpecializedRenderPipeline, SpecializedRenderPipelines, StencilState, TextureFormat,
            VertexAttribute, VertexState, VertexStepMode, binding_types::uniform_buffer,
        },
        renderer::RenderDevice,
        storage::GpuShaderStorageBuffer,
        view::{ExtractedView, ViewTarget, ViewUniform, ViewUniforms},
    },
};

use crate::marching_cubes::resource::MarchingCubesDrawResource;

#[derive(Resource)]
pub struct MarchingCubesDrawPipeline {
    uniform_bind_group_layout: BindGroupLayoutDescriptor,
    view_bind_group_layout: BindGroupLayoutDescriptor,
    pub shader: Handle<Shader>,
}

#[derive(Component)]
pub struct MarchingCubesDrawPipelineId(pub CachedRenderPipelineId);

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct MarchingCubesSpecializeKey {
    pub hdr: bool,
    pub samples: u32,
}

impl FromWorld for MarchingCubesDrawPipeline {
    fn from_world(world: &mut World) -> Self {
        let uniform_bind_group_layout = BindGroupLayoutDescriptor::new(
            "MarchingCubesUniformBindGroupLayout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX_FRAGMENT,
                uniform_buffer::<MarchingCubesUniform>(true),
            ),
        );
        let view_bind_group_layout = BindGroupLayoutDescriptor::new(
            "MarchingCubesViewBindGroupLayout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX_FRAGMENT,
                uniform_buffer::<ViewUniform>(true),
            ),
        );
        let shader = load_embedded_asset!(world, "marching_cubes_view.wgsl");
        Self {
            uniform_bind_group_layout,
            view_bind_group_layout,
            shader,
        }
    }
}

impl SpecializedRenderPipeline for MarchingCubesDrawPipeline {
    type Key = MarchingCubesSpecializeKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let format = if key.hdr {
            ViewTarget::TEXTURE_FORMAT_HDR
        } else {
            TextureFormat::bevy_default()
        };

        let mut shader_defs = vec![];
        if key.hdr {
            shader_defs.push("HDR".into());
        }
        RenderPipelineDescriptor {
            label: Some("MarchingCubesDrawPipeline".into()),
            layout: vec![
                self.uniform_bind_group_layout.clone(),
                self.view_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: self.shader.clone(),
                shader_defs: shader_defs.clone(),
                entry_point: Some("vertex".into()),
                buffers: vec![VertexBufferLayout {
                    array_stride: VertexFormat::Float32x4.size() * 2,
                    step_mode: VertexStepMode::Vertex,
                    attributes: vec![
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: VertexFormat::Float32x4.size(),
                            shader_location: 1,
                        },
                    ],
                }],
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs,
                entry_point: Some("fragment".into()),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multisample: MultisampleState {
                count: key.samples,
                ..default()
            },
            depth_stencil: Some(DepthStencilState {
                format: CORE_3D_DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::GreaterEqual,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            primitive: PrimitiveState {
                cull_mode: Some(Face::Back),
                ..default()
            },
            zero_initialize_workgroup_memory: true,
        }
    }
}

#[derive(Component)]
pub struct MarchingCubesDrawBindings {
    pub uniform_bind_group: BindGroup,
    pub uniform_index: u32,
    pub view_bind_group: BindGroup,
    pub vertex_buffer: Buffer,
    pub indirect_buffer: Buffer,
}

#[derive(Component, ExtractComponent, ShaderType, Clone)]
pub struct MarchingCubesUniform {
    pub world_from_local: Mat4,
}

pub fn prepare_marching_cubes_draw_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    view_uniforms: Res<ViewUniforms>,
    mc_uniforms: Res<ComponentUniforms<MarchingCubesUniform>>,
    pipeline: Res<MarchingCubesDrawPipeline>,
    fluids: Query<(
        Entity,
        &MarchingCubesDrawResource,
        &DynamicUniformIndex<MarchingCubesUniform>,
    )>,
    buffer: Res<RenderAssets<GpuShaderStorageBuffer>>,
) {
    for (entity, mc_resource, mc_uniform_index) in &fluids {
        let (Some(mc_uniform), Some(view_uniform)) =
            (mc_uniforms.binding(), view_uniforms.uniforms.binding())
        else {
            continue;
        };
        let uniform_bind_group = render_device.create_bind_group(
            "MarchingCubesUniformBindGroup",
            &pipeline_cache.get_bind_group_layout(&pipeline.uniform_bind_group_layout),
            &BindGroupEntries::single(mc_uniform),
        );

        let view_bind_group = render_device.create_bind_group(
            "MarchingCubesViewBindGroup",
            &pipeline_cache.get_bind_group_layout(&pipeline.view_bind_group_layout),
            &BindGroupEntries::single(view_uniform),
        );

        let vertex_buffer = buffer.get(&mc_resource.vertices).unwrap().buffer.clone();
        let indirect_buffer = buffer
            .get(&mc_resource.indirect_args)
            .unwrap()
            .buffer
            .clone();

        commands.entity(entity).insert(MarchingCubesDrawBindings {
            uniform_bind_group,
            uniform_index: mc_uniform_index.index(),
            view_bind_group,
            vertex_buffer,
            indirect_buffer,
        });
    }
}

pub fn prepare_marching_cubes_draw_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<MarchingCubesDrawPipeline>>,
    pipeline: Res<MarchingCubesDrawPipeline>,
    views: Query<(Entity, &Msaa, &ExtractedView)>,
) {
    for (entity, msaa, view) in &views {
        let pipeline_id = pipelines.specialize(
            &pipeline_cache,
            &pipeline,
            MarchingCubesSpecializeKey {
                hdr: view.hdr,
                samples: msaa.samples(),
            },
        );

        commands
            .entity(entity)
            .insert(MarchingCubesDrawPipelineId(pipeline_id));
    }
}

pub fn update_transform_uniform(mut fluids: Query<(&mut MarchingCubesUniform, &GlobalTransform)>) {
    for (mut uniform, transform) in &mut fluids {
        uniform.world_from_local = transform.to_matrix();
    }
}
