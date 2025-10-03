//! Tests atlas height doubling works correctly.
//!
//! Press space to add text.
//!
//! Atlas should be squished.
//!
//! Should fail eventually if reached wgpu's texture limit.

use bevy::{
    app::{App, Startup},
    asset::{AssetServer, Assets},
    color::{Color, Srgba},
    light::AmbientLight,
    math::{Vec2, Vec3},
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::{
        AlphaMode, Camera3d, Commands, Mesh, Mesh3d, OrthographicProjection, Plane3d, Projection,
        Res, ResMut, Transform,
    },
    DefaultPlugins,
};
use bevy_rich_text3d::{Text3d, Text3dBounds, Text3dPlugin, Text3dStyling, TextAtlas};

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Text3dPlugin {
            default_atlas_dimension: (1024, 512),
            scale_factor: 2.,
            load_system_fonts: true,
            ..Default::default()
        })
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 800.,
            ..Default::default()
        })
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    let mat = standard_materials.add(StandardMaterial {
        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..Default::default()
    });
    commands.spawn((
        Text3d::new(include_str!("lorem_arabic.txt")),
        Text3dStyling {
            size: 32.,
            color: Srgba::new(1., 1., 0., 1.),
            text_shadow: Some((Srgba::BLACK, Vec2::new(2., -2.))),
            ..Default::default()
        },
        Text3dBounds { width: 600. },
        Mesh3d::default(),
        MeshMaterial3d(mat.clone()),
        Transform::from_xyz(300., 0., 0.),
    ));

    commands.spawn((
        Mesh3d(server.add(Mesh::from(Plane3d::new(Vec3::Z, Vec2::new(200., 200.))))),
        MeshMaterial3d(mat.clone()),
        Transform::from_xyz(-300., 0., 0.),
    ));
    commands.spawn((
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection::default_3d()),
        Transform::from_translation(Vec3::new(0., 0., 1.))
            .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
    ));
}
