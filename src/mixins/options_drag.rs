// Dragging on SunnyMods' list: its scrollbars.
fn sunnymods_options_drag(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<Result<bool>> {
    if !crate::game::sunnymods_ui::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::sunnymods_ui::drag(game).map(|()| true))
}
