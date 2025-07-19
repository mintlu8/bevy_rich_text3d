use std::{
    collections::VecDeque,
    num::NonZero,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, MutexGuard},
};

use bevy::{
    asset::{AssetId, Assets},
    ecs::resource::Resource,
    image::Image,
};
use cosmic_text::{
    ttf_parser::Face, Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Style, Weight,
};

use crate::{
    render::{cache_glyph, CommandEncoder},
    StrokeJoins, Text3dPlugin, TextAtlas,
};

/// An [`Arc<Mutex>`] around [`cosmic_text::FontSystem`],
/// rendering fonts require exclusive access.
#[derive(Debug, Resource, Clone)]
pub struct TextRenderer(pub(crate) Arc<Mutex<TextRendererInner>>);

impl TextRenderer {
    pub fn new(font_system: FontSystem) -> Self {
        Self(Arc::new(Mutex::new(TextRendererInner {
            font_system,
            queue: VecDeque::new(),
        })))
    }

    // Methods uses `mut` to deter `Res` usage as that would block.

    /// Obtain the underlying [`FontSystem`].
    pub fn lock(&mut self) -> FontSystemGuard {
        FontSystemGuard(self.0.lock().unwrap())
    }

    /// Obtain the underlying [`FontSystem`] if not loading.
    pub fn try_lock(&mut self) -> Option<FontSystemGuard> {
        self.0.try_lock().ok().map(FontSystemGuard)
    }
}

/// Mutex guard over a [`FontSystem`].
pub struct FontSystemGuard<'t>(MutexGuard<'t, TextRendererInner>);

impl Deref for FontSystemGuard<'_> {
    type Target = FontSystem;

    fn deref(&self) -> &Self::Target {
        &self.0.font_system
    }
}

impl DerefMut for FontSystemGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.font_system
    }
}

#[derive(Debug)]
pub(crate) struct TextRendererInner {
    pub(crate) font_system: FontSystem,
    pub(crate) queue: VecDeque<(AssetId<TextAtlas>, TextAtlas, Image)>,
}

/// Style that only concerns drawing but not layout.
#[derive(Debug, Clone, Default)]
pub struct DrawStyle {
    pub family: Arc<str>,
    pub size: f32,
    pub stroke: Option<NonZero<u32>>,
    pub weight: Weight,
    pub style: Style,
}

pub(crate) fn family(name: &str) -> Family {
    match name {
        "" | "serif" => Family::Serif,
        "sans-serif" => Family::SansSerif,
        "monospace" => Family::Monospace,
        "cursive" => Family::Cursive,
        "fantasy" => Family::Fantasy,
        _ => Family::Name(name),
    }
}

impl DrawStyle {
    pub fn as_attrs(&self) -> Attrs {
        Attrs::new()
            .family(family(&self.family))
            .weight(self.weight)
            .style(self.style)
    }
}

/// A callback function that helps a loading screen keep track of progress.
///
/// If no callback is needed use `()`.
pub trait TextProgressReportCallback: Send + Sync + 'static {
    /// Called every time a glyph is drawn.
    fn glyph_drawn(&mut self) {}
    /// Called every time a style entry is drawn.
    fn style_drawn(&mut self) {}
    /// Called every time an atlas is drawn.
    fn atlas_drawn(&mut self) {}
}

impl TextProgressReportCallback for () {}

