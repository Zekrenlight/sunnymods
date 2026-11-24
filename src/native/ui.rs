// SunnyMods in the game (not in the Cargo library): the Mods list that takes Sunrise's Mods
// page (the title screen's Mods button and Options > Mods...), drawn and driven here.
use super::Game;
use mod_sunnymods::config::{self, Rows, Sort};
use mod_sunnymods::list::{self, Card, Filter, KeyInfo, Status};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use sunrise_client::Result;
use sunrise_client::hud::HudBatch;
use sunrise_client::options_ui::{Event, Page};
use sunrise_client::screen_draw::{self as d, Draw, Measure, TextField};
use sunrise_client::text::Text;
use winit::keyboard::KeyCode;

const PAD: i32 = 8;
const HEADER: i32 = 30;
const FOOTER: i32 = 30;
const CONTROL: i32 = 18;
const CHIP_H: i32 = 12;
const BUTTON_H: i32 = 20;
const BIG_ICON: i32 = 64;
const WHEEL: f64 = 20.0;
/// The atlas has this many icon slots a frame; rows past it would draw no icon.
const ICON_SLOTS: usize = 60;

// ARGB colours.
// The shared Sunrise dark palette, with the live accent (default Sunrise orange #F9801D).
const WHITE: u32 = 0xFFF6_EEDF;
const TEXT: u32 = 0xFFE8_DCC8;
const SOFT: u32 = 0xFFB9_B2A6;
const DIM: u32 = 0xFF8B_93A7;
const SHADE: u32 = 0xA008_1014;
const PANEL: u32 = 0xFF0F_1118;
const PANEL_EDGE: u32 = 0xFF1E_2330;
const CHIP: u32 = 0xFF1E_2330;
const CHIP_HOVER: u32 = 0xFF2A_3142;
const ROW_HOVER: u32 = 0x14FF_FFFF;
const GREEN: u32 = 0xFF5C_D65C;
const RED: u32 = 0xFFFF_5C5C;
const GOLD: u32 = 0xFFFF_C04A;
const PURPLE: u32 = 0xFFC0_8CFF;
const TRACK: u32 = 0xFF0A_0C11;
const THUMB: u32 = 0xFF2E_3546;
const THUMB_HELD: u32 = 0xFF8B_93A7;

fn argb(c: u32) -> i32 {
    c as i32
}
/// An accent `0xRRGGBB` as opaque ARGB.
fn opaque(rgb: u32) -> u32 {
    0xFF00_0000 | (rgb & 0xFF_FFFF)
}
/// The accent darkened into the panel: the selected row's and chip's fill.
fn tab_on(accent: u32) -> u32 {
    let ch = |s: u32| {
        let a = ((accent >> s) & 0xFF) as f32;
        let b = ((PANEL >> s) & 0xFF) as f32;
        ((a * 0.42 + b * 0.58).round() as u32).min(255) << s
    };
    0xFF00_0000 | ch(16) | ch(8) | ch(0)
}
fn tr(key: &str, english: &str) -> Text {
    Text::translate_with_fallback(key, english, &[])
}
fn inside(r: [i32; 4], m: (i32, i32)) -> bool {
    d::inside(m, r[0], r[1], r[2], r[3])
}

/// Which scrollbar the pointer holds.
#[derive(Clone, Copy, PartialEq)]
enum Bar {
    List,
    Panel,
}

/// The screen's places at one GUI size.
struct Geo {
    search: [i32; 4],
    sort: [i32; 4],
    chips: Vec<(Filter, [i32; 4])>,
    /// The list's rows.
    list: [i32; 4],
    /// The detail panel, its scrolled contents and its buttons.
    panel: [i32; 4],
    body: [i32; 4],
    buttons: [[i32; 4]; 3],
    footer: [[i32; 4]; 4],
}

fn geo(m: &Measure, w: i32, h: i32) -> Geo {
    let lw = (((w - 3 * PAD) as f32) * 0.42) as i32;
    let lw = lw.clamp(140, 240).min(w - 3 * PAD - 120).max(100);
    let lx = PAD;
    let top = HEADER;
    let bottom = h - FOOTER - 4;
    let search = [lx, top, lw - 34, CONTROL];
    let sort = [lx + lw - 32, top, 32, CONTROL];
    let mut chips = Vec::new();
    let mut cx = lx;
    for f in Filter::ALL {
        let cw = m.width(&tr(f.lang(), f.label())) + 8;
        chips.push((f, [cx, top + CONTROL + 4, cw, CHIP_H]));
        cx += cw + 3;
    }
    let list_top = top + CONTROL + 4 + CHIP_H + 4;
    let list = [lx, list_top, lw, (bottom - list_top).max(20)];
    let px = lx + lw + PAD;
    let panel = [px, top, (w - PAD - px).max(60), (bottom - top).max(40)];
    let by = panel[1] + panel[3] - BUTTON_H - 4;
    let bw = ((panel[2] - 8 - 8) / 3).max(20);
    let buttons = [0, 1, 2].map(|i| [panel[0] + 4 + i * (bw + 4), by, bw, BUTTON_H]);
    let body = [panel[0], panel[1], panel[2], (by - 4 - panel[1]).max(10)];
    let fw = ((w - 2 * PAD - 3 * 4) / 4).clamp(40, 110);
    let total = fw * 4 + 3 * 4;
    let fx = (w - total) / 2;
    let fy = h - FOOTER + 4;
    let footer = [0, 1, 2, 3].map(|i| [fx + i * (fw + 4), fy, fw, BUTTON_H]);
    Geo { search, sort, chips, list, panel, body, buttons, footer }
}

struct State {
    cards: Vec<Card>,
    /// Decoded icons by texture key (`sunnymods/icon/<id>`), 128x128 RGBA.
    icons: HashMap<String, Arc<[u8]>>,
    /// Mods whose package was read and has no icon.
    no_icon: std::collections::HashSet<String>,
    /// What the cards were made from, to make them again when it changes.
    signature: u64,
    search: TextField,
    focused_at: Instant,
    filter: Filter,
    sort: Sort,
    shown: Vec<usize>,
    /// The selected card's id.
    selected: Option<String>,
    list_scroll: f64,
    panel_scroll: f64,
    panel_content: i32,
    held: Option<(Bar, f64)>,
    /// Sunrise's own list was asked for: it shows until the Mods pages are left.
    classic: bool,
    /// The Mods pages showed last frame.
    on_mods: bool,
    started: Instant,
}

