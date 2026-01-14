//! Tests spawning text works correctly.

use std::cell::OnceCell;

use bevy::{
    app::{App, Startup, Update},
    asset::{Assets, Handle},
    color::{Color, Srgba},
    ecs::system::Local,
    input::{mouse::MouseButton, ButtonInput},
    light::GlobalAmbientLight,
    math::{Vec2, Vec3},
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::{
        AlphaMode, Camera3d, Commands, Mesh3d, OrthographicProjection, Projection, Res, ResMut,
        Transform,
    },
    DefaultPlugins,
};
use bevy_rich_text3d::{Text3d, Text3dBounds, Text3dPlugin, Text3dStyling, TextAtlas};

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Text3dPlugin {
            default_atlas_dimension: (1024, 512),
            scale_factor: 2.,
            load_system_fonts: true,
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

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection::default_3d()),
        Transform::from_translation(Vec3::new(0., 0., 100.))
            .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
    ));
}

const MESSAGES: &[&str] = &["Hello, World!", "Bevy is the best!", "Ferris loves you!"];

fn update(
    mut commands: Commands,
    input: Res<ButtonInput<MouseButton>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    cell: Local<OnceCell<Handle<StandardMaterial>>>,
    mut z: Local<f32>,
) {
    let mat = cell.get_or_init(|| {
        standard_materials.add(StandardMaterial {
            base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..Default::default()
        })
    });

    if input.just_pressed(MouseButton::Left) {
        commands.spawn((
            Text3d::new(MESSAGES[fastrand::usize(0..3)]),
            Text3dStyling {
                size: 32.,
                color: Srgba::new(1., 1., 0., 1.),
                text_shadow: Some((Srgba::BLACK, Vec2::new(2., -2.))),
                ..Default::default()
            },
            Text3dBounds { width: 600. },
            Mesh3d::default(),
            MeshMaterial3d(mat.clone()),
            Transform::from_xyz(
                fastrand::f32() * 600. - 300.,
                fastrand::f32() * 400. - 200.,
                *z,
            ),
        ));
        *z += 0.1;
    }
}
