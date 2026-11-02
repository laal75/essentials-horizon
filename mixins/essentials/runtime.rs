// Essentials: the usual server commands for a Horizon world — info, quality-
// of-life actions, homes, warps, teleport requests, nick and back.
//
// The three hooks (register, perform, console) delegate here. A player's
// command arrives in `perform` with that player's state swapped into `game`, so
// the actions act on them and `native_feedback` answers them. A world's data
// lives in its `essentials/` folder as JSON, written whole to a temp file and
// renamed over the old one: a backup of the world carries it, and a crash leaves
// the old file or the new one, never half of one. A world that never runs an
// Essentials command never gets the folder, so it is untouched.

use super::Game;
use super::native_server::{NativeCommand, NativeServerPlayer};
use horizon_client::command::args::GameMode;
use horizon_client::text::Text;
use horizon_core::geometry::Vec3;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// A stored position in a dimension (a home, warp, last place, or the target of
/// a pending teleport request).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct Pos {
    dim: String,
    x: f64,
    y: f64,
    z: f64,
}

/// A pending teleport request someone made to this player.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct TpaRequest {
    /// The requester's UUID (decimal string). On /tpaccept, the accepter (the
    /// acting player) moves to the requester — the native command hook can only
    /// move the acting player, so both /tpa and /tpahere bring the target to
    /// the one who asked.
    from: String,
}

