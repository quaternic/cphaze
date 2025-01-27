use bevy::{core_pipeline::core_3d::{Transparent3d, CORE_3D_DEPTH_FORMAT}, ecs::system::lifetimeless::{Read, SRes}, pbr::{MeshPipeline, MeshPipelineKey, MeshPipelineViewLayoutKey, SetMaterialBindGroup, SetMeshBindGroup, SetMeshViewBindGroup}, prelude::*, render::{extract_component::{ExtractComponent, ExtractComponentPlugin}, mesh::{PrimitiveTopology, VertexBufferLayout}, render_asset::RenderAssets, render_phase::{AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand, RenderCommandResult, SetItemPipeline, ViewSortedRenderPhases}, render_resource::{AsBindGroup, BindGroupLayout, BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthStencilState, FragmentState, FrontFace, MultisampleState, PipelineCache, PolygonMode, PrimitiveState, RenderPipelineDescriptor, SpecializedRenderPipeline, SpecializedRenderPipelines, VertexAttribute, VertexFormat, VertexState, VertexStepMode}, renderer::RenderDevice, storage::{GpuShaderStorageBuffer, ShaderStorageBuffer}, view::{self, ExtractedView, RenderVisibleEntities, ViewTarget, VisibilitySystems}, Render, RenderApp, RenderSet}};

use crate::func_xy::ParticleMaterial;

const SHADER_ASSET_PATH: & str= "shaders/particle.wgsl";

pub struct PointCloudPipelinePlugin;
impl Plugin for PointCloudPipelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<PointCloudEntity>::default())
            .add_systems(
                PostUpdate,
                view::check_visibility::<With<PointCloudEntity>>
                    .in_set(VisibilitySystems::CheckVisibility),
            );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else { panic!() };

        render_app
            .init_resource::<SpecializedRenderPipelines<PointCloudPipeline>>()
            .add_render_command::<Transparent3d, DrawPointCloudPipelineCommands>()
            .add_systems(Render, queue_point_cloud_pipeline.in_set(RenderSet::Queue));
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else { return };
        render_app.init_resource::<PointCloudPipeline>();
    }
}


#[derive(Resource)]
struct PointCloudPipeline {
    shader_handle: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
    material_layout: BindGroupLayout,
}
#[derive(Clone, Component, ExtractComponent)]
pub struct PointCloudEntity {
    pub xs: Handle<ShaderStorageBuffer>,
    pub ys: Handle<ShaderStorageBuffer>,
    pub zs: Handle<ShaderStorageBuffer>,
    pub init: u32,
}

type DrawPointCloudPipelineCommands = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetMaterialBindGroup<ParticleMaterial, 2>,
    DrawPointCloud,
);


struct DrawPointCloud;
impl<P> RenderCommand<P> for DrawPointCloud
where
    P: PhaseItem,
{
    type Param = (
        SRes<RenderAssets<GpuShaderStorageBuffer>>,
    );

    type ViewQuery = ();

    type ItemQuery = Read<PointCloudEntity>;
    
    fn render<'w>(
        _item: &P,
        _view: bevy::ecs::query::ROQueryItem<'w, Self::ViewQuery>,
        entity: Option<bevy::ecs::query::ROQueryItem<'w, Self::ItemQuery>>,
        param: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let buffers = param.0.into_inner();
        let entity = entity.unwrap();

        let xs = buffers.get(&entity.xs).unwrap().buffer.slice(..);
        let ys = buffers.get(&entity.ys).unwrap().buffer.slice(..);
        let zs = buffers.get(&entity.zs).unwrap().buffer.slice(..);

        pass.set_vertex_buffer(0, xs);
        pass.set_vertex_buffer(1, ys);
        pass.set_vertex_buffer(2, zs);

        pass.draw(0..entity.init, 0..1);
        RenderCommandResult::Success
    }
}


