use std::marker::PhantomData;

use bevy::{
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::RenderAssets,
        render_resource::{AsBindGroup, BindGroup, PipelineCache},
        renderer::RenderDevice,
        storage::GpuShaderStorageBuffer,
        texture::{FallbackImage, GpuImage},
        Render, RenderApp, RenderSystems,
    },
};

use crate::pipeline::HasBindGroupLayout;

pub(crate) trait FluidComputePass: Sized + Send + Sync + 'static {
    type Pipeline: Resource + FromWorld + HasBindGroupLayout;
    type Resource: Component
        + ExtractComponent
        + Clone
        + AsBindGroup<
            Param = (
                Res<'static, RenderAssets<GpuImage>>,
                Res<'static, FallbackImage>,
                Res<'static, RenderAssets<GpuShaderStorageBuffer>>,
            ),
        >;
    type BG: Component + From<BindGroup>;

    fn register_assets(_app: &mut App) {}
}

pub(crate) struct FluidComputePassPlugin<T: FluidComputePass> {
    marker: PhantomData<T>,
}

impl<T: FluidComputePass> Default for FluidComputePassPlugin<T> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<T: FluidComputePass> Plugin for FluidComputePassPlugin<T> {
    fn build(&self, app: &mut App) {
        T::register_assets(app);

        app.add_plugins(ExtractComponentPlugin::<T::Resource>::default());
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.add_systems(
            Render,
            prepare_bind_groups::<T>.in_set(RenderSystems::PrepareBindGroups),
        );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.init_resource::<T::Pipeline>();
    }
}

fn prepare_bind_groups<'a, T: FluidComputePass>(
    mut commands: Commands,
    pipeline: Res<T::Pipeline>,
    query: Query<(Entity, &T::Resource)>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    mut param: (
        Res<'a, RenderAssets<GpuImage>>,
        Res<'a, FallbackImage>,
        Res<'a, RenderAssets<GpuShaderStorageBuffer>>,
    ),
) {
    for (entity, resource) in &query {
        let bind_group = resource
            .as_bind_group(
                &pipeline.bind_group_layout(),
                &render_device,
                &pipeline_cache,
                &mut param,
            )
            .unwrap()
            .bind_group;

        commands.entity(entity).insert(T::BG::from(bind_group));
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::FluidComputePass;
    use bevy::{
        prelude::*,
        render::{
            extract_component::ExtractComponent,
            render_resource::{AsBindGroup, BindGroup},
            settings::WgpuSettings,
            RenderPlugin,
        },
    };

    use crate::{
        pipeline::{HasBindGroupLayout, SingleComputePipeline},
        plugin::FluidComputePassPlugin,
    };

    #[derive(Resource)]
    struct TestPipeline {
        pipeline: SingleComputePipeline,
    }

    impl FromWorld for TestPipeline {
        fn from_world(world: &mut World) -> Self {
            let pipeline = SingleComputePipeline::new::<TestShaderResource>(
                world,
                "TestPipeline",
                PathBuf::from("test.wgsl"),
                "main",
            );
            Self { pipeline }
        }
    }

    impl HasBindGroupLayout for TestPipeline {
        fn bind_group_layout(&self) -> &bevy::render::render_resource::BindGroupLayoutDescriptor {
            &self.pipeline.bind_group_layout
        }
    }

    struct TestPass;

    #[derive(Component)]
    struct TestBindGroup {
        pub _bind_group: BindGroup,
    }

    impl From<BindGroup> for TestBindGroup {
        fn from(_bind_group: BindGroup) -> Self {
            Self { _bind_group }
        }
    }

    #[derive(Component, Clone, ExtractComponent, AsBindGroup)]
    struct TestShaderResource {}

    impl FluidComputePass for TestPass {
        type Resource = TestShaderResource;
        type Pipeline = TestPipeline;
        type BG = TestBindGroup;
    }

    #[test]
    fn add_plugins() {
        let mut app = App::new();
        app.add_plugins((
            AssetPlugin::default(),
            TaskPoolPlugin::default(),
            RenderPlugin {
                render_creation: WgpuSettings {
                    backends: None,
                    ..default()
                }
                .into(),
                ..default()
            },
        ));
        app.add_plugins(FluidComputePassPlugin::<TestPass>::default());
    }
}
