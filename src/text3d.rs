use bevy::{
    asset::Handle,
    ecs::{
        component::Component,
        entity::Entity,
        lifecycle::HookContext,
        world::{DeferredWorld, Mut},
    },
    image::Image,
};
#[cfg(feature = "reflect")]
use bevy::{ecs::reflect::ReflectComponent, reflect::Reflect};

use crate::{
    styling::SegmentStyle, SharedSegment, Text3dBounds, Text3dDimensionOut, Text3dStyle,
    TextAtlasHandle,
};

/// A rich text component.
///
/// Requires [`Text3dStyle`], [`Text3dBounds`], [`TextAtlasHandle`], [`Text3dDimensionOut`].
#[derive(Debug, Component)]
#[require(Text3dDimensionOut, Text3dBounds, TextAtlasHandle, Text3dStyle)]
#[component(on_remove = text_3d_on_remove)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component))]
pub struct Text3d {
    pub segments: Vec<(Text3dSegment, SegmentStyle)>,
}

/// A string segment in [`Text3d`].
///
/// `Extract` reads data from an entity's [`FetchedTextSegment`](crate::FetchedTextSegment) component.
#[derive(Debug)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
pub enum Text3dSegment {
    /// A string segment.
    String(String),
    /// [`FetchedText`](crate::FetchedText) on an entity.
    Extract(Entity),
    /// Renders an image or emoji inside the text.
    ///
    /// # Limitations
    ///
    /// The image will be copied into the text atlas as is regardless of font size.
    /// The image is only loaded once and cannot change.
    Image {
        /// Image asset.
        image: Handle<Image>,
        /// Represents width / em, usually `1.0` for squares.
        width: f32,
    },
    /// [`FetchedCondition`](crate::FetchedCondition) on an entity.
    SkipIf {
        condition: Entity,
        skip_if: bool,
        offset: usize,
    },
}

impl Text3dSegment {
    pub fn get_external_segment(&self) -> Option<Entity> {
        match self {
            Text3dSegment::Extract(entity) => Some(*entity),
            Text3dSegment::SkipIf { condition, .. } => Some(*condition),
            _ => None,
        }
    }
}

fn text_3d_on_remove(mut world: DeferredWorld, cx: HookContext) {
    let Ok(entity) = world.get_entity(cx.entity) else {
        return;
    };
    let Some(text) = entity.get::<Text3d>() else {
        return;
    };
    let to_be_dropped: Vec<_> = text
        .segments
        .iter()
        .filter_map(|x| x.0.get_external_segment())
        .filter(|entity| {
            world
                .get_entity(*entity)
                .is_ok_and(|e| !e.contains::<SharedSegment>())
        })
        .collect();
    let mut commands = world.commands();
    for entity in to_be_dropped {
        commands.entity(entity).try_despawn();
    }
}

impl Text3d {
    /// Create a simple string without parsing.
    ///
    /// To parse rich text, see [`Text3d::parse`].
    pub fn new(s: impl ToString) -> Self {
        let string = s.to_string();
        Self {
            segments: vec![(Text3dSegment::String(string), Default::default())],
        }
    }

    /// Create a string from a [`FetchedTextSegment`](crate::FetchedTextSegment) on an entity.
    pub fn from_extract(entity: Entity) -> Self {
        Self {
            segments: vec![(Text3dSegment::Extract(entity), Default::default())],
        }
    }

    /// If only contains an owned segment, return that segment as a `&str`.
    pub fn get_single(&self) -> Option<&str> {
        if self.segments.len() != 1 {
            None
        } else if let Some((Text3dSegment::String(s), _)) = self.segments.first() {
            Some(s)
        } else {
            None
        }
    }

    /// If only contains an owned segment, return that segment mutably.
    pub fn get_single_mut(&mut self) -> Option<&mut String> {
        if self.segments.len() != 1 {
            None
        } else if let Some((Text3dSegment::String(s), _)) = self.segments.get_mut(0) {
            Some(s)
        } else {
            None
        }
    }

    /// If only contains an owned segment, return that segment mutably,
    /// without triggering change detection.
    pub fn map_single_mut<'a>(this: &'a mut Mut<Self>) -> Option<Mut<'a, String>> {
        this.reborrow().filter_map_unchanged(Self::get_single_mut)
    }

    /// Obtain a segment from index.
    pub fn get_segment_mut(&mut self, index: usize) -> Option<&mut String> {
        if let Some((Text3dSegment::String(s), _)) = self.segments.get_mut(index) {
            Some(s)
        } else {
            None
        }
    }

    /// Obtain a segment from index without triggering change detection.
    pub fn map_segment_mut<'a>(this: &'a mut Mut<Self>, index: usize) -> Option<Mut<'a, String>> {
        this.reborrow()
            .filter_map_unchanged(|v| Self::get_segment_mut(v, index))
    }
}
