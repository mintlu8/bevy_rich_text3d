use bevy::{
    ecs::component::Component,
    math::{IVec2, Vec2},
};
use cosmic_text::{Style as CosmicStyle, Weight as CosmicWeight};
use std::ops::{Deref, DerefMut};

#[cfg(feature = "reflect")]
use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{Reflect, ReflectDefault},
};

/// Horizontal align of text.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

impl TextAlign {
    pub fn as_fac(&self) -> f32 {
        match self {
            TextAlign::Left => 0.,
            TextAlign::Center => 0.5,
            TextAlign::Right => 1.0,
        }
    }
}

/// Determines the maximum width of rendered text, by default infinite.
#[derive(Debug, Component)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component, Default))]
pub struct Text3dBounds {
    pub width: f32,
}

impl Default for Text3dBounds {
    fn default() -> Self {
        Self { width: f32::MAX }
    }
}

/// Anchor of a text block, usually in `(-0.5, -0.5)..=(0.5, 0.5)`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
pub struct TextAnchor(pub Vec2);

impl Deref for TextAnchor {
    type Target = Vec2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TextAnchor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TextAnchor {
    pub const BOTTOM_LEFT: TextAnchor = TextAnchor::new(-0.5, -0.5);
    pub const BOTTOM_CENTER: TextAnchor = TextAnchor::new(0., -0.5);
    pub const BOTTOM_RIGHT: TextAnchor = TextAnchor::new(0.5, -0.5);

    pub const CENTER_LEFT: TextAnchor = TextAnchor::new(-0.5, 0.);
    pub const CENTER: TextAnchor = TextAnchor::new(0., 0.);
    pub const CENTER_RIGHT: TextAnchor = TextAnchor::new(0.5, 0.);

    pub const TOP_LEFT: TextAnchor = TextAnchor::new(-0.5, 0.5);
    pub const TOP_CENTER: TextAnchor = TextAnchor::new(0., 0.5);
    pub const TOP_RIGHT: TextAnchor = TextAnchor::new(0.5, 0.5);

    pub const fn new(x: f32, y: f32) -> Self {
        TextAnchor(Vec2::new(x, y))
    }
}

/// Size of the output mesh's `Aabb`.
#[derive(Debug, Component, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component))]
pub struct Text3dDimensionOut {
    /// Returns `aabb`'s x and y derived from font's line height.
    pub dimension: Vec2,
    pub(crate) atlas_dimension: IVec2,
}

/// Allows italic or oblique faces to be selected.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
pub enum Style {
    /// A face that is neither italic not obliqued.
    Normal,
    /// A form that is generally cursive in nature.
    Italic,
    /// A typically-sloped version of the regular face.
    Oblique,
}

impl Default for Style {
    #[inline]
    fn default() -> Style {
        Style::Normal
    }
}

impl From<Style> for CosmicStyle {
    fn from(val: Style) -> Self {
        match val {
            Style::Normal => CosmicStyle::Normal,
            Style::Italic => CosmicStyle::Italic,
            Style::Oblique => CosmicStyle::Oblique,
        }
    }
}

impl From<CosmicStyle> for Style {
    fn from(val: CosmicStyle) -> Self {
        match val {
            CosmicStyle::Normal => Style::Normal,
            CosmicStyle::Italic => Style::Italic,
            CosmicStyle::Oblique => Style::Oblique,
        }
    }
}

/// Specifies the weight of glyphs in the font, their degree of blackness or stroke thickness.
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
pub struct Weight(pub u16);

impl Default for Weight {
    #[inline]
    fn default() -> Weight {
        Weight::NORMAL
    }
}

impl Weight {
    /// Thin weight (100), the thinnest value.
    pub const THIN: Weight = Weight(CosmicWeight::THIN.0);
    /// Extra light weight (200).
    pub const EXTRA_LIGHT: Weight = Weight(CosmicWeight::EXTRA_LIGHT.0);
    /// Light weight (300).
    pub const LIGHT: Weight = Weight(CosmicWeight::LIGHT.0);
    /// Normal (400).
    pub const NORMAL: Weight = Weight(CosmicWeight::NORMAL.0);
    /// Medium weight (500, higher than normal).
    pub const MEDIUM: Weight = Weight(CosmicWeight::MEDIUM.0);
    /// Semibold weight (600).
    pub const SEMIBOLD: Weight = Weight(CosmicWeight::SEMIBOLD.0);
    /// Bold weight (700).
    pub const BOLD: Weight = Weight(CosmicWeight::BOLD.0);
    /// Extra-bold weight (800).
    pub const EXTRA_BOLD: Weight = Weight(CosmicWeight::EXTRA_BOLD.0);
    /// Black weight (900), the thickest value.
    pub const BLACK: Weight = Weight(CosmicWeight::BLACK.0);
}

impl From<Weight> for CosmicWeight {
    fn from(val: Weight) -> Self {
        CosmicWeight(val.0)
    }
}

impl From<CosmicWeight> for Weight {
    fn from(val: CosmicWeight) -> Self {
        Weight(val.0)
    }
}

/// Specifies how corners are drawn when a shape is stroked.
///
/// Join affects the four corners of a stroked rectangle, and the connected segments in a stroked path.
///
/// Usually [`StrokeJoin::Round`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
pub enum StrokeJoin {
    #[default]
    Round,
    Miter,
    Bevel,
}

impl From<StrokeJoin> for tiny_skia::LineJoin {
    fn from(val: StrokeJoin) -> Self {
        match val {
            StrokeJoin::Round => tiny_skia::LineJoin::Round,
            StrokeJoin::Bevel => tiny_skia::LineJoin::Bevel,
            StrokeJoin::Miter => tiny_skia::LineJoin::Miter,
        }
    }
}
