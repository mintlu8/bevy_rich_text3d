use std::{f32::consts::PI, num::NonZero};

use bevy::{
    anti_alias::smaa::{Smaa, SmaaPreset},
    app::{App, Startup},
    asset::Assets,
    camera::{Camera3d, PerspectiveProjection},
    color::{Color, Srgba},
    light::{DirectionalLight, GlobalAmbientLight},
    math::{Quat, Vec2, Vec3},
    mesh::{Mesh3d, Meshable},
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::{Commands, Mesh, Plane3d, Projection, ResMut, Transform},
    render::alpha::AlphaMode,
    DefaultPlugins,
};
use bevy_rich_text3d::{Text3d, Text3dPlugin, Text3dStyling, TextAtlas};

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

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    let mat = standard_materials.add(StandardMaterial {
        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
        alpha_mode: AlphaMode::Mask(0.5),
        unlit: true,
        cull_mode: None,
        ..Default::default()
    });

    commands.spawn((
        Text3d::new("Hello World!"),
        Text3dStyling {
            size: 64.,
            stroke: NonZero::new(10),
            color: Srgba::new(1., 0., 0., 1.),
            stroke_color: Srgba::BLACK,
            world_scale: Some(Vec2::splat(0.25)),
            layer_offset: 0.001,
            ..Default::default()
        },
        Mesh3d::default(),
        MeshMaterial3d(mat.clone()),
        Transform {
            translation: Vec3::new(1., 1., 4.),
            rotation: Quat::from_axis_angle(Vec3::Y, -30.),
            scale: Vec3::ONE,
        },
    ));

    commands.spawn((
        Text3d::new("Bevy is the best!"),
        Text3dStyling {
            size: 64.,
            stroke: NonZero::new(10),
            color: Srgba::new(0., 0.4, 1., 1.),
            stroke_color: Srgba::BLACK,
            world_scale: Some(Vec2::splat(0.25)),
            layer_offset: 0.001,
            ..Default::default()
        },
        Mesh3d::default(),
        MeshMaterial3d(mat.clone()),
        Transform {
            translation: Vec3::new(4., 1., 1.),
            rotation: Quat::from_axis_angle(Vec3::Y, 0.),
            scale: Vec3::ONE,
        },
    ));

    commands.spawn((
        Text3d::parse_raw("~~__a lot of layers__~~").unwrap(),
        Text3dStyling {
            size: 64.,
            stroke: NonZero::new(10),
            color: Srgba::new(1., 0.0, 1., 1.),
            stroke_color: Srgba::BLACK,
            world_scale: Some(Vec2::splat(0.5)),
            layer_offset: 0.1,
            ..Default::default()
        },
        Mesh3d::default(),
        MeshMaterial3d(mat.clone()),
        Transform {
            translation: Vec3::new(1., 1., 1.),
            rotation: Quat::from_axis_angle(Vec3::Y, f32::to_radians(45.)),
            scale: Vec3::ONE,
        },
    ));

    // ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(standard_materials.add(StandardMaterial::from_color(Srgba::GREEN))),
        Transform::from_xyz(0., 0., 0.),
    ));

    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 2000.,
            ..Default::default()
        },
        Transform::from_translation(Vec3::new(10., 10., -10.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: PI / 3.,
            aspect_ratio: 1.0,
            near: 0.1,
            far: 1000.,
            ..Default::default()
        }),
        Transform::from_translation(Vec3::new(6., 4., 6.))
            .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        Smaa {
            preset: SmaaPreset::Medium,
        },
    ));
}