fn queue_point_cloud_pipeline(
    pipeline_cache: Res<PipelineCache>,
    custom_mesh_pipeline: Res<PointCloudPipeline>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    transparent_draw_functions: Res<DrawFunctions<Transparent3d>>,
    mut specialized_render_pipelines: ResMut<SpecializedRenderPipelines<PointCloudPipeline>>,
    views: Query<(Entity, &RenderVisibleEntities, &ExtractedView, &Msaa), With<ExtractedView>>,

) {
    let draw_function = transparent_draw_functions
        .read()
        .id::<DrawPointCloudPipelineCommands>();

    for (view_entity, view_visible_entities, view, msaa) in views.iter() {
        let Some(transparent_phase) = transparent_render_phases.get_mut(&view_entity) else { continue };

        let view_key = MeshPipelineKey::from_msaa_samples(msaa.samples())
            | MeshPipelineKey::from_hdr(view.hdr);

        for &entity in view_visible_entities
            .get::<With<PointCloudEntity>>()
            .iter()
        { 
            let mesh_key = view_key | MeshPipelineKey::from_primitive_topology(PrimitiveTopology::PointList);
            let pipeline = specialized_render_pipelines
                .specialize(
                    &pipeline_cache,
                    &custom_mesh_pipeline,
                    (*msaa, mesh_key),
                );

            transparent_phase.add(
                Transparent3d {
                    distance: 0.0, // ?
                    pipeline,
                    entity,
                    draw_function,
                    batch_range: 0..1, // ?
                    extra_index: PhaseItemExtraIndex(0), // ?
                }
            );
        }
    }
}


impl FromWorld for PointCloudPipeline {
    fn from_world(world: &mut World) -> Self {
        let shader_handle = world.resource::<AssetServer>().load(SHADER_ASSET_PATH);

        Self {
            shader_handle,
            mesh_pipeline: MeshPipeline::from_world(world),
            material_layout: ParticleMaterial::bind_group_layout(world.resource::<RenderDevice>()),
        }
    }
}

impl SpecializedRenderPipeline for PointCloudPipeline {
    type Key = (Msaa, MeshPipelineKey);

    fn specialize(
            &self,
            (msaa, mesh_key): Self::Key,
        ) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("Point Cloud Pipeline".into()),
            layout: vec![
                self.mesh_pipeline
                    .get_view_layout(MeshPipelineViewLayoutKey::from(mesh_key))
                    .clone(),
                self.mesh_pipeline.mesh_layouts.model_only.clone(),
                self.material_layout.clone(),
            ],
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: self.shader_handle.clone(),
                shader_defs: vec![],
                entry_point: "vertex".into(),
                buffers: vec![
                    VertexBufferLayout {
                        array_stride: 4,
                        step_mode: VertexStepMode::Vertex,
                        // this needs to match the layout of Vertex
                        attributes: vec![
                            VertexAttribute {
                                format: VertexFormat::Sint32,
                                offset: 0,
                                shader_location: 0,
                            },
                        ],
                    },
                    VertexBufferLayout {
                        array_stride: 4,
                        step_mode: VertexStepMode::Vertex,
                        // this needs to match the layout of Vertex
                        attributes: vec![
                            VertexAttribute {
                                format: VertexFormat::Sint32,
                                offset: 0,
                                shader_location: 1,
                            }, 
                        ],
                    },
                    VertexBufferLayout {
                        array_stride: 4,
                        step_mode: VertexStepMode::Vertex,
                        // this needs to match the layout of Vertex
                        attributes: vec![
                            VertexAttribute {
                                format: VertexFormat::Float32,
                                offset: 0,
                                shader_location: 2,
                            },
                        ],
                    },
                ],
            },
            fragment: Some(FragmentState {
                shader: self.shader_handle.clone(),
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: ViewTarget::TEXTURE_FORMAT_HDR,
                    blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::PointList,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                strip_index_format: None,
                unclipped_depth: true,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: CORE_3D_DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: CompareFunction::GreaterEqual,
                stencil: default(),
                bias: default(),
            }),
            multisample: MultisampleState {
                count: msaa.samples(),
                ..default()
            },
            zero_initialize_workgroup_memory: false,
        }
    }
}
