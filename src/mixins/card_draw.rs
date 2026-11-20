// SPDX-License-Identifier: MIT
// SunnyMods's settings page: its card is drawn instead of Sunrise's generic page.
fn sunnymods_card_options_draw(
    game: &mut crate::game::Game<'_>,
    batch: &mut sunrise_client::hud::HudBatch,
) -> ::core::ops::ControlFlow<()> {
    if !crate::game::sunnymods_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    crate::game::sunnymods_card::draw(game, batch);
    ::core::ops::ControlFlow::Break(())
}
