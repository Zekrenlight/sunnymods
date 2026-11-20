// Keys on SunnyMods' list; Escape (with the search box let go) is Sunrise's, to leave.
fn sunnymods_options_key(
    game: &mut crate::game::Game<'_>,
    key: KeyCode,
    text: Option<&str>,
) -> ::core::ops::ControlFlow<Result<()>> {
    if !crate::game::sunnymods_ui::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    match crate::game::sunnymods_ui::key(game, key, text) {
        Some(done) => ::core::ops::ControlFlow::Break(done),
        None => ::core::ops::ControlFlow::Continue(()),
    }
}
