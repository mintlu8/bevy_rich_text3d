use std::num::NonZero;

use bevy::{
    app::{App, Startup},
    asset::asset_value,
    camera::{Camera2d, OrthographicProjection, Projection},
    color::{Color, Srgba},
    ecs::template::Template,
    light::GlobalAmbientLight,
    math::Vec3,
    mesh::Mesh2d,
    prelude::Commands,
    scene::{bsn, CommandsSceneExt},
    sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d},
    transform::components::Transform,
    DefaultPlugins,
};
use bevy_rich_text3d::{ParseBuilder, Text3d, Text3dPlugin, Text3dStyle, TextAtlas};

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Text3dPlugin {
            load_system_fonts: true,
            ..Default::default()
        })
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 800.,
            ..Default::default()
        })
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    let mat = asset_value(ColorMaterial {
        texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
        alpha_mode: AlphaMode2d::Blend,
        ..Default::default()
    });

    let _ = commands.spawn_scene(bsn! {
        Text3d::bsn_parse_raw("Hello, World!")
        Text3dStyle {
            size: 64.,
            stroke: NonZero::new(10),
            color: Srgba::new(0., 1., 1., 1.),
            stroke_color: Srgba::BLACK,
        }
        Mesh2d::default()
        MeshMaterial2d::<ColorMaterial>({mat.clone_template()})
    });

    let _ = commands.spawn_scene(bsn! {
        Text3d::bsn_parse("Hello, {w}!", ParseBuilder::new().with_parse_value(|_v| {
            Ok(("World".into(), Default::default()))
        }))
        Text3dStyle {
            size: 64.,
            stroke: NonZero::new(10),
            color: Srgba::new(0., 1., 1., 1.),
            stroke_color: Srgba::BLACK,
        }
        Mesh2d::default()
        MeshMaterial2d::<ColorMaterial>({mat.clone_template()})
        Transform::from_translation(Vec3::new(0., -100., 0.))
    });

    let string = "Hello, World!".to_owned();

    let _ = commands.spawn_scene(bsn! {
        Text3d::bsn_parse_raw(string)
        Text3dStyle {
            size: 64.,
            stroke: NonZero::new(10),
            color: Srgba::new(0., 1., 1., 1.),
            stroke_color: Srgba::BLACK,
        }
        Mesh2d::default()
        MeshMaterial2d::<ColorMaterial>({mat.clone_template()})
        Transform::from_translation(Vec3::new(0., 100., 0.))
    });

    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection::default_3d()),
        Transform::from_translation(Vec3::new(0., 0., 1.))
            .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
    ));
}
