use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::RenderAssets,
        render_resource::{
            binding_types::texture_storage_3d, BindGroup, BindGroupEntries,
            BindGroupLayoutDescriptor, BindGroupLayoutEntries, CachedComputePipelineId,
            ComputePass, ComputePipelineDescriptor, PipelineCache, ShaderStages,
            StorageTextureAccess, TextureFormat,
        },
        renderer::RenderDevice,
        texture::GpuImage,
        Render, RenderApp, RenderSystems,
    },
};
use bevy_eulerian_fluid_common::{
    fluid_compute_pipeline::is_pipeline_loaded, texture::new_texture_storage_3d,
};

use crate::{
    core::workgroup::{
        workgroup_size_x, workgroup_size_xyz, workgroup_size_y, workgroup_size_z, WorkgroupShape,
    },
    resource::FluidResources,
};

pub(crate) struct ExtrapolateVelocityPassPlugin;

impl Plugin for ExtrapolateVelocityPassPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "extrapolate_velocity.wgsl");
        embedded_asset!(app, "extrapolate_velocity_initialize.wgsl");
        app.add_plugins((ExtractComponentPlugin::<ExtrapolateVelocityResource>::default(),));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.add_systems(
            Render,
            prepare_bind_groups.in_set(RenderSystems::PrepareBindGroups),
        );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<ExtrapolateVelocityPipeline>();
    }
}

#[derive(Component, Clone, ExtractComponent)]
pub(crate) struct ExtrapolateVelocityResource {
    pub u0: Handle<Image>,
    pub v0: Handle<Image>,
    pub w0: Handle<Image>,
    pub is_u_valid: [Handle<Image>; 2],
    pub is_v_valid: [Handle<Image>; 2],
    pub is_w_valid: [Handle<Image>; 2],
    pub levelset_air0: Handle<Image>,
}

impl ExtrapolateVelocityResource {
    pub fn setup(
        commands: &mut Commands,
        entity: Entity,
        grid_size: UVec3,
        resources: &FluidResources,
        images: &mut ResMut<Assets<Image>>,
    ) {
        let is_u_valid = [
            new_texture_storage_3d(images, grid_size + UVec3::X, TextureFormat::R32Sint),
            new_texture_storage_3d(images, grid_size + UVec3::X, TextureFormat::R32Sint),
        ];
        let is_v_valid = [
            new_texture_storage_3d(images, grid_size + UVec3::Y, TextureFormat::R32Sint),
            new_texture_storage_3d(images, grid_size + UVec3::Y, TextureFormat::R32Sint),
        ];
        let is_w_valid = [
            new_texture_storage_3d(images, grid_size + UVec3::Z, TextureFormat::R32Sint),
            new_texture_storage_3d(images, grid_size + UVec3::Z, TextureFormat::R32Sint),
        ];
        let extrapolate_velocity = ExtrapolateVelocityResource {
            u0: resources.u0.clone(),
            v0: resources.v0.clone(),
            w0: resources.w0.clone(),
            is_u_valid,
            is_v_valid,
            is_w_valid,
            levelset_air0: resources.levelset_air0.clone(),
        };

        commands.entity(entity).insert(extrapolate_velocity);
    }
}

#[derive(Resource)]
pub(crate) struct ExtrapolateVelocityPipeline {
    pub extrapolate_velocity_pipeline: CachedComputePipelineId,
    pub initialize_velocity_valid_pipeline: CachedComputePipelineId,
    extrapolate_velocity_bind_group_layout: BindGroupLayoutDescriptor,
    initialize_velocity_valid_bind_group_layout: BindGroupLayoutDescriptor,
}

