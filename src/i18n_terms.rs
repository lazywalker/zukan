//! Hardcoded translation tables for field labels and value terms.
//!
//! These mirror zukan-assets/source/i18n/terms.json verbatim. That file lives
//! under `source/`, which is NOT shipped in the Release artifact (only `data/`
//! and `icons/` are). To avoid a runtime dependency on a file we can't embed,
//! we vendor the table here as Rust consts.
//!
//! Monster/item name and description translations DO ship in the artifact
//! (baked into data/{monsters,items}.json via the i18n overlay); those are
//! read from the record at runtime by `i18n::localized`.

/// Field-label translations (Type / Species / Element / ...).
///
/// Tuple layout: (English key, ja, zh).
pub const LABELS: &[(&str, &str, &str)] = &[
    ("Type", "種類", "种类"),
    ("Species", "種族", "种族"),
    ("Size", "サイズ", "体型"),
    ("Element", "属性", "属性"),
    ("Weakness", "弱点", "弱点"),
    ("Ailments", "状態異常", "异常状态"),
    ("Location", "生息地", "栖息地"),
    ("Weak(stars)", "弱点(★)", "弱点(★)"),
    ("Sub-species", "亜種", "亚种"),
    ("Games", "登場作品", "登场作品"),
    ("Rarity", "レア度", "稀有度"),
    ("Value", "価値", "价值"),
    ("Carry", "所持数", "携带数"),
    ("Icon type", "アイコン種", "图标类型"),
];

/// Value-term translations (Fire / Flying Wyvern / Large / Ancient Forest / ...).
///
/// Keys are English values straight from the data files. The renderer looks up
/// by English key but displays the translated text, so colors and alignment
/// stay correct across languages.
pub const TERMS: &[(&str, &str, &str)] = &[
    // Sizes
    ("Large", "大型", "大型"),
    ("Small", "小型", "小型"),
    // Species / types
    ("Elder Dragon", "古龍種", "古龙种"),
    ("Flying Wyvern", "飛竜種", "飞龙种"),
    ("Bird Wyvern", "鳥竜種", "鸟龙种"),
    ("Leviathan", "海竜種", "海龙种"),
    ("Fanged Beast", "牙獣種", "牙兽种"),
    ("Brute Wyvern", "獣竜種", "兽龙种"),
    ("Fanged Wyvern", "牙竜種", "牙龙种"),
    ("Herbivore", "草食種", "草食种"),
    ("Neopteron", "甲虫種", "甲虫种"),
    ("Amphibian", "両生種", "两生种"),
    ("Carapaceon", "甲殻種", "甲壳种"),
    ("Temnoceran", "鋏角種", "铗角种"),
    ("Wingdrake", "翼竜", "翼龙"),
    ("Piscine Wyvern", "魚竜種", "鱼龙种"),
    ("Lynian", "獣人種", "兽人种"),
    ("Construct", "造龍", "造龙"),
    ("Snake Wyvern", "蛇竜種", "蛇龙种"),
    ("Fish", "魚類", "鱼类"),
    ("Cephalopod", "頭足種", "头足种"),
    ("Relict", "遺存種", "遗存种"),
    ("Relicts", "遺存種", "遗存种"),
    ("Demi Elder", "半古龍", "半古龙"),
    ("???", "???", "???"),
    // Elements
    ("Fire", "火", "火"),
    ("Water", "水", "水"),
    ("Thunder", "雷", "雷"),
    ("Ice", "氷", "冰"),
    ("Dragon", "龍", "龙"),
    ("Poison", "毒", "毒"),
    // Ailments
    ("Fireblight", "火属性やけ状態", "火属性异常"),
    ("Waterblight", "水属性やけ状態", "水属性异常"),
    ("Thunderblight", "雷属性やけ状態", "雷属性异常"),
    ("Iceblight", "氷属性やけ状態", "冰属性异常"),
    ("Dragonblight", "龍属性やけ状態", "龙属性异常"),
    ("Paralysis", "麻痺", "麻痹"),
    ("Stun", "気絶", "眩晕"),
    ("Sleep", "睡眠", "睡眠"),
    ("Blastblight", "爆破属性やけ状態", "爆破属性异常"),
    ("Bleeding", "裂傷", "裂伤"),
    ("Defence Down", "防御力ダウン", "防御力下降"),
    ("Noxious Poison", "猛毒", "猛毒"),
    ("Deadly Poison", "致命的な毒", "致命毒"),
    ("Soiled", "悪臭", "恶臭"),
    ("Snowman", "雪だるま", "雪人"),
    ("Webbed", "糸拘束", "蛛网束缚"),
    ("Muddy", "泥まみれ", "泥泞"),
    ("Bubble", "泡状態", "泡沫"),
    ("Fatigue", "疲労", "疲劳"),
    ("Frenzy Virus", "狂竜ウイルス", "狂龙病毒"),
    ("Effluvium", "瘴気", "瘴气"),
    ("Life Drain", "体力吸収", "体力吸取"),
    ("Hellfireblight", "業火属性やけ状態", "业火属性异常"),
    ("Confusion", "混乱", "混乱"),
    ("Bloodblight", "血属性やけ状態", "血属性异常"),
    ("Slimeblight", "粘液属性やけ状態", "粘液属性异常"),
    ("Blastscourge", "爆破状態", "爆破状态"),
    ("BlastScourge", "爆破状態", "爆破状态"),
    ("Tarred", "タール状態", "焦油状态"),
    ("Mucus", "粘液", "粘液"),
    ("Boned", "骨状態", "骨化"),
    // Locations (MHW + Wilds)
    ("Ancient Forest", "古代樹の森", "古代树森林"),
    ("Wildspire Waste", "大蟻塚の荒地", "大蚁冢荒地"),
    ("Coral Highlands", "陸珊瑚の台地", "陆珊瑚台地"),
    ("Rotten Vale", "瘴気の谷", "瘴气之谷"),
    ("Elder's Recess", "地脈の収束所", "地脉回廊"),
    ("Hoarfrost Reach", "渡りの凍て地", "永霜冻土"),
    ("Guiding Lands", "導きの地", "聚魔之地"),
    ("Confluence of Fates", "龍結晶の地", "龙结晶之地"),
    ("Caverns of El Dorado", "エルドラドの地下洞", "黄金乡地下洞"),
    ("Secluded Valley", "幽境の谷", "幽境之谷"),
    ("Great Ravine", "大峡谷", "大峡谷"),
    ("Everstream", "地脈", "地脉"),
    ("Ruins of Wyveria", "竜の都", "龙之都"),
    ("Oilwell Basin", "油涌きの谷", "涌油谷"),
    ("Scarlet Forest", "緋色の森", "绯红森林"),
    ("Windward Plains", "風鳴りの谷", "风鸣之谷"),
    ("Iceshard Cliffs", "氷碎の崖", "冰碎崖"),
];

