*This project has been created as part of the 42 curriculum by thsykas, kmathuri.*

# TAP — The Answer Protocol

## Description

TAP is a shared-world retro text adventure: a small multiplayer MUD (Multi-User Dungeon) built around a TCP server implementing the **RFC 42TAP** line-based protocol, with two clients bringing the world to life — a GUI client (egui) and, eventually, a CLI client.

Multiple players connect to the same persistent world, move between rooms, chat (globally, per room, or per group), fight hostile NPCs, complete quests, and manage their inventory. The server is written in Rust with Tokio (async, one task per connection), and the world (rooms, items, NPCs, quests) is loaded from a YAML file at startup.

The goal of the project was to implement the full mandatory command set of RFC 42TAP, a coherent explorable world, a turn-based combat system, a simple quest system, and comprehensive structured server logging — while documenting every point where our implementation makes its own design choices.

## Instructions

The project uses two independent Rust crates (`backend/` for the server, `frontend/` for the GUI client), each built with Cargo, orchestrated by a root `Makefile`. See the **Building and Running** section below for the full list of commands.

Quick start:
```bash
make install
make run-server        # terminal 1
make run-client-gui     # terminal 2
```

## Resources

- [RFC 42TAP](./protocol-rfc.html) — the protocol specification this project implements.
- [Tokio documentation](https://tokio.rs/) — async runtime used by the server.
- [egui / eframe documentation](https://docs.rs/egui/) — GUI toolkit used by the client.
- Classic MUD references: *Multi-User Dungeon* (Roy Trubshaw & Richard Bartle, 1978) as the historical inspiration for shared-world text games.
- [serde / serde_yaml documentation](https://serde.rs/) — world data (de)serialization.

**AI usage:** Claude (Claude Code) was used throughout this project as a pair-programming and review tool, specifically for:
- Auditing the codebase against `tap.pdf` and RFC 42TAP section by section, and reporting non-conformities (missing logging fields, protocol format mismatches, missing error codes, etc.).
- Resolving a multi-file `git merge` conflict between two long-diverged branches (RFC-format fixes on `main` vs. the structured-logging work on `timestamp_log`), by combining both sets of changes function by function.
- Helping design and iterate on the structured logging system, the flood/abuse detection module, and the server-wide/room player counters (`EVT STATS`), with each addition compiled and tested live (via `nc`) before being accepted.
- Diagnosing and explaining Rust compiler errors (borrow/ownership mismatches, missing struct fields, duplicate imports) introduced while implementing the above by hand.

Every suggestion from the AI was reviewed, tested against the real server (not just read), and only kept once its behavior was verified — several early suggestions (e.g. aligning `GROUP CREATE`/`GROUP JOIN` strictly to the RFC's leader-name semantics) were deliberately rejected in favor of documenting the deviation instead, after weighing the trade-off ourselves.

## Architecture

The server (`backend/`) uses Tokio for asynchronous I/O. `server.rs` runs a `TcpListener` accept loop; each accepted connection is handed to `tokio::spawn`, so every client is handled by its own independent async task (`client.rs::handle_client`).

Inside a client's task, `handle_commands` runs a `tokio::select!` loop that concurrently: (1) reads the next line from the socket and dispatches it, and (2) drains an `mpsc::UnboundedReceiver<String>` carrying asynchronous events (chat broadcasts, presence events, group invites, combat notifications, player-count updates) pushed by other tasks. This lets a client stay responsive to both its own commands and events triggered by other players at the same time.

Command dispatch is a **flat `match` on the command name** (inline handling, not a separate dispatcher/router struct) inside `handle_commands`. Each command's actual logic lives in its own module (`move_cmd.rs`, `look.rs`, `items.rs`, `attack.rs`, `quest.rs`, `group.rs`, `chat.rs`, `talk.rs`, `who.rs`), so `client.rs` stays a thin dispatcher that logs, calls the relevant module, and writes the response back.

World state is shared across all connections through a single `Arc<SharedState>`, cloned into every spawned task. `SharedState` groups related state behind independent `tokio::sync::Mutex`es (`players`, `world_data`, `world_state`, `groups`, `abuse`), so unrelated operations (e.g. reading world data vs. mutating player state) don't contend on the same lock.

Every server → client write goes through a single helper (`log_format` in `client.rs`), which writes the line to the socket **and** logs it, guaranteeing that logging and network output can never drift apart.

## Protocol Implementation

The server strictly follows RFC 42TAP for message framing (TCP, UTF-8, one `\n`-terminated line per message), the core command set (`CONNECT`, `LOOK`, `MOVE`, `CHAT`, `TAKE`, `DROP`, `INVENTORY`, `TALK`, `ATTACK`, `STATUS`, `QUEST`, `QUESTS`, `WHO`, `GROUP`, `QUIT`), the `EVT`/`ERR` message shapes, and the standard error codes defined in RFC section 8.2 (`201`, `301`, `401`, `402`, `404`, `405`, `406`, `900`, `901`).

Documented deviations and extensions:

- **`GROUP CREATE` / `GROUP JOIN` identify groups by an arbitrary name chosen by the creator, not by the leader's username as RFC 5.3.1/5.3.3 suggest.** We chose this deliberately: it lets a player create more than one independent group over a session and gives groups memorable names (e.g. `raid1`), whereas the RFC's leader-name scheme limits a player to leading a single group for their entire connection (since the group ID would be tied 1:1 to their username). `GROUP INVITE`, `GROUP JOIN`, and `GROUP LEAVE` are otherwise unaffected — they resolve a group purely by whatever key was used at creation, so the mechanism works identically either way.
- **Additional error codes beyond the RFC's base table**, all still respecting the ABNF's `3DIGIT` error-code requirement: `400` for malformed/incomplete commands not covered by the RFC (`MISSING_NPC_NAME`, `MISSING_ITEM_NAME`, `UNKNOWN_COMMAND`, `UNKNOWN_SCOPE`, `UNKNOWN_GROUP_COMMAND`), and `407`/`409` for combat/inventory edge cases (`NOT_IN_COMBAT`, `ITEM_NOT_USABLE`) introduced by our combat and item-use extensions.
- **`EVT STATS players=<count>`** (RFC section 6.2.4) is broadcast to all connected players whenever a player connects or disconnects, so every client can keep a live server-wide player counter without polling.
- **Item name resolution is an exact, case-sensitive match** against either the canonical ID or the display name (`TAKE item.herbs` and `TAKE Herbes Médicinales` both work). The RFC only says resources "MAY be identified using either" an ID or a display name, without specifying case-sensitivity, so this is a clarification rather than a deviation.

## Combat System

Combat is turn-based and reactive: a player initiates combat with `ATTACK <npc>` against a hostile NPC in their room. Each player tracks a `combat_turn` (`Player`/`Enemy`) and a `combat_target`. Damage is randomized per hit using an XORShift-based `roll(min, max)` in the range 20–30 for both player and NPC attacks (`PLAYER_MIN/MAX` and `NPC_MIN/MAX` in `attack.rs`).

- **`ATTACK`**: while it is the player's turn, deals 20–30 damage to the NPC and passes the turn to the enemy; the *next* `ATTACK` call resolves the enemy's counter-attack instead (this makes the enemy's action always driven by the player's own next command rather than a background timer, keeping the server fully request/response-driven).
- **`DEFEND`**: reduces the incoming hit by 50% (`DEFEND_DAMAGE_PERCENT`) and ripostes back at the NPC for 50% of the *full* roll (`RIPOSTE_PERCENT`) — a defensive trade-off between survivability and damage output.
- **`FLEE`**: has a 50% chance (`FLEE_CHANCE_PERCENT`) to succeed; on success the player escapes to a random adjacent room and combat ends; on failure the enemy gets a free hit.
- **`USE_ITEM` / `USE <item>`**: consumes a consumable inventory item with a `heal` value (defined per-item in the world file) to restore HP, capped at `MAX_HP`.
- **`STATUS`**: reports `hp`, `max_hp`, a derived label (`healthy`/`wounded`/`dead`), and `in_combat`.

Players start at **100 HP** (`MAX_HP`). On reaching 0 HP through any combat path, the player respawns at the world's `initial_room` with **50 HP** (`RESPAWN_HP`), and the defeated NPC's HP is reset to full. Every combat outcome (hit, victory, defeat, flee, item use) is logged as a `COMBAT_RESULT` event and, for room-visible outcomes (victory, defeat, flee, parry), broadcast to the room via `EVT ROOM COMBAT`.

## Quest System

Quest-giver NPCs optionally carry a single `quest` definition in the world file (`id`, `name`, `description`, `objective` item, `reward` item). Progression is driven entirely by repeated `QUEST <npc>` calls:

1. First `QUEST <npc>` call: the quest is added to the player's quest list with status `active`.
2. Subsequent `QUEST <npc>` calls: if the player's inventory contains the `objective` item, it is removed and replaced by the `reward` item, and the quest status becomes `completed`. If the objective isn't held yet, the quest simply stays `active`.
3. Once `completed`, calling `QUEST <npc>` again on the same NPC returns `ERR 406 NO_QUEST_AVAILABLE`.

`QUESTS` lists every quest the player has ever accepted, with a `progress` field (`"1/1"` once the objective is held or the quest is completed, `"0/1"` otherwise). Every activation and completion is logged as a `QUEST_PROG` event. There is no quest chaining or prerequisite system — each NPC exposes at most one independent quest.

## World Design

The world (`backend/test.yaml`) has **8 rooms**: `loc.tavern`, `loc.square`, `loc.shop`, `loc.forest`, `loc.library`, `loc.observatory`, `loc.swamp`, `loc.crypt`.

- **Loop**: `loc.square` ↔ `loc.shop` ↔ `loc.forest` ↔ `loc.square` forms a full circuit, so exploration never dead-ends into a straight line.
- **Branches**: `loc.square` → `loc.library` → `loc.observatory` is an optional dead-end branch off the loop; `loc.forest` → `loc.swamp` → `loc.crypt` is a second one.

**NPCs** (6 total, covering the 3 required roles):
- *Dialogue-only*: `npc.guard` (square), `npc.dryade` (forest), `npc.astrologue` (observatory).
- *Quest-givers*: `npc.taverniere` (tavern, quest `fetch_herbs`), `npc.marchand` (shop, quest `fetch_wood`).
- *Hostile enemy*: `npc.necromancien` (crypt) — the only attackable NPC in the current world.

**Items**: 13 items total, all obtainable, spread across every room; two of them (`item.gold_coin`, `item.potion`) exist only as quest rewards and are never placed directly in a room.

## Server Logging

All server-side logs are single-line JSON objects printed to stdout via `logs_format::log_output(level, event, data)`, with the shape:
```json
{"timestamp":"<RFC3339>","level":"INFO|WARN|ERROR","event":"<EVENT_NAME>","data":{...}}
```

Event types emitted:
- `IP` — new TCP connection accepted (with the peer address).
- `COMMAND` — every line received from a client (player, command, arguments), including during authentication.
- `RESPONSE` — every line sent back to a client (`OK`/`ERR`/`EVT`), logged at `WARN` when it starts with `ERR`, `INFO` otherwise.
- `DISCONNECT` — logged once per connection close, with `player`, `ip`, and `reason` (`"quit"`, `"eof"`, or `"error"`).
- `READ_ERROR` — a socket read failure, at `ERROR` level.
- `TAKEN` / `DROPED` — item movements between a room and a player's inventory.
- `QUEST_PROG` — quest activation and completion.
- `COMBAT_RESULT` — every combat outcome (hit, victory, defeat, flee, item use), at `WARN` for player defeats.
- `COMMAND_FLOOD` — a player exceeding 10 commands within a 5-second sliding window (`flood_systeme.rs`).
- `RAPID_CONNECT` — an IP address exceeding 5 connection attempts within a 5-second sliding window.

Logs are written to stdout and can be redirected to a file (`make run-server > server.log`) or piped for monitoring, e.g.:
```bash
grep '"level":"WARN"' server.log      # abuse patterns + rejected requests
grep '"event":"COMBAT_RESULT"' server.log
```

## Group Contributions

- **thsykas**: structured server logging system (`logs_format.rs`, wiring every command/response/disconnect through it), the flood/abuse detection module, the world map layout and loop/branch design, GUI player counters (`EVT STATS`), and general protocol-conformance auditing against RFC 42TAP.
- **kmathuri**: initial server and client skeletons, the item/quest/group/talk command modules, the GUI foundation (login, room views, chat panel, action buttons), and the turn-based combat system (`ATTACK`/`DEFEND`/`FLEE`/`USE_ITEM`) together with the RFC-format alignment pass (`LOOK`/`MOVE`/`WHO`/`TALK` response shapes, structured error codes).

Both members worked across the server (`backend/`) and the GUI client (`frontend/`) rather than splitting strictly along a server/GUI line.

## Building and Running

Build system: **Cargo**, orchestrated by the root `Makefile`.

| Command | Effect |
|---|---|
| `make install` | Builds (`cargo build`) both the `backend` and `frontend` crates, fetching dependencies. |
| `make run-server` | Runs the server: `cd backend && cargo run <PORT> <MAP_PATH>` (defaults: `PORT=2000`, `MAP_PATH=test.yaml`). |
| `make run-client-gui` | Runs the GUI client: `cd frontend && cargo run <PORT>`. |
| `make run-client` | Connects to the server with `nc localhost <PORT>` — a dedicated CLI client is not implemented yet, so this is currently the way to interact with the server as raw text (send lines exactly matching RFC 42TAP syntax, e.g. `CONNECT alice`). |
| `make lint` | Runs `cargo clippy` on both crates. |
| `make clean` | Removes both crates' `target/` build directories. |
| `make all` | Starts the server and the GUI client together. |

## Testing

There is no automated test suite; the project was tested manually against a running server, in two complementary ways:

**Protocol-level testing (`nc`)** — connect directly and send raw commands to verify exact response formats and logs, e.g.:
```bash
printf 'CONNECT alice\nLOOK\nMOVE south\nTAKE item.herbs\nQUEST npc.taverniere\nWHO\nQUIT\n' | nc localhost 2000
```
The server's stdout is checked in parallel to confirm the matching `COMMAND`/`RESPONSE` log pairs (and `TAKEN`/`QUEST_PROG`/`COMBAT_RESULT` where relevant) appear.

**Multiplayer / GUI testing** — run `make run-server` once, then launch several `make run-client-gui` instances against the same port. This verifies: room presence events (`EVT ROOM PRESENCE ENTER/LEAVE`) appear on the other clients when a player moves, chat messages reach the right scope (room/group/global), the server-wide player counter updates on every connect/disconnect, and group invites/joins/leaves are visible to all members.

**Combat and quests** — from the GUI or via `nc`, move to `loc.crypt` and `ATTACK npc.necromancien` repeatedly to exercise the full turn-based loop (including `DEFEND`, `FLEE`, and dying/respawning at 0 HP); talk to `npc.taverniere` or `npc.marchand`, pick up their objective item, and call `QUEST <npc>` again to confirm reward distribution and the `completed` status.
