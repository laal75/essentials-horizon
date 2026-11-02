// Essentials: push the mod's commands into the server's command tree, asked
// once when a world opens to players. Delegates to the mod's module, which
// knows the usages and their operator levels.
fn register(commands: &mut Vec<NativeCommand>) {
    crate::game::essentials::register_all(commands);
}