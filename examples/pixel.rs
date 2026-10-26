use bevy::{
    DefaultPlugins,
    app::{App, Startup},
    asset::Assets,
    camera::Camera2d,
    color::{Color, Srgba},
    light::GlobalAmbientLight,
    math::Vec3,
    mesh::Mesh2d,
    prelude::{
        Commands, OrthographicProjection, Projection, ResMut, Transform,
    },
    sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d},
};
use bevy_rich_text3d::{LoadFonts, Text3d, Text3dPlugin, Text3dStyle, TextAtlas};

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Text3dPlugin {
            ..Default::default()
        })
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 800.,
            ..Default::default()
        })
        .insert_resource(LoadFonts {
            font_paths: vec!["assets/PixelifySans-Regular.ttf".to_owned()],
            ..Default::default()
        })
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut standard_materials: ResMut<Assets<ColorMaterial>>,
) {
    let mat = standard_materials.add(ColorMaterial {
        texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
        alpha_mode: AlphaMode2d::Mask(0.5),
        ..Default::default()
    });

    commands.spawn((
        Text3d::new("Hello World!"),
        Text3dStyle {
            size: 64.,
            font: "Pixelify".into(),
            //stroke: NonZero::new(10),
            color: Srgba::new(0., 1., 1., 1.),
            stroke_color: Srgba::BLACK,
            ..Default::default()
        },
        Mesh2d::default(),
        MeshMaterial2d(mat.clone()),
        //Transform::from_scale(Vec3::splat(10.))
    ));

    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection::default_3d()),
        Transform::from_translation(Vec3::new(0., 0., 1.))
            .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
    ));
}