impl ExtrapolateVelocityPipeline {
    pub fn is_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        is_pipeline_loaded(pipeline_cache, self.extrapolate_velocity_pipeline)
            && is_pipeline_loaded(pipeline_cache, self.initialize_velocity_valid_pipeline)
    }

    pub fn dispatch(
        &self,
        pipeline_cache: &PipelineCache,
        pass: &mut ComputePass,
        bind_groups: &ExtrapolateVelocityBindGroups,
        workgroup_shape: &WorkgroupShape,
        grid_size: UVec3,
    ) {
        let num_workgroups_u = workgroup_size_x(grid_size, workgroup_shape);
        let num_workgroups_v = workgroup_size_y(grid_size, workgroup_shape);
        let num_workgroups_w = workgroup_size_z(grid_size, workgroup_shape);
        let num_workgroups_uvw = workgroup_size_xyz(grid_size, workgroup_shape);

        pass.push_debug_group("exrapolate_velocity");
        let initialize_velocity_valid_pipeline = pipeline_cache
            .get_compute_pipeline(self.initialize_velocity_valid_pipeline)
            .unwrap();
        let extrapolate_velocity_pipeline = pipeline_cache
            .get_compute_pipeline(self.extrapolate_velocity_pipeline)
            .unwrap();

        pass.set_pipeline(&initialize_velocity_valid_pipeline);
        pass.set_bind_group(0, &bind_groups.initialize_velocity_valid_bind_group, &[]);
        pass.dispatch_workgroups(
            num_workgroups_uvw.x,
            num_workgroups_uvw.y,
            num_workgroups_uvw.z,
        );

        for _ in 0..(10 / 2) {
            pass.set_pipeline(&extrapolate_velocity_pipeline);
            pass.set_bind_group(0, &bind_groups.extrapolate_u_bind_groups[0], &[]);
            pass.dispatch_workgroups(num_workgroups_u.x, num_workgroups_u.y, num_workgroups_u.z);
            pass.set_bind_group(0, &bind_groups.extrapolate_u_bind_groups[1], &[]);
            pass.dispatch_workgroups(num_workgroups_u.x, num_workgroups_u.y, num_workgroups_u.z);

            pass.set_bind_group(0, &bind_groups.extrapolate_v_bind_groups[0], &[]);
            pass.dispatch_workgroups(num_workgroups_v.x, num_workgroups_v.y, num_workgroups_v.z);
            pass.set_bind_group(0, &bind_groups.extrapolate_v_bind_groups[1], &[]);
            pass.dispatch_workgroups(num_workgroups_v.x, num_workgroups_v.y, num_workgroups_v.z);

            pass.set_bind_group(0, &bind_groups.extrapolate_w_bind_groups[0], &[]);
            pass.dispatch_workgroups(num_workgroups_w.x, num_workgroups_w.y, num_workgroups_w.z);
            pass.set_bind_group(0, &bind_groups.extrapolate_w_bind_groups[1], &[]);
            pass.dispatch_workgroups(num_workgroups_w.x, num_workgroups_w.y, num_workgroups_w.z);
        }

        pass.pop_debug_group();
    }
}

impl FromWorld for ExtrapolateVelocityPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline_cache = world.resource::<PipelineCache>();
        let workgroup_shape = world.resource::<WorkgroupShape>();

        let extrapolate_velocity_bind_group_layout = BindGroupLayoutDescriptor::new(
            "extrapolate_velocity_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    texture_storage_3d(TextureFormat::R32Float, StorageTextureAccess::ReadWrite),
                    texture_storage_3d(TextureFormat::R32Sint, StorageTextureAccess::ReadOnly),
                    texture_storage_3d(TextureFormat::R32Sint, StorageTextureAccess::WriteOnly),
                ),
            ),
        );

        let initialize_velocity_valid_bind_group_layout = BindGroupLayoutDescriptor::new(
            "initialize_velocity_valid_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    texture_storage_3d(TextureFormat::R32Sint, StorageTextureAccess::WriteOnly),
                    texture_storage_3d(TextureFormat::R32Sint, StorageTextureAccess::WriteOnly),
                    texture_storage_3d(TextureFormat::R32Sint, StorageTextureAccess::WriteOnly),
                    texture_storage_3d(TextureFormat::R32Float, StorageTextureAccess::ReadOnly),
                ),
            ),
        );

        let extrapolate_velocity_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("extrapolate_velocity_pipeline".into()),
                layout: vec![extrapolate_velocity_bind_group_layout.clone()],
                shader: load_embedded_asset!(world, "extrapolate_velocity.wgsl"),
                entry_point: Some("extrapolate_velocity".into()),
                shader_defs: vec![workgroup_shape.shader_def()],
                ..default()
            });

        let initialize_velocity_valid_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("initialize_velocity_valid_pipeline".into()),
                layout: vec![initialize_velocity_valid_bind_group_layout.clone()],
                shader: load_embedded_asset!(world, "extrapolate_velocity_initialize.wgsl"),
                entry_point: Some("extrapolate_velocity_initialize".into()),
                shader_defs: vec![workgroup_shape.shader_def()],
                ..default()
            });

        Self {
            extrapolate_velocity_pipeline,
            initialize_velocity_valid_pipeline: initialize_velocity_valid_pipeline,
            extrapolate_velocity_bind_group_layout,
            initialize_velocity_valid_bind_group_layout:
                initialize_velocity_valid_bind_group_layout,
        }
    }
}

#[derive(Component)]
pub(crate) struct ExtrapolateVelocityBindGroups {
    extrapolate_u_bind_groups: Box<[BindGroup]>,
    extrapolate_v_bind_groups: Box<[BindGroup]>,
    extrapolate_w_bind_groups: Box<[BindGroup]>,
    initialize_velocity_valid_bind_group: BindGroup,
    // initialize_v_valid_bind_group: BindGroup,
    // initialize_w_valid_bind_group: BindGroup,
}