impl Pos {
    /// The acting player's place (dimension + feet). Only called where a
    /// player's command has their state swapped in.
    fn of(game: &Game) -> Pos {
        let p = game.player.as_ref().expect("Pos::of needs the acting player");
        Pos {
            dim: game.dim.id().to_owned(),
            x: p.position.x,
            y: p.position.y,
            z: p.position.z,
        }
    }
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

/// Essentials' usages, in the order a player would expect them. A name vanilla
/// already has is left out by the server (with a log line), so none of these
/// collide with vanilla's own commands.
pub(crate) fn register_all(commands: &mut Vec<NativeCommand>) {
    // Info.
    commands.push(NativeCommand::new("whereami"));
    commands.push(NativeCommand::new("online"));
    commands.push(NativeCommand::new("whois [target:player]"));
    // Quality-of-life actions (the acting player).
    commands.push(NativeCommand::new("heal"));
    commands.push(NativeCommand::new("feed"));
    commands.push(NativeCommand::new("god"));
    commands.push(NativeCommand::new("fly"));
    commands.push(NativeCommand::new("gmc"));
    commands.push(NativeCommand::new("gms"));
    commands.push(NativeCommand::new("gma"));
    commands.push(NativeCommand::new("gmsp"));
    // Homes (every player keeps their own; operators may not delete others').
    commands.push(NativeCommand::new("sethome [name:word]"));
    commands.push(NativeCommand::new("home [name:word]"));
    commands.push(NativeCommand::new("delhome <name:word>"));
    commands.push(NativeCommand::new("homes"));
    // Warps (operators set and remove; everyone uses).
    commands.push(NativeCommand::op("setwarp <name:word>", 2));
    commands.push(NativeCommand::new("warp [name:word]"));
    commands.push(NativeCommand::op("delwarp <name:word>", 2));
    commands.push(NativeCommand::new("warps"));
    // Teleport requests.
    commands.push(NativeCommand::new("tpa <target:player>"));
    commands.push(NativeCommand::op("tpahere <target:player>", 2));
    commands.push(NativeCommand::new("tpaccept"));
    commands.push(NativeCommand::new("tpdeny"));
    // Back to the last place you stood.
    commands.push(NativeCommand::new("back"));
    // Nick (a display name over your real one, in chat and the Tab list).
    commands.push(NativeCommand::new("nick <name:word>"));
    commands.push(NativeCommand::new("unnick"));
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

/// A player's Essentials command: act on the acting player, answer with
/// `native_feedback`, and set `ran` when handled. A word none of these is is
/// left for the server's "unknown command".
pub(super) fn perform(
    game: &mut Game,
    player: &NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    let words: Vec<&str> = command.split_whitespace().collect();
    let Some(first) = words.first().copied() else {
        return;
    };
    let arg = words.get(1).copied();
    match first {
        "whereami" => whereami(game, ran),
        "online" => online(game, ran),
        "whois" => whois(game, player, arg, ran),
        "heal" => heal(game, ran),
        "feed" => feed(game, ran),
        "god" => god(game, ran),
        "fly" => fly(game, ran),
        "gmc" => gamemode(game, GameMode::Creative, ran),
        "gms" => gamemode(game, GameMode::Survival, ran),
        "gma" => gamemode(game, GameMode::Adventure, ran),
        "gmsp" => gamemode(game, GameMode::Spectator, ran),
        "sethome" => sethome(game, player, arg, ran),
        "home" => home(game, player, arg, ran),
        "delhome" => delhome(game, player, arg, ran),
        "homes" => homes(game, player, ran),
        "setwarp" => setwarp(game, player, arg, ran),
        "warp" => warp(game, Some(player), arg, ran),
        "delwarp" => delwarp(game, player, arg, ran),
        "warps" => warps(game, ran),
        "tpa" => tpa(game, player, arg, ran),
        "tpahere" => tpahere(game, player, arg, ran),
        "tpaccept" => tpaccept(game, player, ran),
        "tpdeny" => tpdeny(game, player, ran),
        "back" => back(game, player, ran),
        "nick" => nick(game, player, arg, ran),
        "unnick" => unnick(game, player, ran),
        _ => {}
    }
}

/// A console Essentials command (no acting player). The read-only listings and
/// the world's warps; the commands that need a player answer that they want one.
pub(super) fn perform_console(game: &mut Game, command: &str, ran: &mut bool) {
    let words: Vec<&str> = command.split_whitespace().collect();
    let Some(first) = words.first().copied() else {
        return;
    };
    let arg = words.get(1).copied();
    match first {
        "online" => online(game, ran),
        "warps" => warps(game, ran),
        "warp" => warp(game, None, arg, ran),
        _ => {
            game.native_feedback(Text::legacy(
                "that Essentials command needs a player to run it in-game",
            ));
            *ran = true;
        }
    }
}

/// A command that reaches the vanilla dispatcher directly (the window's agent:
/// `owner_console_command` -> `run_command`), past `native_perform_command`.
/// Run it here, as the acting player, when it is one of Essentials' — the
/// agent's turn has their state swapped in, so `native_acting_player` and the
/// actions reach them. `Some(true)` when handled (the caller stops the
/// vanilla parser); `None` when it is not one of these, so vanilla runs it.
pub(super) fn run_command(game: &mut Game, command: &str) -> Option<bool> {
    let Some(player) = game.native_acting_player() else {
        return None;
    };
    let mut ran = false;
    perform(game, &player, command, &mut ran);
    if ran {
        Some(true)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Info
// ---------------------------------------------------------------------------

fn whereami(game: &mut Game, ran: &mut bool) {
    let p = Pos::of(game);
    let line = format!(
        "You are at [{}, {}, {}] in {}",
        p.x, p.y, p.z, p.dim
    );
    game.native_feedback(Text::legacy(&line));
    *ran = true;
}

fn online(game: &mut Game, ran: &mut bool) {
    let players = game.native_server_players();
    let line = if players.is_empty() {
        "No players online.".to_owned()
    } else {
        let names: Vec<String> = players.iter().map(|p| p.name.clone()).collect();
        format!("Online ({}): {}", players.len(), names.join(", "))
    };
    game.native_feedback(Text::legacy(&line));
    *ran = true;
}

fn whois(game: &mut Game, player: &NativeServerPlayer, target: Option<&str>, ran: &mut bool) {
    let target = match target {
        Some(name) => game
            .native_server_players()
            .into_iter()
            .find(|p| p.name.eq_ignore_ascii_case(name)),
        None => Some(player.clone()),
    };
    let Some(t) = target else {
        game.native_feedback(Text::legacy("that player is not online"));
        *ran = true;
        return;
    };
    let mode = mode_name(t.game_mode);
    let line = format!(
        "{} — {} (level {}), {} hp, at [{}, {}, {}] in {}",
        t.name,
        mode,
        t.permission,
        t.health,
        t.position[0],
        t.position[1],
        t.position[2],
        t.dimension
    );
    game.native_feedback(Text::legacy(&line));
    *ran = true;
}

fn mode_name(id: i32) -> &'static str {
    match id {
        0 => "survival",
        1 => "creative",
        2 => "adventure",
        3 => "spectator",
        _ => "unknown",
    }
}

// ---------------------------------------------------------------------------
// Actions on the acting player
// ---------------------------------------------------------------------------

fn heal(game: &mut Game, ran: &mut bool) {
    // set_health clamps to the player's maximum, so a high value is "full".
    let healed = game
        .player
        .as_mut()
        .is_some_and(|p| p.set_health(1000.0, false).is_ok());
    let line = if healed {
        "You have been healed"
    } else {
        "I could not heal you"
    };
    game.native_feedback(Text::legacy(line));
    *ran = true;
}

fn feed(game: &mut Game, ran: &mut bool) {
    let fed = game.player.as_mut().is_some_and(|p| {
        p.health.food.food = 20;
        p.health.food.saturation = 20.0;
        p.health.food.exhaustion = 0.0;
        true
    });
    let line = if fed { "Your hunger is restored" } else { "I could not feed you" };
    game.native_feedback(Text::legacy(line));
    *ran = true;
}

fn god(game: &mut Game, ran: &mut bool) {
    let Some(p) = game.player.as_mut() else {
        game.native_feedback(Text::legacy("no player to change"));
        *ran = true;
        return;
    };
    p.invulnerable = !p.invulnerable;
    let line = if p.invulnerable {
        "God mode is now ON"
    } else {
        "God mode is now OFF"
    };
    game.native_feedback(Text::legacy(line));
    *ran = true;
}

fn fly(game: &mut Game, ran: &mut bool) {
    let Some(p) = game.player.as_mut() else {
        game.native_feedback(Text::legacy("no player to change"));
        *ran = true;
        return;
    };
    p.may_fly = !p.may_fly;
    p.flying = p.may_fly;
    let line = if p.may_fly { "You can now fly" } else { "You can no longer fly" };
    game.native_feedback(Text::legacy(line));
    *ran = true;
}

fn gamemode(game: &mut Game, mode: GameMode, ran: &mut bool) {
    let name = match mode {
        GameMode::Creative => "creative",
        GameMode::Survival => "survival",
        GameMode::Adventure => "adventure",
        GameMode::Spectator => "spectator",
    };
    let changed = game.set_game_mode(mode);
    let line = if changed {
        format!("Game mode set to {name}")
    } else {
        format!("You are already in {name}")
    };
    game.native_feedback(Text::legacy(&line));
    *ran = true;
}

// ---------------------------------------------------------------------------
// Homes
// ---------------------------------------------------------------------------

fn sethome(game: &mut Game, player: &NativeServerPlayer, name: Option<&str>, ran: &mut bool) {
    let Some(dir) = data_dir(game) else {
        game.native_feedback(Text::legacy("no world to store homes in"));
        *ran = true;
        return;
    };
    let name = name.unwrap_or("home").to_lowercase();
    let mut homes: BTreeMap<String, BTreeMap<String, Pos>> =
        load(&dir.join("homes.json")).unwrap_or_default();
    let owner = player.uuid.to_string();
    let entry = homes.entry(owner.clone()).or_default();
    let count = entry.len();
    entry.insert(name.clone(), Pos::of(game));
    save(&dir.join("homes.json"), &homes);
    let line = if count == 0 {
        format!("set home '{name}' (your first)")
    } else {
        format!("set home '{name}'")
    };
    game.native_feedback(Text::legacy(&line));
    *ran = true;
}

fn home(game: &mut Game, player: &NativeServerPlayer, name: Option<&str>, ran: &mut bool) {
    let Some(dir) = data_dir(game) else {
        game.native_feedback(Text::legacy("no world to read homes from"));
        *ran = true;
        return;
    };
    let homes: BTreeMap<String, BTreeMap<String, Pos>> =
        load(&dir.join("homes.json")).unwrap_or_default();
    let owner = player.uuid.to_string();
    let Some(entry) = homes.get(&owner) else {
        game.native_feedback(Text::legacy("you have no homes (try /sethome)"));
        *ran = true;
        return;
    };
    let found = match name {
        Some(n) => entry
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(n))
            .map(|(k, v)| (k.clone(), v.clone())),
        None => entry
            .get("home")
            .cloned()
            .map(|v| ("home".to_owned(), v))
            .or_else(|| entry.iter().next().map(|(k, v)| (k.clone(), v.clone()))),
    };
    let Some((name, pos)) = found else {
        let names: Vec<String> = entry.keys().cloned().collect();
        game.native_feedback(Text::legacy(&format!(
            "no home by that name (you have: {})",
            names.join(", ")
        )));
        *ran = true;
        return;
    };
    remember(game, player);
    teleport_to(game, &pos);
    game.native_feedback(Text::legacy(&format!("teleported to home '{name}'")));
    *ran = true;
}

fn delhome(game: &mut Game, player: &NativeServerPlayer, name: Option<&str>, ran: &mut bool) {
    let Some(name) = name else {
        game.native_feedback(Text::legacy("usage: /delhome <name>"));
        *ran = true;
        return;
    };
    let Some(dir) = data_dir(game) else {
        game.native_feedback(Text::legacy("no world to delete homes from"));
        *ran = true;
        return;
    };
    let mut homes: BTreeMap<String, BTreeMap<String, Pos>> =
        load(&dir.join("homes.json")).unwrap_or_default();
    let owner = player.uuid.to_string();
    let Some(entry) = homes.get_mut(&owner) else {
        game.native_feedback(Text::legacy("you have no homes"));
        *ran = true;
        return;
    };
    let Some(key) = entry.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(k, _)| k.clone())
    else {
        game.native_feedback(Text::legacy(&format!("no home named '{name}'")));
        *ran = true;
        return;
    };
    entry.remove(&key);
    if entry.is_empty() {
        homes.remove(&owner);
    }
    save(&dir.join("homes.json"), &homes);
    game.native_feedback(Text::legacy(&format!("deleted home '{key}'")));
    *ran = true;
}

fn homes(game: &mut Game, player: &NativeServerPlayer, ran: &mut bool) {
    let Some(dir) = data_dir(game) else {
        game.native_feedback(Text::legacy("no homes yet"));
        *ran = true;
        return;
    };
    let homes: BTreeMap<String, BTreeMap<String, Pos>> =
        load(&dir.join("homes.json")).unwrap_or_default();
    let owner = player.uuid.to_string();
    let Some(entry) = homes.get(&owner) else {
        game.native_feedback(Text::legacy("you have no homes (try /sethome)"));
        *ran = true;
        return;
    };
    let names: Vec<String> = entry.keys().cloned().collect();
    game.native_feedback(Text::legacy(&format!(
        "your homes: {}",
        names.join(", ")
    )));
    *ran = true;
}

// ---------------------------------------------------------------------------
// Warps
// ---------------------------------------------------------------------------

fn setwarp(game: &mut Game, player: &NativeServerPlayer, name: Option<&str>, ran: &mut bool) {
    if player.permission < 2 {
        game.native_feedback(Text::legacy("you need operator level 2 to set warps"));
        *ran = true;
        return;
    }
    let Some(name) = name else {
        game.native_feedback(Text::legacy("usage: /setwarp <name>"));
        *ran = true;
        return;
    };
    let Some(dir) = data_dir(game) else {
        game.native_feedback(Text::legacy("no world to store warps in"));
        *ran = true;
        return;
    };
    let name = name.to_lowercase();
    let mut warps: BTreeMap<String, Pos> = load(&dir.join("warps.json")).unwrap_or_default();
    warps.insert(name.clone(), Pos::of(game));
    save(&dir.join("warps.json"), &warps);
    game.native_feedback(Text::legacy(&format!("set warp '{name}'")));
    *ran = true;
}

fn warp(
    game: &mut Game,
    player: Option<&NativeServerPlayer>,
    name: Option<&str>,
    ran: &mut bool,
) {
    let Some(dir) = data_dir(game) else {
        game.native_feedback(Text::legacy("no warps yet"));
        *ran = true;
        return;
    };
    let warps: BTreeMap<String, Pos> = load(&dir.join("warps.json")).unwrap_or_default();
    let Some(name) = name else {
        let names: Vec<String> = warps.keys().cloned().collect();
        let line = if names.is_empty() {
            "no warps yet (an operator can /setwarp one)".to_owned()
        } else {
            format!("warps: {}", names.join(", "))
        };
        game.native_feedback(Text::legacy(&line));
        *ran = true;
        return;
    };
    let Some((name, pos)) = warps
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(k, v)| (k.clone(), v.clone()))
    else {
        game.native_feedback(Text::legacy(&format!("no warp named '{name}'")));
        *ran = true;
        return;
    };
    if let Some(p) = player {
        remember(game, p);
    }
    teleport_to(game, &pos);
    game.native_feedback(Text::legacy(&format!("teleported to warp '{name}'")));
    *ran = true;
}

