//! Tests atlas height doubling works correctly.
//!
//! Atlas should be squished.
//!
//! Should fail eventually if reached wgpu's texture limit.

use bevy::{
    DefaultPlugins, app::{App, Startup, Update}, asset::{AssetServer, Assets}, color::{Color, Srgba}, ecs::{
        component::Component,
        hierarchy::ChildOf,
        message::MessageReader,
        query::{Changed, With},
        system::Query,
    }, image::Image, input::keyboard::{KeyCode, KeyboardInput}, light::GlobalAmbientLight, math::{Vec2, Vec3}, pbr::{MeshMaterial3d, StandardMaterial}, prelude::{
        AlphaMode, Camera3d, Commands, Mesh, Mesh3d, OrthographicProjection, Plane3d, Projection,
        Res, ResMut, Transform,
    }
};
use bevy_rectray::{Anchor, Dimension, RectrayFrame, RectrayPlugin, RectrayWindow, Transform2D};
use bevy_rich_text3d::{
    Text3d, Text3dBounds, Text3dDimensionOut, Text3dPlugin, Text3dStyling, TextAtlas,
    TextAtlasHandle,
};

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RectrayPlugin)
        .add_plugins(Text3dPlugin {
            load_system_fonts: true,
            scale_factor: 2.0,
            sync_scale_factor_with_main_window: false,
            ..Default::default()
        })
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 800.,
            ..Default::default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, increment_on_space_press)
        .add_systems(Update, rectray_sync)
        .run();
}

#[derive(Debug, Component)]
pub struct First;

fn rectray_sync(
    mut query: Query<(&Text3dDimensionOut, &mut Dimension), Changed<Text3dDimensionOut>>,
) {
    for (out, mut dim) in query.iter_mut() {
        dim.0 = out.dimension;
    }
}

fn setup(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut atlases: ResMut<Assets<TextAtlas>>,
) {
    let image = images.add(TextAtlas::empty_image(2048, 512));
    let atlas = atlases.add(TextAtlas::new(image.clone()));
    let doubling_mat = standard_materials.add(StandardMaterial {
        base_color_texture: Some(image.clone()),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..Default::default()
    });

    let default_mat = standard_materials.add(StandardMaterial {
        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..Default::default()
    });

    let window = commands
        .spawn((RectrayWindow, RectrayFrame::default()))
        .id();

    let purpose = "Press space to increase font size.\n\n\
        This will increase the number of glyphs cached. \
        The texture will double in height if full, \
        and will eventually fail if reaching texture size limit. \
        The bottom entry shouldn't be affected even if the texture doubles.
    ";

    commands.spawn((
        ChildOf(window),
        Text3d::parse_raw(purpose).unwrap(),
        Text3dStyling {
            size: 12.,
            color: Srgba::WHITE,
            //world_scale: Some(Vec2::splat(12.)),
            ..Default::default()
        },
        Text3dBounds { width: 600. },
        Transform2D {
            anchor: Anchor::TOP_LEFT,
            ..Default::default()
        },
        Mesh3d::default(),
        MeshMaterial3d(default_mat.clone()),
    ));

    commands.spawn((
        Text3d::new(include_str!("lorem_cn.txt")),
        Text3dStyling {
            size: 16.,
            color: Srgba::new(1., 1., 0., 1.),
            ..Default::default()
        },
        Text3dBounds { width: 600. },
        TextAtlasHandle(atlas.clone()),
        Mesh3d::default(),
        MeshMaterial3d(doubling_mat.clone()),
        Transform::from_xyz(300., 150., 0.),
        First,
    ));

    commands.spawn((
        Text3d::new(include_str!("lorem_cn.txt")),
        Text3dStyling {
            size: 16.,
            color: Srgba::new(1., 1., 0., 1.),
            ..Default::default()
        },
        Text3dBounds { width: 600. },
        TextAtlasHandle(atlas.clone()),
        Mesh3d::default(),
        MeshMaterial3d(doubling_mat.clone()),
        Transform::from_xyz(300., -150., 0.),
    ));

    commands.spawn((
        Mesh3d(server.add(Mesh::from(Plane3d::new(Vec3::Z, Vec2::new(200., 200.))))),
        MeshMaterial3d(doubling_mat.clone()),
        Transform::from_xyz(-300., 0., 0.),
    ));
    commands.spawn((
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection::default_3d()),
        Transform::from_translation(Vec3::new(0., 0., 1.))
            .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
    ));
}

pub fn increment_on_space_press(
    mut input: MessageReader<KeyboardInput>,
    mut query: Query<&mut Text3dStyling, With<First>>,
) {
    for key in input.read() {
        if key.key_code == KeyCode::Space && !key.repeat && key.state.is_pressed() {
            for mut style in &mut query {
                style.size += 0.1;
            }
        }
    }
}
