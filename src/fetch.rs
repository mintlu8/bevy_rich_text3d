use std::str::FromStr;

use bevy::ecs::{component::Component, world::Mut};
#[cfg(feature = "reflect")]
use bevy::prelude::{Reflect, ReflectComponent, ReflectDefault};

/// If alongside a [`FetchedTextSegment`] or ['FetchedCondition`], prevent [`Text3d`](crate::Text3d) from despawning it on remove.
#[derive(Debug, Component, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component, Default))]
pub struct SharedSegment;

/// A string segment on a component, as opposed to in a [`Text3d`](crate::Text3d).
///
/// By default [`Text3d`](crate::Text3d) removes all linked [`FetchedTextSegment`] on remove,
/// add [`SharedSegment`] to prevent this behavior.
#[derive(Debug, Component, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component, Default))]
pub struct FetchedText(pub String);

impl FetchedText {
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

/// A string segment on a component, as opposed to in a [`Text3d`](crate::Text3d).
///
/// By default [`Text3d`](crate::Text3d) removes all linked [`FetchedTextSegment`] on remove,
/// add [`SharedSegment`] to prevent this behavior.
#[derive(Debug, Component, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component, Default))]
pub struct FetchedCondition(pub bool);
