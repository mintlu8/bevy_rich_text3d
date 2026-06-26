use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use bevy::{
    app::{App, Startup, Update},
    asset::{AssetId, AssetServer, Assets},
    camera::Camera2d,
    color::Color,
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Local, Query},
    },
    image::Image,
    light::GlobalAmbientLight,
    math::{Vec2, Vec3},
    mesh::Mesh2d,
    prelude::{
        Commands, Mesh, OrthographicProjection, Plane3d, Projection, Res, ResMut, Transform,
    },
    sprite::Sprite,
    sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d},
    time::{Time, Virtual},
    DefaultPlugins,
};
use bevy_rich_text3d::{
    DrawStyle, LoadFonts, Text3d, Text3dPlugin, Text3dStyling, TextAtlas,
    TextProgressReportCallback, TextRenderer,
};

#[derive(Default)]
pub struct Counter(Arc<AtomicU32>);

#[derive(Debug, Component)]
pub struct Swirl;

impl TextProgressReportCallback for Counter {
    fn style_drawn(&mut self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn main() {
    let c = Counter::default();
    let counter = c.0.clone();
    let mut callback = Some(c);
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Text3dPlugin {
            default_atlas_dimension: (8192, 8192),
            scale_factor: 2.,
            sync_scale_factor_with_main_window: false,
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
        .add_systems(
            Update,
            move |mut commands: Commands,
                  timer: Res<Time<Virtual>>,
                  mut local: Local<f32>,
                  mut query: Query<(Entity, &mut Transform), With<Swirl>>| {
                if counter.load(Ordering::Relaxed) >= 133 {
                    for (entity, _) in &query {
                        commands.entity(entity).despawn();
                    }
                }
                *local += timer.delta_secs();
                if *local > 0.1 {
                    *local -= 0.1;
                    for (_, mut transform) in &mut query {
                        transform.rotate_local_z(f32::to_radians(-45.0));
                    }
                }
            },
        )
        .add_systems(
            Startup,
            move |settings: Res<Text3dPlugin>,
                  text_renderer: Res<TextRenderer>,
                  mut atlases: ResMut<Assets<TextAtlas>>,
                  mut images: ResMut<Assets<Image>>| {
                let task = text_renderer.prepare_images_cloned(
                    &settings,
                    [(
                        AssetId::default(),
                        (16..150).map(|x| {
                            (
                                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
                                DrawStyle {
                                    size: x as f32,
                                    family: "Roboto".into(),
                                    ..Default::default()
                                },
                            )
                        }),
                    )],
                    &mut atlases,
                    &mut images,
                    callback.take().unwrap(),
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
                    Sprite {
                        image: server.load("loading.png"),
                        custom_size: Some(Vec2::splat(128.)),
                        ..Default::default()
                    },
                    Swirl,
                ));
                commands.spawn((
                    Mesh2d(server.add(Mesh::from(Plane3d::new(Vec3::Z, Vec2::new(200., 200.))))),
                    MeshMaterial2d(mat.clone()),
                    Transform::IDENTITY,
                ));
                commands.spawn((
                    Text3d::new("Hello World!"),
                    Text3dStyling {
                        font: "Roboto".into(),
                        size: 64.,
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
