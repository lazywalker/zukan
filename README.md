# ZUKAN

<!-- Repository badges -->

[![Rust](https://img.shields.io/badge/rust-2024--edition-orange)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/lazywalker/zukan/actions/workflows/ci.yml/badge.svg)](https://github.com/lazywalker/zukan/actions)
[![crates.io](https://img.shields.io/crates/v/zukan.svg)](https://crates.io/crates/zukan)
[![codecov](https://codecov.io/gh/lazywalker/zukan/branch/master/graph/badge.svg)](https://codecov.io/gh/lazywalker/zukan)
[![Dependabot](https://img.shields.io/badge/Dependabot-enabled-brightgreen.svg)](https://github.com/lazywalker/zukan/network/updates)
[![Maintenance](https://img.shields.io/maintenance/yes/2026)](https://github.com/lazywalker/zukan)

zukan(図鑑) is a Monster Hunter bestiary in your terminal. 
Renders creature and item icons as half-block ANSI art, 
with an optional info card. All assets are
embedded at compile time — a single offline binary, no external data files.

Asset data comes from the companion repo [zukan-assets](https://github.com/lazywalker/zukan-assets).

## Install

**Prebuilt binary** — download from the
[Releases page](https://github.com/lazywalker/zukan/releases/latest):

```bash
# Linux/macOS (pick the asset matching your platform)
tar xzf zukan-<version>-x86_64-linux.tar.gz
sudo install -m 755 zukan /usr/local/bin/
```

**cargo install** — builds from source on crates.io:

```bash
cargo install zukan
```

For asset refresh, local-checkout overrides, make targets, developing notes, see [DEVELOPING.md](DEVELOPING.md).

## Usage

```bash
# Show a monster's icon (sprite → stdout, name → stderr)
zukan rathalos

# Icon + info card
zukan rathalos --detail

# Typo-tolerant: rathalas → rathalos
zukan rathalas

# Multiple monsters, one per block
zukan rathalos rathian diablos

# Items instead of monsters
zukan --item mega-potion --detail

# Browse
zukan --random                  # random monster
zukan --random --game mhwilds   # random monster from Wilds
zukan --list mhwilds            # list all Wilds monsters

# Languages: en (default) / ja / zh
zukan rathalos --detail --lang ja
```

### Flags

| Flag | Purpose |
|---|---|
| `<query>...` | One or more monster/item names (fuzzy, typo-tolerant) |
| `--item` | Query the item database instead of monsters |
| `--detail` | Show the info card next to the icon |
| `--no-card` | Icon only (overrides `--detail` and config) |
| `--width N` | Icon width in terminal columns (0 = default 32 monsters / 24 items; else 24..=48) |
| `--lang en\|ja\|zh` | Display language (`auto` = use config) |
| `--random` | Pick a random monster/item |
| `--game CODE` | Filter `--random` to a game (e.g. `mhw`, `MHW`, `mhwilds`) |
| `--list GAME` | List all monsters in a game, then exit |
| `--hide-name` | Suppress the name line on stderr |
| `--all` / `-a` | Render every match instead of just the best one |

### Output convention

The sprite goes to **stdout**, the name to **stderr** — so you can pipe the art
on its own:

```bash
zukan rathalos --hide-name | some-other-tool
```

## Configuration

Optional config file at `~/.config/zukan/config` (or the platform equivalent).
Plain `KEY = VAL`, one per line, `#` for comments. Explicit CLI flags always
win over config.

```sh
language = en              # en | ja | zh
default_width = 0          # 0 = built-in defaults (32 monsters / 24 items); else must be 24..=48
default_game =             # preferred game code for icon selection
show_card_by_default = false # show the info card without needing --detail
```

