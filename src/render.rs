use bevy::{
    asset::{AssetId, Assets, RenderAssetUsages},
    color::{ColorToComponents, LinearRgba},
    ecs::{
        change_detection::DetectChanges,
        system::{Query, Res, ResMut},
        world::{Mut, Ref},
    },
    image::Image,
    math::{FloatOrd, IVec2, Rect, Vec2, Vec3, Vec4},
    render::mesh::{Indices, Mesh, Mesh2d, Mesh3d, PrimitiveTopology, VertexAttributeValues},
};
use cosmic_text::{
    ttf_parser::{Face, GlyphId, OutlineBuilder},
    Attrs, Buffer, Family, Metrics, Shaping, Weight, Wrap,
};
use std::num::NonZero;
use zeno::{Cap, Command as ZCommand, Format, Join, Mask, Stroke, Style, Transform, Vector};

use crate::{
    fetch::FetchedTextSegment,
    styling::{GlyphEntry, StrokeJoins},
    text3d::{Text3d, Text3dSegment},
    GlyphMeta, Text3dBounds, Text3dDimensionOut, Text3dPlugin, Text3dStyling, TextAtlas,
    TextAtlasHandle, TextRenderer,
};

fn corners(rect: Rect) -> [[f32; 2]; 4] {
    [
        [rect.min.x, rect.min.y],
        [rect.max.x, rect.min.y],
        [rect.min.x, rect.max.y],
        [rect.max.x, rect.max.y],
    ]
}

// Take the allocation if possible but clear the data.
macro_rules! recycle_mesh {
    ($mesh: expr, $attr: ident, $ty: ident) => {
        if let Some(VertexAttributeValues::$ty(mut v)) = $mesh.remove_attribute(Mesh::$attr) {
            v.clear();
            v
        } else {
            Vec::new()
        }
    };
}

fn default_mesh() -> Mesh {
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<Vec3>::new())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<Vec3>::new())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, Vec::<Vec2>::new())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, Vec::<Vec2>::new())
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, Vec::<Vec4>::new())
        .with_inserted_indices(Indices::U16(Vec::new()))
}

fn get_mesh<'t>(
    mesh2d: &mut Option<Mut<Mesh2d>>,
    mesh3d: &mut Option<Mut<Mesh3d>>,
    meshes: &'t mut Assets<Mesh>,
) -> Option<&'t mut Mesh> {
    let mut id = mesh2d
        .as_ref()
        .map(|x| x.id())
        .or_else(|| mesh3d.as_ref().map(|x| x.id()))?;
    if id == AssetId::default() {
        let handle = meshes.add(default_mesh());
        id = handle.id();
        if let Some(handle_2d) = mesh2d {
            handle_2d.0 = handle.clone();
        }
        if let Some(handle_3d) = mesh3d {
            handle_3d.0 = handle;
        }
    }
    meshes.get_mut(id)
}

