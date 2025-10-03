//! This is used to circumvent bevy's long running issue of image change does not reflect onto materials.
//! This module will likely be removed in the future.
//!
//! Any dynamic text may want to register here.
//!

use std::marker::PhantomData;

#[cfg(feature = "3d")]
use bevy::pbr::{Material, MeshMaterial3d};
#[cfg(feature = "2d")]
use bevy::sprite_render::{Material2d, MeshMaterial2d};
use bevy::{
    app::{Plugin, PostUpdate},
    asset::Assets,
    ecs::{
        change_detection::ResMut,
        query::Changed,
        schedule::{IntoScheduleConfigs, SystemSet},
        system::Query,
    },
};

use crate::Text3dDimensionOut;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct TouchMaterialSet;

macro_rules! impl_mat {
    ($name: ident, $ty:ident, $comp: ident, $f:ident) => {
        /// This plugin must be added if you want text changes to affect the material, this works by
        /// mutably dereferencing the material to signal a change.
        ///
        /// Currently there is a bug/issue in bevy that prevents image change from updating material,
        /// this will likely be removed in the future if the issue gets resolved.
        pub struct $name<T: $ty>(PhantomData<T>);

        impl<T: $ty> Default for $name<T> {
            fn default() -> Self {
                Self(PhantomData)
            }
        }

        fn $f<T: $ty>(
            mut materials: ResMut<Assets<T>>,
            query: Query<&$comp<T>, Changed<Text3dDimensionOut>>,
        ) {
            for handle in &query {
                let _ = materials.get_mut(handle.0.id());
            }
        }

        impl<T: $ty> Plugin for $name<T> {
            fn build(&self, app: &mut bevy::app::App) {
                app.add_systems(PostUpdate, $f::<T>.in_set(TouchMaterialSet));
            }
        }
    };
}

#[cfg(feature = "2d")]
impl_mat!(
    TouchTextMaterial2dPlugin,
    Material2d,
    MeshMaterial2d,
    touch_text_material2d
);

#[cfg(feature = "3d")]
impl_mat!(
    TouchTextMaterial3dPlugin,
    Material,
    MeshMaterial3d,
    touch_text_material
);