fn delwarp(game: &mut Game, player: &NativeServerPlayer, name: Option<&str>, ran: &mut bool) {
    if player.permission < 2 {
        game.native_feedback(Text::legacy("you need operator level 2 to remove warps"));
        *ran = true;
        return;
    }
    let Some(name) = name else {
        game.native_feedback(Text::legacy("usage: /delwarp <name>"));
        *ran = true;
        return;
    };
    let Some(dir) = data_dir(game) else {
        game.native_feedback(Text::legacy("no world to delete warps from"));
        *ran = true;
        return;
    };
    let mut warps: BTreeMap<String, Pos> = load(&dir.join("warps.json")).unwrap_or_default();
    let Some(key) = warps.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(k, _)| k.clone())
    else {
        game.native_feedback(Text::legacy(&format!("no warp named '{name}'")));
        *ran = true;
        return;
    };
    warps.remove(&key);
    save(&dir.join("warps.json"), &warps);
    game.native_feedback(Text::legacy(&format!("deleted warp '{key}'")));
    *ran = true;
}

fn warps(game: &mut Game, ran: &mut bool) {
    let Some(dir) = data_dir(game) else {
        game.native_feedback(Text::legacy("no warps yet"));
        *ran = true;
        return;
    };
    let warps: BTreeMap<String, Pos> = load(&dir.join("warps.json")).unwrap_or_default();
    let names: Vec<String> = warps.keys().cloned().collect();
    let line = if names.is_empty() {
        "no warps yet".to_owned()
    } else {
        format!("warps: {}", names.join(", "))
    };
    game.native_feedback(Text::legacy(&line));
    *ran = true;
}

