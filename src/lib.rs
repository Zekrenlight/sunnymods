//! SunnyMods 1.0.0 for Sunrise, by Zekrenlight (MIT).
//!
//! Takes over Sunrise's own Mods list (the title screen's Mods button and Options > Mods...):
//! each mod's real `icon.png`, its name, version, authors, licence, links and full
//! description, search (`@author`, `#id`), filter chips, sorting, the keys a mod adds, and a
//! Settings button for every mod with a settings page. Browse Mods, the mods folder and
//! Sunrise's own list stay one click away.
//!
//! This crate holds what needs no game types: the settings and the list's model (search,
//! filters, sorting). The screen itself is `src/native/ui.rs`.

pub mod config;
pub mod list;
#[cfg(test)]
mod icon_check;

pub use config::*;

use mod_api::{
    ClientHost, ClientTickPostEvent, Hooks, KeyRegistration, Registrar, RegistrationContext,
    SettingChangedEvent, SettingSpec,
};
use std::sync::atomic::{AtomicBool, Ordering};

/// Set by the Mods key, taken by the game half, which opens the list.
static OPEN_LIST: AtomicBool = AtomicBool::new(false);

/// Whether the list was asked for (the Mods key) since the last call.
pub fn take_open_list() -> bool {
    OPEN_LIST.swap(false, Ordering::Relaxed)
}

#[derive(Default)]
struct SunnyMods;

fn label(key: &str) -> String {
    format!("sunnymods.setting.{key}")
}

fn toggle(r: &mut dyn Registrar, key: &str, default: bool) {
    let l = label(key);
    let _ = r.register_setting(SettingSpec::toggle(key, l.clone(), default).tooltip(format!("{l}.tooltip")));
}

fn choice(r: &mut dyn Registrar, key: &str, options: &[&str], default: usize) {
    let l = label(key);
    let _ = r.register_setting(
        SettingSpec::choice(key, l.clone(), options.iter().copied(), default).tooltip(format!("{l}.tooltip")),
    );
}

fn color(r: &mut dyn Registrar, key: &str, default: u32) {
    let l = label(key);
    let _ = r.register_setting(SettingSpec::color(key, l.clone(), default).tooltip(format!("{l}.tooltip")));
}

/// Reads every setting into the copy the game half uses.
fn sync(host: &dyn ClientHost) {
    let d = Config::default();
    let b = |key: &str, default: bool| host.setting_bool(key).unwrap_or(default);
    let c = |key: &str| host.setting_choice(key).unwrap_or(0);
    set_config(Config {
        take_over: b(KEY_TAKE_OVER, d.take_over),
        sort: Sort::from_index(c(KEY_SORT)),
        rows: Rows::from_index(c(KEY_ROWS)),
        show_ids: b(KEY_IDS, d.show_ids),
        accent: host.setting_color(KEY_ACCENT).unwrap_or(d.accent) & 0xFF_FFFF,
    });
}

#[mod_api::sunrise_mod]
impl Hooks for SunnyMods {
    fn register(&self, r: &mut dyn Registrar, _cx: &RegistrationContext) {
        r.register_key_mapping(
            KeyRegistration::new("open", "key.keyboard.unknown")
                .label("sunnymods.key.open")
                .category("sunnymods"),
        );
        let d = Config::default();
        toggle(r, KEY_TAKE_OVER, d.take_over);
        choice(r, KEY_SORT, &Sort::OPTIONS, 0);
        choice(r, KEY_ROWS, &Rows::OPTIONS, 0);
        toggle(r, KEY_IDS, d.show_ids);
        color(r, KEY_ACCENT, d.accent);
    }

    fn setting_changed(&self, host: &mut dyn ClientHost, event: &mut SettingChangedEvent<'_>) {
        if event.mod_id == ID {
            sync(host);
        }
    }

    fn client_tick_post(&self, host: &mut dyn ClientHost, _event: &mut ClientTickPostEvent) {
        if !config_loaded() {
            sync(host);
        }
        if host.consume_click(ID, "open") {
            OPEN_LIST.store(true, Ordering::Relaxed);
        }
    }
}
