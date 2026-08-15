use crate::FetchedTextSegment;
use bevy::ecs::{
    change_detection::DetectChanges, component::Component, entity::Entity, query::Without,
    system::Query, world::EntityRef,
};
#[cfg(feature = "reflect")]
use bevy::prelude::{Reflect, ReflectComponent, ReflectDefault};

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
