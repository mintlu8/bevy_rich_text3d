use bevy::{
    app::{App, Startup},
    asset::{Asset, Assets},
    color::Color,
    light::GlobalAmbientLight,
    math::Vec3,
    mesh::{Mesh, MeshVertexAttribute, MeshVertexBufferLayoutRef, VertexFormat},
    pbr::{
        ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
        MaterialPlugin, MeshMaterial3d, StandardMaterial,
    },
    prelude::{
        AlphaMode, Camera3d, Commands, Mesh3d, OrthographicProjection, Projection, ResMut,
        Transform,
    },
    reflect::TypePath,
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
    DefaultPlugins,
};
use bevy_rich_text3d::{
    GlyphMeta, MeshExport, MeshExportEntry, Text3d, Text3dPlugin, Text3dStyling, TextAtlas,
};

#[derive(Debug, Clone, TypePath, AsBindGroup, Asset)]
pub struct SpookyShader {
    #[uniform(100)]
    pub frequency: f32,
    #[uniform(101)]
    pub intensity: f32,
}

impl MaterialExtension for SpookyShader {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Path("custom_attribute.wgsl".into())
    }
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path("custom_attribute.wgsl".into())
    }

    fn specialize(
        _: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(5),
            MY_ATTRIBUTE.at_shader_location(30),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

const MY_ATTRIBUTE: MeshVertexAttribute =
    MeshVertexAttribute::new("text_attribute", 666, VertexFormat::Float32x4);

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
                Text3d::parse_raw("Something {v-1:**CRAZY**} is happening!").unwrap(),
                Text3dStyling {
                    size: 64.0,
                    export: MeshExport::Custom(vec![
                        MeshExportEntry::new(MY_ATTRIBUTE, &[
                            GlyphMeta::RandomPerGlyph,
                            GlyphMeta::MagicNumber,
                            GlyphMeta::Advance,
                            GlyphMeta::PerGlyphAdvance,
                        ])
                    ]),
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
