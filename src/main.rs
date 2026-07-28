//! zukan: Monster Hunter bestiary in your terminal.
//!
//! Half-block ANSI sprites + data cards, all assets embedded at compile time.

mod card;
mod cli;
mod color;
mod config;
mod data;
mod database;
mod fuzzy;
mod i18n;
mod i18n_terms;
mod render;
mod search;

use std::process::ExitCode;

use argh::FromArgs;

use cli::Args;
use config::Config;
use data::{Item, Monster};
use database::Database;

fn main() -> ExitCode {
    let args: Args = argh::from_env();

    if args.version {
        println!("zukan {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    // No-op invocation: print --help and exit 0, like git/kubectl. Done before
    // loading the DB so it's instant.
    if args.query.is_empty() && !args.random && args.list.is_none() {
        // argh only emits help when --help is passed; synthesize it so a bare
        // `zukan` invocation behaves like git/kubectl.
        match Args::from_args(&["zukan"], &["--help"]) {
            Ok(_) => {}
            Err(early) => {
                // status Ok = help requested (stdout), Err = parse error (stderr).
                if early.status.is_ok() {
                    print!("{}", early.output);
                } else {
                    eprint!("{}", early.output);
                }
            }
        }
        return ExitCode::SUCCESS;
    }

    let cfg = Config::load();

    let db = match Database::load() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("zukan: failed to load embedded data: {e}");
            return ExitCode::from(1);
        }
    };

    match run(&db, &cfg, &args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(RunError::NotFound { kind, query }) => {
            eprintln!("zukan: no {kind} matches {query:?}");
            ExitCode::from(1)
        }
        Err(RunError::Ambiguous {
            kind,
            query,
            candidates,
        }) => {
            eprintln!("zukan: multiple {kind}s match {query:?}:");
            for c in candidates {
                eprintln!("  {c}");
            }
            eprintln!("pass --all to show every match, or refine the query.");
            ExitCode::from(2)
        }
        Err(RunError::Usage(msg)) => {
            eprintln!("zukan: {msg}");
            ExitCode::from(2)
        }
    }
}

