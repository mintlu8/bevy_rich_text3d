use std::str::FromStr;

use bevy::ecs::{
    change_detection::DetectChanges,
    component::Component,
    entity::Entity,
    query::Without,
    system::Query,
    world::{EntityRef, Mut},
};
#[cfg(feature = "reflect")]
use bevy::prelude::{Reflect, ReflectComponent, ReflectDefault};

/// If alongside a [`FetchedTextSegment`], prevent [`Text3d`](crate::Text3d) from despawning it on remove.
#[derive(Debug, Component, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component, Default))]
pub struct SharedTextSegment;

/// A string segment on a component, as opposed to in a [`Text3d`](crate::Text3d).
///
/// By default [`Text3d`](crate::Text3d) removes all linked [`FetchedTextSegment`] on remove,
/// add [`SharedTextSegment`] to prevent this behavior.
#[derive(Debug, Component, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component, Default))]
pub struct FetchedTextSegment(pub String);

impl FetchedTextSegment {
    pub const EMPTY: Self = Self(String::new());

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Set and trigger change detection if a string like value is changed.
    pub fn set_if_changed(mut this: Mut<Self>, value: impl AsRef<str> + ToString) {
        if this.0 != value.as_ref() {
            this.0 = value.to_string()
        }
    }

    /// Set and trigger change detection if a parsable value is changed.
    pub fn write_if_changed<T: ToString + FromStr + Eq>(mut this: Mut<Self>, value: T) {
        if let Ok(val) = this.0.parse::<T>() {
            if val == value {
                return;
            }
        }
        this.0 = value.to_string()
    }
}

/// A component that fetches data as a string from the world.
#[derive(Component)]
#[require(FetchedTextSegment)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component, from_reflect = false))]
pub struct TextFetch {
    entity: Entity,
    #[cfg_attr(feature = "reflect", reflect(ignore))]
    fetch: Box<dyn FnMut(EntityRef) -> Option<String> + Send + Sync>,
}

impl TextFetch {
    /// Create a text fetcher that fetches a string from a single component if the component changes.
    pub fn fetch_component<C: Component>(
        entity: Entity,
        mut fetch: impl (FnMut(&C) -> String) + Send + Sync + 'static,
    ) -> Self {
        TextFetch {
            entity,
            fetch: Box::new(move |entity: EntityRef| {
                if let Some(component) = entity.get_ref::<C>() {
                    if component.is_changed() {
                        return Some(fetch(&component));
                    }
                }
                None
            }),
        }
    }

    /// Create a text fetcher that fetches from an [`EntityRef`].
    pub fn fetch_entity_ref(
        entity: Entity,
        fetch: impl (FnMut(EntityRef) -> Option<String>) + Send + Sync + 'static,
    ) -> Self {
        TextFetch {
            entity,
            fetch: Box::new(fetch),
        }
    }
}

/// Triggers the [`TextFetch`] component.
pub fn text_fetch_system(
    mut channels: Query<(&mut TextFetch, &mut FetchedTextSegment)>,
    other: Query<EntityRef, Without<TextFetch>>,
) {
    for (mut channel, mut text) in channels.iter_mut() {
        if let Ok(entity_ref) = other.get(channel.entity) {
            if let Some(output) = (channel.fetch)(entity_ref) {
                text.0 = output;
            }
        }
    }
}
