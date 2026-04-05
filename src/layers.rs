use std::{
    num::NonZero,
    ops::{BitOr, BitOrAssign},
};

use bevy::{asset::AssetId, color::Srgba, image::Image, math::Vec2};

use crate::{
    export::TextMeshFaceCategory, line::LineMode, SegmentStyle, Text3dSegment, Text3dStyling,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Layer(u8);

impl BitOr for Layer {
    type Output = Layer;

    fn bitor(self, rhs: Self) -> Self::Output {
        Layer(self.0 | rhs.0)
    }
}

impl BitOrAssign for Layer {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[allow(non_upper_case_globals)]
impl Layer {
    pub const NoShadow: Layer = Layer(0x80);
    pub const Strikethrough: Layer = Layer(0x4);
    /// Depend on the offset, either fill or stroke.
    pub const Higher: Layer = Layer(0x2);
    pub const Underline: Layer = Layer(0x1);

    pub const None: Layer = Layer(0);
}

pub enum DrawType {
    Glyph(Option<NonZero<u32>>),
    Line(Option<NonZero<u32>>, LineMode),
    Image(AssetId<Image>),
}

pub struct DrawRequest {
    pub sort: Layer,
    pub request: DrawType,
    pub color: Srgba,
    pub offset: Vec2,
    pub category: TextMeshFaceCategory,
}

impl Text3dStyling {
    /// Note: Things drawn last gets rendered first.
    pub(crate) fn fill_draw_requests(
        &self,
        segment: &Text3dSegment,
        attrs: &SegmentStyle,
        requests: &mut Vec<DrawRequest>,
    ) {
        requests.clear();
        if let Text3dSegment::Image { image, .. } = segment {
            let color = attrs.fill_color.unwrap_or(self.color);
            requests.push(DrawRequest {
                sort: Layer::None,
                request: DrawType::Image(image.id()),
                color,
                offset: Vec2::ZERO,
                category: TextMeshFaceCategory::Image,
            });
            return;
        }
        #[allow(non_snake_case)]
        let FILL = if self.stroke_in_front {
            Layer::None
        } else {
            Layer::Higher
        };
        #[allow(non_snake_case)]
        let STROKE = if self.stroke_in_front {
            Layer::Higher
        } else {
            Layer::None
        };
        let fill = attrs.fill.unwrap_or(self.fill);
        let stroke = attrs.stroke.or(self.stroke);
        let fill_color = attrs.fill_color.unwrap_or(self.color);
        let stroke_color = attrs.stroke_color.unwrap_or(self.stroke_color);
        let fill_stroke: &[_] = match (fill, stroke) {
            (true, None) => &[(None, fill_color, FILL)],
            (true, Some(stroke)) => &[
                (None, fill_color, FILL),
                (Some(stroke), stroke_color, STROKE),
            ],
            (false, None) => &[],
            (false, Some(stroke)) => &[(Some(stroke), stroke_color, STROKE)],
        };
        let normal_shadow: &[_] = match self.text_shadow {
            Some((color, offset)) => &[
                (None, Vec2::ZERO, Layer::NoShadow),
                (Some(color), offset, Layer::None),
            ],
            None => &[(None, Vec2::ZERO, Layer::None)],
        };
        for (shadow_color, offset, shadow_layer) in normal_shadow.iter().copied() {
            for (stroke, color, regular_layer) in fill_stroke.iter().copied() {
                let category = if shadow_color.is_some() {
                    TextMeshFaceCategory::Shadow
                } else if stroke.is_some() {
                    TextMeshFaceCategory::Stroke
                } else {
                    TextMeshFaceCategory::Fill
                };
                requests.push(DrawRequest {
                    request: DrawType::Glyph(stroke),
                    color: shadow_color.unwrap_or(color),
                    offset,
                    sort: regular_layer | shadow_layer,
                    category,
                });
                if attrs.underline.is_some_and(|x| x) {
                    requests.push(DrawRequest {
                        request: DrawType::Line(stroke, LineMode::Underline),
                        color: shadow_color.unwrap_or(color),
                        offset,
                        sort: regular_layer | shadow_layer | Layer::Underline,
                        category,
                    });
                }
                if attrs.strikethrough.is_some_and(|x| x) {
                    requests.push(DrawRequest {
                        request: DrawType::Line(stroke, LineMode::Strikethrough),
                        color: shadow_color.unwrap_or(color),
                        offset,
                        sort: regular_layer | shadow_layer | Layer::Strikethrough,
                        category,
                    });
                }
            }
        }
    }
}
