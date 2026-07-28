//! End-to-end integration tests that exercise the compiled binary.
//!
//! These spawn the actual `zukan` executable so they cover CLI parsing,
//! embedded-asset loading, search, render, and output formatting together.
//! Assets are baked in at build time (rust-embed reads `assets/`), so the
//! binary is self-contained; no env vars needed at run time.
//!
//! `XDG_CONFIG_HOME` is pointed at a throwaway empty dir for each spawned
//! child, so the user's real `~/.config/zukan/config` (which may flip
//! show_card_by_default and other defaults) can't change what these tests
//! assert.

use std::process::Command;

fn zukan() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_zukan"));
    // dirs::config_dir() honors XDG_CONFIG_HOME on Linux. Point it at an empty
    // dir so no zukan/config is found; the binary then uses built-in defaults.
    let tmp = std::env::temp_dir().join("zukan-integration-xdg");
    let _ = std::fs::create_dir_all(&tmp);
    cmd.env("XDG_CONFIG_HOME", &tmp);
    cmd
}

#[test]
fn exact_slug_renders_icon_and_name() {
    let out = zukan()
        .args(["rathalos"])
        .output()
        .expect("zukan binary runs");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // Icon goes to stdout (contains the half-block glyph).
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains('▀'), "stdout should contain half-block art");
    // Name goes to stderr.
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Rathalos"),
        "stderr should contain the name"
    );
}

#[test]
fn hide_name_suppresses_stderr() {
    let out = zukan()
        .args(["rathalos", "--hide-name"])
        .output()
        .expect("runs");
    assert!(out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).trim().is_empty(),
        "stderr should be empty with --hide-name"
    );
}

#[test]
fn typo_resolves_via_levenshtein() {
    let out = zukan().args(["rathalas"]).output().expect("runs");
    assert!(out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("Rathalos"),
        "rathalas should resolve to Rathalos"
    );
}

#[test]
fn ambiguous_query_lists_candidates_and_exits_2() {
    let out = zukan().args(["rath"]).output().expect("runs");
    assert!(!out.status.success());
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("multiple monsters match"));
    assert!(stderr.contains("Rathalos"));
}

#[test]
fn all_flag_renders_every_match() {
    // "diablo" alone is ambiguous (4 monsters); --all must render all of them,
    // not just the first (regression guard for the old returns-one bug).
    let out = zukan().args(["diablo", "--all"]).output().expect("runs");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    for name in [
        "Apex Diablos",
        "Black Diablos",
        "Bloodbath Diablos",
        "Diablos",
    ] {
        assert!(
            stderr.contains(name),
            "--all must render {name:?}; stderr was:\n{stderr}"
        );
    }
    // Each name is printed once (one icon block per match).
    assert_eq!(
        stderr.matches("Diablos").count(),
        4,
        "expected 4 Diablos-family names, got different count"
    );
}

#[test]
fn all_flag_with_single_match_still_renders() {
    // --all on a unique query is a no-op (single match -> single render).
    let out = zukan().args(["rathalos", "--all"]).output().expect("runs");
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        stderr.matches("Rathalos").count(),
        1,
        "single match should render once"
    );
}

#[test]
fn not_found_exits_1() {
    let out = zukan().args(["zzzz-not-a-monster"]).output().expect("runs");
    assert!(!out.status.success());
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("no monster matches"));
}

#[test]
fn detail_card_renders() {
    let out = zukan()
        .args(["rathalos", "--detail"])
        .output()
        .expect("runs");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Card layout: a horizontal rule of box-drawing chars under the title.
    assert!(stdout.contains('─'));
    // And at least one labeled field.
    assert!(stdout.contains("Type:") || stdout.contains("種類:"));
}

#[test]
fn japanese_localization() {
    let out = zukan()
        .args(["rathalos", "--detail", "--lang", "ja"])
        .output()
        .expect("runs");
    assert!(out.status.success());
    // Card mode: the name is the card's title line on stdout (no separate
    // stderr name line).
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("リオレウス"), "ja name in card title");
    assert!(stdout.contains("飛竜種"), "ja type label in card");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("リオレウス"),
        "card mode must not duplicate the name on stderr"
    );
}

#[test]
fn item_mode_works() {
    let out = zukan()
        .args(["--item", "dragonite-ore", "--detail"])
        .output()
        .expect("runs");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Dragonite Ore"));
    assert!(stdout.contains("Rarity:"));
}

#[test]
fn list_game_prints_roster() {
    let out = zukan().args(["--list", "mhwilds"]).output().expect("runs");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Monster Hunter Wilds"));
    assert!(stdout.contains("monsters)"));
    // A few known Wilds monsters.
    assert!(stdout.contains("rathalos"));
}

#[test]
fn random_returns_zero_exit() {
    let out = zukan().args(["--random"]).output().expect("runs");
    assert!(out.status.success());
    // Random picks some name on stderr (non-empty).
    assert!(!String::from_utf8_lossy(&out.stderr).trim().is_empty());
}

#[test]
fn random_with_game_filter_picks_a_wilds_monster() {
    let out = zukan()
        .args(["--random", "--game", "mhwilds"])
        .output()
        .expect("runs");
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    let name = stderr.trim();
    assert!(
        !name.is_empty(),
        "random --game should print a name on stderr"
    );

    // Cross-check: the picked name (minus the trailing English slug suffix)
    // must appear in the Wilds roster.
    let list = zukan().args(["--list", "mhwilds"]).output().expect("runs");
    let roster = String::from_utf8_lossy(&list.stdout);
    // The localized name on stderr is "Local Name EnglishName"; take the last
    // token, which is also the slug lookup.
    let english = name.split_whitespace().last().unwrap_or(name);
    assert!(
        roster.contains(english),
        "random --game mhwilds picked {english:?}, which is not in the Wilds roster"
    );
}

#[test]
fn multi_query_renders_each() {
    let out = zukan()
        .args(["rathalos", "rathian"])
        .output()
        .expect("runs");
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Rathalos"));
    assert!(stderr.contains("Rathian"));
}

#[test]
fn invalid_lang_errors() {
    let out = zukan()
        .args(["rathalos", "--lang", "klingon"])
        .output()
        .expect("runs");
    assert!(!out.status.success());
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn no_args_prints_help_and_exits_0() {
    // No query / --random / --list: print --help (like git/kubectl), exit 0.
    let out = zukan().output().expect("runs");
    assert!(out.status.success(), "no-args should exit 0");
    // clap writes --help to stdout.
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Usage: zukan"),
        "should print usage: {stdout}"
    );
    assert!(
        stdout.contains("--random"),
        "help should list flags: {stdout}"
    );
}
