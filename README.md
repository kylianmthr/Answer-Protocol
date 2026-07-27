*This project has been created as part of the 42 curriculum by thsykas, kmathuri.*

# TAP — The Answer Protocol

## Description

TAP is a shared-world retro text adventure: a small multiplayer MUD (Multi-User Dungeon) built around a TCP server implementing the **RFC 42TAP** line-based protocol, with two clients bringing the world to life — a GUI client (egui) and a CLI client.

Multiple players connect to the same persistent world, move between rooms, chat (globally, per room, or per group), fight hostile NPCs, complete quests, and manage their inventory. The server is written in Rust with Tokio (async, one task per connection), and the world (rooms, items, NPCs, quests) is loaded and validated from a YAML file at startup.

The goal of the project was to implement the full mandatory command set of RFC 42TAP, a coherent explorable world, a turn-based combat system, a simple quest system, and comprehensive structured server logging — while documenting every point where our implementation makes its own design choices.

## Instructions

The project uses three independent Rust crates (`backend/` for the server, `frontend/` for the GUI client, `client_cli/` for the CLI client), each built with Cargo, orchestrated by a root `Makefile`. See the **Building and Running** section below for the full list of commands.

Quick start:
```bash
make install
make run-server         # terminal 1
make run-client-gui     # terminal 2, or:
make run-client         # terminal 2 (CLI instead of GUI)
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

Every suggestion from the AI was reviewed, tested against the real server (not just read), and only kept once its behavior was verified. Some suggestions were rejected outright, and others were debated between us before being accepted: the group-identification scheme (arbitrary group name vs. the RFC's leader-name semantics) was implemented and tested both ways before we settled on the RFC-conformant version described below.

## Architecture

The server (`backend/`) uses Tokio for asynchronous I/O. `server.rs` runs a `TcpListener` accept loop; each accepted connection is handed to `tokio::spawn`, so every client is handled by its own independent async task (`client.rs::handle_client`).

Inside a client's task, `handle_commands` runs a `tokio::select!` loop that concurrently: (1) reads the next line from the socket and dispatches it, and (2) drains an `mpsc::UnboundedReceiver<String>` carrying asynchronous events (chat broadcasts, presence events, group invites, combat notifications, room refreshes, player-count updates) pushed by other tasks. This lets a client stay responsive to both its own commands and events triggered by other players at the same time.

Command dispatch is a **flat `match` on the command name** (inline handling, not a separate dispatcher/router struct) inside `handle_commands`. Each command's actual logic lives in its own module (`move_cmd.rs`, `look.rs`, `items.rs`, `attack.rs`, `quest.rs`, `group.rs`, `chat.rs`, `talk.rs`, `who.rs`), so `client.rs` stays a thin dispatcher that logs, calls the relevant module, and writes the response back.

World state is shared across all connections through a single `Arc<SharedState>`, cloned into every spawned task. `SharedState` groups related state behind independent `tokio::sync::Mutex`es (`players`, `world_data`, `world_state`, `groups`, `abuse`), so unrelated operations (e.g. reading world data vs. mutating player state) don't contend on the same lock.

Every server → client write goes through a single helper (`log_format` in `client.rs`), which writes the line to the socket **and** logs it, guaranteeing that logging and network output can never drift apart.

**World validation at startup.** The YAML file is parsed and then cross-checked before the listener is opened: every exit must point at an existing room, every item and NPC referenced by a room must exist in the `items` / `npcs` tables, every quest `objective` and `reward` must be a declared item, and `initial_room` must exist. If any reference is dangling, the server **panics with an explicit message and never starts** — we preferred a hard failure over booting a half-broken world, since a silent fallback would produce protocol errors that look like server bugs during play.

**Clients.** The CLI (`client_cli/`) is a raw pass-through (see *Building and Running*). The GUI (`frontend/`, egui) displays the current room's description, items, NPCs and exits with live updates, shows the inventory, exposes buttons for every protocol action (`LOOK`, `MOVE`, `TAKE`, `DROP`, `TALK`, `ATTACK`, `STATUS`, `QUEST`, `QUESTS`, `WHO`, `GROUP`, `QUIT`, plus `DEFEND`/`FLEE`/`USE`), accepts both item IDs and display names in its input fields, refreshes the room view automatically after `TAKE`/`DROP` and after any room event pushed by the server, keeps the chat view (Global / Room / Group tabs) separate from the raw protocol log view, displays NPC dialogue returned by `TALK`, and shows two live counters: players in the current room and players on the server.

## Protocol Implementation

The server strictly follows RFC 42TAP for message framing (TCP, UTF-8, one `\n`-terminated line per message), the greeting (`OK hello proto=1` sent on connection), the core command set (`CONNECT`, `LOOK`, `MOVE`, `CHAT`, `TAKE`, `DROP`, `INVENTORY`, `TALK`, `ATTACK`, `STATUS`, `QUEST`, `QUESTS`, `WHO`, `GROUP`, `QUIT`), the `EVT`/`ERR` message shapes, and the standard error codes defined in RFC section 8.2 (`201`, `301`, `401`, `402`, `404`, `405`, `406`, `900`, `901`).

**Groups follow the RFC's leader-name semantics.** `GROUP CREATE` registers a group keyed by the creator's username, and `GROUP INVITE` / `GROUP JOIN` / `GROUP LEAVE` resolve a group by its leader's name, as described in RFC 5.3.1/5.3.3. We initially implemented arbitrary group names (which would have allowed one player to lead several groups at once), but since CLI and GUI clients must remain interchangeable between groups working on this project in parallel, a client written against the RFC would send `GROUP JOIN <leader>` and fail against a name-keyed server. Conformance won over the extra flexibility, and the group is destroyed when its leader disconnects.

**Item resolution.** `TAKE` and `DROP` accept either the item ID (`item.herbs`) or the display name (`Herbs`), matched case-insensitively, and **multi-word display names are fully supported**: the rest of the line after the command word is taken as a single item designator, so `TAKE Frothy Ale` and `DROP item.frothy_ale` both resolve to the same unique instance. Items exist as single instances in the world state: taking one removes it from the room (no duplication) and dropping it makes it immediately visible to every other player in that room.

Documented extensions:

- **Additional error codes beyond the RFC's base table**, all still respecting the ABNF's `3DIGIT` error-code requirement: `400` for malformed/incomplete commands not covered by the RFC (`MISSING_NPC_NAME`, `MISSING_ITEM_NAME`, `UNKNOWN_COMMAND`, `UNKNOWN_SCOPE`, `UNKNOWN_GROUP_COMMAND`), and `407`/`409` for combat/inventory edge cases (`NOT_IN_COMBAT`, `ITEM_NOT_USABLE`) introduced by our combat and item-use extensions. A client that ignores unknown codes and only reads the `ERR` prefix still behaves correctly.
- **`EVT STATS players=<count>`** (RFC section 6.2.4) is broadcast to all connected players whenever a player connects or disconnects, so every client can keep a live server-wide player counter without polling.

## Combat System

Combat is turn-based and reactive: a player initiates combat with `ATTACK <npc>` against a hostile NPC in their room. Each player tracks a `combat_turn` (`Player`/`Enemy`) and a `combat_target`. Damage is randomized per hit using an XORShift-based `roll(min, max)` in the range 20–30 for both player and NPC attacks (`PLAYER_MIN/MAX` and `NPC_MIN/MAX` in `attack.rs`).

**Initiative order:** the player always acts first, and the enemy's turn is resolved on the player's *next* combat command rather than by a background timer — this keeps the server fully request/response-driven, so a client never receives combat damage it did not ask for, and the protocol stays strictly synchronous outside of `EVT` broadcasts.

- **`ATTACK`**: while it is the player's turn, deals 20–30 damage to the NPC and passes the turn to the enemy; the *next* `ATTACK` call resolves the enemy's counter-attack instead.
- **`DEFEND`**: reduces the incoming hit by 50% (`DEFEND_DAMAGE_PERCENT`) and ripostes back at the NPC for 50% of the *full* roll (`RIPOSTE_PERCENT`) — a defensive trade-off between survivability and damage output.
- **`FLEE`**: has a 50% chance (`FLEE_CHANCE_PERCENT`) to succeed; on success the player escapes to a random adjacent room and combat ends; on failure the enemy gets a free hit.
- **`USE_ITEM` / `USE <item>`**: consumes a consumable inventory item with a `heal` value (defined per-item in the world file) to restore HP, capped at `MAX_HP`.
- **`STATUS`**: reports `hp`, `max_hp`, a derived label (`healthy`/`wounded`/`dead`), and `in_combat`.

Enemy HP is **not hard-coded**: it comes from each NPC's `stats.hp` in the world file, so different enemy types can have different difficulty without touching the code. In the current world, `npc.necromancien` has **100 HP** — deliberately equal to a player's maximum, so a straight `ATTACK`-only exchange is roughly even and `DEFEND`/`USE` are what actually tip the fight.

Players start at **100 HP** (`MAX_HP`). On reaching 0 HP through any combat path, the player respawns at the world's `initial_room` with **50 HP** (`RESPAWN_HP`), and the defeated NPC's HP is reset to full. Every combat outcome (hit, victory, defeat, flee, item use) is logged as a `COMBAT_RESULT` event and, for room-visible outcomes (victory, defeat, flee, parry), broadcast to the room via `EVT ROOM COMBAT`.

## Quest System

Quest-giver NPCs optionally carry a single `quest` definition in the world file (`id`, `name`, `description`, `objective` item, `reward` item). Progression is driven entirely by repeated `QUEST <npc>` calls:

1. First `QUEST <npc>` call: the quest is added to the player's quest list with status `active`.
2. Subsequent `QUEST <npc>` calls: if the player's inventory contains the `objective` item, it is removed and replaced by the `reward` item, and the quest status becomes `completed`. If the objective isn't held yet, the quest simply stays `active`.
3. Once `completed`, calling `QUEST <npc>` again on the same NPC returns `ERR 406 NO_QUEST_AVAILABLE`.

Completion validation is therefore inventory-based and re-checked on every call, which means the objective item can be obtained in any order (picked up before ever talking to the giver, taken from the ground after another player dropped it, etc.) — there is no hidden per-quest counter that could drift out of sync with the actual inventory.

`QUESTS` lists every quest the player has ever accepted, with a `progress` field (`"1/1"` once the objective is held or the quest is completed, `"0/1"` otherwise). Every activation and completion is logged as a `QUEST_PROG` event. There is no quest chaining or prerequisite system — each NPC exposes at most one independent quest.

## World Design

The world (`backend/test.yaml`) has **8 rooms**: `loc.tavern`, `loc.square`, `loc.shop`, `loc.forest`, `loc.library`, `loc.observatory`, `loc.swamp`, `loc.crypt`.

- **Loop**: `loc.square` ↔ `loc.shop` ↔ `loc.forest` ↔ `loc.square` forms a full circuit, so exploration never dead-ends into a straight line.
- **Branches**: `loc.square` → `loc.library` → `loc.observatory` is an optional dead-end branch off the loop; `loc.forest` → `loc.swamp` → `loc.crypt` is a second one.

**NPCs** (6 total, covering the 3 required roles):
- *Dialogue-only*: `npc.guard` (square), `npc.dryade` (forest), `npc.astrologue` (observatory).
- *Quest-givers*: `npc.taverniere` (tavern, quest `fetch_herbs`), `npc.marchand` (shop, quest `fetch_wood`).
- *Hostile enemy*: `npc.necromancien` (crypt, 100 HP) — the only attackable NPC in the current world.

**Items**: 13 items total, all obtainable, spread across every room; two of them (`item.gold_coin`, `item.potion`) exist only as quest rewards and are never placed directly in a room. Each item declares an `id` and a `name`, and both are accepted by `TAKE`/`DROP` (see *Protocol Implementation*).

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
- `REFRESH` — a real-time room-state refresh pushed to a client because *another* player changed something visible in their room (movement, take/drop, combat outcome). This is what keeps the GUI room view live without polling, and logging it makes it possible to verify from the server side that every affected client was actually notified.
- `QUEST_PROG` — quest activation and completion.
- `COMBAT_RESULT` — every combat outcome (hit, victory, defeat, flee, item use), at `WARN` for player defeats.
- `COMMAND_FLOOD` — a player exceeding 10 commands within a 5-second sliding window (`flood_systeme.rs`).
- `RAPID_CONNECT` — an IP address exceeding 5 connection attempts within a 5-second sliding window.

Logging is fire-and-forget on stdout with no lock held across a write and no I/O to a remote sink, so it stays off the critical path of command handling.

Logs are written to stdout and can be redirected to a file (`make run-server > server.log`) or piped for monitoring, e.g.:
```bash
grep '"level":"WARN"' server.log       # abuse patterns + rejected requests
grep '"event":"COMBAT_RESULT"' server.log
grep -E '"event":"(COMMAND_FLOOD|RAPID_CONNECT)"' server.log
```

## Group Contributions

- **thsykas**: structured server logging system (`logs_format.rs`, wiring every command/response/disconnect through it), the `REFRESH` real-time notification path and its logging, the flood/abuse detection module, the world map layout and loop/branch design, the world-validation pass at startup, GUI player counters (`EVT STATS`), and general protocol-conformance auditing against RFC 42TAP.
- **kmathuri**: initial server and client skeletons, the item/quest/group/talk command modules, item resolution by ID *and* display name (including multi-word names), the GUI foundation (login, room views, chat panel, action buttons), and the turn-based combat system (`ATTACK`/`DEFEND`/`FLEE`/`USE_ITEM`) together with the RFC-format alignment pass (`LOOK`/`MOVE`/`WHO`/`TALK` response shapes, structured error codes).

The move of `GROUP CREATE`/`JOIN` to the RFC's leader-name semantics was done jointly, since it touched both the group module and the GUI's group panel.

Both members worked across the server (`backend/`) and the GUI client (`frontend/`) rather than splitting strictly along a server/GUI line.

## Building and Running

Build system: **Cargo**, orchestrated by the root `Makefile`.

| Command | Effect |
|---|---|
| `make install` | Builds (`cargo build`) the `backend`, `frontend`, and `client_cli` crates, fetching dependencies. |
| `make run-server` | Runs the server: `cd backend && cargo run <PORT> <MAP_PATH>` (defaults: `PORT=2000`, `MAP_PATH=test.yaml`). |
| `make run-client-gui` | Runs the GUI client: `cd frontend && cargo run <PORT>`. |
| `make run-client` | Runs the CLI client: `cd client_cli && cargo run <PORT>`. Prompts interactively for the server address (e.g. `127.0.0.1:2000`), then behaves as a direct pass-through to the server. |
| `make lint` | Runs `cargo clippy` on all three crates. |
| `make clean` | Removes all three crates' `target/` build directories. |
| `make all` | Pre-builds and starts the server and the GUI client together (server first, with a short delay before the GUI connects, to avoid a connection-refused race on a cold build). |

**CLI Command Interface Choice:** the CLI (`client_cli/`) implements the first option offered by the subject — it sends the user's input **directly** to the server using raw RFC 42TAP syntax (e.g. typing `CONNECT alice` or `MOVE north` verbatim), with no translation layer. Reading from the socket and reading from stdin run as two independent Tokio tasks (`tokio::select!`), so the client keeps printing incoming `EVT`/`OK`/`ERR` lines in real time even while idle on user input.

## Testing

There is no automated test suite; the project was tested manually against a running server, in several complementary ways:

**Protocol-level testing (`nc`)** — connect directly and send raw commands to verify exact response formats and logs, e.g.:
```bash
printf 'CONNECT alice\nLOOK\nMOVE south\nTAKE Herbs\nQUEST npc.taverniere\nWHO\nQUIT\n' | nc localhost 2000
```
The server's stdout is checked in parallel to confirm the matching `COMMAND`/`RESPONSE` log pairs (and `TAKEN`/`QUEST_PROG`/`COMBAT_RESULT`/`REFRESH` where relevant) appear. The same sequence is replayed with `TAKE item.herbs` and with a multi-word display name to confirm both resolution paths hit the same instance.

**World validation** — start the server with a deliberately broken YAML (an exit pointing at a nonexistent room, or a quest rewarding an undeclared item): the server must panic with an explicit message and never open the listening socket.

**Multiplayer / GUI testing** — run `make run-server` once, then launch several `make run-client-gui` instances against the same port. This verifies: room presence events (`EVT ROOM PRESENCE ENTER/LEAVE`) appear on the other clients when a player moves, an item taken by one player disappears from the other clients' room view without any manual refresh, chat messages reach the right scope (room/group/global), the server-wide and per-room player counters update on every connect/disconnect, and group invites/joins/leaves are visible to all members.

**Group semantics** — with two clients: `alice` sends `GROUP CREATE`, `bob` sends `GROUP JOIN alice` (the leader's name, not an arbitrary label), then `CHAT GROUP ...` must reach only group members; `alice` disconnecting must dissolve the group.

**CLI responsiveness testing** — with one `make run-client` connected and idle (no input typed), connect a second client (GUI, CLI, or `nc`) to the same server: the first CLI must print the resulting `EVT STATS players=<n>` (and any room presence event, if in the same room) immediately, without needing to press Enter itself — confirming the read and write loops are truly independent.

**Combat and quests** — from the GUI or via `nc`, move to `loc.crypt` and `ATTACK npc.necromancien` repeatedly to exercise the full turn-based loop (including `DEFEND`, `FLEE`, `USE`, and dying/respawning at 0 HP); talk to `npc.taverniere` or `npc.marchand`, pick up their objective item, and call `QUEST <npc>` again to confirm reward distribution and the `completed` status, then a third time to confirm `ERR 406 NO_QUEST_AVAILABLE`.