/// Look up a value-term translation by English key.
///
/// `Lang::En` returns the input verbatim. Otherwise tries exact match, then
/// Title-cased (so "fire" and "FIRE" both resolve to the "Fire" entry), then
/// falls back to the original `text`.
pub fn term(text: &str, lang: crate::i18n::Lang) -> String {
    if matches!(lang, crate::i18n::Lang::En) {
        return text.to_string();
    }
    let want_ja = matches!(lang, crate::i18n::Lang::Ja);
    // Exact key first.
    for (en, ja, zh) in TERMS {
        if *en == text {
            return (if want_ja { ja } else { zh }).to_string();
        }
    }
    // Title-cased fallback.
    let titled = titlecase(text);
    for (en, ja, zh) in TERMS {
        if *en == titled {
            return (if want_ja { ja } else { zh }).to_string();
        }
    }
    text.to_string()
}

/// Look up a field-label translation.
///
/// `Lang::En` returns the input verbatim.
pub fn label(text: &str, lang: crate::i18n::Lang) -> String {
    if matches!(lang, crate::i18n::Lang::En) {
        return text.to_string();
    }
    let want_ja = matches!(lang, crate::i18n::Lang::Ja);
    for (en, ja, zh) in LABELS {
        if *en == text {
            return (if want_ja { ja } else { zh }).to_string();
        }
    }
    text.to_string()
}

/// Minimal Title-Case: capitalize the first ASCII letter, lowercase the rest.
/// Matches Python's `str.title()` closely enough for our lookup needs (the
/// terms table is the source of truth; this only bridges case variants).
fn titlecase(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut up_next = true;
    for c in s.chars() {
        if c.is_ascii() {
            if up_next && c.is_ascii_alphabetic() {
                out.push(c.to_ascii_uppercase());
                up_next = false;
            } else {
                out.push(c.to_ascii_lowercase());
            }
        } else {
            out.push(c);
            up_next = false;
        }
    }
    out
}