// ---------------------------------------------------------------------------
// Teleport requests
// ---------------------------------------------------------------------------

fn tpa(game: &mut Game, player: &NativeServerPlayer, target: Option<&str>, ran: &mut bool) {
    request(game, player, target, ran);
}

fn tpahere(game: &mut Game, player: &NativeServerPlayer, target: Option<&str>, ran: &mut bool) {
    if player.permission < 2 {
        game.native_feedback(Text::legacy("you need operator level 2 to use /tpahere"));
        *ran = true;
        return;
    }
    request(game, player, target, ran);
}

fn request(
    game: &mut Game,
    player: &NativeServerPlayer,
    target: Option<&str>,
    ran: &mut bool,
) {
    let Some(target_name) = target else {
        game.native_feedback(Text::legacy("usage: /tpa <player>"));
        *ran = true;
        return;
    };
    let Some(t) = game
        .native_server_players()
        .into_iter()
        .find(|p| p.name.eq_ignore_ascii_case(target_name))
    else {
        game.native_feedback(Text::legacy(&format!("'{target_name}' is not online")));
        *ran = true;
        return;
    };
    if t.uuid == player.uuid {
        game.native_feedback(Text::legacy("you cannot request a teleport to yourself"));
        *ran = true;
        return;
    }
    let Some(dir) = data_dir(game) else {
        game.native_feedback(Text::legacy("no world to track requests in"));
        *ran = true;
        return;
    };
    let mut tpa: BTreeMap<String, TpaRequest> = load(&dir.join("tpa.json")).unwrap_or_default();
    tpa.insert(
        t.uuid.to_string(),
        TpaRequest {
            from: player.uuid.to_string(),
        },
    );
    save(&dir.join("tpa.json"), &tpa);
    let msg = format!("{} asks you to come to them (/tpaccept or /tpdeny)", player.name);
    let _ = game.native_tell(t.uuid, Text::legacy(&msg));
    game.native_feedback(Text::legacy(&format!(
        "asked {} to come to you (waiting to answer)",
        t.name
    )));
    *ran = true;
}

