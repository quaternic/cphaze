

use bevy::{core_pipeline::core_3d::CORE_3D_DEPTH_FORMAT, prelude::*, render::{render_asset::RenderAssetUsages, render_resource::{AsBindGroup, CompareFunction, DepthStencilState, ShaderRef}, view::NoFrustumCulling}};

pub fn plugin(
    app: &mut App,
) {
    // FIXME: sometimes the plane is not loaded correctly
    app
        .add_plugins(MaterialPlugin::<PlaneMaterial>::default())
        .add_systems(Startup, load_assets)
        ;
}


#[derive(Resource, Reflect)]
struct VisualizationPlane {
    mesh: Handle<Mesh>,
    material: Handle<PlaneMaterial>,
}

fn load_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PlaneMaterial>>,
) {
    let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![
        [ 0., 0., 0.], [ 1., 1., 0.], [-1., 1., 0.],
        [ 0., 0., 0.], [-1., 1., 0.], [-1.,-1., 0.],
        [ 0., 0., 0.], [-1.,-1., 0.], [ 1.,-1., 0.],
        [ 0., 0., 0.], [ 1.,-1., 0.], [ 1., 1., 0.],
    ]);
    
    let mesh = meshes.add(mesh);
    let material = materials.add(PlaneMaterial {
        time: 0,
        origin: Vec3::new(0.0, 0.0, 0.0),
        x: Vec3::X,
        y: Vec3::Y,
        xy_bounds: IVec4::from_array([i32::MIN, i32::MIN, i32::MAX, i32::MAX]),
    });
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material.clone()),
        NoFrustumCulling,
    ));
        
    commands.insert_resource(VisualizationPlane { mesh, material });
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Resource, Default)]
pub struct PlaneMaterial {
    #[uniform(0)]
    pub time: i32,
    #[uniform(1)]
    pub origin: Vec3,
    #[uniform(2)]
    pub x: Vec3,
    #[uniform(3)]
    pub y: Vec3,
    #[uniform(4)]
    pub xy_bounds: IVec4,
}

impl Material for PlaneMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/plane.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "shaders/plane.wgsl".into()
    }
    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError>
    {
        descriptor.primitive.unclipped_depth = true;
        descriptor.primitive.cull_mode = None;
        descriptor.depth_stencil = Some(DepthStencilState {
            format: CORE_3D_DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: CompareFunction::GreaterEqual,
            stencil: default(),
            bias: default(),
        });
        Ok(())
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Premultiplied
    }
}