pub fn text_render(
    settings: Res<Text3dPlugin>,
    font_system: ResMut<TextRenderer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut atlases: ResMut<Assets<TextAtlas>>,
    mut text_query: Query<(
        Ref<Text3d>,
        Ref<Text3dBounds>,
        Ref<Text3dStyling>,
        &TextAtlasHandle,
        Option<&mut Mesh2d>,
        Option<&mut Mesh3d>,
        &mut Text3dDimensionOut,
    )>,
    segments: Query<Ref<FetchedTextSegment>>,
) {
    let Ok(mut lock) = font_system.0.try_lock() else {
        return;
    };
    let mut redraw = false;
    if font_system.is_changed() {
        redraw = true;
    }
    // Add asynchronously drawn text.
    for (id, atlas, image) in lock.queue.drain(..) {
        let img_id = atlas.image.id();
        images.insert(img_id, image);
        atlases.insert(id, atlas);
        redraw = true;
    }
    let font_system = &mut lock.font_system;
    let scale_factor = settings.scale_factor;
    for (text, bounds, styling, atlas, mut mesh2d, mut mesh3d, mut output) in text_query.iter_mut()
    {
        let Some(atlas) = atlases.get_mut(atlas.0.id()) else {
            return;
        };

        if atlas.image.id() == AssetId::default() || !images.contains(atlas.image.id()) {
            atlas.image = images.add(TextAtlas::empty_image(
                settings.default_atlas_dimension.0,
                settings.default_atlas_dimension.1,
            ))
        };

        let Some(image) = images.get_mut(atlas.image.id()) else {
            return;
        };

        // Change detection.
        if !redraw && !text.is_changed() && !bounds.is_changed() && !styling.is_changed() {
            let mut unchanged = true;
            for segment in &text.segments {
                if let Text3dSegment::Extract(entity) = &segment.0 {
                    if segments.get(*entity).is_ok_and(|x| x.is_changed()) {
                        unchanged = false;
                        break;
                    }
                }
            }
            if unchanged {
                let Some(image) = images.get(atlas.image.id()) else {
                    continue;
                };
                let new_dimension = IVec2::new(image.width() as i32, image.height() as i32);
                if output.atlas_dimension == new_dimension {
                    continue;
                }

                let Some(mesh) = get_mesh(&mut mesh2d, &mut mesh3d, &mut meshes) else {
                    continue;
                };

                let Some(VertexAttributeValues::Float32x2(uv0)) =
                    mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
                else {
                    continue;
                };
                for [x, y] in uv0 {
                    *x *= output.atlas_dimension.x as f32 / new_dimension.x as f32;
                    *y *= output.atlas_dimension.y as f32 / new_dimension.y as f32;
                }
                output.atlas_dimension = new_dimension;
                continue;
            }
        }

        let mut buffer = Buffer::new(
            font_system,
            Metrics::new(styling.size, styling.size * styling.line_height),
        );
        buffer.set_wrap(font_system, Wrap::WordOrGlyph);
        buffer.set_size(font_system, Some(bounds.width), None);
        buffer.set_tab_width(font_system, styling.tab_width);

        buffer.set_rich_text(
            font_system,
            text.segments
                .iter()
                .enumerate()
                .map(|(idx, (text, style))| {
                    (
                        match text {
                            Text3dSegment::String(s) => s.as_str(),
                            Text3dSegment::Extract(e) => segments
                                .get(*e)
                                .map(|x| x.into_inner().as_str())
                                .unwrap_or(""),
                        },
                        style.as_attr(&styling).metadata(idx),
                    )
                }),
            &Attrs::new()
                .family(Family::Name(&styling.font))
                .style(styling.style.into())
                .weight(styling.weight.into()),
            Shaping::Advanced,
            None,
        );

        buffer.shape_until_scroll(font_system, true);

        let Some(mesh) = get_mesh(&mut mesh2d, &mut mesh3d, &mut meshes) else {
            continue;
        };

        let mut positions = recycle_mesh!(mesh, ATTRIBUTE_POSITION, Float32x3);
        let mut normals = recycle_mesh!(mesh, ATTRIBUTE_NORMAL, Float32x3);
        let mut uv0 = recycle_mesh!(mesh, ATTRIBUTE_UV_0, Float32x2);
        let mut uv1 = recycle_mesh!(mesh, ATTRIBUTE_UV_1, Float32x2);
        let mut colors = recycle_mesh!(mesh, ATTRIBUTE_COLOR, Float32x4);

        let mut indices = if let Some(Indices::U16(indices)) = mesh.remove_indices() {
            indices
        } else {
            Vec::new()
        };
        indices.clear();

        let mut width = 0.0f32;
        let mut sum_width = 0.0f32;

        let mut idx = 0;
        let mut real_index = 0;

        let mut tess_commands = CommandEncoder::default();
        let mut height = 0.0f32;
        for run in buffer.layout_runs() {
            width = width.max(run.line_w);
            height = height.max(run.line_top + run.line_height);
            for glyph in run.glyphs.iter() {
                let Some((_, attrs)) = text.segments.get(glyph.metadata) else {
                    continue;
                };
                let dx = -run.line_w * styling.align.as_fac();

                let fills = attrs.fill.unwrap_or(styling.fill);
                let stroke = attrs.stroke.or(styling.stroke);

                let renders: &[_] = match (fills, stroke) {
                    (true, None) => &[(None, attrs.fill_color.unwrap_or(styling.color), 0.)],
                    (false, Some(stroke)) => &[(
                        Some(stroke),
                        attrs.stroke_color.unwrap_or(styling.stroke_color),
                        0.,
                    )],
                    (true, Some(stroke)) if styling.stroke_offset > 0.0 => &[
                        (
                            None,
                            attrs.fill_color.unwrap_or(styling.color),
                            styling.stroke_offset,
                        ),
                        (
                            Some(stroke),
                            attrs.stroke_color.unwrap_or(styling.stroke_color),
                            0.,
                        ),
                    ],
                    (true, Some(stroke)) => &[
                        (
                            Some(stroke),
                            attrs.stroke_color.unwrap_or(styling.stroke_color),
                            -styling.stroke_offset,
                        ),
                        (None, attrs.fill_color.unwrap_or(styling.color), 0.),
                    ],
                    (false, None) => &[],
                };

                for (stroke, color, z) in renders.iter().copied() {
                    let Some((pixel_rect, base)) = atlas
                        .glyphs
                        .get(&GlyphEntry {
                            font: glyph.font_id,
                            glyph_id: glyph.glyph_id,
                            size: FloatOrd(glyph.font_size),
                            weight: styling.weight,
                            stroke,
                        })
                        .copied()
                        .or_else(|| {
                            font_system
                                .db()
                                .with_face_data(glyph.font_id, |file, _| {
                                    let Ok(face) = Face::parse(file, 0) else {
                                        return None;
                                    };
                                    cache_glyph(
                                        scale_factor,
                                        atlas,
                                        image,
                                        &mut tess_commands,
                                        glyph,
                                        stroke,
                                        styling.stroke_joins,
                                        attrs.weight.unwrap_or(styling.weight).into(),
                                        face,
                                    )
                                })
                                .flatten()
                        })
                    else {
                        continue;
                    };

                    let i = idx as u16 * 4;
                    indices.extend([i, i + 1, i + 2, i + 1, i + 3, i + 2]);
                    idx += 1;

                    let local_x0 = glyph.x + base.x;
                    let local_x1 = local_x0 + pixel_rect.width() / scale_factor;

                    let x0 = local_x0 + dx;
                    let y0 = glyph.y + base.y - run.line_y;
                    let x1 = local_x1 + dx;
                    let y1 = y0 + pixel_rect.height() / scale_factor;
                    positions.extend([[x0, y0, z], [x1, y0, z], [x0, y1, z], [x1, y1, z]]);

                    normals.extend([[0., 0., 1.]; 4]);

                    // First we cache the pixel position since the texture may be resized.
                    uv0.extend(corners(pixel_rect));

                    let mut uv1_buffer = [[0., 0.], [0., 0.], [0., 0.], [0., 0.]];

                    for (meta_type, i) in [(styling.uv1.0, 0), (styling.uv1.1, 1)] {
                        match meta_type {
                            GlyphMeta::Index => {
                                for pair in &mut uv1_buffer {
                                    pair[i] = real_index as f32;
                                }
                            }
                            GlyphMeta::Advance => {
                                let x0 = (local_x0 + sum_width) / styling.size;
                                let x1 = (local_x0 + sum_width) / styling.size;
                                uv1_buffer[0][i] = x0;
                                uv1_buffer[1][i] = x1;
                                uv1_buffer[2][i] = x0;
                                uv1_buffer[3][i] = x1;
                            }
                            GlyphMeta::PerGlyphAdvance => {
                                let x = (glyph.x
                                    + base.x
                                    + pixel_rect.width() / scale_factor / 2.0
                                    + sum_width)
                                    / styling.size;
                                uv1_buffer[0][i] = x;
                                uv1_buffer[1][i] = x;
                                uv1_buffer[2][i] = x;
                                uv1_buffer[3][i] = x;
                            }
                            GlyphMeta::MagicNumber => {
                                uv1_buffer[0][i] = attrs.magic_number.unwrap_or(0.);
                                uv1_buffer[1][i] = attrs.magic_number.unwrap_or(0.);
                                uv1_buffer[2][i] = attrs.magic_number.unwrap_or(0.);
                                uv1_buffer[3][i] = attrs.magic_number.unwrap_or(0.);
                            }
                            GlyphMeta::RowX => (),
                            GlyphMeta::ColY => (),
                        }
                    }

                    uv1.extend(uv1_buffer);

                    colors.extend([LinearRgba::from(color).to_f32_array(); 4]);
                }
                real_index += 1;
            }
            sum_width += run.line_w;
        }

        let min = positions
            .chunks(4)
            .map(|arr| arr[0][0])
            .reduce(f32::min)
            .unwrap_or(0.0);
        let max = positions
            .chunks(4)
            .map(|arr| arr[3][0])
            .reduce(f32::max)
            .unwrap_or(0.0);

        let dimension = Vec2::new(max - min, height);
        let center = Vec2::new((max + min) / 2., -height / 2.);
        let offset = *styling.anchor * dimension - center;
        let bb_min = Vec2::new(min, -height);

        for (meta_type, i) in [(styling.uv1.0, 0), (styling.uv1.1, 1)] {
            match meta_type {
                GlyphMeta::RowX => {
                    for (uv1, position) in uv1.iter_mut().zip(positions.iter()) {
                        uv1[i] = (position[0] - bb_min.x) / dimension.x;
                    }
                }
                GlyphMeta::ColY => {
                    for (uv1, position) in uv1.iter_mut().zip(positions.iter()) {
                        uv1[i] = (position[1] - bb_min.y) / dimension.y;
                    }
                }
                _ => (),
            }
        }

        positions.iter_mut().for_each(|[x, y, _]| {
            *x += offset.x;
            *y += offset.y;
        });

        output.dimension = dimension;
        output.atlas_dimension = IVec2::new(image.width() as i32, image.height() as i32);

        let inv_width = 1.0 / image.width() as f32;
        let inv_height = 1.0 / image.height() as f32;

        uv0.iter_mut().for_each(|[x, y]| {
            *x *= inv_width;
            *y *= inv_height;
        });

        if !positions.is_empty() {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv0);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, uv1);
            mesh.insert_indices(Indices::U16(indices));
        } else {
            // Placeholder, since empty mesh panics on some platforms.
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 0.0, 0.0]; 3]);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; 3]);
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0, 0.0, 0.0, 0.0]; 3]);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; 3]);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, vec![[0.0, 0.0]; 3]);
            mesh.insert_indices(Indices::U16(vec![0, 1, 2]));
        }
    }
}

