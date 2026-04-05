use std::sync::{Arc, OnceLock};

use crate::{emoji::create_emoji_font, LoadFonts, Text3dPlugin, TextRenderer};
use bevy::{
    ecs::resource::Resource,
    ecs::system::{Commands, ResMut},
    log::error,
};
use cosmic_text::fontdb::Database;

#[derive(Debug, Resource)]
pub struct LoadCosmicFonts(pub(crate) Arc<OnceLock<TextRenderer>>);

impl Text3dPlugin {
    pub fn load_fonts_blocking(&self, fonts: LoadFonts) -> TextRenderer {
        let empty = Database::new();
        let locale = self
            .locale
            .clone()
            .or_else(sys_locale::get_locale)
            .unwrap_or_else(|| "en-US".to_string());
        let mut system = cosmic_text::FontSystem::new_with_locale_and_db(locale, empty);
        if self.load_system_fonts {
            system.db_mut().load_system_fonts();
        }
        for path in &fonts.font_paths {
            if let Err(err) = system.db_mut().load_font_file(path) {
                error!("Error loading font {path}: {err}.");
            };
        }
        for path in fonts.font_directories {
            system.db_mut().load_fonts_dir(path);
        }
        for data in fonts.font_embedded {
            system.db_mut().load_font_data(data.to_vec());
        }
        if !self.serif_family.is_empty() {
            system.db_mut().set_serif_family(&self.serif_family);
        }
        if !self.sans_serif_family.is_empty() {
            system
                .db_mut()
                .set_sans_serif_family(&self.sans_serif_family);
        }
        if !self.cursive_family.is_empty() {
            system.db_mut().set_cursive_family(&self.cursive_family);
        }
        if !self.monospace_family.is_empty() {
            system.db_mut().set_monospace_family(&self.monospace_family);
        }
        if !self.fantasy_family.is_empty() {
            system.db_mut().set_fantasy_family(&self.fantasy_family);
        }
        system.db_mut().load_font_data(create_emoji_font(
            &self.placeholder_family,
            &self.placeholder_glyph_widths,
            self.placeholder_glyph_origin,
        ));
        TextRenderer::new(system)
    }

    pub fn load_fonts_concurrent(&self, fonts: LoadFonts) -> LoadCosmicFonts {
        let locale = self
            .locale
            .clone()
            .or_else(sys_locale::get_locale)
            .unwrap_or_else(|| "en-US".to_string());

        let sender = Arc::new(OnceLock::new());
        let receiver = sender.clone();

        let settings = self.clone();

        std::thread::spawn(move || {
            let empty = cosmic_text::fontdb::Database::new();
            let mut system = cosmic_text::FontSystem::new_with_locale_and_db(locale, empty);
            if settings.load_system_fonts {
                system.db_mut().load_system_fonts();
            }
            for path in fonts.font_paths {
                if let Err(err) = system.db_mut().load_font_file(&path) {
                    error!("Error loading font {path}: {err}.");
                };
            }
            for path in fonts.font_directories {
                system.db_mut().load_fonts_dir(path);
            }
            for data in fonts.font_embedded {
                system.db_mut().load_font_data(data.to_vec());
            }
            if !settings.serif_family.is_empty() {
                system.db_mut().set_serif_family(&settings.serif_family);
            }
            if !settings.sans_serif_family.is_empty() {
                system
                    .db_mut()
                    .set_sans_serif_family(&settings.sans_serif_family);
            }
            if !settings.cursive_family.is_empty() {
                system.db_mut().set_cursive_family(&settings.cursive_family);
            }
            if !settings.monospace_family.is_empty() {
                system
                    .db_mut()
                    .set_monospace_family(&settings.monospace_family);
            }
            if !settings.fantasy_family.is_empty() {
                system.db_mut().set_fantasy_family(&settings.fantasy_family);
            }
            system.db_mut().load_font_data(create_emoji_font(
                &settings.placeholder_family,
                &settings.placeholder_glyph_widths,
                settings.placeholder_glyph_origin,
            ));
            sender.set(TextRenderer::new(system))
        });
        LoadCosmicFonts(receiver)
    }
}

pub fn load_cosmic_fonts_system(mut commands: Commands, mut load: ResMut<LoadCosmicFonts>) {
    if let Some(system) = Arc::get_mut(&mut load.0) {
        if let Some(system) = system.take() {
            commands.insert_resource(system);
            commands.remove_resource::<LoadCosmicFonts>();
        }
    }
}
