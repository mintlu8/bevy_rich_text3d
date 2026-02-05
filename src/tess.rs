use crate::{styling::GlyphEntry, TextAtlas};
use bevy::{
    image::Image,
    math::{Rect, Vec2},
};
use tiny_skia::{
    BlendMode, Color, ColorSpace, FillRule, LineCap, Paint, PathBuilder, PixmapMut,
    Rect as SkiaRect, Shader, Stroke, Transform,
};
use ttf_parser::OutlineBuilder;

#[derive(Debug, Default)]
pub(crate) struct PathEncoder {
    pub commands: PathBuilder,
}

impl OutlineBuilder for PathEncoder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.commands.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.commands.line_to(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.commands.quad_to(x1, y1, x, y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.commands.cubic_to(x1, y1, x2, y2, x, y);
    }

    fn close(&mut self) {
        self.commands.close();
    }
}

impl PathEncoder {
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn push_rect(&mut self, min_x: f32, min_y: f32, max_x: f32, max_y: f32) {
        if let Some(rect) = SkiaRect::from_ltrb(min_x, min_y, max_x, max_y) {
            self.commands.push_rect(rect);
        }
    }
}

impl PathEncoder {
    /// Returns a rectangle and an additional offset, keep in mind both has to be applied scale factor before usage.
    ///
    /// * `scale`: the value multiplied to the original glyph of size `units_per_em` (usually 1000 or so),
    ///   so size * scale_factor / units_per_em for regular usage
    pub fn tess_glyph(
        self,
        stroke: Option<f32>,
        scale: f32,
        atlas: &mut TextAtlas,
        image: &mut Image,
        entry: GlyphEntry,
    ) -> Option<(Rect, Vec2)> {
        let image_width = image.width();
        let image_height = image.height();
        let paint = Paint {
            shader: Shader::SolidColor(Color::WHITE),
            blend_mode: BlendMode::Source,
            anti_alias: true,
            colorspace: ColorSpace::Linear,
            force_hq_pipeline: false,
        };
        let path = self.commands.finish()?;
        let path = path.transform(Transform::from_scale(scale, scale))?;
        let boundary = path.compute_tight_bounds()?;
        if let Some(stroke) = stroke {
            let boundary = boundary.outset(stroke, stroke).unwrap();
            let w = boundary.width().ceil() as usize;
            let h = boundary.height().ceil() as usize;
            let base = Vec2::new(boundary.left(), boundary.top());
            let pixel_rect = atlas.cache(image, entry, base, w, h);
            let stroke = Stroke {
                // Different from the original zeno implementation.
                width: stroke * 2.0,
                miter_limit: 4.0,
                line_cap: LineCap::Round,
                line_join: entry.join.into(),
                dash: None,
            };
            let transform =
                Transform::from_translate(pixel_rect.min.x - base.x, pixel_rect.min.y - base.y);
            let bytes = image.data.as_mut()?;
            let mut pixmap = PixmapMut::from_bytes(bytes, image_width, image_height)?;
            pixmap.stroke_path(&path, &paint, &stroke, transform, None);
            Some((pixel_rect, base))
        } else {
            let w = boundary.width().ceil() as usize;
            let h = boundary.height().ceil() as usize;
            let base = Vec2::new(boundary.left(), boundary.top());
            let pixel_rect = atlas.cache(image, entry, base, w, h);
            let transform =
                Transform::from_translate(pixel_rect.min.x - base.x, pixel_rect.min.y - base.y);
            let bytes = image.data.as_mut()?;
            let mut pixmap = PixmapMut::from_bytes(bytes, image_width, image_height)?;
            pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
            Some((pixel_rect, base))
        }
    }
}