fn tpaccept(game: &mut Game, player: &NativeServerPlayer, ran: &mut bool) {
    let Some(dir) = data_dir(game) else {
        game.native_feedback(Text::legacy("no pending request"));
        *ran = true;
        return;
    };
    let mut tpa: BTreeMap<String, TpaRequest> = load(&dir.join("tpa.json")).unwrap_or_default();
    let me = player.uuid.to_string();
    let Some(req) = tpa.remove(&me) else {
        game.native_feedback(Text::legacy("you have no pending teleport request"));
        *ran = true;
        return;
    };
    save(&dir.join("tpa.json"), &tpa);
    let Some(from) = game
        .native_server_players()
        .into_iter()
        .find(|p| p.uuid.to_string() == req.from)
    else {
        game.native_feedback(Text::legacy("the requester is no longer online"));
        *ran = true;
        return;
    };
    // The accepter (the acting player) moves to the requester.
    remember(game, player);
    teleport_to(game, &Pos {
        dim: from.dimension.clone(),
        x: from.position[0],
        y: from.position[1],
        z: from.position[2],
    });
    game.native_feedback(Text::legacy(&format!("you accepted — moving to {}", from.name)));
    *ran = true;
}

fn tpdeny(game: &mut Game, player: &NativeServerPlayer, ran: &mut bool) {
    let Some(dir) = data_dir(game) else {
        game.native_feedback(Text::legacy("no pending request"));
        *ran = true;
        return;
    };
    let mut tpa: BTreeMap<String, TpaRequest> = load(&dir.join("tpa.json")).unwrap_or_default();
    let me = player.uuid.to_string();
    if tpa.remove(&me).is_some() {
        save(&dir.join("tpa.json"), &tpa);
        game.native_feedback(Text::legacy("you declined the request"));
    } else {
        game.native_feedback(Text::legacy("you have no pending teleport request"));
    }
    *ran = true;
}

