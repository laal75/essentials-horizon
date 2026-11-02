# Essentials

The usual server commands for a Horizon world, as a native mod: **info**,
**quality-of-life actions**, **homes**, **warps**, **teleport requests**,
**nick** and **back** — the day-to-day commands a survival server wants, in the
style of EssentialsX and MoreCommands.

It builds on Horizon's server command hook points (`native_register_commands`,
`native_perform_command`, `native_perform_console_command`) plus the dispatcher
itself (so the window's agent, which runs commands straight through it, reaches
the mod too), so a stock Horizon client needs nothing: no client-side mod, no
resource pack, no plugin. The commands appear in every player's command tree
(with tab-completion, filtered by their operator level, exactly as vanilla's are)
and run on the server that simulates the world — single player, a LAN host, or a
dedicated server.

**Off by default in effect.** The mod adds only its commands; a world that never
runs one is untouched. Its data lives in the world's own folder
(`essentials/`), written whole to a temp file and renamed over the old one, so a
backup of the world carries it and a crash leaves the old file or the new one —
never half of one.

## Commands

| Command | Who | Does |
|---|---|---|
| `/whereami` | everyone | your position and dimension |
| `/online` | everyone | who is online |
| `/whois [player]` | everyone | a player's mode, level, health and position (yours if no name) |
| `/heal` | everyone | restore your health to full |
| `/feed` | everyone | restore your hunger and saturation |
| `/god` | everyone | toggle invulnerability |
| `/fly` | everyone | toggle flight |
| `/gmc` `/gms` `/gma` `/gmsp` | everyone | set your gamemode to creative / survival / adventure / spectator |
| `/sethome [name]` | everyone | set a home here (default name `home`) |
| `/home [name]` | everyone | teleport to one of your homes |
| `/delhome <name>` | everyone | delete one of your homes |
| `/homes` | everyone | list your homes |
| `/setwarp <name>` | level 2 | set a world warp here |
| `/warp [name]` | everyone | list warps, or teleport to one |
| `/delwarp <name>` | level 2 | delete a warp |
| `/warps` | everyone | list the world's warps |
| `/tpa <player>` | everyone | ask a player to come to you |
| `/tpahere <player>` | level 2 | ask a player to come to you (operator form) |
| `/tpaccept` | everyone | accept the pending request — you move to the requester |
| `/tpdeny` | everyone | decline the pending request |
| `/back` | everyone | teleport to where you were before your last home/warp/tpa |
| `/nick <name>` | everyone | show a display name in chat and the Tab list |
| `/unnick` | everyone | clear your display name |

A name vanilla already has (so `/msg`, `/list`, `/tp`, `/gamemode`, `/kill`,
`/give`…) is deliberately not taken: use vanilla's for those.

## Install

A native mod: install it from the Horizon launcher (Mods → install the `.hnmod`)
or drop it in your game's `horizon-native-mods/` folder, then build. It is pinned
to the Horizon commit it was made on (see Compatibility); the launcher builds it
on that commit.

## Verified

- Builds into the pinned Horizon: the bundle plans, applies and the whole
  `horizon-client` compiles clean (`cargo check --locked --offline`, no warnings).
- The command logic, the data store and the teleport are the code in
  `mixins/essentials/runtime.rs`, read straight against the pinned source.

## Compatibility

- Horizon: the commit in `mixins/essentials/horizon-mixin.toml`
  (`[compatibility] horizon_revision`).
- horizon-mixins: the version that plans/applies this bundle (0.6.0).
- Single player, LAN and dedicated server: the commands run where the world is
  simulated, through the server's command hook points.

## Not in it (yet)

- Moving the *requester* on `/tpa`: the native command hook can only act on the
  acting player, so both `/tpa` and `/tpahere` bring the target to you (you move
  on `/tpaccept`). A "you come to them" direction needs the server to run the
  teleport outside a player's hook — a follow-on.
- Cross-dimension teleports: a home or warp in another dimension is refused for
  now; the place is stored, so it works the moment cross-level travel is wired.
- `/seen`, `/vanish`, `/killall`, `/repair`, `/hat`, `/explode`, `/enchantmax`,
  the economy and market: the natural next commands, on the same hook points and
  store.

## Written with an agent

This mod was developed with an AI agent (OpenCode, Qwen3.8 27B) working from the
Horizon source and its native-mods docs. The agent wrote the code and docs; the
decisions, the pins and the trust in the native build are the maintainer's.

## Layout

- `mixins/essentials/` — the bundle: `horizon-mixin.toml`, the four small hook
  helpers (`register.rs`, `perform.rs`, `console.rs`, `command.rs`) and the mod's
  logic in `runtime.rs` (staged as `crate::game::essentials`).
- `docs/commands.md` — the command reference.
- `tools/package.py` — builds the install package (`essentials.hnmod`) from the
  bundle: the `horizon-native.json` manifest plus `mixins/essentials/`.
- `LICENSE` (MIT).

## Build and test

```sh
# plan + apply the bundle into a staged workspace, then compile the game
horizon-mixins plan  --workspace <horizon> --mods mixins --out plan.json
horizon-mixins apply --workspace <horizon> --mods mixins --out stage
( cd stage && cargo check --locked --offline -p horizon-client --no-default-features )

# build the install package
python3 tools/package.py            # -> essentials.hnmod
```
