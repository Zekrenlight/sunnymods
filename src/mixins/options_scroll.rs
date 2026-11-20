// The wheel on SunnyMods' list.
fn sunnymods_options_scroll(
    game: &mut crate::game::Game<'_>,
    amount: f32,
) -> ::core::ops::ControlFlow<Result<()>> {
    if !crate::game::sunnymods_ui::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::sunnymods_ui::scroll(game, amount))
}
