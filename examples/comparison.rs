//! Compares with bevy's text implementations.

use bevy::{
    app::{App, Startup, Update},
    asset::{AssetServer, Assets},
    camera::Camera2d,
    color::{Color, Srgba},
    ecs::{
        component::Component,
        message::MessageReader,
        query::{With, Without},
        system::{Local, Query},
    },
    input::keyboard::{KeyCode, KeyboardInput},
    light::GlobalAmbientLight,
    math::Vec3,
    mesh::Mesh2d,
    prelude::{Commands, Res, ResMut, Transform},
    sprite::Text2d,
    sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d},
    text::TextFont,
    ui::{widget::Text, Node, UiRect, Val},
    DefaultPlugins,
};
use bevy_rectray::RectrayPlugin;
use bevy_rich_text3d::{LoadFonts, Text3d, Text3dPlugin, Text3dStyling, TextAtlas};

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RectrayPlugin)
        .add_plugins(Text3dPlugin {
            ..Default::default()
        })
        .insert_resource(LoadFonts {
            font_paths: vec!["./assets/Roboto-Regular.ttf".into()],
            ..Default::default()
        })
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 800.,
            ..Default::default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

#[derive(Debug, Component)]
pub struct First;

const SIZE: f32 = 12.;

fn setup(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut standard_materials: ResMut<Assets<ColorMaterial>>,
) {
    let default_mat = standard_materials.add(ColorMaterial {
        texture: Some(TextAtlas::DEFAULT_IMAGE),
        alpha_mode: AlphaMode2d::Blend,
        ..Default::default()
    });

    commands.spawn((
        Text3d::new("Rt3d: An example sentence:"),
        Text3dStyling {
            size: SIZE,
            font: "Roboto".into(),
            color: Srgba::WHITE,
            ..Default::default()
        },
        Transform {
            translation: Vec3::new(0., -SIZE * 2., 0.),
            ..Default::default()
        },
        Mesh2d::default(),
        MeshMaterial2d(default_mat.clone()),
    ));

    commands.spawn((
        Text2d::new("T2d: An example sentence:"),
        TextFont {
            font_size: SIZE,
            font: server.load("Roboto-Regular.ttf"),
            ..Default::default()
        },
        Transform {
            translation: Vec3::new(0., SIZE * 2., 0.),
            ..Default::default()
        },
    ));

    commands.spawn((
        Camera2d,
        Transform::from_translation(Vec3::new(0., 0., 1.))
            .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
    ));

    commands.spawn((
        Node {
            margin: UiRect {
                left: Val::Auto,
                right: Val::Auto,
                top: Val::Auto,
                bottom: Val::Auto,
            },
            ..Default::default()
        },
        Text::new("UI: An example sentence:"),
        TextFont {
            font_size: SIZE,
            font: server.load("Roboto-Regular.ttf"),
            ..Default::default()
        },
    ));
}

pub fn update(
    mut input: MessageReader<KeyboardInput>,
    mut rt3d: Query<(&mut Text3dStyling, &mut Transform), Without<TextFont>>,
    mut t2d: Query<(&mut TextFont, &mut Transform), Without<Text>>,
    mut ui: Query<&mut TextFont, With<Text>>,
    mut entry: Local<usize>,
) {
    const SIZES: [f32; 8] = [12.0, 16.0, 24.0, 32.0, 48.0, 64.0, 6.0, 8.0];
    for key in input.read() {
        if key.key_code == KeyCode::Space && !key.repeat && key.state.is_pressed() {
            *entry = (*entry + 1) % 8;
            let size = SIZES[*entry];
            for (mut style, mut transform) in &mut rt3d {
                style.size = size;
                transform.translation.y = -size * 2.;
            }

            for (mut style, mut transform) in &mut t2d {
                style.font_size = size;
                transform.translation.y = size * 2.;
            }
            for mut style in &mut ui {
                style.font_size = size;
            }
        }
    }
}