// ---------------------------------------------------------------------------
// Back
// ---------------------------------------------------------------------------

fn back(game: &mut Game, player: &NativeServerPlayer, ran: &mut bool) {
    let Some(dir) = data_dir(game) else {
        game.native_feedback(Text::legacy("no record of where you were"));
        *ran = true;
        return;
    };
    let last: BTreeMap<String, Pos> = load(&dir.join("last.json")).unwrap_or_default();
    let Some(pos) = last.get(&player.uuid.to_string()) else {
        game.native_feedback(Text::legacy("no record of where you were"));
        *ran = true;
        return;
    };
    teleport_to(game, pos);
    game.native_feedback(Text::legacy("back to where you were"));
    *ran = true;
}

/// Remember the acting player's place, before a teleport moves them, so /back
/// has somewhere to go.
fn remember(game: &mut Game, player: &NativeServerPlayer) {
    if let Some(dir) = data_dir(game) {
        let mut last: BTreeMap<String, Pos> = load(&dir.join("last.json")).unwrap_or_default();
        last.insert(player.uuid.to_string(), Pos::of(game));
        save(&dir.join("last.json"), &last);
    }
}

// ---------------------------------------------------------------------------
// Nick
// ---------------------------------------------------------------------------

fn nick(game: &mut Game, player: &NativeServerPlayer, name: Option<&str>, ran: &mut bool) {
    let Some(name) = name else {
        game.native_feedback(Text::legacy("usage: /nick <name>"));
        *ran = true;
        return;
    };
    if name.is_empty() || name.chars().count() > 16 {
        game.native_feedback(Text::legacy("a nick is 1 to 16 characters"));
        *ran = true;
        return;
    };
    if game
        .native_server_players()
        .iter()
        .any(|p| p.name.eq_ignore_ascii_case(name) && p.uuid != player.uuid)
    {
        game.native_feedback(Text::legacy("that name is already taken"));
        *ran = true;
        return;
    }
    let ok = game.native_set_nickname(player.uuid, Some(name));
    let line = if ok {
        format!("your nick is now '{name}' (use /unnick to clear it)")
    } else {
        "I could not set your nick".to_owned()
    };
    game.native_feedback(Text::legacy(&line));
    *ran = true;
}

