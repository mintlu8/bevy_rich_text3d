use bevy::{
    color::Srgba,
    ecs::component::Component,
    math::{FloatOrd, Vec2},
};
use cosmic_text::{fontdb::ID, Attrs};
use std::{num::NonZeroU32, sync::Arc};

use crate::{prepare::family, MeshExport, StrokeJoin, Style, TextAlign, TextAnchor, Weight};

#[cfg(feature = "reflect")]
use bevy::prelude::{Reflect, ReflectComponent, ReflectDefault};

/// Default text style of a rich text component.
#[derive(Debug, Component, Clone)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component, Default))]
pub struct Text3dStyling {
    /// Size of the font, corresponding to world space units.
    ///
    /// Ths is cached per unique value so keep the amount of unique values of `size` low,
    /// use [`Text3dStyling::world_scale`] instead for small variations.
    pub size: f32,
    /// Name of the font, by default `"serif"`.
    ///
    /// For `"serif"`, `"sans-serif"`, `"monospace"`, `"cursive"` and `"fantasy"`,
    /// use one of the default fonts set in `cosmic_text`.
    pub font: Arc<str>,
    /// Style of the font, i.e. italic.
    pub style: Style,
    /// Weight of the font, i.e. bold.
    pub weight: Weight,
    /// Horizontal alignment of the font.
    pub align: TextAlign,
    /// Where local `[0, 0]` is inside the text block's Aabb.
    pub anchor: TextAnchor,
    /// Height of a line multiplied by font size, by default `1.0`.
    pub line_height: f32,
    /// Color of fill.
    pub color: Srgba,
    /// Color of stroke.
    pub stroke_color: Srgba,
    /// If not set, will not render the fill of the font, making it hollow.
    pub fill: bool,
    /// If set, renders a stroke or outline of the font.
    ///
    /// The value represents a percentage of the font size and should be
    /// in `1..10` for hollow text and `1..20` for outline.
    ///
    /// Ths is cached per unique value so be sure not to use too many of them.
    pub stroke: Option<NonZeroU32>,
    /// If true, render stroke in front.
    pub stroke_in_front: bool,
    /// The shape of the stroke line joins, usually [`StrokeJoin::Round`].
    pub stroke_join: StrokeJoin,
    /// Sets the distance between different layers, i.e. stroke and fill.
    ///
    /// `0.0` likely works for transparent render modes and opaque2d, but
    /// not for opaque3d.
    pub layer_offset: f32,
    /// Determines what to extract as uv1.
    pub export: MeshExport,
    /// Tab in terms of spaces, default 4.
    pub tab_width: u16,
    /// If set, overwrite the size of `em` in the generated mesh.
    ///
    /// By default the mesh size is relative to [`Text3dStyling::size`], which is equivalent to `Some((size, size))`.
    pub world_scale: Option<Vec2>,
    /// If `Some`, render a text shadow.
    pub text_shadow: Option<(Srgba, Vec2)>,
}

impl Default for Text3dStyling {
    fn default() -> Self {
        Self {
            size: 16.,
            color: Srgba::WHITE,
            font: Default::default(),
            style: Default::default(),
            weight: Default::default(),
            align: Default::default(),
            anchor: TextAnchor::CENTER,
            stroke_color: Srgba::WHITE,
            fill: true,
            stroke: Default::default(),
            line_height: 1.0,
            layer_offset: 0.01,
            stroke_in_front: false,
            stroke_join: StrokeJoin::Round,
            export: MeshExport::None,
            tab_width: 4,
            world_scale: None,
            text_shadow: None,
        }
    }
}

/// Text style of a segment.
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
pub struct SegmentStyle {
    pub font: Option<Arc<str>>,
    pub fill_color: Option<Srgba>,
    pub stroke_color: Option<Srgba>,
    pub fill: Option<bool>,
    pub stroke: Option<NonZeroU32>,
    pub weight: Option<Weight>,
    pub style: Option<Style>,
    pub underline: Option<bool>,
    pub strikethrough: Option<bool>,
    /// Can be referenced by [`GlyphMeta::MagicNumber`].
    pub magic_number: Option<f32>,
}

impl SegmentStyle {
    pub fn as_attr<'t>(&'t self, base: &'t Text3dStyling) -> Attrs<'t> {
        let family_name = self.font.as_ref().map(Arc::as_ref).unwrap_or(&base.font);
        let family = family(family_name);
        Attrs::new()
            .weight(self.weight.unwrap_or(base.weight).into())
            .style(self.style.unwrap_or(base.style).into())
            .family(family)
    }

    pub fn join(&self, other: Self) -> Self {
        SegmentStyle {
            font: other.font.or_else(|| self.font.clone()),
            fill_color: other.fill_color.or(self.fill_color),
            stroke_color: other.stroke_color.or(self.stroke_color),
            fill: other.fill.or(self.fill),
            stroke: other.stroke.or(self.stroke),
            weight: other.weight.or(self.weight),
            underline: other.underline.or(self.underline),
            strikethrough: other.strikethrough.or(self.strikethrough),
            style: other.style.or(self.style),
            magic_number: other.magic_number.or(self.magic_number),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphEntry {
    pub font: ID,
    pub glyph_id: GlyphTextureOf,
    pub join: StrokeJoin,
    pub size: FloatOrd,
    pub weight: Weight,
    pub stroke: Option<NonZeroU32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlyphTextureOf {
    Id(u16),
    UnderlineTexture,
    StrikethroughTexture,
}

impl From<u16> for GlyphTextureOf {
    fn from(id: u16) -> Self {
        GlyphTextureOf::Id(id)
    }
}
