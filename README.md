# SunnyMods

A Sunrise client mod by Zekrenlight (MIT).
Version 1.0.0 · Minecraft Java 26.3 · Sunrise API 0.2 · Mod SDK `9425651` · MIT licence.

SunnyMods takes over Sunrise's own Mods list: the title screen's **Mods** button and
**Options > Mods...** both open it. Nothing else changes; turn "Use SunnyMods" off on its
settings page (or press **Sunrise List** at the bottom) to see Sunrise's list again.

## The list
- Each mod's own `icon.png`, sharp at 32 px (20 px in compact rows); a lettered tile for mods
  without one. Mods built into the game are read from their installed package
  (`sunrise-native-mods/<id>` beside the game, or `mods/`), as the game's own list keeps
  only whether they have an icon.
- Name, version, authors and the first sentence of the description on every row, with a
  status dot (green running, grey not loaded, red failed, purple dev) and a
  gear for mods with a settings page.
- Search by name, id, author, licence or description. `@name` looks only at the authors,
  `#id` only at the ids. Typing anywhere starts a search; Ctrl+F focuses it.
- Filter chips: All, Running, Settings (mods with a settings page), Issues (failed, not
  loaded, or with loading notices). Tab steps through them.
- Sort: name A to Z, Z to A, author, or settings first (the button beside the search).
- Only mods are listed: the game and Sunrise themselves are left out.

## The panel
- A 64 px icon, the name, `v1.0.0` and `#id` (click to copy), the status, and how many
  settings and keys the mod has.
- Authors, licence (a link when the mod gives one), credits, the full description, the keys
  the mod adds with what they are bound to now, any failure and loading notices.
- **Settings** opens the mod's settings page (Done or Escape comes back to the list);
  **Website** and **Issues** open the mod's links after Sunrise's confirmation. A second
  click on the selected mod, or Enter, also opens its settings.

## Footer
- **Browse Mods** (Sunrise's mod browser), **Mods Folder**, **Sunrise List** (Sunrise's own
  list until you leave it) and **Done**.

## Keys
- Open the Mods List: not bound at first (Options > Controls > SunnyMods). From play it opens
  the list straight away.
- On the list: arrows, Page Up/Down, Home/End to move, Enter for Settings, Tab for the
  chips, Escape to leave the search box or the list.

## Settings
Use SunnyMods, Sort, Rows (comfortable or compact), Show Mod IDs,
Accent Colour.

## Notes
- Which mods run is still the launcher's choice, as in Sunrise's own list.
- The pre-load startup menu keeps Sunrise's list.

## Look and settings

SunnyMods uses the shared Sunrise dark look: a `#0F1118` card with a 2 px accent edge, tabs and
charcoal chips. The accent is Sunrise orange `#F9801D` at first; change it with **Accent colour**
in the settings card (eight presets, or Reset).
The mods list and its panel use the same card, chips and accent.

The settings card opens from SunnyMods' **Settings** button, from Sunrise's own Mods list, or
from the mod's settings key. **Key binds › Rebind** switches to Sunrise's classic page to change keys.

## Compatibility

| Piece | Pin |
| --- | --- |
| Minecraft Java | 26.3 |
| Sunrise API | 0.2 |
| Mod SDK (`[native].sunrise`) | `94256510f74e6843754e0a93fd9e4da7e45b1056` (`9425651`, 19 Nov 2026) |
| Mixins (`[native].mixins`) | `e59da9094f0c2fdda6c627fe9eff22dec6a906f0` |

SunnyMods works on its own and alongside the other Sunrise mods by Zekrenlight. Its settings-card
hooks act only on its own Options page, so several mods can be installed together without any
conflicting replacements. A mod with native hooks works only on the exact SDK build it was made for.

## Building

```sh
cargo sunrise-mod           # fetches the pinned SDK into .sunrise/ (gitignored)
cargo sunrise-mod check     # type-checks against the pinned SDK
cargo test --lib            # unit tests, no game needed
cargo sunrise-mod build     # target/sunrise-mod/sunnymods.smod
```

`python3 tools/make_icon.py icon.png` redraws the icon.

To move to a newer SDK, run `cargo sunrise-mod update --sunrise <build>` (it rewrites the
`sunrise` and `mixins` lines in `sunrise-mod.toml`), then check and rebuild.

Install the `.smod` with sunrise-launcher (**Add from file**), or point **Add from GitHub** at
this repository. Native mods run with full access to your computer; read the source first.

## License

MIT © 2026 Zekrenlight. See [LICENSE](LICENSE).
