//! Showcases and stress tests underlines strikethroughs and their various interactions.

use std::num::NonZero;

use bevy::{
    app::{App, Startup, Update},
    asset::Assets,
    camera::Camera2d,
    color::{Color, Srgba},
    ecs::{hierarchy::ChildOf, query::Changed, system::Query},
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
    ConditionOutput, LoadFonts, ParseBuilder, ParseError, Text3d, Text3dDimensionOut, Text3dPlugin,
    Text3dStyling, TextAtlas,
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
        .run();
}

fn rectray_sync(
    mut query: Query<(&Text3dDimensionOut, &mut Dimension), Changed<Text3dDimensionOut>>,
) {
    for (out, mut dim) in query.iter_mut() {
        dim.0 = out.dimension;
    }
}

fn setup(mut commands: Commands, mut standard_materials: ResMut<Assets<ColorMaterial>>) {
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

    let parse = |s: &str| match s {
        "true" => Ok(ConditionOutput::Constant(true)),
        "false" => Ok(ConditionOutput::Constant(false)),
        _ => Err(ParseError::Custom("Ahh".to_owned())),
    };

    commands.spawn((
        ChildOf(layout),
        Transform2D::default(),
        Text3d::parse(
            "H{?true:e{?false:o{?true:1}{?true:4}}l}l{?!true:7}{?!false:o{?false:world}{?!true:!}}",
            ParseBuilder::new().with_parse_condition(parse),
        )
        .unwrap(),
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
}
