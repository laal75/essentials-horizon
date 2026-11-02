# Essentials commands

The commands Essentials adds, and the details a server operator would want.

## Info

### `/whereami`
Your feet position and dimension, e.g. `You are at [416, 76, -356] in minecraft:overworld`.

### `/online`
The players on the server, in the order they joined: `Online (2): laal75, Player24518`.
Also works from the dedicated server's console.

### `/whois [player]`
A player's gamemode, operator level, health and position. With no argument, it
reports the player who ran it. `whois Alex — creative (level 4), 20 hp, at [0, 64, 0] in minecraft:overworld`.

## Quality-of-life actions

These act on the player who runs them.

- **`/heal`** — health back to full (clamped to your maximum).
- **`/feed`** — hunger and saturation back to full.
- **`/god`** — toggle invulnerability (like creative's).
- **`/fly`** — toggle the ability to fly, and your current flight state.
- **`/gmc` `/gms` `/gma` `/gmsp`** — set your gamemode to creative, survival,
  adventure or spectator. (Vanilla's `/gamemode` does the same; these are the
  short forms EssentialsX and MoreCommands use, since `/gamemode` is already
  taken by vanilla.)

## Homes

Each player keeps their own homes, in the world's `essentials/homes.json`.

- **`/sethome [name]`** — set a home at your current position. The default name
  is `home`; give a name for more than one.
- **`/home [name]`** — teleport to one of your homes. No name goes to `home` (or
  your first, if you have no `home`).
- **`/delhome <name>`** — delete one of your homes.
- **`/homes`** — list your homes.

## Warps

World-wide, set by an operator (level 2), in `essentials/warps.json`.

- **`/setwarp <name>`** (level 2) — set a warp at your current position.
- **`/warp [name]`** — no name lists the world's warps; a name teleports there.
- **`/delwarp <name>`** (level 2) — delete a warp.
- **`/warps`** — list the world's warps. Also works from the console.

## Teleport requests

Stored in `essentials/tpa.json` until answered.

- **`/tpa <player>`** — ask a player to come to you. They get
  `<you> asks you to come to them (/tpaccept or /tpdeny)`.
- **`/tpahere <player>`** (level 2) — the operator form of the same request.
- **`/tpaccept`** — accept the pending request: **you move to the requester**.
- **`/tpdeny`** — decline it.

Because the native command hook can only act on the player whose command is
running, `/tpaccept` moves the accepter to the requester (both `/tpa` and
`/tpahere` bring the target to the one who asked). A "you go to them" direction
is a follow-on.

## Back

- **`/back`** — teleport to where you were before your last `/home`, `/warp` or
  `/tpaccept`. Essentials remembers your place before each of those, in
  `essentials/last.json`.

## Nick

A display name over your real one, shown in chat and the Tab list; hovering the
shown name gives the real one, which stays in the log, commands, selectors and
the server's events (Horizon's `native_set_nickname`).

- **`/nick <name>`** — set it (1 to 16 characters, not already taken).
- **`/unnick`** — clear it.

## Console

From the dedicated server's console (or `horizon-server console`): `/online` and
`/warps` list, and `/warp <name>` teleports. The commands that need a player
(`heal`, `home`, `tpa`, …) answer that they want one run in-game.

## Data

Everything is in the world's `essentials/` folder (created on first use), so a
backup of the world carries it:

| File | Holds |
|---|---|
| `homes.json` | each player's homes (name → dimension + position) |
| `warps.json` | the world's warps (name → dimension + position) |
| `tpa.json` | pending teleport requests (target → requester) |
| `last.json` | each player's last remembered place (for `/back`) |

Each file is written whole to a temp file and renamed over the old one, so a
crash leaves the old file or the new one, never half of one.
