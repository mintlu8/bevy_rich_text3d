use bevy::{
    app::{App, Startup},
    asset::{Asset, Assets},
    color::Color,
    light::GlobalAmbientLight,
    math::Vec3,
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
use bevy_rich_text3d::{GlyphMeta, Text3d, Text3dPlugin, Text3dStyling, TextAtlas};

#[derive(Debug, Clone, TypePath, AsBindGroup, Asset)]
pub struct SpookyShader {
    #[uniform(100)]
    pub frequency: f32,
    #[uniform(101)]
    pub intensity: f32,
}

impl MaterialExtension for SpookyShader {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Path("wobble.wgsl".into())
    }
}

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, SpookyShader>>::default())
        .add_plugins(Text3dPlugin {
            default_atlas_dimension: (1024, 512),
            load_system_fonts: true,
            ..Default::default()
        })
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 800.,
            ..Default::default()
        })
        .add_systems(Startup, |mut commands: Commands,
            mut mats: ResMut<Assets<ExtendedMaterial<StandardMaterial, SpookyShader>>>,
        | {
            let mat = mats.add(
                ExtendedMaterial {
                    base: StandardMaterial {
                        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        ..Default::default()
                    },
                    extension: SpookyShader {
                        frequency: 2.,
                        intensity: 9.,
                    },
                }
            );
            commands.spawn((
                Text3d::parse_raw("Something {s-4, s-white, transparent, v-1:SPOOKY} is happening!").unwrap(),
                Text3dStyling {
                    size: 64.0,
                    uv1: (GlyphMeta::RandomPerGlyph, GlyphMeta::MagicNumber),
                    ..Default::default()
                },
                Mesh3d::default(),
                MeshMaterial3d(mat.clone()),
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
