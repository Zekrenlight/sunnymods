// On Sunrise's Mods page, SunnyMods' list is drawn instead.
fn sunnymods_options_draw(
    game: &mut crate::game::Game<'_>,
    batch: &mut sunrise_client::hud::HudBatch,
) -> ::core::ops::ControlFlow<()> {
    if !crate::game::sunnymods_ui::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    crate::game::sunnymods_ui::draw(game, batch);
    ::core::ops::ControlFlow::Break(())
}
