// SPDX-License-Identifier: MIT
// The wheel on SunnyMods's settings card.
fn sunnymods_card_options_scroll(
    game: &mut crate::game::Game<'_>,
    amount: f32,
) -> ::core::ops::ControlFlow<Result<()>> {
    if !crate::game::sunnymods_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::sunnymods_card::scroll(game, amount))
}
