use std::{borrow::Borrow, ops::Deref, str::FromStr};

use bevy::ecs::{component::Component, world::Mut};
#[cfg(feature = "reflect")]
use bevy::prelude::{Reflect, ReflectComponent, ReflectDefault};

use crate::Text3dSegment;

/// If alongside a [`FetchedText`] or [`FetchedCondition`], prevent [`Text3d`](crate::Text3d) from despawning the entity on remove.
#[derive(Debug, Clone, Copy, Component, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component, Default))]
pub struct SharedSegment;

/// A string segment on an [`Entity`](bevy::ecs::entity::Entity) that can be referenced by a [`Text3d`](crate::Text3d).
///
/// By default [`Text3d`](crate::Text3d) removes all linked [`FetchedText`] on remove,
/// add [`SharedSegment`] to prevent this behavior.
///
/// # Change Detection
///
/// As long as change detection is triggered on this component, associated text will be rebuilt.
/// Users should take care to not mutably dereference this component if no changes are needed,
/// functions like [`FetchedText::write_if_changed`] or [`FetchedText::set_if_changed`] can help
/// in this regard.
#[derive(Debug, Clone, Component, Default)]
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

impl Text3dSegment {
    /// Set and trigger change detection if a string like value is changed.
    pub fn set_if_changed(mut this: Mut<String>, value: impl AsRef<str> + ToString) {
        if *this != value.as_ref() {
            *this = value.to_string()
        }
    }

    /// Set and trigger change detection if a parsable value is changed.
    pub fn write_if_changed<T: ToString + FromStr + Eq>(mut this: Mut<String>, value: T) {
        if let Ok(val) = this.parse::<T>() {
            if val == value {
                return;
            }
        }
        *this = value.to_string()
    }
}

impl AsRef<str> for FetchedText {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for FetchedText {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Deref for FetchedText {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

/// A boolean condition on an [`Entity`](bevy::ecs::entity::Entity) that can be referenced by a [`Text3d`](crate::Text3d).
///
/// By default [`Text3d`](crate::Text3d) removes all linked [`FetchedCondition`] on remove,
/// add [`SharedSegment`] to prevent this behavior.
///
/// # Change Detection
///
/// As long as change detection is triggered on this component, associated text will be rebuilt.
#[derive(Debug, Clone, Component, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component, Default))]
pub struct FetchedCondition(pub bool);

impl FetchedCondition {
    pub const fn get(&self) -> bool {
        self.0
    }

    /// Set and trigger change detection if value is changed.
    pub fn set_if_changed(mut this: Mut<Self>, value: bool) {
        if this.0 != value {
            this.0 = value
        }
    }
}
