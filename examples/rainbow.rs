use std::num::NonZero;

use bevy::{
    app::{App, Startup},
    asset::{Asset, Assets},
    color::{Color, Srgba},
    light::GlobalAmbientLight,
    math::{Vec2, Vec3},
    pbr::{ExtendedMaterial, MaterialExtension, MaterialPlugin, MeshMaterial3d, StandardMaterial},
    prelude::{
        AlphaMode, Camera3d, Commands, Mesh3d, OrthographicProjection, Projection, ResMut,
        Transform,
    },
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    DefaultPlugins,
};
use bevy_rich_text3d::{
    GlyphMeta, MeshExport, Text3d, Text3dBounds, Text3dPlugin, Text3dStyling, TextAlign, TextAtlas,
};

#[derive(Debug, Clone, TypePath, AsBindGroup, Asset)]
pub struct UVTextShader {}

impl MaterialExtension for UVTextShader {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path("uv.wgsl".into())
    }
}

#[derive(Debug, Clone, TypePath, AsBindGroup, Asset)]
pub struct RainbowShader {}

impl MaterialExtension for RainbowShader {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path("rainbow.wgsl".into())
    }
}

#[derive(Debug, Clone, TypePath, Asset, AsBindGroup)]
struct SplitShader {}

impl MaterialExtension for SplitShader {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        ShaderRef::Path("split.wgsl".into())
    }
}

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, UVTextShader>>::default())
        .add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, RainbowShader>>::default())
        .add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, SplitShader>>::default())
        .add_plugins(Text3dPlugin {
            load_system_fonts: true,
            ..Default::default()
        })
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 800.,
            ..Default::default()
        })
        .add_systems(Startup, |mut commands: Commands,
            mut mats: ResMut<Assets<ExtendedMaterial<StandardMaterial, UVTextShader>>>,
            mut mats2: ResMut<Assets<ExtendedMaterial<StandardMaterial, RainbowShader>>>,
            mut mats3: ResMut<Assets<ExtendedMaterial<StandardMaterial, SplitShader>>>,
        | {
            let mat = mats.add(
                ExtendedMaterial {
                    base: StandardMaterial {
                        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        ..Default::default()
                    },
                    extension: UVTextShader {},
                }
            );
            commands.spawn((
                Text3d::new(include_str!("lorem.txt")),
                Text3dStyling {
                    align: TextAlign::Left,
                    export: MeshExport::Uv1(GlyphMeta::UvX, GlyphMeta::UvY),
                    ..Default::default()
                },
                Text3dBounds { width: 500. },
                Mesh3d::default(),
                MeshMaterial3d(mat.clone()),
            ));

            commands.spawn((
                Text3d::new(include_str!("lorem.txt")),
                Text3dStyling {
                    align: TextAlign::Left,
                    export: MeshExport::Uv1(GlyphMeta::GlyphUvX, GlyphMeta::GlyphUvY),
                    ..Default::default()
                },
                Text3dBounds { width: 500. },
                Mesh3d::default(),
                MeshMaterial3d(mat.clone()),
                Transform::from_translation(Vec3::new(0., -250., 0.)),
            ));

            let mat2 = mats2.add(
                ExtendedMaterial {
                    base: StandardMaterial {
                        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        ..Default::default()
                    },
                    extension: RainbowShader {},
                }
            );
            commands.spawn((
                Text3d::new("Lorem ipsum dolor sit amet."),
                Text3dStyling {
                    align: TextAlign::Left,
                    size: 64.,
                    export: MeshExport::Uv1(GlyphMeta::Advance, GlyphMeta::PerGlyphAdvance),
                    ..Default::default()
                },
                Mesh3d::default(),
                MeshMaterial3d(mat2.clone()),
                Transform::from_translation(Vec3::new(0., 200., 0.)),
            ));

            let mat3 = mats3.add(
                ExtendedMaterial {
                    base: StandardMaterial {
                        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        ..Default::default()
                    },
                    extension: SplitShader {},
                }
            );

            commands.spawn((
                Text3d::parse_raw("~~__categorized__~~").unwrap(),
                Text3dStyling {
                    size: 64.,
                    stroke: NonZero::new(10),
                    text_shadow: Some((Srgba::BLACK, Vec2::new(4., 4.))),
                    export: MeshExport::Uv1(GlyphMeta::Category, GlyphMeta::Category),
                    ..Default::default()
                },
                Mesh3d::default(),
                MeshMaterial3d(mat3.clone()),
                Transform::from_translation(Vec3::new(0., 300., 0.)),
            ));

            commands.spawn((
                Camera3d::default(),
                Projection::Orthographic(OrthographicProjection::default_3d()),
                Transform::from_translation(Vec3::new(0., 0., 1.))
                    .looking_at(Vec3::new(0., 0., 0.), Vec3::Y)
            ));
        })
        .run();
}