impl TextRenderer {
    /// Creates a function task that renders text to a [`TextAtlas`].
    ///
    /// This function should either be ran synchronously before app startup
    /// or be sent to another thread during a loading screen.
    ///
    /// While the task is running concurrently, the text rendering system
    /// will be paused and no new text will be drawn.
    ///
    /// The [`TextAtlas`] and [`Image`] will be REPLACED after the task finishes.
    /// You should not call `prepare_task` with the same atlas
    /// or image multiple times, or modify them concurrently in the `World`.
    pub fn prepare_task<S, I>(
        &self,
        settings: &Text3dPlugin,
        workload: impl IntoIterator<Item = (AssetId<TextAtlas>, TextAtlas, Image, I)>
            + Send
            + Sync
            + 'static,
        mut callback: impl TextProgressReportCallback,
    ) -> impl FnOnce() + Send + Sync + 'static
    where
        S: AsRef<str> + 'static,
        I: IntoIterator<Item = (S, DrawStyle)>,
    {
        let font_system = self.clone();
        let scale_factor = settings.scale_factor;
        move || {
            let mut guard = font_system.0.lock().unwrap();
            let TextRendererInner { font_system, queue } = guard.deref_mut();
            let mut tess_commands = CommandEncoder::default();
            for (id, mut atlas, mut image, workload) in workload {
                for (str, style) in workload {
                    let mut buffer = Buffer::new(font_system, Metrics::new(style.size, style.size));
                    buffer.set_text(
                        font_system,
                        str.as_ref(),
                        &style.as_attrs(),
                        Shaping::Advanced,
                    );
                    buffer.shape_until_scroll(font_system, false);
                    let stroke = style.stroke;
                    let weight = style.weight;
                    for run in buffer.layout_runs() {
                        for glyph in run.glyphs {
                            font_system.db().with_face_data(glyph.font_id, |file, _| {
                                let Ok(face) = Face::parse(file, 0) else {
                                    return;
                                };
                                cache_glyph(
                                    scale_factor,
                                    &mut atlas,
                                    &mut image,
                                    &mut tess_commands,
                                    glyph,
                                    stroke,
                                    StrokeJoins::default(),
                                    weight,
                                    face,
                                );
                            });
                            callback.glyph_drawn();
                        }
                    }
                    callback.style_drawn();
                }
                queue.push_back((id, atlas, image));
                callback.atlas_drawn();
            }
        }
    }

    /// Creates a function task that renders text to a [`TextAtlas`].
    ///
    /// This function prepare atlases by cloning the underlying images.
    /// See [`TextRenderer::prepare_task`] for details.
    pub fn prepare_images_cloned<S, I>(
        &self,
        settings: &Text3dPlugin,
        workload: impl IntoIterator<Item = (AssetId<TextAtlas>, I)> + Send + Sync + 'static,
        atlases: &mut Assets<TextAtlas>,
        images: &mut Assets<Image>,
        callback: impl TextProgressReportCallback,
    ) -> impl FnOnce() + Send + Sync + 'static
    where
        S: AsRef<str> + 'static,
        I: IntoIterator<Item = (S, DrawStyle)> + Send + Sync + 'static,
    {
        let workload: Vec<_> = workload
            .into_iter()
            .filter_map(|(id, iter)| {
                let atlas = atlases.get(id)?.clone();
                let image = images.get(atlas.image.id())?.clone();
                Some((id, atlas, image, iter))
            })
            .collect();
        self.prepare_task(settings, workload, callback)
    }

    /// Creates a function task that renders text to a [`TextAtlas`].
    ///
    /// This function prepare atlases by removing the underlying atlases and images
    /// and readd them after the task finishes.
    /// If spawned as a thread, the images will not be available until render finishes.
    /// See [`TextRenderer::prepare_task`] for details.
    pub fn prepare_images<S, I>(
        &self,
        settings: &Text3dPlugin,
        workload: impl IntoIterator<Item = (AssetId<TextAtlas>, I)> + Send + Sync + 'static,
        atlases: &mut Assets<TextAtlas>,
        images: &mut Assets<Image>,
        callback: impl TextProgressReportCallback,
    ) -> impl FnOnce() + Send + Sync + 'static
    where
        S: AsRef<str> + 'static,
        I: IntoIterator<Item = (S, DrawStyle)> + Send + Sync + 'static,
    {
        let workload: Vec<_> = workload
            .into_iter()
            .filter_map(|(id, iter)| {
                let atlas = atlases.remove(id)?.clone();
                let image = images.remove(atlas.image.id())?.clone();
                Some((id, atlas, image, iter))
            })
            .collect();
        self.prepare_task(settings, workload, callback)
    }
}
