// Ctrl+A, C, V and X in SunnyMods' search box.
fn sunnymods_options_shortcut(
    game: &mut crate::game::Game<'_>,
    shortcut: sunrise_client::clipboard::EditShortcut,
) -> ::core::ops::ControlFlow<bool> {
    if !crate::game::sunnymods_ui::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::sunnymods_ui::shortcut(game, shortcut))
}
