<div align="center">

# Azeroth Survivors

**An independent WoW survival-game implementation on a from-scratch WoW 1.12.1
client engine (Rust + Bevy), inspired by [Aethus’s Azeroth Survivors](https://wow.aethus.pl/).**

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue.svg)](LICENSE-MIT)

</div>

This repository implements the design in [our intentions](docs/intentions/) and
[the Survivors specification](docs/SURVIVORS.md) on the benilla/wenilla engine. It is
inspired by Aethus’s game; it is not a build of that game's source code. Implementation
is ongoing; the [acceptance contract](docs/SURVIVORS-IMPLEMENTATION.md) tracks the intended
scope, including content and screens that are not complete yet.

Pick a class and a battleground, then survive: WASD to move, your abilities cast themselves,
enemies swarm in waves, you level up and pick upgrades, and a boss shows up every couple of
minutes. **No server, no login, no account** — fully offline and self-contained.

## Interface gallery

Actual checkout captures from **2026-10-24**, at a **1440 × 900 logical viewport**
(2880 × 1800 Retina render, UI scale 1.171875), exported to 1440 × 900 JPEGs.
These show the current implementation, including unfinished pages; they are not the
[intended designs](docs/intentions/) or proof of responsiveness at every resolution.
See [capture instructions](docs/SURVIVORS-DEVELOPMENT.md#readme-captures) to refresh them.

<table>
  <tr>
    <td width="50%"><strong>Front door</strong><br><img src="docs/screenshots/current-front-door.jpg" alt="Front door: Class, hero and twelve intended destinations; incomplete content is visibly locked."><br>Class, hero and twelve intended destinations; incomplete content is visibly locked.</td>
    <td width="50%"><strong>Character</strong><br><img src="docs/screenshots/current-character.jpg" alt="Character: Current legacy character controls; the intended paper-doll layout is unfinished."><br>Current legacy character controls; the intended paper-doll layout is unfinished.</td>
  </tr>
  <tr>
    <td width="50%"><strong>Collection</strong><br><img src="docs/screenshots/current-collection.jpg" alt="Collection: Slot and rarity catalog, item details and equipment controls on a validation account."><br>Slot and rarity catalog, item details and equipment controls on a validation account.</td>
    <td width="50%"><strong>Drop Tables</strong><br><img src="docs/screenshots/current-drop-tables.jpg" alt="Drop Tables: Current text view; missing content and layout work remain."><br>Current text view; missing content and layout work remain.</td>
  </tr>
  <tr>
    <td width="50%"><strong>Codex</strong><br><img src="docs/screenshots/current-codex.jpg" alt="Codex: Spell and item reference view; footer spacing still needs cleanup."><br>Spell and item reference view; footer spacing still needs cleanup.</td>
    <td width="50%"><strong>Talents</strong><br><img src="docs/screenshots/current-talents.jpg" alt="Talents: Warrior tree presentation; this image does not imply every talent effect is implemented."><br>Warrior tree presentation; this image does not imply every talent effect is implemented.</td>
  </tr>
  <tr>
    <td width="50%"><strong>Achievements</strong><br><img src="docs/screenshots/current-achievements.jpg" alt="Achievements: Achievement categories and account summary."><br>Achievement categories and account summary.</td>
    <td width="50%"><strong>Rewards</strong><br><img src="docs/screenshots/current-rewards.jpg" alt="Rewards: Current reward list and cosmetic categories."><br>Current reward list and cosmetic categories.</td>
  </tr>
  <tr>
    <td width="50%"><strong>Appearance</strong><br><img src="docs/screenshots/current-appearance.jpg" alt="Appearance: Legacy appearance picker; overlapping controls and old zone choices still need replacement."><br>Legacy appearance picker; overlapping controls and old zone choices still need replacement.</td>
    <td width="50%"><strong>Run Options</strong><br><img src="docs/screenshots/current-run-options.jpg" alt="Run Options: Currently displays a difficulty notice, not a dedicated options dialog."><br>Currently displays a difficulty notice, not a dedicated options dialog.</td>
  </tr>
  <tr>
    <td width="50%"><strong>Combat HUD</strong><br><img src="docs/screenshots/current-combat.jpg" alt="Combat HUD: Elwynn at run start: health, experience, minimap, abilities and damage meter."><br>Elwynn at run start: health, experience, minimap, abilities and damage meter.</td>
    <td width="50%"><strong>Level-up cards</strong><br><img src="docs/screenshots/current-level-up.jpg" alt="Level-up cards: Responsive upgrade choices with card-local shine and rarity glow."><br>Responsive upgrade choices with card-local shine and rarity glow.</td>
  </tr>
  <tr>
    <td width="50%"><strong>Pause</strong><br><img src="docs/screenshots/current-pause.jpg" alt="Pause: Resume, settings, abandon and quit controls."><br>Resume, settings, abandon and quit controls.</td>
    <td width="50%"><strong>Settings</strong><br><img src="docs/screenshots/current-settings.jpg" alt="Settings: Sound, graphics and the persisted auto-push camera toggle."><br>Sound, graphics and the persisted auto-push camera toggle.</td>
  </tr>
</table>

Current victory/defeat results, reward claims, equipped appearances and responsive
viewport variants still need fresh captures. Earlier screenshots below are historical.

<details>
<summary>Earlier checkout captures</summary>

<table>
  <tr>
    <td width="50%"><img src="docs/screenshots/menu.jpg" alt="The pick screen: race, gender and class on the left, sixteen zones on the right, the hero in between"></td>
    <td width="50%"><img src="docs/screenshots/combat.jpg" alt="A Mage holding off Elwynn Forest's wolves with frost, fire and arcane"></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/level-up.jpg" alt="Level up: three spell cards, coloured by rarity, each showing exactly what its next rank changes"></td>
    <td><img src="docs/screenshots/results.jpg" alt="The results screen after a death in Redridge Mountains"></td>
  </tr>
</table>

</details>

### In this build

- **A three-column front door and Armory** using client glue art, with character appearance,
  collection, talents and progression screens. The intended campaign lists **twelve zones**;
  destinations with missing world, encounter or drop content remain visibly locked. These
  screens are under development, and the full campaign is not yet playable.
- **Level-up cards** that deal in, tilt toward the cursor and lift on hover. Their WoW item-quality
  colour says how strong a pick is, and each card lists exactly what changes, like
  `Damage 20 » 26` or `Targets 2 » 3`.
- **A class movement spell on F** from level 5, aimed at the cursor: Blink, Charge, Sprint, Dash,
  Fade, Ghost Wolf, Aspect of the Cheetah, Blessing of Freedom, Howl of Terror.
- **A camera you can free:** locked top-down by default with the hero facing the cursor. **C**
  unlocks it to swing and tilt, and the wheel zooms from over the shoulder out to 70 yards.
- A light sharpen-and-colour pass over the picture (**G** toggles it).

## Play it

```bash
git clone https://github.com/OutThisLife/azeroth-survivors-re
cd azeroth-survivors-re
make build     # fetch game data + install toolchain + build the wasm (~5.5 GB download)
make start     # → open http://127.0.0.1:8090/
```

`make build` chains three re-runnable scripts, each skipping what's already done:

| Step | What it does |
| --- | --- |
| `scripts/fetch-data.sh` | downloads the WoW 1.12.1 client and extracts its MPQs |
| `scripts/web-setup.sh` | installs the wasm toolchain (wasi-sdk, wasm-bindgen, binaryen) |
| `scripts/web-build.sh` | builds the client into `web/dist/` |

**Prerequisites:** [Rust](https://rustup.rs/), `make`, `curl`, `unzip`, and a bash shell. On Windows, run the wasm scripts from Git Bash; PowerShell and cmd can't run them. The native game is the other way around: from Cursor's terminal, run `scripts\survivors.cmd`.

### Windows, native

```text
scripts\survivors.cmd
```

That runs `cargo survivors`: the `play` profile, release optimizations with incremental compilation. The first build compiles the engine. After that, an edit recompiles the crates that changed and leaves the rest of the stack in `target\play`. `cargo build --release` does not — it rebuilds the client crate from scratch, so it is the wrong daily loop.

The script points `WOW_DATA` at `wow-data\Data` when that folder is already there. The program it produces is `target\play\warcraft-survivors.exe`. Closing the window does not throw the build away; the next launch compiles only if the source changed.

Use `cargo survivors-rules` for the engine-independent rule tests. Native development builds
also reload `crates/survivors-core/tuning.txt` while running, so damage, movement, experience
and pickup multipliers can be adjusted without a Rust build. See
[development and distribution](docs/SURVIVORS-DEVELOPMENT.md) for cache boundaries, live
tuning validation, and the planned prompted updater.

Run `make` (or `make help`) to list every target.

## Controls

See [CONTROLS.md](CONTROLS.md). The short version: **WASD** to move, **Space** to jump, **F**
for your class's movement spell, the **mouse wheel** to zoom, **C** to unlock the camera, **Esc**
to pause, **1/2/3** to pick a level-up card.

## The game data (not in this repo)

This repo ships **no Blizzard content** — no art, models, sounds, maps, MPQ contents, or
FrameXML. The game reads a WoW 1.12.1 client's `Data` folder at runtime, which you supply.

`make build` (via `scripts/fetch-data.sh`) downloads the **English WoW 1.12.1 (build 5875)**
client from [archive.org](https://archive.org/details/WoW_1.12.1-5875) (~5.5 GB) and extracts the
MPQs into `wow-data/Data/` (git-ignored, resumable, skipped when already present). Or point at a
client you already have with `--data` / `WOW_DATA`.

### Changing the download source

If the archive.org item moves or you have your own mirror, override the URL without editing
anything:

```bash
make build WOW_DATA_URL=https://your-mirror.example/client.zip
```

Or set it permanently by editing the single clearly-marked `DEFAULT_DATA_URL` line at the top of
`scripts/fetch-data.sh`. Either way the URL must be a direct link to a zip holding the client's
`Data/*.MPQ` files.

## How it's built

The survivors mode is `crates/benilla-app/src/survivors/`, running on two upstream projects:

- [benilla](https://github.com/samwhosung/benilla) — a from-scratch WoW 1.12.1 client.
- [wenilla](https://github.com/Arnesen/wenilla) — benilla compiled to WebAssembly, with a local
  host and realm service.

## Legal

This independent fan project is inspired by **Aethus’s Azeroth Survivors**. It is not
affiliated with or endorsed by Blizzard Entertainment and ships no Blizzard game files. The screenshots in `docs/screenshots/`
show World of Warcraft art rendered from a user-supplied client; that art is © Blizzard
Entertainment. World of Warcraft is a trademark of Blizzard Entertainment, Inc. The engine code in
this repository is licensed under
[MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE), at your option; the vendored components under
`third_party/` keep their own upstream licenses.
