use std::num::NonZero;

use bevy::{
    app::{App, Startup},
    asset::{AssetId, AssetServer, Assets},
    camera::Camera2d,
    color::{Color, Srgba},
    image::Image,
    light::AmbientLight,
    math::{Vec2, Vec3},
    mesh::Mesh2d,
    prelude::{
        Commands, Mesh, OrthographicProjection, Plane3d, Projection, Res, ResMut, Transform,
    },
    sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d},
    DefaultPlugins,
};
use bevy_rich_text3d::{DrawStyle, Text3d, Text3dPlugin, Text3dStyling, TextAtlas, TextRenderer};

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Text3dPlugin {
            load_system_fonts: true,
            scale_factor: 2.,
            sync_scale_factor_with_main_window: false,
            ..Default::default()
        })
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 800.,
            ..Default::default()
        })
        .add_systems(
            Startup,
            |settings: Res<Text3dPlugin>,
             text_renderer: Res<TextRenderer>,
             mut atlases: ResMut<Assets<TextAtlas>>,
             mut images: ResMut<Assets<Image>>| {
                let task = text_renderer.prepare_images_cloned(
                    &settings,
                    [(
                        AssetId::default(),
                        [(
                            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz,.-=+/~",
                            DrawStyle {
                                size: 16.,
                                ..Default::default()
                            },
                        )],
                    )],
                    &mut atlases,
                    &mut images,
                    (),
                );
                std::thread::spawn(task);
            },
        )
        .add_systems(
            Startup,
            |mut commands: Commands,
             server: Res<AssetServer>,
             mut standard_materials: ResMut<Assets<ColorMaterial>>| {
                let mat = standard_materials.add(ColorMaterial {
                    texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
                    alpha_mode: AlphaMode2d::Blend,
                    ..Default::default()
                });
                commands.spawn((
                    Mesh2d(server.add(Mesh::from(Plane3d::new(Vec3::Z, Vec2::new(200., 200.))))),
                    MeshMaterial2d(mat.clone()),
                    Transform::IDENTITY,
                ));
                commands.spawn((
                    Text3d::new("Hello World!"),
                    Text3dStyling {
                        size: 64.,
                        stroke: NonZero::new(10),
                        color: Srgba::new(0., 1., 1., 1.),
                        stroke_color: Srgba::BLACK,
                        ..Default::default()
                    },
                    Mesh2d::default(),
                    MeshMaterial2d(mat.clone()),
                ));
                commands.spawn((
                    Camera2d,
                    Projection::Orthographic(OrthographicProjection::default_3d()),
                    Transform::from_translation(Vec3::new(0., 0., 1.))
                        .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
                ));
            },
        )
        .run();
}
