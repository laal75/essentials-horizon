// Essentials: the window's agent (and any other path) runs a command straight
// through the vanilla dispatcher (`run_command`), past the native command
// hooks. Catch Essentials' commands at the top of it, before the vanilla
// parser rejects them as unknown. `Break(true)` answers the command (the body
// is skipped); `Continue(())` leaves a command none of these is to vanilla.
fn command_head(
    game: &mut crate::game::Game<'_>,
    command: &str,
) -> core::ops::ControlFlow<bool> {
    match crate::game::essentials::run_command(game, command) {
        Some(result) => core::ops::ControlFlow::Break(result),
        None => core::ops::ControlFlow::Continue(()),
    }
}