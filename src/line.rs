use std::num::NonZero;

use bevy::{
    image::Image,
    math::{FloatOrd, Rect, Vec2},
};
use cosmic_text::{fontdb::ID, FontSystem, LayoutGlyph};
use ttf_parser::Face;

use crate::{
    styling::{GlyphEntry, GlyphTextureOf},
    tess::PathEncoder,
    SegmentStyle, Text3dSegment, Text3dStyling, TextAtlas,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct LineRun {
    pub min_index: usize,
    pub max_index: usize,
    pub min_offset: f32,
    pub max_offset: f32,
    pub size: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum LineMode {
    Underline,
    Strikethrough,
}

/// So that contains never returns true.
impl Default for LineRun {
    fn default() -> Self {
        Self {
            min_index: usize::MAX,
            max_index: usize::MAX,
            min_offset: 0.0,
            max_offset: 0.0,
            size: 0.0,
        }
    }
}

impl LineRun {
    pub fn contains(&self, glyph: &LayoutGlyph) -> bool {
        !(self.min_index > glyph.end || self.max_index < glyph.start)
    }

    pub fn uv_range(&self, min: f32, max: f32, stroke: f32) -> UvRanges {
        let r_min = self.min_offset - stroke;
        let r_max = self.max_offset + stroke;
        let corner = (r_max - r_min).min(self.size + stroke * 2.0) / 2.0;
        let f = |x| match x {
            x if x <= self.min_offset => 0.0,
            x if x >= self.max_offset => 1.0,
            x if x <= self.min_offset + corner => (x - self.min_offset) / corner / 2.0,
            x if x >= self.max_offset - corner => (self.max_offset - x) / corner / 2.0 + 0.5,
            _ => 0.5,
        };
        if min < min + corner && max > min + corner {
            let center = min + corner;
            UvRanges::Two([((min, f(min)), (center, 0.5)), ((center, 0.5), (max, 0.5))])
        } else if min < max - corner && max > max - corner {
            let center = max - corner;
            UvRanges::Two([((min, 0.5), (center, 0.5)), ((center, 0.5), (max, f(max)))])
        } else {
            UvRanges::One(((min, f(min)), (max, f(max))))
        }
    }
}

pub enum UvRanges {
    One(((f32, f32), (f32, f32))),
    Two([((f32, f32), (f32, f32)); 2]),
}

impl UvRanges {
    pub fn iter(&self) -> impl Iterator<Item = ((f32, f32), (f32, f32))> + use<'_> {
        match self {
            UvRanges::One(v) => std::array::from_ref(v).iter().copied(),
            UvRanges::Two(v) => v.iter().copied(),
        }
    }
}

impl From<LineMode> for GlyphTextureOf {
    fn from(value: LineMode) -> Self {
        match value {
            LineMode::Underline => GlyphTextureOf::UnderlineTexture,
            LineMode::Strikethrough => GlyphTextureOf::StrikethroughTexture,
        }
    }
}

impl LineMode {
    pub fn select<T>(&self, underline: T, strikethrough: T) -> T {
        match self {
            LineMode::Underline => underline,
            LineMode::Strikethrough => strikethrough,
        }
    }

    /// Requires a valid first point.
    pub fn new_run(
        &self,
        size: f32,
        mut index: usize,
        glyphs: &[LayoutGlyph],
        text: &[(Text3dSegment, SegmentStyle)],
    ) -> LineRun {
        let first = &glyphs[index];
        let mut result = LineRun {
            min_index: first.start,
            max_index: first.end,
            min_offset: first.x,
            max_offset: first.x + first.w,
            size,
        };
        loop {
            index += 1;
            let Some(next) = glyphs.get(index) else {
                break;
            };
            if next.font_id != first.font_id || next.font_size != first.font_size {
                break;
            }
            let Some((_, next_style)) = text.get(next.metadata) else {
                break;
            };
            if !self.validate(next_style) {
                break;
            }
            result.max_index = next.end;
            result.max_offset = next.x + next.w;
        }
        result
    }

    pub fn validate(&self, style: &SegmentStyle) -> bool {
        match self {
            LineMode::Underline => style.underline.unwrap_or_default(),
            LineMode::Strikethrough => style.strikethrough.unwrap_or_default(),
        }
    }

    pub fn boundary(
        &self,
        glyphs: &[LayoutGlyph],
        segments: &[(Text3dSegment, SegmentStyle)],
        index: usize,
        stroke: f32,
    ) -> (f32, f32) {
        let current = &glyphs[index];
        let mut min = current.x - stroke;
        let mut max = current.x + current.w + stroke;
        if let Some(prev) = glyphs.get(index.wrapping_sub(1)) {
            if prev.font_id == current.font_id && prev.font_size == current.font_size {
                if let Some((_, style)) = segments.get(prev.metadata) {
                    if self.validate(style) {
                        min = (prev.x + prev.w + current.x) / 2.;
                    }
                }
            }
        }
        if let Some(next) = glyphs.get(index.wrapping_add(1)) {
            if next.font_id == current.font_id && next.font_size == current.font_size {
                if let Some((_, style)) = segments.get(next.metadata) {
                    if self.validate(style) {
                        max = (current.x + current.w + next.x) / 2.;
                    }
                }
            }
        }
        (min, max)
    }

    pub fn size(&self, font_system: &mut FontSystem, id: ID, size: f32) -> f32 {
        font_system
            .db()
            .with_face_data(id, |file, _| {
                let Ok(face) = Face::parse(file, 0) else {
                    return None;
                };
                let metrics = match self {
                    LineMode::Underline => face.underline_metrics()?,
                    LineMode::Strikethrough => face.underline_metrics()?,
                };
                Some(metrics.thickness as f32 / face.units_per_em() as f32 * size)
            })
            .flatten()
            .unwrap_or(size)
    }

    pub fn get_line_rect(
        &self,
        font_system: &mut FontSystem,
        size: f32,
        min: f32,
        max: f32,
        stroke: f32,
        glyph: &LayoutGlyph,
    ) -> Option<Rect> {
        font_system
            .db()
            .with_face_data(glyph.font_id, |file, _| {
                let Ok(face) = Face::parse(file, 0) else {
                    return None;
                };
                let metrics = match self {
                    LineMode::Underline => face.underline_metrics()?,
                    LineMode::Strikethrough => face.strikeout_metrics()?,
                };
                let base = metrics.position as f32 / face.units_per_em() as f32 * size;
                let height = metrics.thickness as f32 / face.units_per_em() as f32 * size;
                Some(Rect {
                    min: Vec2::new(min, base - height - stroke),
                    max: Vec2::new(max, base + stroke),
                })
            })
            .flatten()
    }

    pub fn get_atlas_rect(
        &self,
        font_system: &mut FontSystem,
        font: ID,
        scale_factor: f32,
        atlas: &mut TextAtlas,
        image: &mut Image,
        attrs: &SegmentStyle,
        style: &Text3dStyling,
        stroke: Option<NonZero<u32>>,
    ) -> Option<Rect> {
        let entry = GlyphEntry {
            font,
            glyph_id: (*self).into(),
            join: style.stroke_join,
            size: FloatOrd(style.size),
            weight: attrs.weight.unwrap_or(style.weight),
            stroke,
        };
        atlas
            .glyphs
            .get(&entry)
            .copied()
            .map(|(a, _)| a)
            .or_else(|| {
                font_system
                    .db()
                    .with_face_data(font, |file, _| {
                        let Ok(face) = Face::parse(file, 0) else {
                            return None;
                        };
                        self.cache_texture(
                            entry,
                            style.size,
                            scale_factor,
                            atlas,
                            image,
                            stroke,
                            face,
                        )
                    })
                    .flatten()
            })
    }

    pub fn cache_texture(
        &self,
        entry: GlyphEntry,
        size: f32,
        scale_factor: f32,
        atlas: &mut TextAtlas,
        image: &mut Image,
        stroke: Option<NonZero<u32>>,
        face: Face,
    ) -> Option<Rect> {
        let metrics = match self {
            LineMode::Underline => face.underline_metrics(),
            LineMode::Strikethrough => face.strikeout_metrics(),
        }?;
        let unit_per_em = face.units_per_em() as f32;
        let d = metrics.thickness as f32 / unit_per_em * size * scale_factor;

        let mut tess_commands = PathEncoder::default();
        tess_commands.push_rect(0., 0., d, d);
        let stroke = stroke.map(|x| x.get() as f32 * size / 100.);

        tess_commands
            .tess_glyph(stroke, 1., atlas, image, entry)
            .map(|(x, _)| x)
    }
}
