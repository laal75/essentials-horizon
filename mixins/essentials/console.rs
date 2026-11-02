// Essentials: a command from the dedicated server's console (no acting player;
// the source is the console at permission level 4). Dispatch it in the mod's
// module and set `ran` when one of its commands was handled.
fn console(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    crate::game::essentials::perform_console(game, command, ran);
}
