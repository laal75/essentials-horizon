// Essentials: a player's command (their state swapped in). Dispatch it in the
// mod's module: it acts on the acting player, answers through native_feedback,
// and sets `ran` when one of its commands was handled.
fn perform(
    game: &mut crate::game::Game<'_>,
    player: &NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    crate::game::essentials::perform(game, player, command, ran);
}