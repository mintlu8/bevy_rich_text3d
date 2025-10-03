use bevy::ecs::{
    component::Component,
    entity::Entity,
    lifecycle::HookContext,
    world::{DeferredWorld, Mut},
};
#[cfg(feature = "reflect")]
use bevy::{ecs::reflect::ReflectComponent, reflect::Reflect};

use crate::{
    styling::SegmentStyle, Text3dBounds, Text3dDimensionOut, Text3dStyling, TextAtlasHandle,
};

/// A rich text component.
///
/// Requires [`Text3dStyling`], [`Text3dBounds`], [`TextAtlasHandle`], [`Text3dDimensionOut`].
#[derive(Debug, Component)]
#[require(Text3dDimensionOut, Text3dBounds, TextAtlasHandle, Text3dStyling)]
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
    String(String),
    Extract(Entity),
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
        .filter_map(|x| match &x.0 {
            Text3dSegment::String(_) => None,
            Text3dSegment::Extract(entity) => Some(*entity),
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
}
