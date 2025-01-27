use bevy::prelude::*;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut spawn = |x, c| {

        let mesh = Mesh::new(
            bevy::render::mesh::PrimitiveTopology::LineList,
            bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD
        ).with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![Vec3::ZERO, x]);
    
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial::from_color(c))),
        ));
    };
    spawn(Vec3::X, LinearRgba::RED);
    spawn(Vec3::Y, LinearRgba::GREEN);
    spawn(Vec3::Z, LinearRgba::BLUE);
}
