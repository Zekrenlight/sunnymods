//! SunnyMods' settings: their keys, defaults and the copy the game half reads.

use std::sync::Mutex;

pub const ID: &str = "sunnymods";

pub const KEY_TAKE_OVER: &str = "take_over";
pub const KEY_SORT: &str = "sort";
pub const KEY_ROWS: &str = "rows";
pub const KEY_IDS: &str = "show_ids";
pub const KEY_ACCENT: &str = "accent";

/// How the list is ordered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sort {
    NameAz,
    NameZa,
    Author,
    SettingsFirst,
}

impl Sort {
    pub const OPTIONS: [&'static str; 4] = [
        "sunnymods.sort.name_az",
        "sunnymods.sort.name_za",
        "sunnymods.sort.author",
        "sunnymods.sort.settings_first",
    ];
    pub const ALL: [Sort; 4] = [Sort::NameAz, Sort::NameZa, Sort::Author, Sort::SettingsFirst];
    pub fn from_index(i: u32) -> Self {
        match i {
            1 => Self::NameZa,
            2 => Self::Author,
            3 => Self::SettingsFirst,
            _ => Self::NameAz,
        }
    }
    pub fn index(self) -> u32 {
        match self {
            Self::NameAz => 0,
            Self::NameZa => 1,
            Self::Author => 2,
            Self::SettingsFirst => 3,
        }
    }
    /// The short label on the sort button.
    pub fn short(self) -> &'static str {
        match self {
            Self::NameAz => "A-Z",
            Self::NameZa => "Z-A",
            Self::Author => "By",
            Self::SettingsFirst => "Set",
        }
    }
    pub fn next(self) -> Self {
        Self::from_index((self.index() + 1) % 4)
    }
}

/// How tall the list's rows are.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rows {
    Comfortable,
    Compact,
}

impl Rows {
    pub const OPTIONS: [&'static str; 2] = ["sunnymods.rows.comfortable", "sunnymods.rows.compact"];
    pub fn from_index(i: u32) -> Self {
        match i {
            1 => Self::Compact,
            _ => Self::Comfortable,
        }
    }
    /// The icon's side in GUI pixels.
    pub fn icon(self) -> i32 {
        match self {
            Self::Comfortable => 32,
            Self::Compact => 20,
        }
    }
    /// A row's height.
    pub fn height(self) -> i32 {
        self.icon() + 6
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    /// SunnyMods shows the Mods list (title screen and Options > Mods...).
    pub take_over: bool,
    pub sort: Sort,
    pub rows: Rows,
    /// Each row shows the mod's id under its name.
    pub show_ids: bool,
    /// The accent colour, `0xRRGGBB`.
    pub accent: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            take_over: true,
            sort: Sort::NameAz,
            rows: Rows::Comfortable,
            show_ids: false,
            accent: 0xF9801D,
        }
    }
}

static CONFIG: Mutex<Option<Config>> = Mutex::new(None);

pub fn set_config(config: Config) {
    *CONFIG.lock().unwrap_or_else(|e| e.into_inner()) = Some(config);
}
pub fn config() -> Config {
    CONFIG.lock().unwrap_or_else(|e| e.into_inner()).unwrap_or_default()
}
pub fn config_loaded() -> bool {
    CONFIG.lock().unwrap_or_else(|e| e.into_inner()).is_some()
}
