// Once a frame: the Mods key, and letting Sunrise's own list go once its pages are left.
fn sunnymods_frame(game: &mut crate::game::Game<'_>) {
    crate::game::sunnymods_ui::frame(game);
}