#[derive(Debug)]
enum RunError {
    NotFound {
        kind: &'static str,
        query: String,
    },
    Ambiguous {
        kind: &'static str,
        query: String,
        candidates: Vec<String>,
    },
    Usage(&'static str),
}

fn run(db: &Database, cfg: &Config, args: &Args) -> Result<(), RunError> {
    // --lang wins; "auto" falls back to config, then en.
    let lang_str = if args.lang == "auto" {
        if cfg.language.is_empty() {
            "en"
        } else {
            &cfg.language
        }
    } else {
        &args.lang
    };
    let lang = match i18n::Lang::parse(lang_str) {
        Some(l) => l,
        None => {
            return Err(RunError::Usage(
                "--lang must be one of: en, ja, zh (or auto)",
            ));
        }
    };

    // Effective width: explicit --width wins; otherwise config.
    let width = if args.width != 0 {
        args.width
    } else {
        cfg.default_width
    };

    // Effective card mode: flags override config.
    let show_card = if args.no_card {
        false
    } else if args.detail {
        true
    } else {
        cfg.show_card_by_default
    };

    // --list <game>: print all monsters in that game, then exit.
    if let Some(game_token) = &args.list {
        return list_game(db, game_token, lang);
    }

    // Resolve --game filter (normalized code) for query/random restriction.
    let game_filter = args
        .game
        .as_deref()
        .map(|g| color::normalize_game(g).unwrap_or(g));

    if args.item {
        let targets: Vec<usize> = if args.random {
            vec![
                search::random(&db.items, make_item_predicate(game_filter))
                    .ok_or(RunError::Usage("no items match the filter"))?,
            ]
        } else if args.query.is_empty() {
            return Err(RunError::Usage(
                "provide an item name, or use --item --random",
            ));
        } else {
            // Each query token resolves independently: `zukan --item potion herb`
            // is two separate lookups.
            resolve_many(db, &args.query, args.all, resolve_item)?
        };
        for (i, &t) in targets.iter().enumerate() {
            if i > 0 {
                println!();
            }
            render_item(&db.items[t], width, show_card, args.hide_name, lang);
        }
        return Ok(());
    }

    // Monster mode (default).
    let targets: Vec<usize> = if args.random {
        vec![
            search::random(&db.monsters, make_monster_predicate(game_filter))
                .ok_or(RunError::Usage("no monsters match the filter"))?,
        ]
    } else if args.query.is_empty() {
        return Err(RunError::Usage(
            "provide a monster name, or use --random / --list",
        ));
    } else {
        resolve_many(db, &args.query, args.all, resolve_monster)?
    };

    for (i, &t) in targets.iter().enumerate() {
        if i > 0 {
            println!();
        }
        render_monster(
            &db.monsters[t],
            width,
            show_card,
            args.hide_name,
            game_filter,
            lang,
        );
    }
    Ok(())
}

/// Filter monsters by game code. `None` matches everything.
fn make_monster_predicate(game: Option<&str>) -> impl Fn(&Monster) -> bool {
    let game = game.map(str::to_string);
    move |m: &Monster| match &game {
        None => true,
        Some(g) => m.games.iter().any(|e| e.game == *g),
    }
}

/// Filter items by game code. Item sources use API vocabulary ("wilds"), which
/// `normalize_game` maps to "mhwilds", so we compare against the normalized form.
fn make_item_predicate(game: Option<&str>) -> impl Fn(&Item) -> bool {
    let game = game.map(str::to_string);
    move |it: &Item| match &game {
        None => true,
        Some(g) => it.sources.iter().any(|s| {
            let normalized = color::normalize_game(&s.game).unwrap_or(&s.game);
            normalized == g
        }),
    }
}

/// `--list <game>`: print every monster in `game`, one per line, by slug.
fn list_game(db: &Database, token: &str, lang: i18n::Lang) -> Result<(), RunError> {
    let game = match color::normalize_game(token) {
        Some(g) => g,
        None => {
            return Err(RunError::Usage(
                "unknown game; try mhw, MHW, or 'Monster Hunter World'",
            ));
        }
    };
    let mut entries: Vec<&Monster> = db
        .monsters
        .iter()
        .filter(|m| m.games.iter().any(|e| e.game == game))
        .collect();
    // Sort by slug for stable output.
    entries.sort_by(|a, b| a.slug.cmp(&b.slug));

    let full_title = crate::color::game_full_title(game);
    let full_title: &str = if full_title.is_empty() {
        game
    } else {
        full_title
    };
    println!("{full_title}  ({} monsters)", entries.len());
    for m in entries {
        let (loc_name, _) = i18n::monster_localized(m, lang);
        // Tint by first element so the roster reads as color-grouped.
        let colored = color::name_colored_by_element(&loc_name, &m.elements);
        println!("  {colored}  ({})", m.slug);
    }
    Ok(())
}

/// Resolve each query token to indices, flattening into one ordered list.
/// With `--all`, a broad token like "rath" expands to every match; without it,
/// a multi-match token yields an `Ambiguous` error.
fn resolve_many(
    db: &Database,
    queries: &[String],
    show_all: bool,
    resolver: fn(&Database, &str, bool) -> Result<Vec<usize>, RunError>,
) -> Result<Vec<usize>, RunError> {
    let mut out = Vec::new();
    for q in queries {
        out.extend(resolver(db, q, show_all)?);
    }
    Ok(out)
}

/// Resolve a monster query to indices (one for a unique match, all with
/// `--all`, or `Ambiguous` for a multi-match without `--all`).
fn resolve_monster(db: &Database, query: &str, show_all: bool) -> Result<Vec<usize>, RunError> {
    let hits = search::search(&db.monsters, query, 16);
    resolve_hits("monster", db, query, show_all, hits, |db, i| {
        let m = &db.monsters[i];
        format!("{} ({})", m.name, m.slug)
    })
}

/// Resolve an item query (same rules as `resolve_monster`).
fn resolve_item(db: &Database, query: &str, show_all: bool) -> Result<Vec<usize>, RunError> {
    let hits = search::search(&db.items, query, 16);
    resolve_hits("item", db, query, show_all, hits, |db, i| {
        let it = &db.items[i];
        format!("{} ({})", it.name, it.slug)
    })
}

/// Shared resolution core: empty yields `NotFound`, one hit auto-displays, many
/// hits expand with `--all` or yield `Ambiguous`.
fn resolve_hits(
    kind: &'static str,
    db: &Database,
    query: &str,
    show_all: bool,
    hits: Vec<usize>,
    fmt_candidate: impl Fn(&Database, usize) -> String,
) -> Result<Vec<usize>, RunError> {
    if hits.is_empty() {
        return Err(RunError::NotFound {
            kind,
            query: query.to_string(),
        });
    }
    if hits.len() == 1 || show_all {
        return Ok(hits);
    }
    let candidates = hits.iter().map(|&i| fmt_candidate(db, i)).collect();
    Err(RunError::Ambiguous {
        kind,
        query: query.to_string(),
        candidates,
    })
}

/// Render a monster's icon (and optional card) to stdout.
///
/// In plain mode the localized name goes to stderr first so it lands above the
/// sprite when the streams merge. In card mode the name is the card's title
/// line, so no stderr line is emitted (avoids duplication). `--hide-name`
/// suppresses both.
fn render_monster(
    m: &Monster,
    width: u32,
    show_card: bool,
    hide_name: bool,
    game_override: Option<&str>,
    lang: i18n::Lang,
) {
    // Name to stderr first (plain mode only).
    if !hide_name && !show_card {
        let (loc_name, _) = i18n::monster_localized(m, lang);
        eprintln!("{loc_name}");
    }

    let icon_path =
        best_icon_path(m, game_override).unwrap_or_else(|| format!("icons/mhw/{}.png", m.slug));

    let img = Database::asset(&icon_path).and_then(|data| {
        image::load_from_memory(data).map_err(|e| database::LoadError::Parse {
            file: "icon",
            error: e.to_string(),
        })
    });

    let w = if width != 0 { width } else { 32 };

    match (img, show_card) {
        (Ok(img), true) => {
            println!("{}", card::render_monster(&img, m, w, lang));
        }
        (Ok(img), false) => {
            println!("{}", render::render_halfblock(&img, w, true));
        }
        (Err(e), _) => {
            eprintln!("zukan: could not load icon {icon_path}: {e}");
        }
    }
}

/// Render an item's icon (and optional card) to stdout; name to stderr.
///
/// Items without an `icon` field get a placeholder block; the card still shows
/// their metadata.
fn render_item(it: &Item, width: u32, show_card: bool, hide_name: bool, lang: i18n::Lang) {
    // Name to stderr first (plain mode only).
    if !hide_name && !show_card {
        let (loc_name, _) =
            i18n::item_localized(&it.name, it.description.as_deref(), &it.i18n, lang);
        eprintln!("{loc_name}");
    }

    let icon_path = it.icon.as_ref().map(|p| format!("icons/{p}"));
    let img = icon_path.as_ref().and_then(|p| {
        Database::asset(p)
            .and_then(|data| {
                image::load_from_memory(data).map_err(|e| database::LoadError::Parse {
                    file: "icon",
                    error: e.to_string(),
                })
            })
            .ok()
    });

    let w = if width != 0 { width } else { 24 };

    match (img.as_ref(), show_card) {
        (Some(img), true) => {
            println!("{}", card::render_item(Some(img), it, w, lang));
        }
        (Some(img), false) => {
            println!("{}", render::render_halfblock(img, w, true));
        }
        (None, true) => {
            println!("{}", card::render_item(None, it, w, lang));
        }
        (None, false) => {
            // No icon, no card; name (if not hidden) was already printed.
        }
    }
}

/// Pick which game's icon to render for a monster.
///
/// `game_override` (`--game`) wins if that game has an icon entry; otherwise
/// we scan `PREFERENCE`. The order is tuned for low-res readability, not release
/// order: simpler/bolder art (Stories, older mainline) downscales cleaner than
/// the photoreal modern titles. See `docs/zukan-design.md` and AGENTS.md.
fn best_icon_path(m: &Monster, game_override: Option<&str>) -> Option<String> {
    // First match wins. Stories → older mainline → modern mainline.
    const PREFERENCE: &[&str] = &[
        "mhst2", "mhst", "mh4u", "mh3u", "mhfu", "mhgu", "mhwilds", "mhwi", "mhw", "mhrise", "mhrs",
    ];

    let path_for = |e: &data::GameEntry| format!("icons/{}", e.icon.as_ref().unwrap());

    // (1) --game override wins if present and has an icon; otherwise fall
    // through to the preference scan (no error).
    if let Some(want) = game_override
        && let Some(entry) = m.games.iter().find(|g| g.game == want && g.icon.is_some())
    {
        return Some(path_for(entry));
    }

    // (2) Preference scan.
    for game in PREFERENCE {
        if let Some(entry) = m.games.iter().find(|g| g.game == *game && g.icon.is_some()) {
            return Some(path_for(entry));
        }
    }
    // (3) Fall back to any entry with an icon.
    m.games
        .iter()
        .find_map(|g| g.icon.clone())
        .map(|icon| format!("icons/{icon}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_simpler_art() {
        // mhst2 wins over mhw despite mhw being newer.
        let m = Monster {
            id: String::new(),
            name: "Test".into(),
            slug: "test".into(),
            kind: "Test".into(),
            species: None,
            is_large: true,
            sub_species: Vec::new(),
            elements: Vec::new(),
            ailments: Vec::new(),
            weakness: Vec::new(),
            games: vec![
                data::GameEntry {
                    game: "mhw".into(),
                    info: None,
                    danger: None,
                    icon: Some("mhw/test.png".into()),
                    icon_source: None,
                },
                data::GameEntry {
                    game: "mhst2".into(),
                    info: None,
                    danger: None,
                    icon: Some("mhst2/test.png".into()),
                    icon_source: None,
                },
            ],
            numeric: None,
            i18n: Default::default(),
        };
        assert_eq!(
            best_icon_path(&m, None).as_deref(),
            Some("icons/mhst2/test.png")
        );
    }

    #[test]
    fn skips_entries_without_icon() {
        // mhst2 preferred but iconless; falls through to mhfu.
        let m = Monster {
            id: String::new(),
            name: "Test".into(),
            slug: "test".into(),
            kind: "Test".into(),
            species: None,
            is_large: true,
            sub_species: Vec::new(),
            elements: Vec::new(),
            ailments: Vec::new(),
            weakness: Vec::new(),
            games: vec![
                data::GameEntry {
                    game: "mhst2".into(),
                    info: None,
                    danger: None,
                    icon: None,
                    icon_source: None,
                },
                data::GameEntry {
                    game: "mhfu".into(),
                    info: None,
                    danger: None,
                    icon: Some("mhfu/test.png".into()),
                    icon_source: None,
                },
            ],
            numeric: None,
            i18n: Default::default(),
        };
        assert_eq!(
            best_icon_path(&m, None).as_deref(),
            Some("icons/mhfu/test.png")
        );
    }

    #[test]
    fn override_picks_that_game() {
        // --game mhw overrides the default mhst2 preference.
        let m = Monster {
            id: String::new(),
            name: "Test".into(),
            slug: "test".into(),
            kind: "Test".into(),
            species: None,
            is_large: true,
            sub_species: Vec::new(),
            elements: Vec::new(),
            ailments: Vec::new(),
            weakness: Vec::new(),
            games: vec![
                data::GameEntry {
                    game: "mhw".into(),
                    info: None,
                    danger: None,
                    icon: Some("mhw/test.png".into()),
                    icon_source: None,
                },
                data::GameEntry {
                    game: "mhst2".into(),
                    info: None,
                    danger: None,
                    icon: Some("mhst2/test.png".into()),
                    icon_source: None,
                },
            ],
            numeric: None,
            i18n: Default::default(),
        };
        assert_eq!(
            best_icon_path(&m, Some("mhw")).as_deref(),
            Some("icons/mhw/test.png")
        );
    }

    #[test]
    fn override_falls_back_when_game_missing() {
        // --game mhrs but monster not in mhrs; falls back, not errors.
        let m = Monster {
            id: String::new(),
            name: "Test".into(),
            slug: "test".into(),
            kind: "Test".into(),
            species: None,
            is_large: true,
            sub_species: Vec::new(),
            elements: Vec::new(),
            ailments: Vec::new(),
            weakness: Vec::new(),
            games: vec![data::GameEntry {
                game: "mhst2".into(),
                info: None,
                danger: None,
                icon: Some("mhst2/test.png".into()),
                icon_source: None,
            }],
            numeric: None,
            i18n: Default::default(),
        };
        assert_eq!(
            best_icon_path(&m, Some("mhrs")).as_deref(),
            Some("icons/mhst2/test.png")
        );
    }

    #[test]
    fn override_falls_back_when_no_icon() {
        // --game mhw but the mhw entry has no icon; falls back to preference.
        let m = Monster {
            id: String::new(),
            name: "Test".into(),
            slug: "test".into(),
            kind: "Test".into(),
            species: None,
            is_large: true,
            sub_species: Vec::new(),
            elements: Vec::new(),
            ailments: Vec::new(),
            weakness: Vec::new(),
            games: vec![
                data::GameEntry {
                    game: "mhw".into(),
                    info: None,
                    danger: None,
                    icon: None,
                    icon_source: None,
                },
                data::GameEntry {
                    game: "mhst2".into(),
                    info: None,
                    danger: None,
                    icon: Some("mhst2/test.png".into()),
                    icon_source: None,
                },
            ],
            numeric: None,
            i18n: Default::default(),
        };
        assert_eq!(
            best_icon_path(&m, Some("mhw")).as_deref(),
            Some("icons/mhst2/test.png")
        );
    }
}
