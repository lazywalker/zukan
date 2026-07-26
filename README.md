# ZUKAN

zukan(図鑑) is a Monster Hunter bestiary in your terminal. 
Renders creature and item icons as half-block ANSI art, 
with an optional info card. All assets are
embedded at compile time — a single offline binary, no external data files.

Asset data comes from the companion repo [zukan-assets](https://github.com/lazywalker/zukan-assets).

## Install

Prebuilt binaries will land on the Releases page once CI is wired. For now,
build from source:

```bash
git clone https://github.com/lazywalker/zukan
cd zukan
make release        # optimized build, ~9 MB binary at target/release/zukan
```

The first build downloads the latest `zukan-assets` Release (~6 MB of icon +
data) into the in-tree `assets/` directory and embeds it.

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
| `--width N` | Icon width in terminal columns (default 32 monsters / 24 items) |
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

Optional config file at `~/.config/zukan/config.toml` (or the platform
equivalent). Explicit CLI flags always win over config.

```toml
language = "en"              # en | ja | zh
default_width = 0            # 0 = built-in defaults (32 monsters / 24 items)
default_game = ""            # preferred game code for icon selection
show_card_by_default = false # show the info card without needing --detail
```