impl Default for State {
    fn default() -> Self {
        let mut search = TextField::new(0, 0, 100, CONTROL);
        search.max_length = 64;
        search.hint = Some(tr("sunnymods.search", "Search mods, @author or #id").with_color(0x808080));
        Self {
            cards: Vec::new(),
            icons: HashMap::new(),
            no_icon: Default::default(),
            signature: 0,
            search,
            focused_at: Instant::now(),
            filter: Filter::All,
            sort: Sort::NameAz,
            shown: Vec::new(),
            selected: None,
            list_scroll: 0.0,
            panel_scroll: 0.0,
            panel_content: 0,
            held: None,
            classic: false,
            on_mods: false,
            started: Instant::now(),
        }
    }
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> Option<R> {
    STATE.with(|s| s.try_borrow_mut().ok().map(|mut s| f(&mut s)))
}

/// SunnyMods shows on this frame: the Options screen is on its Mods page, the mod is on and
/// Sunrise's own list wasn't asked for.
pub(super) fn active(game: &Game<'_>) -> bool {
    game.screen == super::Screen::Options
        && game.ui.options.page() == Page::Mods
        && config::config().take_over
        && !with(|s| s.classic).unwrap_or(false)
}

/// Once a frame: the Mods key, and forgetting Sunrise's list once its pages are left.
pub(super) fn frame(game: &mut Game<'_>) {
    let options = game.screen == super::Screen::Options;
    let page = game.ui.options.page();
    let on_mods = options && matches!(page, Page::Mods | Page::NativeMod(_) | Page::ModBrowser);
    with(|s| {
        if !on_mods {
            s.classic = false;
            s.held = None;
        }
        if !(options && page == Page::Mods) {
            s.search.focused = false;
        }
        if on_mods && !s.on_mods {
            // Opened afresh: the sort the settings ask for, the top of the list.
            s.sort = config::config().sort;
            s.filter = Filter::All;
            s.search.value.clear();
            s.search.cursor = 0;
            s.search.highlight = 0;
            s.search.display = 0;
            s.list_scroll = 0.0;
            s.panel_scroll = 0.0;
            s.signature = 0;
        }
        s.on_mods = on_mods;
    });
    if mod_sunnymods::take_open_list() && game.screen == super::Screen::Playing {
        game.open_options_at(false, Page::Mods);
        // Opened from play, the pointer is still the game's: it is let go, and leaving goes
        // back to the full pause menu.
        game.mining = None;
        game.pause_menu = true;
        game.capture(false);
    }
}

fn hash_str(h: &mut u64, s: &str) {
    for b in s.bytes() {
        *h = (*h ^ u64::from(b)).wrapping_mul(0x0100_0000_01B3);
    }
    *h = (*h ^ 0xFF).wrapping_mul(0x0100_0000_01B3);
}

/// What the installed list looks like now.
fn signature(game: &Game<'_>) -> u64 {
    let mods = &game.settings.mods;
    let mut h = 0xCBF2_9CE4_8422_2325u64;
    for (i, m) in mods.installed.iter().enumerate() {
        hash_str(&mut h, &m.id);
        hash_str(&mut h, &m.title);
        hash_str(&mut h, m.problem.as_deref().unwrap_or(""));
        hash_str(&mut h, if mods.is_on(i) { "on" } else { "off" });
        if let Some(page) = &m.native {
            hash_str(&mut h, &page.settings.len().to_string());
            for k in &page.mappings {
                let bound = mods.keys.get(&m.id).and_then(|keys| keys.get(&k.id)).unwrap_or(&k.key);
                hash_str(&mut h, bound);
            }
        }
        if let Some(info) = mods.info.get(&m.id) {
            hash_str(&mut h, &info.version);
            hash_str(&mut h, &info.icon.as_ref().map_or(0, |i| i.len()).to_string());
        }
    }
    h
}

/// Makes the cards again from the installed mods, when they changed.
fn refresh(game: &Game<'_>, s: &mut State) {
    let sig = signature(game);
    if sig == s.signature && !s.cards.is_empty() {
        return;
    }
    s.signature = sig;
    let mods = &game.settings.mods;
    let mut cards: Vec<Card> = Vec::new();
    let mut icons = HashMap::new();
    for (index, m) in mods.installed.iter().enumerate() {
        // Only mods: never the game or Sunrise itself, should the list ever carry them.
        if is_base_game(&m.id) {
            continue;
        }
        let info = mods.info.get(&m.id);
        let key = icon_key(&m.id);
        let icon = match s.icons.remove(&key) {
            Some(icon) => Some(icon),
            None => info
                .and_then(|i| i.icon.as_deref())
                .and_then(sunrise_client::pack_screen::load_icon)
                .or_else(|| package_icon(&m.id, &mut s.no_icon)),
        };
        let has_icon = icon.is_some();
        if let Some(icon) = icon {
            icons.insert(key, icon);
        }
        let status = if let Some(problem) = &m.problem {
            Status::Failed(problem.clone())
        } else if !mods.is_on(index) {
            Status::Off
        } else {
            Status::Running
        };
        let keys = m.native.as_ref().map_or_else(Vec::new, |page| {
            page.mappings
                .iter()
                .map(|k| KeyInfo {
                    label: k.label.clone(),
                    key: mods
                        .keys
                        .get(&m.id)
                        .and_then(|keys| keys.get(&k.id))
                        .cloned()
                        .unwrap_or_else(|| k.key.clone()),
                })
                .collect()
        });
        cards.push(Card {
            id: m.id.clone(),
            name: info.map_or_else(|| m.title.clone(), |i| i.name.clone()),
            version: info.map(|i| i.version.clone()).unwrap_or_default(),
            authors: info.map(|i| i.authors.clone()).unwrap_or_default(),
            credits: info.map(|i| i.credits.clone()).unwrap_or_default(),
            license: info.map(|i| i.license.clone()).unwrap_or_default(),
            license_url: info.and_then(|i| i.license_url.clone()),
            homepage: info.and_then(|i| i.homepage.clone()),
            issues: info.and_then(|i| i.issues.clone()),
            description: m.description.clone(),
            index: Some(index),
            settings: m.native.as_ref().map_or(0, |p| p.settings.len()),
            keys,
            status,
            warnings: info.map(|i| i.warnings.clone()).unwrap_or_default(),
            has_icon,
            dev: m.dev,
            native: m.native.is_some(),
        });
    }
    s.cards = cards;
    s.icons = icons;
}

/// The game and Sunrise, which the list leaves out.
fn is_base_game(id: &str) -> bool {
    matches!(id, "minecraft" | "sunrise" | "java" | "sunrise-client" | "sunrise_client")
}

/// A mod's `icon.png` read from its own package, for when the game's list has none for it.
///
/// Sunrise lists the native mods (the ones built into the game) again from their installed
/// packages after loading, and that list keeps only whether a package has an icon, not the
/// icon: so every native mod would show a letter. The package is where Sunrise itself finds
/// it: `sunrise-native-mods/<id>` beside the game, else the game folder's `mods/`. Ids found
/// to have none are remembered in `missing` so the folders aren't read again.
fn package_icon(id: &str, missing: &mut std::collections::HashSet<String>) -> Option<Arc<[u8]>> {
    use sunrise_mod_package::{Package, find, mod_list};
    if missing.contains(id) {
        return None;
    }
    let read = |path: &std::path::Path| -> Option<Arc<[u8]>> {
        let package = Package::read(path).ok()?;
        if package.manifest.id != id {
            return None;
        }
        sunrise_client::pack_screen::load_icon(package.icon.as_ref()?.bytes())
    };
    let native = std::env::current_exe()
        .ok()
        .map(|exe| exe.with_file_name(mod_list::NATIVE_DIR).join(id));
    let found = native.as_deref().and_then(read).or_else(|| {
        let dir = std::env::current_dir().unwrap_or_default();
        find(&dir.join(mod_list::DIR))
            .into_iter()
            .chain(find(&dir.join(mod_list::NATIVE_DIR)))
            .chain(native.as_deref().and_then(|p| p.parent()).map(find).unwrap_or_default())
            .find_map(|path| read(&path))
    });
    if found.is_none() {
        missing.insert(id.to_owned());
    }
    found
}

fn icon_key(id: &str) -> String {
    format!("sunnymods/icon/{id}")
}

/// The cards the search, the chip and the sort keep; the selection stays when it can.
fn reshow(s: &mut State) {
    s.shown = list::shown(&s.cards, &s.search.value, s.filter, s.sort);
    let still = s
        .selected
        .as_ref()
        .is_some_and(|id| s.shown.iter().any(|&i| &s.cards[i].id == id));
    if !still {
        let old = s.selected.take();
        let first = s.shown.first();
        s.selected = first.map(|&i| s.cards[i].id.clone());
        if s.selected != old {
            s.panel_scroll = 0.0;
        }
    }
}

fn selected(s: &State) -> Option<&Card> {
    let id = s.selected.as_ref()?;
    s.cards.iter().find(|c| &c.id == id)
}

fn row_h() -> i32 {
    config::config().rows.height()
}

fn list_content(s: &State) -> i32 {
    s.shown.len() as i32 * row_h()
}

fn max_scroll(content: i32, view: i32) -> f64 {
    f64::from((content - view).max(0))
}

fn thumb(content: i32, view: i32, scroll: f64) -> Option<(i32, i32)> {
    if content <= view || view <= 0 {
        return None;
    }
    let len = ((view * view) / content.max(1)).clamp(16, view);
    let room = view - len;
    let at = (scroll / max_scroll(content, view) * f64::from(room)).round() as i32;
    Some((at, len))
}

fn clamp_scrolls(s: &mut State, g: &Geo) {
    s.list_scroll = s.list_scroll.clamp(0.0, max_scroll(list_content(s), g.list[3]));
    s.panel_scroll = s.panel_scroll.clamp(0.0, max_scroll(s.panel_content, g.body[3] - 8));
}

/// A tile for a mod without an icon: its initial on a colour of its own.
fn placeholder(out: &mut Vec<Draw>, m: &Measure, card: &Card, x: i32, y: i32, size: i32) {
    let mut h = 0u32;
    for b in card.id.bytes() {
        h = h.wrapping_mul(31).wrapping_add(u32::from(b));
    }
    const TILES: [u32; 6] = [0xFF3A_6EA5, 0xFF6A_4C9C, 0xFF2E_8B6E, 0xFFB0_5A2A, 0xFF8C_3B5A, 0xFF4B_6B2F];
    let c = TILES[(h as usize) % TILES.len()];
    d::fill(out, x + 1, y, x + size - 1, y + size, argb(c));
    d::fill(out, x, y + 1, x + size, y + size - 1, argb(c));
    d::fill(out, x + 1, y + size - 3, x + size - 1, y + size, argb(0x40000000));
    let initial: String = card.name.chars().next().unwrap_or('?').to_uppercase().collect();
    let t = Text::literal(initial).bold();
    let tw = m.width(&t);
    d::text(out, t, x + (size - tw) / 2, y + (size - 8) / 2, argb(WHITE), true);
}

fn icon(out: &mut Vec<Draw>, m: &Measure, s: &State, card: &Card, x: i32, y: i32, size: i32, used: &mut usize) {
    let key = icon_key(&card.id);
    if card.has_icon && s.icons.contains_key(&key) && *used < ICON_SLOTS {
        *used += 1;
        d::blit(out, &key, x, y, (0.0, 0.0), size, size, [size, size]);
    } else {
        placeholder(out, m, card, x, y, size);
    }
}

fn status_colour(card: &Card) -> u32 {
    match card.status {
        Status::Running if card.dev => PURPLE,
        Status::Running => GREEN,
        Status::Off => DIM,
        Status::Failed(_) => RED,
    }
}

fn status_text(card: &Card) -> Text {
    match &card.status {
        Status::Running if card.dev => Text::literal("Running (dev)"),
        Status::Running => Text::literal("Running"),
        Status::Off => Text::literal("Not loaded: turn it on in the launcher"),
        Status::Failed(why) => Text::literal(format!("Failed: {why}")),
    }
}

fn panel_box(out: &mut Vec<Draw>, r: [i32; 4]) {
    let [x, y, w, h] = r;
    d::fill(out, x, y, x + w, y + h, argb(PANEL));
    d::fill(out, x, y, x + w, y + 1, argb(PANEL_EDGE));
    // The 2 px top edge in the accent, as on every Sunrise-dark card.
    d::fill(out, x, y, x + w, y + 2, argb(opaque(config::config().accent)));
    d::fill(out, x, y + h - 1, x + w, y + h, argb(PANEL_EDGE));
    d::fill(out, x, y, x + 1, y + h, argb(PANEL_EDGE));
    d::fill(out, x + w - 1, y, x + w, y + h, argb(PANEL_EDGE));
}

fn scrollbar(out: &mut Vec<Draw>, r: [i32; 4], content: i32, scroll: f64, held: bool) {
    if let Some((at, len)) = thumb(content, r[3], scroll) {
        let x = r[0] + r[2] - 4;
        d::fill(out, x, r[1], x + 3, r[1] + r[3], argb(TRACK));
        let c = if held { THUMB_HELD } else { THUMB };
        d::fill(out, x, r[1] + at, x + 3, r[1] + at + len, argb(c));
    }
}

fn now_ms(s: &State) -> i64 {
    s.started.elapsed().as_millis() as i64
}

/// The list's draws.
fn draw_list(out: &mut Vec<Draw>, m: &Measure, s: &State, g: &Geo, mouse: (i32, i32), used: &mut usize) {
    let cfg = config::config();
    let accent = opaque(cfg.accent);
    let [x, y, w, h] = g.list;
    panel_box(out, [x, y, w, h]);
    let rh = cfg.rows.height();
    let size = cfg.rows.icon();
    out.push(Draw::Scissor { x0: x + 1, y0: y + 1, x1: x + w - 1, y1: y + h - 1 });
    let first = (s.list_scroll / f64::from(rh)).floor().max(0.0) as usize;
    let rows = (h / rh) as usize + 2;
    for (n, &ci) in s.shown.iter().enumerate().skip(first).take(rows) {
        let card = &s.cards[ci];
        let ry = y + n as i32 * rh - s.list_scroll.round() as i32;
        let row = [x + 1, ry, w - 2, rh];
        let picked = s.selected.as_deref() == Some(card.id.as_str());
        if picked {
            d::fill(out, row[0], ry, row[0] + row[2], ry + rh, argb(tab_on(accent)));
            d::fill(out, row[0], ry, row[0] + 2, ry + rh, argb(accent));
        } else if inside(row, mouse) && inside(g.list, mouse) {
            d::fill(out, row[0], ry, row[0] + row[2], ry + rh, argb(ROW_HOVER));
        }
        icon(out, m, s, card, x + 5, ry + 3, size, used);
        let tx = x + 5 + size + 5;
        let right = x + w - 8;
        // The status dot, and a gear for a settings page.
        let dot = status_colour(card);
        d::fill(out, right - 4, ry + 5, right, ry + 9, argb(dot));
        let mut name_room = right - 8 - tx;
        if card.has_settings() {
            let gear = Text::literal("\u{2699}");
            let gw = m.width(&gear);
            d::text(out, gear, right - 8 - gw, ry + 3, argb(accent), false);
            name_room -= gw + 3;
        }
        let name = Text::literal(card.name.clone()).bold();
        let name = if m.width(&name) > name_room { d::clip(m, &name, name_room) } else { name };
        let nw = m.width(&name);
        d::text(out, name, tx, ry + 4, argb(WHITE), true);
        let mut tag = card.version.clone();
        if cfg.show_ids {
            tag = if tag.is_empty() { card.id.clone() } else { format!("{tag} \u{00B7} {}", card.id) };
        }
        if !tag.is_empty() && nw + 6 < name_room {
            let t = Text::literal(tag);
            let t = if m.width(&t) > name_room - nw - 6 { d::clip(m, &t, name_room - nw - 6) } else { t };
            d::text(out, t, tx + nw + 5, ry + 4, argb(DIM), false);
        }
        let room = right - tx;
        let line2 = if card.authors.is_empty() { String::new() } else { format!("by {}", card.authors_line()) };
        match cfg.rows {
            Rows::Comfortable => {
                if !line2.is_empty() {
                    let t = Text::literal(line2);
                    let t = if m.width(&t) > room { d::clip(m, &t, room) } else { t };
                    d::text(out, t, tx, ry + 15, argb(SOFT), false);
                }
                let summary = card.summary();
                if !summary.is_empty() {
                    let t = Text::literal(summary.to_owned());
                    let t = if m.width(&t) > room { d::clip(m, &t, room) } else { t };
                    d::text(out, t, tx, ry + 26, argb(DIM), false);
                }
            }
            Rows::Compact => {
                let line = if line2.is_empty() { card.summary().to_owned() } else { line2 };
                if !line.is_empty() {
                    let t = Text::literal(line);
                    let t = if m.width(&t) > room { d::clip(m, &t, room) } else { t };
                    d::text(out, t, tx, ry + 15, argb(DIM), false);
                }
            }
        }
    }
    if s.shown.is_empty() {
        let t = Text::literal("No mods match");
        let tw = m.width(&t);
        d::text(out, t, x + (w - tw) / 2, y + 12, argb(DIM), false);
    }
    out.push(Draw::NoScissor);
    scrollbar(out, g.list, list_content(s), s.list_scroll, s.held.is_some_and(|h| h.0 == Bar::List));
}

/// A line of the panel's body: the draw and its height. Clickable lines carry what a click
/// does.
enum Click {
    Link(String),
    Copy(String),
}

/// The detail panel's scrolled body. Returns its height and the clickable spots.
fn draw_body(out: &mut Vec<Draw>, m: &Measure, s: &State, card: &Card, g: &Geo, mouse: (i32, i32), used: &mut usize) -> (i32, Vec<([i32; 4], Click)>) {
    let cfg = config::config();
    let accent = opaque(cfg.accent);
    let [bx, by, bw, bh] = g.body;
    let left = bx + 8;
    let width = bw - 16;
    let top = by + 6 - s.panel_scroll.round() as i32;
    let mut clicks = Vec::new();
    let mut y = top;
    out.push(Draw::Scissor { x0: bx + 1, y0: by + 1, x1: bx + bw - 1, y1: by + bh });
    let big = if bw < 220 { 40 } else { BIG_ICON };
    icon(out, m, s, card, left, y, big, used);
    let hx = left + big + 8;
    let hroom = (left + width - hx).max(10);
    let name = Text::literal(card.name.clone()).bold();
    let name = if m.width(&name) > hroom { d::clip(m, &name, hroom) } else { name };
    d::text(out, name, hx, y + 2, argb(WHITE), true);
    // The version and the id: a click copies.
    let mut cx = hx;
    for (value, label) in [(&card.version, "version"), (&card.id, "id")] {
        if value.is_empty() {
            continue;
        }
        let t = Text::literal(if label == "id" { format!("#{value}") } else { format!("v{value}") });
        let tw = m.width(&t).min(hroom);
        let r = [cx, y + 13, tw, 9];
        let hover = inside(r, mouse) && inside(g.body, mouse);
        let t = if hover { t.underlined() } else { t };
        d::text(out, t, cx, y + 13, argb(if hover { TEXT } else { DIM }), false);
        if hover {
            d::tooltip(out, m, &Text::literal(format!("Click to copy the {label}")), mouse);
        }
        clicks.push((r, Click::Copy(value.clone())));
        cx += tw + 8;
    }
    let st = status_text(card);
    let st = if m.width(&st) > hroom { d::clip(m, &st, hroom) } else { st };
    d::text(out, st, hx, y + 25, argb(status_colour(card)), false);
    let mut extras = Vec::new();
    if card.settings > 0 {
        extras.push(format!("{} setting{}", card.settings, if card.settings == 1 { "" } else { "s" }));
    }
    if !card.keys.is_empty() {
        extras.push(format!("{} key{}", card.keys.len(), if card.keys.len() == 1 { "" } else { "s" }));
    }
    if !extras.is_empty() {
        d::text(out, Text::literal(extras.join(" \u{00B7} ")), hx, y + 37, argb(accent), false);
    }
    y += big + 8;
    let rule = |out: &mut Vec<Draw>, y: i32| d::fill(out, left, y, left + width, y + 1, argb(0x30FF_FFFF));
    let heading = |out: &mut Vec<Draw>, y: &mut i32, label: &str| {
        d::text(out, Text::literal(label.to_owned()).bold(), left, *y, argb(accent), false);
        *y += 11;
    };
    let wrapped = |out: &mut Vec<Draw>, y: &mut i32, text: Text, colour: u32| {
        for line in m.split(&text, width) {
            d::text(out, line, left, *y, argb(colour), false);
            *y += 10;
        }
    };
    rule(out, y - 3);
    // Facts: authors, licence (a link when it has one), credits.
    let fact = |out: &mut Vec<Draw>, y: &mut i32, label: &str, value: &str| -> [i32; 4] {
        let l = Text::literal(format!("{label} "));
        let lw = m.width(&l);
        d::text(out, l, left, *y, argb(SOFT), false);
        let lines = m.split(&Text::literal(value.to_owned()), (width - lw).max(20));
        let start = *y;
        let mut vw = 0;
        for line in lines {
            vw = vw.max(m.width(&line));
            d::text(out, line, left + lw, *y, argb(TEXT), false);
            *y += 10;
        }
        [left + lw, start, vw, *y - start]
    };
    if !card.authors.is_empty() {
        let label = if card.authors.len() == 1 { "Author" } else { "Authors" };
        fact(out, &mut y, label, &card.authors_line());
    }
    if !card.license.is_empty() {
        let link = list::safe_link(card.license_url.as_deref()).map(str::to_owned);
        let r = fact(out, &mut y, "Licence", &card.license);
        if let Some(link) = link {
            if inside(r, mouse) && inside(g.body, mouse) {
                d::fill(out, r[0], r[1] + 9, r[0] + r[2], r[1] + 10, argb(TEXT));
                d::tooltip(out, m, &Text::literal(link.clone()), mouse);
            }
            clicks.push((r, Click::Link(link)));
        }
    } else {
        fact(out, &mut y, "Licence", "not stated");
    }
    if !card.credits.trim().is_empty() {
        fact(out, &mut y, "Credits", card.credits.trim());
    }
    y += 4;
    if !card.description.trim().is_empty() {
        heading(out, &mut y, "About");
        for (i, para) in card.description.trim().split('\n').enumerate() {
            if i > 0 {
                y += 2;
            }
            if para.trim().is_empty() {
                continue;
            }
            wrapped(out, &mut y, Text::literal(para.trim().to_owned()), TEXT);
        }
        y += 4;
    }
    if !card.keys.is_empty() {
        heading(out, &mut y, "Keys");
        for k in &card.keys {
            let key = sunrise_client::keybinds::display_name(&k.key);
            let key = if k.key == "key.keyboard.unknown" { Text::literal("Not bound") } else { key };
            let kw = m.width(&key);
            let label = Text::translate(&k.label, &[]);
            let room = (width - kw - 8).max(10);
            let label = if m.width(&label) > room { d::clip(m, &label, room) } else { label };
            d::text(out, label, left, y, argb(SOFT), false);
            d::text(out, key, left + width - kw, y, argb(TEXT), false);
            y += 10;
        }
        y += 4;
    }
    if let Status::Failed(why) = &card.status {
        heading(out, &mut y, "Problem");
        wrapped(out, &mut y, Text::literal(why.clone()), RED);
        y += 4;
    }
    if !card.warnings.is_empty() {
        heading(out, &mut y, "Notices");
        for (key, args) in &card.warnings {
            let args: Vec<Text> = args.iter().map(|a| Text::literal(a.clone())).collect();
            wrapped(out, &mut y, Text::translate(key, &args), GOLD);
            y += 2;
        }
    }
    out.push(Draw::NoScissor);
    (y - top, clicks)
}

struct Labels {
    footer: [Text; 4],
    buttons: [Text; 3],
}

fn labels() -> Labels {
    Labels {
        footer: [
            tr("sunnymods.button.browse", "Browse Mods"),
            tr("sunnymods.button.folder", "Mods Folder"),
            tr("sunnymods.button.classic", "Sunrise List"),
            tr("sunnymods.button.done", "Done"),
        ],
        buttons: [
            tr("sunnymods.button.settings", "Settings"),
            tr("sunnymods.button.website", "Website"),
            tr("sunnymods.button.issues", "Issues"),
        ],
    }
}

/// Draws the screen over the menu background, in place of Sunrise's Mods page.
pub(super) fn draw(game: &mut Game<'_>, batch: &mut HudBatch) {
    let (w, h) = (batch.width, batch.height);
    let in_world = game.ui.options.in_world;
    sunrise_client::ui::menu_background(batch, &game.gui.atlas, 0.0, 0.0, w, h, in_world);
    let mouse = (game.cursor.0.floor() as i32, game.cursor.1.floor() as i32);
    let atlas = &game.gui.atlas;
    let m = Measure(&atlas.font);
    let draws = STATE.with(|cell| {
        let Ok(mut s) = cell.try_borrow_mut() else { return Vec::new() };
        let mut shade = Vec::new();
        d::fill(&mut shade, 0, 0, w as i32, h as i32, argb(SHADE));
        refresh(game, &mut s);
        reshow(&mut s);
        let g = geo(&m, w as i32, h as i32);
        let [sx, sy, sw, sh] = g.search;
        s.search.x = sx;
        s.search.y = sy;
        s.search.width = sw;
        s.search.height = sh;
        clamp_scrolls(&mut s, &g);
        let cfg = config::config();
        let accent = opaque(cfg.accent);
        let mut out = shade;
        let now = now_ms(&s);
        // The title and the counts.
        let title = tr("sunnymods.title", "Mods").bold();
        let tw = m.width(&title);
        d::text(&mut out, title, (w as i32 - tw) / 2, 7, argb(accent), true);
        let (total, running, issues) = list::counts(&s.cards);
        let mut sub = format!("{total} installed \u{00B7} {running} running");
        if issues > 0 {
            sub.push_str(&format!(" \u{00B7} {issues} failed"));
        }
        let sub = Text::literal(sub);
        let subw = m.width(&sub);
        d::text(&mut out, sub, (w as i32 - subw) / 2, 18, argb(SOFT), false);
        // The search, the sort and the chips.
        let blink = s.focused_at.elapsed().as_millis() as u64;
        s.search.draw(&mut out, &m, blink, None);
        let [ox, oy, ow, oh] = g.sort;
        let sort_hover = inside(g.sort, mouse);
        d::button(&mut out, &m, &Text::literal(s.sort.short()), ox, oy, ow, oh, true, sort_hover, now);
        if sort_hover {
            let name = Text::translate_with_fallback(Sort::OPTIONS[s.sort.index() as usize], "", &[]);
            d::tooltip(&mut out, &m, &Text::literal("Sorted: ").append(name).append(Text::literal(". Click for the next order.")), mouse);
        }
        for (f, r) in &g.chips {
            let on = *f == s.filter;
            let hover = inside(*r, mouse);
            let [cx, cy, cw, ch] = *r;
            let bg = if on { tab_on(accent) } else if hover { CHIP_HOVER } else { CHIP };
            d::fill(&mut out, cx + 1, cy, cx + cw - 1, cy + ch, argb(bg));
            d::fill(&mut out, cx, cy + 1, cx + cw, cy + ch - 1, argb(bg));
            let t = tr(f.lang(), f.label());
            let lw = m.width(&t);
            d::text(&mut out, t, cx + (cw - lw) / 2, cy + 2, argb(if on { WHITE } else { TEXT }), on);
        }
        let mut used = 0;
        draw_list(&mut out, &m, &s, &g, mouse, &mut used);
        // The panel.
        panel_box(&mut out, g.panel);
        let card = selected(&s).cloned();
        let l = labels();
        match &card {
            Some(card) => {
                let (content, _) = draw_body(&mut out, &m, &s, card, &g, mouse, &mut used);
                s.panel_content = content;
                scrollbar(&mut out, g.body, content + 8, s.panel_scroll, s.held.is_some_and(|h| h.0 == Bar::Panel));
                let active = [
                    card.has_settings(),
                    list::safe_link(card.homepage.as_deref()).is_some(),
                    list::safe_link(card.issues.as_deref()).is_some(),
                ];
                for (i, r) in g.buttons.iter().enumerate() {
                    let hover = active[i] && inside(*r, mouse);
                    d::button(&mut out, &m, &l.buttons[i], r[0], r[1], r[2], r[3], active[i], hover, now);
                }
            }
            None => {
                let t = Text::literal("Pick a mod to see it here");
                let tw = m.width(&t);
                d::text(&mut out, t, g.panel[0] + (g.panel[2] - tw) / 2, g.panel[1] + 20, argb(DIM), false);
            }
        }
        for (i, r) in g.footer.iter().enumerate() {
            let hover = inside(*r, mouse);
            d::button(&mut out, &m, &l.footer[i], r[0], r[1], r[2], r[3], true, hover, now);
        }
        if inside(g.footer[2], mouse) {
            d::tooltip(&mut out, &m, &Text::literal("Shows Sunrise's own Mods list until you leave it."), mouse);
        }
        sunrise_client::pack_screen::bind_icon_slots(&s.icons, &mut out, atlas, &mut batch.patches);
        out
    });
    let tooltip = d::paint(batch, atlas, &draws);
    d::paint_tooltip(batch, atlas, &draws, tooltip);
}

/// What a click asks of the game, done once the screen's state is let go.
enum Act {
    None,
    Click,
    Events(Vec<Event>),
    Settings(usize),
    Browse,
    Classic,
    Done,
}

/// A mouse button on the screen.
pub(super) fn mouse(game: &mut Game<'_>, pressed: bool, right: bool) -> Result<()> {
    if !pressed {
        with(|s| s.held = None);
        return Ok(());
    }
    if right {
        return Ok(());
    }
    let (w, h) = game.gui_size();
    let mouse = (game.cursor.0.floor() as i32, game.cursor.1.floor() as i32);
    let (fx, fy) = (f64::from(game.cursor.0), f64::from(game.cursor.1));
    let act = {
        let atlas = &game.gui.atlas;
        let m = Measure(&atlas.font);
        with(|s| {
            let g = geo(&m, w as i32, h as i32);
            let was_focused = s.search.focused;
            s.search.focused = false;
            if inside(g.search, mouse) {
                s.search.focused = true;
                if !was_focused {
                    s.focused_at = Instant::now();
                }
                s.search.click(&m, fx, false, false);
                return Act::None;
            }
            if inside(g.sort, mouse) {
                s.sort = s.sort.next();
                reshow(s);
                return Act::Click;
            }
            for (f, r) in &g.chips {
                if inside(*r, mouse) {
                    s.filter = *f;
                    s.list_scroll = 0.0;
                    reshow(s);
                    return Act::Click;
                }
            }
            // Scrollbars.
            for (bar, r, content, scroll) in [
                (Bar::List, g.list, list_content(s), s.list_scroll),
                (Bar::Panel, g.body, s.panel_content + 8, s.panel_scroll),
            ] {
                let track = [r[0] + r[2] - 6, r[1], 6, r[3]];
                if inside(track, mouse) && let Some((at, len)) = thumb(content, r[3], scroll) {
                    let grab = fy - f64::from(r[1] + at);
                    let grab = if grab < 0.0 || grab > f64::from(len) { f64::from(len) / 2.0 } else { grab };
                    s.held = Some((bar, grab));
                    drag_bar(s, &g, fy);
                    return Act::None;
                }
            }
            if inside(g.list, mouse) {
                let rh = row_h();
                let n = ((fy - f64::from(g.list[1]) + s.list_scroll) / f64::from(rh)).floor();
                if n >= 0.0 && let Some(&ci) = s.shown.get(n as usize) {
                    let id = s.cards[ci].id.clone();
                    if s.selected.as_deref() == Some(id.as_str()) {
                        // A second click on the selected mod opens its settings.
                        if let Some(i) = s.cards[ci].index.filter(|_| s.cards[ci].has_settings()) {
                            return Act::Settings(i);
                        }
                    } else {
                        s.selected = Some(id);
                        s.panel_scroll = 0.0;
                    }
                    return Act::Click;
                }
                return Act::None;
            }
            let Some(card) = selected(s).cloned() else {
                return footer_act(&g, mouse);
            };
            if inside(g.buttons[0], mouse) && card.has_settings() {
                if let Some(i) = card.index {
                    return Act::Settings(i);
                }
            }
            if inside(g.buttons[1], mouse) && let Some(url) = list::safe_link(card.homepage.as_deref()) {
                return Act::Events(vec![Event::ButtonClick, Event::OpenLink(url.to_owned())]);
            }
            if inside(g.buttons[2], mouse) && let Some(url) = list::safe_link(card.issues.as_deref()) {
                return Act::Events(vec![Event::ButtonClick, Event::OpenLink(url.to_owned())]);
            }
            if inside(g.body, mouse) {
                // The clickable spots of the body, as last drawn.
                let mut scratch = Vec::new();
                let mut used = ICON_SLOTS;
                let (_, clicks) = draw_body(&mut scratch, &m, s, &card, &g, mouse, &mut used);
                for (r, click) in clicks {
                    if inside(r, mouse) {
                        return match click {
                            Click::Link(url) => Act::Events(vec![Event::ButtonClick, Event::OpenLink(url)]),
                            Click::Copy(text) => Act::Events(vec![Event::ButtonClick, Event::CopyText(text)]),
                        };
                    }
                }
                return Act::None;
            }
            footer_act(&g, mouse)
        })
        .unwrap_or(Act::None)
    };
    perform(game, act)
}

fn footer_act(g: &Geo, mouse: (i32, i32)) -> Act {
    if inside(g.footer[0], mouse) {
        Act::Browse
    } else if inside(g.footer[1], mouse) {
        Act::Events(vec![Event::ButtonClick, Event::OpenModFolder])
    } else if inside(g.footer[2], mouse) {
        Act::Classic
    } else if inside(g.footer[3], mouse) {
        Act::Done
    } else {
        Act::None
    }
}

fn perform(game: &mut Game<'_>, act: Act) -> Result<()> {
    match act {
        Act::None => Ok(()),
        Act::Click => game.handle_option_events(vec![Event::ButtonClick]),
        Act::Events(events) => game.handle_option_events(events),
        Act::Settings(i) => {
            game.ui.options.show(Page::NativeMod(i));
            game.handle_option_events(vec![Event::ButtonClick, Event::NativeSettings(i)])
        }
        Act::Browse => {
            game.ui.options.show(Page::ModBrowser);
            game.handle_option_events(vec![Event::ButtonClick, Event::BrowseMods])
        }
        Act::Classic => {
            with(|s| s.classic = true);
            game.ui.options.mods_screen = None;
            game.handle_option_events(vec![Event::ButtonClick])
        }
        Act::Done => {
            game.handle_option_events(vec![Event::ButtonClick])?;
            with(|s| s.search.focused = false);
            // Leaves as the page's Escape does (back to Options in a world, out on the title).
            game.options_key(KeyCode::Escape, None).map(|_| ())
        }
    }
}

fn drag_bar(s: &mut State, g: &Geo, y: f64) {
    let Some((bar, grab)) = s.held else { return };
    let (r, content) = match bar {
        Bar::List => (g.list, list_content(s)),
        Bar::Panel => (g.body, s.panel_content + 8),
    };
    let Some((_, len)) = thumb(content, r[3], 0.0) else { return };
    let room = f64::from((r[3] - len).max(1));
    let at = (y - grab - f64::from(r[1])).clamp(0.0, room);
    let value = at / room * max_scroll(content, r[3]);
    match bar {
        Bar::List => s.list_scroll = value,
        Bar::Panel => s.panel_scroll = value,
    }
}

/// The pointer moved with a button held: a held scrollbar follows it.
pub(super) fn drag(game: &mut Game<'_>) -> Result<()> {
    let (w, h) = game.gui_size();
    let y = f64::from(game.cursor.1);
    let atlas = &game.gui.atlas;
    let m = Measure(&atlas.font);
    with(|s| {
        let g = geo(&m, w as i32, h as i32);
        drag_bar(s, &g, y);
    });
    Ok(())
}

/// The wheel: the list or the panel, whichever the pointer is over.
pub(super) fn scroll(game: &mut Game<'_>, amount: f32) -> Result<()> {
    let (w, h) = game.gui_size();
    let mouse = (game.cursor.0.floor() as i32, game.cursor.1.floor() as i32);
    let atlas = &game.gui.atlas;
    let m = Measure(&atlas.font);
    with(|s| {
        let g = geo(&m, w as i32, h as i32);
        let step = f64::from(amount) * WHEEL;
        if inside(g.panel, mouse) {
            s.panel_scroll -= step;
        } else {
            s.list_scroll -= step;
        }
        clamp_scrolls(s, &g);
    });
    Ok(())
}

/// Moves the selection by `delta` rows and keeps it in view.
fn step(s: &mut State, g: &Geo, delta: i32) {
    if s.shown.is_empty() {
        return;
    }
    let at = s
        .selected
        .as_ref()
        .and_then(|id| s.shown.iter().position(|&i| &s.cards[i].id == id));
    let next = match at {
        Some(at) => (at as i32 + delta).clamp(0, s.shown.len() as i32 - 1) as usize,
        None => 0,
    };
    s.selected = Some(s.cards[s.shown[next]].id.clone());
    s.panel_scroll = 0.0;
    let rh = f64::from(row_h());
    let top = next as f64 * rh;
    let view = f64::from(g.list[3]);
    if top < s.list_scroll {
        s.list_scroll = top;
    } else if top + rh > s.list_scroll + view {
        s.list_scroll = top + rh - view;
    }
}

/// A key on the screen: `None` lets Sunrise have it (Escape, to leave).
pub(super) fn key(game: &mut Game<'_>, key: KeyCode, text: Option<&str>) -> Option<Result<()>> {
    let shift = game.has_shift_down();
    let ctrl = game.has_control_down();
    let (w, h) = game.gui_size();
    let act = {
        let atlas = &game.gui.atlas;
        let m = Measure(&atlas.font);
        with(|s| {
            let g = geo(&m, w as i32, h as i32);
            if key == KeyCode::Escape {
                if s.search.focused {
                    s.search.focused = false;
                    return Some(Act::None);
                }
                return None;
            }
            if s.search.focused {
                if matches!(key, KeyCode::ArrowUp | KeyCode::ArrowDown | KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Tab) {
                    s.search.focused = false;
                } else {
                    let before = s.search.value.clone();
                    if s.search.key(&m, key, shift, ctrl).is_none()
                        && let Some(t) = text.filter(|t| !t.chars().any(char::is_control))
                    {
                        s.search.insert_text(&m, t);
                    }
                    if s.search.value != before {
                        s.list_scroll = 0.0;
                        reshow(s);
                    }
                    return Some(Act::None);
                }
            }
            match key {
                KeyCode::ArrowUp => step(s, &g, -1),
                KeyCode::ArrowDown => step(s, &g, 1),
                KeyCode::PageUp => step(s, &g, -(g.list[3] / row_h()).max(1)),
                KeyCode::PageDown => step(s, &g, (g.list[3] / row_h()).max(1)),
                KeyCode::Home => step(s, &g, -(s.shown.len() as i32)),
                KeyCode::End => step(s, &g, s.shown.len() as i32),
                KeyCode::Enter | KeyCode::NumpadEnter => {
                    if let Some(card) = selected(s)
                        && card.has_settings()
                        && let Some(i) = card.index
                    {
                        return Some(Act::Settings(i));
                    }
                }
                KeyCode::KeyF if ctrl => {
                    s.search.focused = true;
                    s.focused_at = Instant::now();
                }
                KeyCode::Tab => {
                    let at = Filter::ALL.iter().position(|f| *f == s.filter).unwrap_or(0);
                    let n = Filter::ALL.len();
                    s.filter = Filter::ALL[if shift { (at + n - 1) % n } else { (at + 1) % n }];
                    s.list_scroll = 0.0;
                    reshow(s);
                }
                _ => {
                    // Typing starts a search.
                    if !ctrl && let Some(t) = text.filter(|t| !t.is_empty() && !t.chars().any(char::is_control)) {
                        s.search.focused = true;
                        s.focused_at = Instant::now();
                        s.search.insert_text(&m, t);
                        s.list_scroll = 0.0;
                        reshow(s);
                    }
                }
            }
            Some(Act::None)
        })
        .flatten()
    }?;
    Some(perform(game, act))
}

/// Ctrl+A, C, V and X in the search box.
pub(super) fn shortcut(game: &mut Game<'_>, shortcut: sunrise_client::clipboard::EditShortcut) -> bool {
    let atlas = &game.gui.atlas;
    let m = Measure(&atlas.font);
    let clipboard = &mut game.clipboard;
    with(|s| {
        if !s.search.focused {
            return false;
        }
        let before = s.search.value.clone();
        s.search.shortcut(&m, shortcut, clipboard);
        if s.search.value != before {
            s.list_scroll = 0.0;
            reshow(s);
        }
        true
    })
    .unwrap_or(false)
}
