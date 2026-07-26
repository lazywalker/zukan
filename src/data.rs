//! Serde model for zukan-assets data/{monsters,items}.json.
//!
//! Conventions:
//!   - `#[serde(default)]` on optional fields so the model tolerates
//!     forward-compatible additions (unknown keys ignored, missing keys filled).
//!   - Field names match JSON keys, except `type` (Rust keyword) -> `kind`.
//!   - `numeric` has three independent sub-structures (mhw / wilds / mhgu),
//!     each modeled as its own struct behind `Option`.
//!   - `wilds.weaknesses[]` is polymorphic on `kind`; modeled as an untagged
//!     enum (serde picks the variant by which key is present).
//!
//! Only fields the renderer/card reads are strictly typed. Deeply nested
//! reward/recipe/protection trees are skipped; serde ignores unknown fields.
//! The schema is wider than what we render today, so unread fields are allowed
//! dead_code by design (forward-compat for future detail views).

// Schema fields are deserialized for completeness / forward compatibility even
// when not yet read by a renderer. Silence the per-field dead-code lint.
#![allow(dead_code)]

use serde::Deserialize;

// ========== monsters ===

#[derive(Debug, Clone, Deserialize)]
pub struct Monster {
    pub id: String,
    pub name: String,
    pub slug: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub species: Option<String>,
    #[serde(default)]
    pub is_large: bool,
    #[serde(default)]
    pub sub_species: Vec<String>,
    #[serde(default)]
    pub elements: Vec<String>,
    #[serde(default)]
    pub ailments: Vec<String>,
    #[serde(default)]
    pub weakness: Vec<String>,
    #[serde(default)]
    pub games: Vec<GameEntry>,
    #[serde(default)]
    pub numeric: Option<NumericData>,
    #[serde(default)]
    pub i18n: I18nMap,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct GameEntry {
    #[serde(default)]
    pub game: String,
    #[serde(default)]
    pub game_full: String,
    #[serde(default)]
    pub info: Option<String>,
    #[serde(default)]
    pub danger: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub icon_source: Option<String>,
}

// ========== numeric ====

#[derive(Debug, Clone, Deserialize, Default)]
pub struct NumericData {
    #[serde(default)]
    pub mhw: Option<MhwNumeric>,
    #[serde(default)]
    pub wilds: Option<WildsNumeric>,
    #[serde(default)]
    pub mhgu: Option<MhguNumeric>,
}

/// MHW-style numeric: text + per-element star weaknesses + locations.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct MhwNumeric {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub weaknesses: Vec<ElementStars>,
    #[serde(default)]
    pub locations: Vec<NamedLocation>,
}

/// Wilds-style numeric: text + polymorphic weaknesses + part hitzones + tips.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct WildsNumeric {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub weaknesses: Vec<WildsWeakness>,
    #[serde(default)]
    pub locations: Vec<NamedLocation>,
    #[serde(default)]
    pub tips: Option<String>,
}

/// MHGU-style numeric: HP + numeric hitzones + trap flags + status tolerances.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct MhguNumeric {
    #[serde(default)]
    pub base_hp: Option<u64>,
    #[serde(default)]
    pub hitzones: Vec<Hitzones>,
    #[serde(default)]
    pub status: Vec<StatusTolerance>,
    #[serde(default)]
    pub traps: Option<Traps>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ElementStars {
    #[serde(default)]
    pub element: String,
    #[serde(default)]
    pub stars: u32,
    #[serde(default)]
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct NamedLocation {
    #[serde(default)]
    pub name: String,
}

/// Wilds weaknesses are polymorphic on `kind`. Untagged enum: serde tries each
/// variant in order, picking the first whose fields match. The discriminator is
/// which of `element` / `status` / `effect` is set.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum WildsWeakness {
    Element {
        #[serde(default)]
        element: String,
        #[serde(default)]
        level: u32,
        #[serde(default)]
        kind: Option<String>,
    },
    Status {
        #[serde(default)]
        status: String,
        #[serde(default)]
        level: u32,
        #[serde(default)]
        kind: Option<String>,
    },
    Effect {
        #[serde(default)]
        effect: String,
        #[serde(default)]
        level: u32,
        #[serde(default)]
        kind: Option<String>,
    },
}

impl WildsWeakness {
    pub fn level(&self) -> u32 {
        match self {
            WildsWeakness::Element { level, .. }
            | WildsWeakness::Status { level, .. }
            | WildsWeakness::Effect { level, .. } => *level,
        }
    }

    /// English token for color lookup and label display.
    pub fn token(&self) -> &str {
        match self {
            WildsWeakness::Element { element, .. } => element,
            WildsWeakness::Status { status, .. } => status,
            WildsWeakness::Effect { effect, .. } => effect,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Hitzones {
    #[serde(default)]
    pub part: String,
    #[serde(default)]
    pub cut: i64,
    #[serde(default)]
    pub impact: i64,
    #[serde(default)]
    pub shot: i64,
    #[serde(default)]
    pub fire: i64,
    #[serde(default)]
    pub water: i64,
    #[serde(default)]
    pub ice: i64,
    #[serde(default)]
    pub thunder: i64,
    #[serde(default)]
    pub dragon: i64,
    #[serde(default)]
    pub ko: i64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct StatusTolerance {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub initial: i64,
    #[serde(default)]
    pub increase: i64,
    #[serde(default)]
    pub max: i64,
    #[serde(default)]
    pub duration: i64,
    #[serde(default)]
    pub damage: i64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Traps {
    #[serde(default)]
    pub shock: bool,
    #[serde(default)]
    pub pitfall: bool,
    #[serde(default)]
    pub flash: bool,
    #[serde(default)]
    pub sonic: bool,
    #[serde(default)]
    pub dung: bool,
    #[serde(default)]
    pub meat: bool,
}

// ============ items ====

#[derive(Debug, Clone, Deserialize)]
pub struct Item {
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub rarity: Option<u32>,
    #[serde(default)]
    pub value: Option<u64>,
    #[serde(default)]
    pub carry_limit: Option<u32>,
    #[serde(default)]
    pub sources: Vec<ItemSource>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub icon_source: Option<String>,
    #[serde(default)]
    pub wilds_icon: Option<WildsIcon>,
    #[serde(default)]
    pub i18n: I18nMap,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ItemSource {
    #[serde(default)]
    pub game: String,
    #[serde(default)]
    pub id: i64,
}

/// Wilds-only icon metadata. No PNG on disk; it's a kind/color tag, so `Item`s
/// without a real `icon` field get a textual fallback in the card.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct WildsIcon {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub color_id: Option<i64>,
    #[serde(default)]
    pub color: Option<String>,
}

// ============= i18n ====

/// Map of language code to translation entry. Real records always carry ja+zh.
/// `Default` is an empty map (English fallback applies in `i18n.rs`).
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct I18nMap {
    pub ja: Option<I18nEntry>,
    pub zh: Option<I18nEntry>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct I18nEntry {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub desc: Option<String>,
    /// Provenance: "official" / "manual" / "ai". Not displayed; kept for
    /// future tooling (audit / overlay regeneration).
    #[serde(default)]
    pub source: Option<String>,
}
