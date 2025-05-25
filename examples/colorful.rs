use bevy::{
    app::{App, Startup, Update},
    asset::Assets,
    color::{Color, Srgba},
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    math::Vec3,
    pbr::{AmbientLight, MeshMaterial3d, StandardMaterial},
    prelude::{
        AlphaMode, Camera3d, Commands, Component, Mesh3d, OrthographicProjection, Projection,
        Query, Res, ResMut, Transform, With,
    },
    DefaultPlugins,
};
use bevy_rich_text3d::{
    FetchedTextSegment, ParseError, Text3d, Text3dBounds, Text3dPlugin, Text3dSegment,
    Text3dStyling, TextAlign, TextAtlas,
};

#[derive(Debug, Component)]
pub struct FetchFPS;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Text3dPlugin {
            load_system_fonts: true,
            ..Default::default()
        })
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 800.,
            ..Default::default()
        })
        .add_systems(
            Startup,
            |mut commands: Commands, mut standard_materials: ResMut<Assets<StandardMaterial>>| {
                let mat = standard_materials.add(StandardMaterial {
                    base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..Default::default()
                });
                let text = Text3d::parse(
                    "{s-20, s-black:<Time Bomb>}: Deals {orange:**explosion**} {bg-red:damage} equal to {red:*fps*}, which is {s-20, s-black, red:{fps}}!",
                    |s| {
                        if s == "fps" {
                            Ok(Text3dSegment::Extract(
                                commands.spawn((FetchedTextSegment::EMPTY, FetchFPS)).id(),
                            ))
                        } else {
                            Err(ParseError::Custom(format!("Bad value {s}.")))
                        }
                    },
                    |s| Err(ParseError::Custom(format!("Bad style {s}."))),
                )
                .unwrap();
                commands.spawn((
                    text,
                    Text3dStyling {
                        size: 32.,
                        color: Srgba::new(0., 1., 1., 1.),
                        align: TextAlign::Center,
                        ..Default::default()
                    },
                    Text3dBounds { width: 400. },
                    Mesh3d::default(),
                    MeshMaterial3d(mat.clone()),
                ));

                commands.spawn((
                    Camera3d::default(),
                    Projection::Orthographic(OrthographicProjection::default_3d()),
                    Transform::from_translation(Vec3::new(0., 0., 1.))
                        .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
                ));
            },
        )
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(
            Update,
            |fps: Res<DiagnosticsStore>,
             mut query: Query<&mut FetchedTextSegment, With<FetchFPS>>| {
                let Some(fps) = fps.get(&FrameTimeDiagnosticsPlugin::FPS) else {
                    return;
                };
                let Some(fps) = fps.smoothed() else {
                    return;
                };
                let fps_text = format!("{:.0}", fps);
                for mut segment in &mut query {
                    if segment.as_str() != fps_text {
                        segment.0 = fps_text.clone();
                    }
                }
            },
        )
        .run();
}
