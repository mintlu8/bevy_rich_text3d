use std::marker::PhantomData;

use bevy::{
    asset::{uuid::Uuid, Asset, Assets, Handle, RenderAssetUsages},
    ecs::component::Component,
    image::Image,
    log::info,
    math::{IRect, IVec2, Vec2},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use rustc_hash::FxHashMap;

use crate::styling::GlyphEntry;

#[cfg(feature = "reflect")]
use bevy::{ecs::reflect::ReflectComponent, reflect::Reflect};

/// Backing image handle and atlas of [`Text3d`](crate::Text3d).
#[derive(Debug, Clone, Default, Asset)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(not(feature = "reflect"), derive(bevy::reflect::TypePath))]
pub struct TextAtlas {
    pub(crate) image: Handle<Image>,
    #[cfg_attr(feature = "reflect", reflect(ignore))]
    pub(crate) glyphs: FxHashMap<GlyphEntry, (IRect, Vec2)>,
    pub(crate) pointer: IVec2,
    pub(crate) descent: usize,
}

const PADDING: usize = 2;

impl TextAtlas {
    /// The image used by [`TextAtlas::default()`].
    #[allow(deprecated)]
    pub const DEFAULT_IMAGE: Handle<Image> = Handle::Uuid(
        Uuid::from_u128(0x9a5c50eb057602509c7836bb327807e1),
        PhantomData,
    );

    /// Create a new empty [`TextAtlas`].
    ///
    /// The image can be created via [`TextAtlas::empty_image`].
    pub fn new(image: Handle<Image>) -> Self {
        Self {
            image,
            ..Default::default()
        }
    }

    /// Create an empty [`Image`] filled with transparent white `(255, 255, 255, 0)`.
    pub fn empty_image(width: usize, height: usize) -> Image {
        Image::new(
            Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            vec![[255, 255, 255, 0]; width * height].into_flattened(),
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::all(),
        )
    }

    /// Allocate space for a glyph, resizes if necessary but does not draw.
    pub fn allocate(
        &mut self,
        image: &mut Image,
        glyph: GlyphEntry,
        base: Vec2,
        width: usize,
        height: usize,
    ) -> IRect {
        if let Some((rect, _)) = self.glyphs.get(&glyph) {
            return *rect;
        }
        if self.pointer.x as usize + width + PADDING > image.width() as usize {
            self.pointer.x = 0;
            self.pointer.y += self.descent.max(height) as i32 + PADDING as i32;
            self.descent = 0;
        }
        if image.data.is_none() {
            return Default::default();
        }
        macro_rules! data {
            ($($tt:tt)*) => {
                image.data.as_mut().unwrap()[$($tt)*]
            };
        }
        self.descent = self.descent.max(height);
        if self.pointer.y as usize + self.descent + PADDING >= image.height() as usize {
            info!("Text atlas size expanded!");
            let old_dim = (image.width() * image.height()) as usize;
            image.resize(Extent3d {
                width: image.width(),
                height: image.height() * 2,
                depth_or_array_layers: 1,
            });
            for i in old_dim..old_dim * 2 {
                data![i * 4] = 255;
                data![i * 4 + 1] = 255;
                data![i * 4 + 2] = 255;
            }
        };
        let dimension = IVec2::new(width as i32, height as i32);

        let output = IRect {
            min: self.pointer,
            max: self.pointer + dimension,
        };

        self.glyphs.insert(glyph, (output, base));
        self.pointer.x += dimension.x + PADDING as i32;

        output
    }

    /// Clear all cached glyphs and repaint the image as transparent white.
    pub fn clear(&mut self, images: &mut Assets<Image>) {
        self.pointer = IVec2::ZERO;
        self.glyphs.clear();
        if let Some(img) = images.get_mut(self.image.id()) {
            for chunk in img.data.as_mut().unwrap().chunks_mut(4) {
                chunk[0] = 255;
                chunk[1] = 255;
                chunk[2] = 255;
                chunk[3] = 0;
            }
        }
    }
}

/// [`Component`] of a [`Handle<TextAtlas>`](TextAtlas), if left as default,
/// will use the shared [`TextAtlas::DEFAULT_IMAGE`] as
/// the underlying image.
#[derive(Debug, Clone, Component, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component))]
pub struct TextAtlasHandle(pub Handle<TextAtlas>);
