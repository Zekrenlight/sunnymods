// SPDX-License-Identifier: MIT
// The mouse buttons on SunnyMods's settings card.
fn sunnymods_card_options_mouse(
    game: &mut crate::game::Game<'_>,
    pressed: bool,
    right: bool,
) -> ::core::ops::ControlFlow<Result<bool>> {
    if !crate::game::sunnymods_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::sunnymods_card::mouse(game, pressed, right).map(|()| true))
}
