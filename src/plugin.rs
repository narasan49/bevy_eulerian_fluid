use std::marker::PhantomData;

use bevy::{
    ecs::{schedule::ScheduleConfigs, system::ScheduleSystem},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        Render, RenderApp, RenderSystems,
    },
};

pub(crate) trait FluidComputePass: Sized + Send + Sync + 'static {
    type P: Resource + FromWorld;
    type Resource: Component + ExtractComponent + Clone;

    fn register_assets(_app: &mut App) {}

    fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem>;
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
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            T::prepare_bind_groups_system().in_set(RenderSystems::PrepareBindGroups),
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<T::P>();
    }
}

#[cfg(test)]
mod test {
    use super::FluidComputePass;
    use bevy::{
        ecs::{schedule::ScheduleConfigs, system::ScheduleSystem},
        prelude::*,
        render::{
            extract_component::ExtractComponent,
            render_asset::RenderAssets,
            render_resource::{AsBindGroup, BindGroup, BindGroupLayout},
            renderer::RenderDevice,
            storage::GpuShaderStorageBuffer,
            texture::{FallbackImage, GpuImage},
        },
    };

    use crate::{pipeline::Pipeline, plugin::FluidComputePassPlugin};

    struct TestPlugin;

    #[derive(Resource)]
    struct TestPipeline {
        bind_group_layout: BindGroupLayout,
    }

    impl Pipeline for TestPipeline {
        fn is_pipeline_state_ready(
            &self,
            _pipeline_cache: &bevy::render::render_resource::PipelineCache,
        ) -> bool {
            true
        }
    }

    impl FromWorld for TestPipeline {
        fn from_world(world: &mut World) -> Self {
            let render_device = world.resource::<RenderDevice>();
            let bind_group_layout = TestShaderResource::bind_group_layout(render_device);
            Self { bind_group_layout }
        }
    }

    #[derive(Component, Clone, ExtractComponent, AsBindGroup)]
    struct TestShaderResource {}

    #[derive(Component)]
    struct TestBindGroups {
        _bind_group: BindGroup,
    }

    impl FluidComputePass for TestPlugin {
        type P = TestPipeline;
        type Resource = TestShaderResource;

        fn prepare_bind_groups_system() -> ScheduleConfigs<ScheduleSystem> {
            prepare_bind_groups.into_configs()
        }
    }

    fn prepare_bind_groups<'a>(
        mut commands: Commands,
        pipeline: Res<TestPipeline>,
        query: Query<(Entity, &TestShaderResource)>,
        render_device: Res<RenderDevice>,
        mut param: (
            Res<'a, RenderAssets<GpuImage>>,
            Res<'a, FallbackImage>,
            Res<'a, RenderAssets<GpuShaderStorageBuffer>>,
        ),
    ) {
        for (e, res) in &query {
            let _bind_group = res
                .as_bind_group(&pipeline.bind_group_layout, &render_device, &mut param)
                .unwrap()
                .bind_group;

            commands.entity(e).insert(TestBindGroups { _bind_group });
        }
    }

    #[test]
    fn add_plugins() {
        let mut app = App::new();
        app.add_plugins(FluidComputePassPlugin::<TestPlugin>::default());
    }
}