pub(crate) fn cache_glyph(
    scale_factor: f32,
    atlas: &mut TextAtlas,
    image: &mut Image,
    tess_commands: &mut CommandEncoder,
    glyph: &cosmic_text::LayoutGlyph,
    stroke: Option<NonZero<u32>>,
    stroke_joins: StrokeJoins,
    weight: Weight,
    face: Face,
) -> Option<(Rect, Vec2)> {
    let unit_per_em = face.units_per_em() as f32;
    let entry = GlyphEntry {
        font: glyph.font_id,
        glyph_id: glyph.glyph_id,
        size: FloatOrd(glyph.font_size),
        weight: weight.into(),
        stroke,
    };
    tess_commands.commands.clear();
    face.outline_glyph(GlyphId(glyph.glyph_id), tess_commands)?;
    let (alpha_map, bb) = if let Some(stroke) = stroke {
        Mask::new(&tess_commands.commands)
            .style(Style::Stroke(Stroke {
                width: stroke.get() as f32 * unit_per_em / 100.,
                start_cap: Cap::Round,
                end_cap: Cap::Round,
                join: match stroke_joins {
                    StrokeJoins::Miter => Join::Miter,
                    StrokeJoins::Round => Join::Round,
                    _ => Join::Bevel,
                },
                ..Default::default()
            }))
            .transform(Some(Transform::scale(
                glyph.font_size / unit_per_em * scale_factor,
                glyph.font_size / unit_per_em * scale_factor,
            )))
            .format(Format::Alpha)
            .render()
    } else {
        Mask::new(&tess_commands.commands)
            .transform(Some(Transform::scale(
                glyph.font_size / unit_per_em * scale_factor,
                glyph.font_size / unit_per_em * scale_factor,
            )))
            .format(Format::Alpha)
            .render()
    };
    let (w, h) = (bb.width as usize, bb.height as usize);
    let base = Vec2::new(bb.left as f32, bb.top as f32) / scale_factor;
    let pixel_rect = atlas.cache(image, entry, base, w, h, |buffer, pitch| {
        for x in 0..w {
            for y in 0..h {
                buffer[y * pitch + x * 4 + 3] = alpha_map[y * w + x]
            }
        }
        IVec2::new(w as i32, h as i32)
    });
    Some((pixel_rect, base))
}

#[derive(Debug, Default)]
pub(crate) struct CommandEncoder {
    commands: Vec<ZCommand>,
}

impl OutlineBuilder for CommandEncoder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.commands.push(ZCommand::MoveTo(Vector::new(x, y)));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.commands.push(ZCommand::LineTo(Vector::new(x, y)));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.commands
            .push(ZCommand::QuadTo(Vector::new(x1, y1), Vector::new(x, y)));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.commands.push(ZCommand::CurveTo(
            Vector::new(x1, y1),
            Vector::new(x2, y2),
            Vector::new(x, y),
        ));
    }

    fn close(&mut self) {
        self.commands.push(ZCommand::Close);
    }
}
