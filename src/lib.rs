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
    ClientHost, ClientTickPostEvent, Host, Hooks, KeyRegistration, Registrar, RegistrationContext,
    SettingChangedEvent, SettingKey, SettingSpec,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

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

fn toggle(r: &mut dyn Registrar, key: &str, default: bool) -> Option<SettingKey> {
    let l = label(key);
    r.register_setting(SettingSpec::toggle(key, l.clone(), default).tooltip(format!("{l}.tooltip"))).ok()
}

fn choice(r: &mut dyn Registrar, key: &str, options: &[&str], default: usize) -> Option<SettingKey> {
    let l = label(key);
    r.register_setting(
        SettingSpec::choice(key, l.clone(), options.iter().copied(), default).tooltip(format!("{l}.tooltip")),
    ).ok()
}

fn color(r: &mut dyn Registrar, key: &str, default: u32) -> Option<SettingKey> {
    let l = label(key);
    r.register_setting(SettingSpec::color(key, l.clone(), default).tooltip(format!("{l}.tooltip"))).ok()
}

struct SettingKeys {
    take_over: Option<SettingKey>,
    sort: Option<SettingKey>,
    rows: Option<SettingKey>,
    ids: Option<SettingKey>,
    accent: Option<SettingKey>,
}

static SETTING_KEYS: OnceLock<SettingKeys> = OnceLock::new();

fn bool_of(host: &dyn Host, key: Option<SettingKey>, default: bool) -> bool {
    key.and_then(|k| host.setting_bool(k).ok()).unwrap_or(default)
}
fn choice_of(host: &dyn Host, key: Option<SettingKey>, default: u32) -> u32 {
    key.and_then(|k| host.setting_choice(k).ok()).unwrap_or(default)
}

/// Reads every setting into the copy the game half uses.
fn sync(host: &dyn Host) {
    let d = Config::default();
    let Some(k) = SETTING_KEYS.get() else { return };
    set_config(Config {
        take_over: bool_of(host, k.take_over, d.take_over),
        sort: Sort::from_index(choice_of(host, k.sort, 0)),
        rows: Rows::from_index(choice_of(host, k.rows, 0)),
        show_ids: bool_of(host, k.ids, d.show_ids),
        accent: k.accent.and_then(|a| host.setting_color(a).ok()).unwrap_or(d.accent) & 0xFF_FFFF,
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
        let keys = SettingKeys {
            take_over: toggle(r, KEY_TAKE_OVER, d.take_over),
            sort: choice(r, KEY_SORT, &Sort::OPTIONS, 0),
            rows: choice(r, KEY_ROWS, &Rows::OPTIONS, 0),
            ids: toggle(r, KEY_IDS, d.show_ids),
            accent: color(r, KEY_ACCENT, d.accent),
        };
        let _ = SETTING_KEYS.set(keys);
    }

    fn setting_changed(&self, host: &dyn Host, event: &mut SettingChangedEvent<'_>) {
        if event.mod_id == ID {
            sync(host);
        }
    }

    fn client_tick_post(&self, host: &mut dyn ClientHost, _event: &mut ClientTickPostEvent) {
        if !config_loaded() {
            sync(host);
        }
        if host.mod_key_mapping(ID, "open").and_then(|m| m.consume_click(host)).unwrap_or(false) {
            OPEN_LIST.store(true, Ordering::Relaxed);
        }
    }
}