fn prepare_bind_groups(
    mut commands: Commands,
    pipeline: Res<ExtrapolateVelocityPipeline>,
    query: Query<(Entity, &ExtrapolateVelocityResource)>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    gpu_images: Res<RenderAssets<GpuImage>>,
) {
    for (entity, resource) in &query {
        let mut extrapolate_u_bind_groups = Vec::with_capacity(2);
        let mut extrapolate_v_bind_groups = Vec::with_capacity(2);
        let mut extrapolate_w_bind_groups = Vec::with_capacity(2);
        let u0 = gpu_images.get(&resource.u0).unwrap();
        let v0 = gpu_images.get(&resource.v0).unwrap();
        let w0 = gpu_images.get(&resource.w0).unwrap();
        let is_u_valid = [
            gpu_images.get(&resource.is_u_valid[0]).unwrap(),
            gpu_images.get(&resource.is_u_valid[1]).unwrap(),
        ];
        let is_v_valid = [
            gpu_images.get(&resource.is_v_valid[0]).unwrap(),
            gpu_images.get(&resource.is_v_valid[1]).unwrap(),
        ];
        let is_w_valid = [
            gpu_images.get(&resource.is_w_valid[0]).unwrap(),
            gpu_images.get(&resource.is_w_valid[1]).unwrap(),
        ];
        let levelset_air0 = gpu_images.get(&resource.levelset_air0).unwrap();
        for i in 0..2 {
            extrapolate_u_bind_groups.push(
                render_device.create_bind_group(
                    Some(format!("extrapolate_u_bind_group_{i}").as_str()),
                    &pipeline_cache
                        .get_bind_group_layout(&pipeline.extrapolate_velocity_bind_group_layout),
                    &BindGroupEntries::sequential((
                        &u0.texture_view,
                        &is_u_valid[i].texture_view,
                        &is_u_valid[1 - i].texture_view,
                    )),
                ),
            );
            extrapolate_v_bind_groups.push(
                render_device.create_bind_group(
                    Some(format!("extrapolate_v_bind_group_{i}").as_str()),
                    &pipeline_cache
                        .get_bind_group_layout(&pipeline.extrapolate_velocity_bind_group_layout),
                    &BindGroupEntries::sequential((
                        &v0.texture_view,
                        &is_v_valid[i].texture_view,
                        &is_v_valid[1 - i].texture_view,
                    )),
                ),
            );
            extrapolate_w_bind_groups.push(
                render_device.create_bind_group(
                    Some(format!("extrapolate_w_bind_group_{i}").as_str()),
                    &pipeline_cache
                        .get_bind_group_layout(&pipeline.extrapolate_velocity_bind_group_layout),
                    &BindGroupEntries::sequential((
                        &w0.texture_view,
                        &is_w_valid[i].texture_view,
                        &is_w_valid[1 - i].texture_view,
                    )),
                ),
            );
        }

        let initialize_velocity_valid_bind_group = render_device.create_bind_group(
            Some(format!("initialize_u_valid_bind_group").as_str()),
            &pipeline_cache
                .get_bind_group_layout(&pipeline.initialize_velocity_valid_bind_group_layout),
            &BindGroupEntries::sequential((
                &is_u_valid[0].texture_view,
                &is_v_valid[0].texture_view,
                &is_w_valid[0].texture_view,
                &levelset_air0.texture_view,
            )),
        );

        // let initialize_v_valid_bind_group = render_device.create_bind_group(
        //     Some(format!("initialize_v_valid_bind_group").as_str()),
        //     &pipeline_cache
        //         .get_bind_group_layout(&pipeline.initialize_velocity_valid_bind_group_layout),
        //     &BindGroupEntries::sequential((
        //         &is_v_valid[0].texture_view,
        //         &levelset_air0.texture_view,
        //     )),
        // );

        // let initialize_w_valid_bind_group = render_device.create_bind_group(
        //     Some(format!("initialize_w_valid_bind_group").as_str()),
        //     &pipeline_cache
        //         .get_bind_group_layout(&pipeline.initialize_velocity_valid_bind_group_layout),
        //     &BindGroupEntries::sequential((
        //         &is_w_valid[0].texture_view,
        //         &levelset_air0.texture_view,
        //     )),
        // );

        commands
            .entity(entity)
            .insert(ExtrapolateVelocityBindGroups {
                initialize_velocity_valid_bind_group,
                extrapolate_u_bind_groups: extrapolate_u_bind_groups.into_boxed_slice(),
                extrapolate_v_bind_groups: extrapolate_v_bind_groups.into_boxed_slice(),
                extrapolate_w_bind_groups: extrapolate_w_bind_groups.into_boxed_slice(),
            });
    }
}
