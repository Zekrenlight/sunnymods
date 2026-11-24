// SPDX-License-Identifier: MIT
// Dragging on SunnyMods's settings card: its sliders and scrollbar.
fn sunnymods_card_options_drag(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<Result<bool>> {
    if !crate::game::sunnymods_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::sunnymods_card::drag(game).map(|()| true))
}
