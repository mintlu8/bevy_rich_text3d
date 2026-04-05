#![allow(clippy::collapsible_if)]
use std::{cell::OnceCell, num::NonZero};

use bevy::{
    app::{App, Startup, Update},
    asset::{uuid_handle, AssetServer, Assets, Handle},
    camera::Camera2d,
    color::{Color, Srgba},
    ecs::{
        hierarchy::ChildOf,
        query::Changed,
        system::{Local, Query, Res},
    },
    image::Image,
    light::GlobalAmbientLight,
    math::{primitives::Plane3d, Vec2, Vec3},
    mesh::{Mesh, Mesh2d},
    prelude::{Commands, OrthographicProjection, Projection, ResMut, Transform},
    sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d},
    time::Time,
    DefaultPlugins,
};
use bevy_rectray::{
    layout::{Container, LayoutObject, ParagraphLayout, Rev, X, Y},
    Dimension, RectrayFrame, RectrayPlugin, RectrayWindow, Transform2D,
};
use bevy_rich_text3d::{
    ParseError, SegmentStyle, Text3d, Text3dDimensionOut, Text3dPlugin, Text3dSegment,
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
        .add_systems(Startup, setup)
        .add_systems(Update, rectray_sync)
        .add_systems(Update, load)
        .run();
}

fn rectray_sync(
    mut query: Query<(&Text3dDimensionOut, &mut Dimension), Changed<Text3dDimensionOut>>,
) {
    for (out, mut dim) in query.iter_mut() {
        dim.0 = out.dimension;
    }
}

static EMOJI_SMILE: Handle<Image> = uuid_handle!("1347c9b7-c46a-48e7-b7b8-023a354b7cac");

fn setup(
    mut commands: Commands,
    mut standard_materials: ResMut<Assets<ColorMaterial>>,
    server: Res<AssetServer>,
) {
    let parse = |input: &str| -> Result<(Text3dSegment, SegmentStyle), ParseError> {
        let white = SegmentStyle {
            fill_color: Some(Srgba::WHITE),
            ..Default::default()
        };
        match input {
            "spiral" => Ok((
                Text3dSegment::Image {
                    image: server.load("spiral.png"),
                    width: 1.,
                },
                white,
            )),
            "wide" => Ok((
                Text3dSegment::Image {
                    image: server.load("spiral.png"),
                    width: 2.,
                },
                white,
            )),
            "thin" => Ok((
                Text3dSegment::Image {
                    image: server.load("spiral.png"),
                    width: 0.5,
                },
                white,
            )),
            "smile" => Ok((
                Text3dSegment::Image {
                    image: EMOJI_SMILE.clone(),
                    width: 1.,
                },
                white,
            )),
            "ultrawide" => Ok((
                Text3dSegment::Image {
                    image: EMOJI_SMILE.clone(),
                    width: 6.,
                },
                white,
            )),
            _ => Err(ParseError::NotSupported("Expected spiral.")),
        }
    };

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

    commands.spawn((
        Mesh2d(server.add(Mesh::from(Plane3d::new(Vec3::Z, Vec2::new(200., 200.))))),
        MeshMaterial2d(mat.clone()),
        Transform::from_translation(Vec3::new(0., 100., -1.)),
    ));

    commands.spawn((
        ChildOf(layout),
        Transform2D::default(),
        Text3d::parse("Spend 2 {spiral} to do 15 damage!", parse, |_| {
            Err(ParseError::NotSupported(""))
        })
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
        ChildOf(layout),
        Transform2D::default(),
        Text3d::parse("thin {thin} wide {wide}", parse, |_| {
            Err(ParseError::NotSupported(""))
        })
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
        ChildOf(layout),
        Transform2D::default(),
        Text3d::parse("Smile {smile}", parse, |_| {
            Err(ParseError::NotSupported(""))
        })
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
        ChildOf(layout),
        Transform2D::default(),
        Text3d::parse("Ultra {ultrawide} Wide!", parse, |_| {
            Err(ParseError::NotSupported(""))
        })
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

/// Delay loading the smile emoji.
fn load(
    time: Res<Time>,
    server: Res<AssetServer>,
    cell: Local<OnceCell<Handle<Image>>>,
    mut images: ResMut<Assets<Image>>,
) {
    if images.get(&EMOJI_SMILE).is_none() {
        if time.elapsed().as_secs_f32() > 5.0 {
            let handle = cell.get_or_init(|| server.load("smile.png"));
            if let Some(im) = images.get(handle) {
                let result = im.clone();
                let _ = images.insert(&EMOJI_SMILE, result);
                println!("Loaded!");
            }
        }
    }
}
