# Developing zukan

Notes for contributors building / hacking on zukan from source. For end-user
usage see [README.md](README.md).

## Build

zukan is a Rust 2024 binary crate. The only non-obvious part is asset
acquisition: icons + data are embedded at compile time from the companion
[zukan-assets](https://github.com/lazywalker/zukan-assets) Release.

```bash
git clone https://github.com/lazywalker/zukan
cd zukan
make release        # optimized build, ~9 MB binary at target/release/zukan
```

The first build downloads the latest `zukan-assets` Release (~6 MB of icon +
data) into the in-tree `assets/` directory and embeds it. The payload is
gitignored (only `assets/.gitkeep` is tracked).

## Asset lifecycle

`build.rs` auto-downloads via `ureq` when `assets/{data,icons}` are missing,
but that path can be unreliable behind corporate/sandboxed networks. The
Makefile uses `curl + tar` instead, which works everywhere:

```bash
make download       # fetch latest zukan-assets Release into assets/
make assets         # download only if assets/{data,icons} are missing
make clean          # wipe target/ AND the cached assets
```

To force a refresh after a new `zukan-assets` Release: `make clean && make download`,
or just `rm -rf assets/data assets/icons && make build`.

## Make targets

```bash
make                # default: print the help menu
make check          # clippy + fmt check (run before committing)
make test           # unit + integration tests
make build          # debug build
make release        # optimized release build
make doc            # rustdoc (warnings as errors)
make cov            # coverage report (needs `cargo install cargo-llvm-cov`)
make all            # check + test + build
```

All `make` targets are thin wrappers over `cargo`; you can call `cargo`
directly if you prefer.

## Quality gates

Before pushing, these must pass (enforced by `make check && make test`):

```bash
cargo clippy  --all-features --tests -- -D warnings
cargo test    --bins --tests
cargo fmt     --all -- --check
cargo doc     --no-deps --package zukan   # with RUSTDOCFLAGS="-D warnings"
cargo llvm-cov --bins --tests --summary-only   # target ≥ 80%
```

## Architecture

See [`docs/zukan-design.md`](../docs/zukan-design.md) for the full design doc
(module map, data flow, rendering decisions, the showie comparison). Module
roles in brief:

- `main.rs` — entry + CLI dispatch + icon-source preference
- `cli.rs` — clap `Args`
- `data.rs` — serde model (Monster/Item/NumericData/WildsWeakness)
- `database.rs` — rust-embed + JSON deserialize + slug→index maps
- `search.rs` — 4-tier match (exact slug → name → substring → Levenshtein ≤2)
- `fuzzy.rs` — Levenshtein (no external crate)
- `render.rs` — half-block ANSI renderer (alpha `< 16`, sticky-bg fixed)
- `card.rs` — neofetch-style layout (CJK-aware)
- `color.rs` — element/ailment 256-color tables + game code↔abbr
- `i18n.rs` + `i18n_terms.rs` — localization (hardcoded term tables)
- `config.rs` — `~/.config/zukan/config` (KEY=VAL parser)
