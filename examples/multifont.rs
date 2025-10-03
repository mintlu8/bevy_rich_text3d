//! Tests multi-font works correctly.
use bevy::{
    app::{App, Startup},
    asset::Assets,
    color::{Color, Srgba},
    light::AmbientLight,
    math::Vec3,
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::{
        AlphaMode, Camera3d, Commands, Mesh3d, OrthographicProjection, Projection, ResMut,
        Transform,
    },
    DefaultPlugins,
};
use bevy_rich_text3d::{LoadFonts, Text3d, Text3dBounds, Text3dPlugin, Text3dStyling, TextAtlas};

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Text3dPlugin {
            load_system_fonts: true,
            asynchronous_load: true,
            ..Default::default()
        })
        .insert_resource(LoadFonts {
            font_paths: vec![
                "./assets/Roboto-Regular.ttf".into(),
                "./assets/Ponomar-Regular.ttf".into(),
            ],
            ..Default::default()
        })
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 800.,
            ..Default::default()
        })
        .add_systems(
            Startup,
            |mut commands: Commands, mut standard_materials: ResMut<Assets<StandardMaterial>>| {
                let mat = standard_materials.add(StandardMaterial {
                    base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..Default::default()
                });
                commands.spawn((
                    Text3d::new("System Serif"),
                    Text3dStyling {
                        size: 32.,
                        color: Srgba::new(1., 1., 0., 1.),
                        ..Default::default()
                    },
                    Text3dBounds { width: 600. },
                    Mesh3d::default(),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_xyz(0., -64., 0.),
                ));

                commands.spawn((
                    Text3d::new("System Monospace"),
                    Text3dStyling {
                        font: "monospace".into(),
                        size: 32.,
                        color: Srgba::new(1., 1., 0., 1.),
                        ..Default::default()
                    },
                    Text3dBounds { width: 600. },
                    Mesh3d::default(),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_xyz(0., -128., 0.),
                ));

                commands.spawn((
                    Text3d::new("Roboto"),
                    Text3dStyling {
                        font: "Roboto".into(),
                        size: 32.,
                        color: Srgba::new(1., 1., 0., 1.),
                        ..Default::default()
                    },
                    Text3dBounds { width: 600. },
                    Mesh3d::default(),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_xyz(0., 0., 0.),
                ));

                commands.spawn((
                    Text3d::new("Ponomar"),
                    Text3dStyling {
                        font: "Ponomar".into(),
                        size: 32.,
                        color: Srgba::new(1., 1., 0., 1.),
                        ..Default::default()
                    },
                    Text3dBounds { width: 600. },
                    Mesh3d::default(),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_xyz(0., 64., 0.),
                ));

                commands.spawn((
                    Camera3d::default(),
                    Projection::Orthographic(OrthographicProjection::default_3d()),
                    Transform::from_translation(Vec3::new(0., 0., 1.))
                        .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
                ));
            },
        )
        .run();
}