fn unnick(game: &mut Game, player: &NativeServerPlayer, ran: &mut bool) {
    let ok = game.native_set_nickname(player.uuid, None);
    let line = if ok {
        "your nick is cleared".to_owned()
    } else {
        "I could not clear your nick".to_owned()
    };
    game.native_feedback(Text::legacy(&line));
    *ran = true;
}

// ---------------------------------------------------------------------------
// Teleport and storage helpers
// ---------------------------------------------------------------------------

/// Move the acting player to a stored position, in their current dimension. A
/// place in another dimension is refused for now (a follow-on); the player is
/// left where they are and told.
fn teleport_to(game: &mut Game, pos: &Pos) {
    let in_same_dim = game.dim.id() == pos.dim;
    if !in_same_dim {
        game.native_feedback(Text::legacy(&format!(
            "that place is in {} — cross-dimension teleports are not on yet",
            pos.dim
        )));
        return;
    }
    let v = Vec3::new(pos.x, pos.y, pos.z);
    if let Some(p) = game.player.as_mut() {
        p.position = v;
        p.velocity = Vec3::new(0.0, 0.0, 0.0);
        p.fall_distance = 0.0;
        p.ground.on_ground = false;
    }
    game.previous_position = v;
}

/// The world's Essentials data folder (created on first write), beside the
/// world's own files so a backup carries it.
fn data_dir(game: &Game) -> Option<PathBuf> {
    Some(game.native_world_dir()?.join("essentials"))
}

/// Read a JSON value from `path`, or None when it is missing or unreadable.
fn load<T: serde::de::DeserializeOwned>(path: &Path) -> Option<T> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

/// Write `value` to `path` whole: serialize to a temp file in the same folder,
/// then rename it over the old one, so a crash leaves the old file or the new
/// one, never half of one.
fn save<T: serde::Serialize>(path: &Path, value: &T) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let tmp = path.with_file_name(format!(".{}.tmp", path.file_name().unwrap().to_string_lossy()));
    if let Ok(json) = serde_json::to_string_pretty(value) {
        let _ = std::fs::write(&tmp, json);
        let _ = std::fs::rename(&tmp, path);
    }
}
