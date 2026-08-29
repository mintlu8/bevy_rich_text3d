//! Tests basic a11y feature with a screen reader, use up/down arrow keys to cycle through selections.

use std::num::NonZero;

use accesskit::{Node, Role};
use bevy::{
    a11y::AccessibilityNode,
    app::{App, Startup, Update},
    asset::{AssetServer, Assets},
    camera::Camera2d,
    color::{Color, Srgba},
    ecs::{
        entity::Entity,
        hierarchy::ChildOf,
        query::Changed,
        resource::Resource,
        system::{Query, Res},
    },
    input::{keyboard::KeyCode, ButtonInput},
    input_focus::{FocusCause, InputFocus},
    light::GlobalAmbientLight,
    math::{Vec2, Vec3},
    mesh::Mesh2d,
    prelude::{Commands, OrthographicProjection, Projection, ResMut, Transform},
    sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d},
    DefaultPlugins,
};
use bevy_rectray::{
    layout::{Container, LayoutObject, ParagraphLayout, Rev, X, Y},
    Dimension, RectrayFrame, RectrayPlugin, RectrayWindow, Transform2D,
};
use bevy_rich_text3d::{
    LoadFonts, ParseBuilder, SegmentStyle, Text3d, Text3dDimensionOut, Text3dPlugin, Text3dSegment,
    Text3dStyle, TextAtlas,
};

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Text3dPlugin {
            load_system_fonts: true,
            ..Default::default()
        })
        .add_plugins(RectrayPlugin)
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 800.,
            ..Default::default()
        })
        .insert_resource(LoadFonts {
            font_paths: vec![
                "./assets/Roboto-Regular.ttf".into(),
                "./assets/Ponomar-Regular.ttf".into(),
            ],
            ..Default::default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, rectray_sync)
        .add_systems(Update, update)
        .run();
}

fn rectray_sync(
    mut query: Query<(&Text3dDimensionOut, &mut Dimension), Changed<Text3dDimensionOut>>,
) {
    for (out, mut dim) in query.iter_mut() {
        dim.0 = out.dimension;
    }
}

#[derive(Resource)]
pub struct List([Entity; 5]);

fn setup(
    mut commands: Commands,
    mut standard_materials: ResMut<Assets<ColorMaterial>>,
    server: Res<AssetServer>,
) {
    let mat = standard_materials.add(ColorMaterial {
        texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
        alpha_mode: AlphaMode2d::Blend,
        ..Default::default()
    });

    let window = commands
        .spawn((RectrayFrame::default(), RectrayWindow))
        .id();

    let layout = commands
        .spawn((
            ChildOf(window),
            Container {
                layout: LayoutObject::new(ParagraphLayout::<Rev<Y>, X>::new()),
                ..Default::default()
            },
            Dimension(Vec2::new(800., 600.)),
        ))
        .id();

    let a = commands
        .spawn((
            ChildOf(layout),
            Transform2D::default(),
            Text3d::parse_raw("1").unwrap(),
            Text3dStyle {
                size: 64.,
                stroke: NonZero::new(10),
                color: Srgba::new(0., 1., 1., 1.),
                stroke_color: Srgba::BLACK,
                ..Default::default()
            },
            AccessibilityNode(Node::new(Role::TextRun)),
            Mesh2d::default(),
            MeshMaterial2d(mat.clone()),
        ))
        .id();

    let b = commands
        .spawn((
            ChildOf(layout),
            Transform2D::default(),
            Text3d::parse_raw("2").unwrap(),
            Text3dStyle {
                size: 64.,
                stroke: NonZero::new(10),
                color: Srgba::new(0., 1., 1., 1.),
                stroke_color: Srgba::BLACK,
                ..Default::default()
            },
            AccessibilityNode(Node::new(Role::TextRun)),
            Mesh2d::default(),
            MeshMaterial2d(mat.clone()),
        ))
        .id();

    let c = commands
        .spawn((
            ChildOf(layout),
            Transform2D::default(),
            Text3d::parse_raw("3").unwrap(),
            Text3dStyle {
                size: 64.,
                color: Srgba::new(0., 1., 1., 1.),
                ..Default::default()
            },
            AccessibilityNode(Node::new(Role::TextRun)),
            Mesh2d::default(),
            MeshMaterial2d(mat.clone()),
        ))
        .id();

    let d = commands
        .spawn((
            ChildOf(layout),
            Transform2D::default(),
            Text3d::parse_raw("Hello, World!").unwrap(),
            Text3dStyle {
                size: 64.,
                stroke: NonZero::new(10),
                color: Srgba::new(0., 1., 1., 1.),
                stroke_color: Srgba::BLACK,
                ..Default::default()
            },
            AccessibilityNode(Node::new(Role::TextRun)),
            Mesh2d::default(),
            MeshMaterial2d(mat.clone()),
        ))
        .id();

    let e = commands
        .spawn((
            ChildOf(layout),
            Transform2D::default(),
            Text3d::parse(
                "Bevy is the best {laugh}!",
                ParseBuilder::new().with_parse_value(|_| {
                    Ok((
                        Text3dSegment::image_alt_text(
                            server.load("smile.png"),
                            1.,
                            "smile".to_owned(),
                        ),
                        SegmentStyle::default(),
                    ))
                }),
            )
            .unwrap(),
            Text3dStyle {
                size: 64.,
                stroke: NonZero::new(10),
                color: Srgba::new(0., 1., 1., 1.),
                stroke_color: Srgba::BLACK,
                ..Default::default()
            },
            AccessibilityNode(Node::new(Role::TextRun)),
            Mesh2d::default(),
            MeshMaterial2d(mat.clone()),
        ))
        .id();

    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection::default_3d()),
        Transform::from_translation(Vec3::new(0., 0., 1.))
            .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
    ));
    commands.insert_resource(List([a, b, c, d, e]));
}

fn update(buttons: Res<ButtonInput<KeyCode>>, mut focus: ResMut<InputFocus>, items: Res<List>) {
    if buttons.just_pressed(KeyCode::ArrowDown) {
        if let Some(e) = focus.get() {
            if let Some(p) = items.0.iter().position(|x| *x == e) {
                focus.set(items.0[(p + 1).rem_euclid(5)], FocusCause::Navigated);
            } else {
                focus.set(items.0[0], FocusCause::Navigated);
            }
        } else {
            focus.set(items.0[0], FocusCause::Navigated);
        }
    } else if buttons.just_pressed(KeyCode::ArrowUp) {
        if let Some(e) = focus.get() {
            if let Some(p) = items.0.iter().position(|x| *x == e) {
                focus.set(items.0[(p - 1).rem_euclid(5)], FocusCause::Navigated);
            } else {
                focus.set(items.0[0], FocusCause::Navigated);
            }
        } else {
            focus.set(items.0[0], FocusCause::Navigated);
        }
    }
}
