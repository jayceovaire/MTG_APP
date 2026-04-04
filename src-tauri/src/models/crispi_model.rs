use crate::models::card_model::Card;
use serde::{Deserialize, Serialize};
use regex::Regex;
use once_cell::sync::Lazy;
use std::collections::{HashSet, HashMap};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Role {
    LAND,
    RAMP,
    DRAW,
    REMOVAL,
    TUTOR,
    PROTECTION,
    FIXING,
    ENGINE,
    WINCON,
    STAX,
    RECURSION,
    FAST_MANA,
    RITUAL,
    TREASURE_BURST,
    SAC_MANA,
    COST_REDUCTION,
    FAST_MANA_ONE_SHOT,
    BURST_DRAW,
    MASS_DRAW,
    WHEEL,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeckArchetype {
    Turbo,
    Midrange,
    Stax,
    CommanderEngine,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QualityTier {
    Premium,
    Efficient,
    Slow,
    LowImpact,
}

impl QualityTier {
    pub fn weight(&self) -> f32 {
        match self {
            QualityTier::Premium => 1.5,
            QualityTier::Efficient => 1.0,
            QualityTier::Slow => 0.75,
            QualityTier::LowImpact => 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrispiDimension {
    pub score: u8,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrispiEvaluation {
    pub total_score: f32,
    pub raw_score: f32,
    pub amv_multiplier: f32,
    pub combo_multiplier: f32,
    pub final_multiplier: f32,
    pub commander_mv_penalty: f32,
    pub land_score: f32,
    pub role_score: f32,
    pub interpretation: String,
    pub bracket: u8,
    pub archetype: DeckArchetype,
    pub detected_combos: Vec<String>,
    pub consistency: CrispiDimension,
    pub resilience: CrispiDimension,
    pub interaction: CrispiDimension,
    pub speed: CrispiDimension,
    pub pivotability: CrispiDimension,
}

static RAMP_PATTERNS: &[&str] = &[
    r"search your library for .* land",
    r"put .* land .* onto the battlefield",
    r"add \{",
    r"add one mana of any color",
    r"create a treasure",
    r"untap target land",
    r"untap target artifact",
    r"discard this card: search your library for a basic land",
];

static DRAW_PATTERNS: &[&str] = &[
    r"draw .* card",
    r"whenever you draw",
    r"whenever an opponent .* draws",
    r"investigate",
    r"connive",
    r"explore",
    r"exile the top .* of your library. .* you may play",
    r"look at the top .* of your library .* put .* into your hand",
];

static REMOVAL_PATTERNS: &[&str] = &[
    r"destroy target",
    r"exile target",
    r"counter target spell",
    r"return target .* to its owner's hand",
    r"sacrifice target",
    r"damage to target",
    r"deals .* damage to each",
    r"change the target",
];

static TUTOR_PATTERNS: &[&str] = &[
    r"search your library for a card",
    r"search your library for an artifact",
    r"search your library for an instant",
    r"search your library for an enchantment",
    r"search your library for a creature",
    r"search your library for a .* card",
];

static PROTECTION_PATTERNS: &[&str] = &[
    r"hexproof",
    r"indestructible",
    r"phase out",
    r"protection from",
    r"can't be countered",
    r"ward \{",
    r"counter target spell",
    r"change the target",
];

static FIXING_PATTERNS: &[&str] = &[
    r"add one mana of any color",
    r"add two mana of any one color",
    r"any color",
    r"search your library for .* land",
];

static STAX_PATTERNS: &[&str] = &[
    r"can't cast more than",
    r"spells cost .* more to cast",
    r"players can't",
    r"skip their",
    r"unless an opponent pays",
    r"each opponent must pay",
    r"opponents can't",
    r"unless they pay \{",
    r"unless that player pays \{",
    r"pay \{.*\} for each",
    r"players must pay",
    r"creatures enter the battlefield tapped",
    r"nonbasic lands are",
    r"nonbasic lands enter the battlefield tapped",
    r"players can't untap more than",
    r"creatures entering the battlefield don't",
    r"activated abilities can't be activated",
    r"can't cast more than one spell each turn",
    r"cant' attack",
];

static ENGINE_PATTERNS: &[&str] = &[
    r"whenever you",
    r"at the beginning of",
    r"each upkeep",
    r"each end step",
    r"whenever an opponent",
];

static WINCON_PATTERNS: &[&str] = &[
    r"you win the game",
    r"target player loses the game",
    r"infinite",
];

static FAST_MANA_PATTERNS: &[&str] = &[
    r"add \{.*\}\{.*\}", 
    r"add .* mana",
];

static RECURSION_PATTERNS: &[&str] = &[
    r"return .* from your graveyard",
    r"put .* from your graveyard",
    r"cast .* from your graveyard",
    r"exile .* from your graveyard",
    r"rebound",
];

static RITUAL_PATTERNS: &[&str] = &[
    r"add \{.*\}{3,}", // At least 3 mana symbols
    r"add three or more",
    r"add .* for each",
    r"add \{r\}\{r\}\{r\}",
    r"add \{b\}\{b\}\{b\}",
    r"add \{g\}\{g\}\{g\}",
    r"add \{u\}\{u\}\{u\}",
    r"add \{w\}\{w\}\{w\}",
];

static TREASURE_BURST_PATTERNS: &[&str] = &[
    r"create .* treasure token", // handles token and tokens
    r"create x treasure tokens",
    r"whenever .* create a treasure",
];

static SAC_MANA_PATTERNS: &[&str] = &[
    r"sacrifice .*\s*:\s*add",
    r"sacrifice .*\s*:\s*create .* treasure",
    r"sacrifice .* add \{",
];

static COST_REDUCTION_PATTERNS: &[&str] = &[
    r"cost .* less to cast",
    r"rather than pay its mana cost",
    r"you may pay .* rather than pay",
];

static FAST_MANA_ONE_SHOT_PATTERNS: &[&str] = &[
    r"sacrifice .*\s*:\s*add \{.*\}{2,}",
    r"sacrifice .* add .* mana",
    r"sacrifice .*\s*:\s*create .* treasure",
];

static BURST_DRAW_PATTERNS: &[&str] = &[
    r"draw two cards",
    r"draw card for each",
    r"draw cards for each",
];

static MASS_DRAW_PATTERNS: &[&str] = &[
    r"draw three or more",
    r"draw cards equal to",
    r"draw x cards",
];

static WHEEL_PATTERNS: &[&str] = &[
    r"each player discards .* hand .* draws",
    r"discard your hand .* draw",
];

static RAMP_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| RAMP_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static DRAW_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| DRAW_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static REMOVAL_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| REMOVAL_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static TUTOR_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| TUTOR_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static PROTECTION_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| PROTECTION_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static FIXING_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| FIXING_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static STAX_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| STAX_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static ENGINE_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| ENGINE_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static WINCON_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| WINCON_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static FAST_MANA_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| FAST_MANA_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static RECURSION_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| RECURSION_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static RITUAL_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| RITUAL_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static TREASURE_BURST_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| TREASURE_BURST_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static SAC_MANA_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| SAC_MANA_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static COST_REDUCTION_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| COST_REDUCTION_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static FAST_MANA_ONE_SHOT_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| FAST_MANA_ONE_SHOT_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static BURST_DRAW_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| BURST_DRAW_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static MASS_DRAW_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| MASS_DRAW_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static WHEEL_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| WHEEL_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());

static MULTI_COLOR_LAND_PATTERNS: &[&str] = &[
    r"add \{.\} or \{.\}",
    r"add \{.\}, \{.\}, or",
    r"add one mana of any color",
    r"add one mana of any one color",
];

static MULTI_MANA_LAND_PATTERNS: &[&str] = &[
    r"add \{.\}\{.\}",
];

static MULTI_COLOR_LAND_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| MULTI_COLOR_LAND_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static MULTI_MANA_LAND_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| MULTI_MANA_LAND_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());

static ANY_TUTOR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"search your library for a card").unwrap());
static FREE_SPELL_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"rather than pay this spell's mana cost|without paying its mana cost").unwrap());
static NON_TAPPING_ACTIVATION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:^|\.)\s*([^:\.]+):").unwrap());

static RE_REMINDER: Lazy<Regex> = Lazy::new(|| Regex::new(r"\(.*?\)").unwrap());
static RE_SPACES: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

pub fn normalize_text(text: &str) -> String {
    let mut normalized = text.to_lowercase();
    normalized = RE_REMINDER.replace_all(&normalized, "").to_string();
    normalized = normalized.replace('\n', " ");
    normalized = RE_SPACES.replace_all(&normalized, " ").to_string();
    normalized.trim().to_string()
}

pub fn normalize_card_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

struct TwoCardComboDef {
    card_a: &'static str,
    card_b: &'static str,
    effects: &'static [&'static str],
    prereqs: u8,
}

static TWO_CARD_COMBOS: &[TwoCardComboDef] = &[
    TwoCardComboDef { card_a: "Demonic Consultation", card_b: "Thassa's Oracle", effects: &[], prereqs: 0 },
    TwoCardComboDef { card_a: "Exquisite Blood", card_b: "Sanguine Bond", effects: &["lifegain", "damage"], prereqs: 1 },
    TwoCardComboDef { card_a: "Tainted Pact", card_b: "Thassa's Oracle", effects: &[], prereqs: 1 },
    TwoCardComboDef { card_a: "Exquisite Blood", card_b: "Vito, Thorn of the Dusk Rose", effects: &["lifegain", "damage"], prereqs: 1 },
    TwoCardComboDef { card_a: "Dramatic Reversal", card_b: "Isochron Scepter", effects: &["mana"], prereqs: 1 },
    TwoCardComboDef { card_a: "Dualcaster Mage", card_b: "Twinflame", effects: &["ETB", "LTB"], prereqs: 1 },
    TwoCardComboDef { card_a: "Bloodthirsty Conqueror", card_b: "Vito, Thorn of the Dusk Rose", effects: &["lifegain", "damage"], prereqs: 1 },
    TwoCardComboDef { card_a: "Niv-Mizzet, Parun", card_b: "Curiosity", effects: &["draw", "damage"], prereqs: 1 },
    TwoCardComboDef { card_a: "Gravecrawler", card_b: "Phyrexian Altar", effects: &["ETB", "LTB"], prereqs: 1 },
    TwoCardComboDef { card_a: "Bloodthirsty Conqueror", card_b: "Sanguine Bond", effects: &["lifegain", "damage"], prereqs: 1 },
    TwoCardComboDef { card_a: "Chatterfang, Squirrel General", card_b: "Pitiless Plunderer", effects: &["mana", "LTB"], prereqs: 1 },
    TwoCardComboDef { card_a: "Exquisite Blood", card_b: "Enduring Tenacity", effects: &["lifegain", "damage"], prereqs: 1 },
    TwoCardComboDef { card_a: "Bloodchief Ascension", card_b: "Mindcrank", effects: &["mill", "lifegain"], prereqs: 1 },
    TwoCardComboDef { card_a: "Basalt Monolith", card_b: "Forsaken Monument", effects: &["mana"], prereqs: 0 },
    TwoCardComboDef { card_a: "Exquisite Blood", card_b: "Marauding Blight-Priest", effects: &["lifegain", "damage"], prereqs: 1 },
    TwoCardComboDef { card_a: "Peregrin Took", card_b: "Nuka-Cola Vending Machine", effects: &["draw", "mana"], prereqs: 1 },
    TwoCardComboDef { card_a: "Dualcaster Mage", card_b: "Molten Duplication", effects: &["ETB", "LTB"], prereqs: 1 },
    TwoCardComboDef { card_a: "Bruvac the Grandiloquent", card_b: "Maddening Cacophony", effects: &["mill"], prereqs: 0 },
    TwoCardComboDef { card_a: "Rings of Brighthearth", card_b: "Basalt Monolith", effects: &["mana"], prereqs: 0 },
    TwoCardComboDef { card_a: "Felidar Guardian", card_b: "Restoration Angel", effects: &["ETB", "LTB"], prereqs: 0 },
    TwoCardComboDef { card_a: "Walking Ballista", card_b: "Heliod, Sun-Crowned", effects: &["damage", "lifegain"], prereqs: 1 },
    TwoCardComboDef { card_a: "Peregrine Drake", card_b: "Deadeye Navigator", effects: &["ETB", "LTB"], prereqs: 1 },
    TwoCardComboDef { card_a: "The Reaver Cleaver", card_b: "Aggravated Assault", effects: &["mana", "damage"], prereqs: 1 },
    TwoCardComboDef { card_a: "Niv-Mizzet, Parun", card_b: "Ophidian Eye", effects: &["draw", "damage"], prereqs: 1 },
    TwoCardComboDef { card_a: "Bloodthirsty Conqueror", card_b: "Marauding Blight-Priest", effects: &["lifegain", "damage"], prereqs: 1 },
    TwoCardComboDef { card_a: "Squee, the Immortal", card_b: "Food Chain", effects: &["mana", "ETB"], prereqs: 0 },
    TwoCardComboDef { card_a: "Blasphemous Act", card_b: "Repercussion", effects: &["damage"], prereqs: 0 },
    TwoCardComboDef { card_a: "Niv-Mizzet, Parun", card_b: "Tandem Lookout", effects: &["draw", "damage"], prereqs: 1 },
    TwoCardComboDef { card_a: "Peregrin Took", card_b: "Experimental Confectioner", effects: &["draw"], prereqs: 1 },
    TwoCardComboDef { card_a: "Staff of Domination", card_b: "Priest of Titania", effects: &["draw", "mana"], prereqs: 2 },
    TwoCardComboDef { card_a: "Sword of Feast and Famine", card_b: "Aggravated Assault", effects: &["mana"], prereqs: 1 },
    TwoCardComboDef { card_a: "Blowfly Infestation", card_b: "Nest of Scarabs", effects: &["ETB", "LTB"], prereqs: 0 },
    TwoCardComboDef { card_a: "Dualcaster Mage", card_b: "Saw in Half", effects: &["ETB"], prereqs: 1 },
    TwoCardComboDef { card_a: "Kiki-Jiki, Mirror Breaker", card_b: "Zealous Conscripts", effects: &["ETB"], prereqs: 0 },
    TwoCardComboDef { card_a: "Staff of Domination", card_b: "Elvish Archdruid", effects: &["draw", "mana"], prereqs: 2 },
    TwoCardComboDef { card_a: "Staff of Domination", card_b: "Marwyn, the Nurturer", effects: &["draw", "mana"], prereqs: 1 },
    TwoCardComboDef { card_a: "Dualcaster Mage", card_b: "Heat Shimmer", effects: &["ETB", "LTB"], prereqs: 1 },
    TwoCardComboDef { card_a: "Niv-Mizzet, the Firemind", card_b: "Curiosity", effects: &["draw", "damage"], prereqs: 1 },
    TwoCardComboDef { card_a: "Ashaya, Soul of the Wild", card_b: "Quirion Ranger", effects: &["ETB"], prereqs: 1 },
    TwoCardComboDef { card_a: "Scurry Oak", card_b: "Ivy Lane Denizen", effects: &["ETB"], prereqs: 0 },
    TwoCardComboDef { card_a: "Godo, Bandit Warlord", card_b: "Helm of the Host", effects: &["ETB"], prereqs: 1 },
    TwoCardComboDef { card_a: "Blowfly Infestation", card_b: "Hapatra, Vizier of Poisons", effects: &["ETB", "LTB"], prereqs: 1 },
    TwoCardComboDef { card_a: "Staff of Domination", card_b: "Circle of Dreams Druid", effects: &["draw", "mana"], prereqs: 2 },
    TwoCardComboDef { card_a: "Combat Celebrant", card_b: "Helm of the Host", effects: &["ETB"], prereqs: 1 },
    TwoCardComboDef { card_a: "Bruvac the Grandiloquent", card_b: "Traumatize", effects: &["mill"], prereqs: 0 },
    TwoCardComboDef { card_a: "Storm-Kiln Artist", card_b: "Haze of Rage", effects: &["mana"], prereqs: 1 },
    TwoCardComboDef { card_a: "Basalt Monolith", card_b: "Forensic Gadgeteer", effects: &["mana"], prereqs: 0 },
    TwoCardComboDef { card_a: "Exquisite Blood", card_b: "Starscape Cleric", effects: &["lifegain", "damage"], prereqs: 1 },
    TwoCardComboDef { card_a: "Malcolm, Keen-Eyed Navigator", card_b: "Glint-Horn Buccaneer", effects: &["damage", "draw"], prereqs: 1 },
    TwoCardComboDef { card_a: "Kaalia of the Vast", card_b: "Master of Cruelties", effects: &[], prereqs: 1 },
    TwoCardComboDef { card_a: "Karn, the Great Creator", card_b: "Mycosynth Lattice", effects: &[], prereqs: 0 },
    TwoCardComboDef { card_a: "Dualcaster Mage", card_b: "Electroduplicate", effects: &["ETB", "LTB"], prereqs: 1 },
    TwoCardComboDef { card_a: "Ondu Spiritdancer", card_b: "Secret Arcade // Dusty Parlor", effects: &["ETB"], prereqs: 1 },
    TwoCardComboDef { card_a: "Neheb, the Eternal", card_b: "Aggravated Assault", effects: &["mana"], prereqs: 1 },
    TwoCardComboDef { card_a: "Aggravated Assault", card_b: "Selvala, Heart of the Wilds", effects: &["mana"], prereqs: 1 },
    TwoCardComboDef { card_a: "Devoted Druid", card_b: "Swift Reconfiguration", effects: &["mana"], prereqs: 1 },
    TwoCardComboDef { card_a: "Savage Ventmaw", card_b: "Aggravated Assault", effects: &["mana"], prereqs: 1 },
    TwoCardComboDef { card_a: "Cut Your Losses", card_b: "Fraying Sanity", effects: &["mill"], prereqs: 1 },
    TwoCardComboDef { card_a: "Satya, Aetherflux Genius", card_b: "Lightning Runner", effects: &["ETB", "LTB"], prereqs: 1 },
    TwoCardComboDef { card_a: "Bloodthirsty Conqueror", card_b: "Starscape Cleric", effects: &["lifegain", "damage"], prereqs: 1 },
    TwoCardComboDef { card_a: "Umbral Mantle", card_b: "Priest of Titania", effects: &["mana"], prereqs: 3 },
    TwoCardComboDef { card_a: "Bloom Tender", card_b: "Freed from the Real", effects: &["mana"], prereqs: 1 },
    TwoCardComboDef { card_a: "Duskmantle Guildmage", card_b: "Mindcrank", effects: &["mill", "damage"], prereqs: 1 },
    TwoCardComboDef { card_a: "Rosie Cotton of South Lane", card_b: "Scurry Oak", effects: &["ETB"], prereqs: 0 },
    TwoCardComboDef { card_a: "Bruvac the Grandiloquent", card_b: "Cut Your Losses", effects: &["mill"], prereqs: 1 },
    TwoCardComboDef { card_a: "Mycosynth Lattice", card_b: "Vandalblast", effects: &[], prereqs: 0 },
    TwoCardComboDef { card_a: "Solphim, Mayhem Dominus", card_b: "Heartless Hidetsugu", effects: &["damage"], prereqs: 0 },
    TwoCardComboDef { card_a: "Eternal Scourge", card_b: "Food Chain", effects: &["mana", "ETB"], prereqs: 2 },
    TwoCardComboDef { card_a: "Chain of Smog", card_b: "Witherbloom Apprentice", effects: &["lifegain"], prereqs: 2 },
    TwoCardComboDef { card_a: "Staff of Domination", card_b: "Selvala, Heart of the Wilds", effects: &["draw", "mana"], prereqs: 2 },
    TwoCardComboDef { card_a: "Combat Celebrant", card_b: "Kiki-Jiki, Mirror Breaker", effects: &["ETB"], prereqs: 0 },
    TwoCardComboDef { card_a: "Umbral Mantle", card_b: "Elvish Archdruid", effects: &["mana"], prereqs: 3 },
    TwoCardComboDef { card_a: "Abdel Adrian, Gorion's Ward", card_b: "Animate Dead", effects: &["ETB"], prereqs: 5 },
    TwoCardComboDef { card_a: "Umbral Mantle", card_b: "Marwyn, the Nurturer", effects: &["mana"], prereqs: 2 },
];

pub fn infer_roles(card: &Card) -> HashSet<Role> {
    let mut roles = HashSet::new();

    if card.is_land() {
        roles.insert(Role::LAND);
    }
    
    if let Some(oracle_text) = card.oracle_text() {
        let normalized = normalize_text(oracle_text);

        if RAMP_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::RAMP);
        }
        if DRAW_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::DRAW);
        }
        if REMOVAL_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::REMOVAL);
        }
        if TUTOR_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::TUTOR);
        }
        if PROTECTION_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::PROTECTION);
        }
        if FIXING_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::FIXING);
        }
        
        // STAX check with "you may pay" guard clause
        let phrases: Vec<&str> = normalized.split('.').collect();
        for phrase in phrases {
            let trimmed = phrase.trim();
            if !trimmed.is_empty() && !trimmed.contains("you may pay") {
                if STAX_REGEX.iter().any(|re| re.is_match(trimmed)) &&
                   !(roles.contains(&Role::LAND) && !roles.contains(&Role::TUTOR) && !roles.contains(&Role::RECURSION)) {
                    roles.insert(Role::STAX);
                    break;
                }
            }
        }

        if ENGINE_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::ENGINE);
        }
        if WINCON_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::WINCON);
        }
        if RECURSION_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::RECURSION);
        }

        if RITUAL_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::RITUAL);
        }
        if TREASURE_BURST_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::TREASURE_BURST);
        }
        if SAC_MANA_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::SAC_MANA);
        }
        if COST_REDUCTION_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::COST_REDUCTION);
        }
        if FAST_MANA_ONE_SHOT_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::FAST_MANA_ONE_SHOT);
        }
        if BURST_DRAW_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::BURST_DRAW);
        }
        if MASS_DRAW_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::MASS_DRAW);
        }
        if WHEEL_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::WHEEL);
        }

        // Fast Mana Check
        if FAST_MANA_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::FAST_MANA);
        } else if card.mana_value() <= 1 && RAMP_REGEX.iter().any(|re| re.is_match(&normalized)) && !card.is_land() {
            roles.insert(Role::FAST_MANA);
        }
    }

    roles
}

pub fn classify_card(card: &Card, roles: &HashSet<Role>) -> QualityTier {
    let oracle_text = card.oracle_text().map(|t| normalize_text(t)).unwrap_or_default();

    // Land Quality Classification
    if card.is_land() {
        let enters_tapped = oracle_text.contains("enters the battlefield tapped") && !oracle_text.contains("unless");
        let produces_multi_color = MULTI_COLOR_LAND_REGEX.iter().any(|re| re.is_match(&oracle_text));
        let produces_multi_mana = MULTI_MANA_LAND_REGEX.iter().any(|re| re.is_match(&oracle_text));
        
        let has_utility = roles.contains(&Role::TUTOR) || 
                          roles.contains(&Role::RECURSION) || 
                          roles.contains(&Role::REMOVAL) || 
                          roles.contains(&Role::PROTECTION) || 
                          roles.contains(&Role::STAX) || 
                          roles.contains(&Role::DRAW) || 
                          roles.contains(&Role::ENGINE) || 
                          roles.contains(&Role::WINCON);

        // Premium: Enters untapped AND (produces >1 mana OR >1 color OR provides tutor / recursion / interaction effect)
        if !enters_tapped && (produces_multi_mana || produces_multi_color || has_utility) {
            return QualityTier::Premium;
        }
        
        // Efficient: Enters tapped BUT has a strong effect OR Enters untapped and only produces a single color (basic/mono)
        if enters_tapped && (has_utility || produces_multi_mana) {
            return QualityTier::Efficient;
        }
        if !enters_tapped {
             return QualityTier::Efficient;
        }
        
        // Slow: Enters tapped and only produces mana
        return QualityTier::Slow;
    }

    let mv = card.mana_value();
    let is_inst = is_instant_speed(card);
    let is_creature_artifact = card.is_creature() || card.is_artifact();

    // Premium criteria
    let mut is_premium = (mv <= 2 && !roles.is_empty()) ||
        FREE_SPELL_REGEX.is_match(&oracle_text) ||
        roles.contains(&Role::FAST_MANA) ||
        (roles.contains(&Role::TUTOR) && ANY_TUTOR_REGEX.is_match(&oracle_text) && mv <= 2) ||
        ((roles.contains(&Role::REMOVAL) || roles.contains(&Role::PROTECTION)) && is_inst && mv <= 2) ||
        (roles.contains(&Role::WINCON) && mv <= 3);

    // Refinement: For creatures/artifacts, require non-tapping activation or immediate impact
    if is_premium && is_creature_artifact {
        let has_non_tap = has_non_tapping_activation(&oracle_text);
        let impact = oracle_text.contains("enters the battlefield") || 
                     oracle_text.contains("when you cast") || 
                     oracle_text.contains("ward") ||
                     roles.contains(&Role::STAX); // Stax is usually immediate impact
        
        if !has_non_tap && !impact {
            is_premium = false;
        }
    }

    if is_premium {
        return QualityTier::Premium;
    }

    // Slow criteria: MV >= 4 and no combo/fast-mana/tutor
    if mv >= 4 && !roles.contains(&Role::WINCON) && !roles.contains(&Role::FAST_MANA) && !roles.contains(&Role::TUTOR) {
        return QualityTier::Slow;
    }

    // Efficient: Default for most playable cards, or mono-colored ramp (which might have been Premium before)
    if !roles.is_empty() {
        return QualityTier::Efficient;
    }

    // Default
    QualityTier::LowImpact
}

pub fn is_instant_speed(card: &Card) -> bool {
    if card.is_instant() {
        return true;
    }
    if let Some(oracle_text) = card.oracle_text() {
        let normalized = normalize_text(oracle_text);
        if normalized.contains("flash") {
            return true;
        }
    }
    false
}

fn has_non_tapping_activation(text: &str) -> bool {
    for cap in NON_TAPPING_ACTIVATION_REGEX.captures_iter(text) {
        let cost = cap.get(1).unwrap().as_str();
        if !cost.contains("{t}") && !cost.contains("tap") {
            return true;
        }
    }
    false
}

fn detect_archetype(
    stax_signal: f32,
    commander_engine_signal: f32,
    turbo_signal: f32,
    midrange_signal: f32,
) -> DeckArchetype {
    if stax_signal >= 8.0 {
        DeckArchetype::Stax
    } else if commander_engine_signal > 0.0 && turbo_signal > 8.0 {
        DeckArchetype::CommanderEngine
    } else if turbo_signal > midrange_signal {
        DeckArchetype::Turbo
    } else {
        DeckArchetype::Midrange
    }
}

pub fn calculate_crispi(mainboard: &[Card], commanders: &[Card], n_gc: u32) -> CrispiEvaluation {
    let mut total_mv = 0.0;
    let mut non_land_count = 0;
    
    let mut consistency_weighted = 0.0;
    let mut resilience_weighted = 0.0;
    let mut interaction_weighted = 0.0;
    let mut pivotability_weighted = 0.0;

    let mut fast_mana_count = 0;
    let mut free_interaction_count = 0;
    let mut premium_tutor_count = 0;
    let mut stax_count = 0;
    let mut wincon_count_efficient = 0;
    let mut draw_count_weighted = 0.0;
    let mut engine_count_weighted = 0.0;
    let mut premium_draw_count = 0;
    let mut explosive_mana_points = 0.0;
    let mut explosive_draw_points = 0.0;
    let mut stax_signal_weighted = 0.0;

    let mut process_card = |card: &Card| {
        let roles = infer_roles(card);
        let tier = classify_card(card, &roles);
        let weight = tier.weight();

        if !card.is_land() {
            total_mv += card.mana_value() as f32;
            non_land_count += 1;
        }

        let mv = card.mana_value();
        let is_inst = is_instant_speed(card);

        // Consistency: Tutors, Draw, Engine
        if roles.contains(&Role::TUTOR) {
            consistency_weighted += weight;
            if tier == QualityTier::Premium && mv <= 2 && ANY_TUTOR_REGEX.is_match(&normalize_text(card.oracle_text().unwrap_or_default())) {
                premium_tutor_count += 1;
            }
        }
        if roles.contains(&Role::DRAW) { 
            draw_count_weighted += weight; 
            if tier == QualityTier::Premium {
                premium_draw_count += 1;
            }
        }
        if roles.contains(&Role::ENGINE) { engine_count_weighted += weight; }

        if roles.contains(&Role::RITUAL) { explosive_mana_points += 2.5; }
        if roles.contains(&Role::TREASURE_BURST) { explosive_mana_points += 2.0; }
        if roles.contains(&Role::SAC_MANA) { explosive_mana_points += 1.8; }
        if roles.contains(&Role::COST_REDUCTION) { explosive_mana_points += 1.5; }
        if roles.contains(&Role::FAST_MANA_ONE_SHOT) { explosive_mana_points += 1.8; }
        if roles.contains(&Role::FAST_MANA) { explosive_mana_points += 1.0; }

        if roles.contains(&Role::BURST_DRAW) { explosive_draw_points += 1.5; }
        if roles.contains(&Role::MASS_DRAW) { explosive_draw_points += 2.0; }
        if roles.contains(&Role::WHEEL) { explosive_draw_points += 2.5; }

        // Interaction: Removal & Stax
        if roles.contains(&Role::REMOVAL) {
            if is_inst {
                interaction_weighted += weight;
                // Free interaction check
                let is_free = mv == 0 || (card.oracle_text().map(|t| FREE_SPELL_REGEX.is_match(&normalize_text(t))).unwrap_or(false));
                if is_free {
                    free_interaction_count += 1;
                }
            }
        }
        if roles.contains(&Role::STAX) {
            interaction_weighted += weight;
            stax_count += 1;
            stax_signal_weighted += if card.is_land() { 0.3 } else { 1.0 };
        }

        // Resilience: Protection, Recursion
        if roles.contains(&Role::PROTECTION) {
            resilience_weighted += weight;
            // Instant protection can also count as free interaction if free
            if is_inst {
                let is_free = mv == 0 || (card.oracle_text().map(|t| FREE_SPELL_REGEX.is_match(&normalize_text(t))).unwrap_or(false));
                if is_free {
                    free_interaction_count += 1;
                }
            }
        }
        if roles.contains(&Role::RECURSION) {
            resilience_weighted += weight;
        }

        // Speed: Fast Mana, Efficient Wincons
        if roles.contains(&Role::FAST_MANA) {
            fast_mana_count += 1;
        }

        if roles.contains(&Role::WINCON) {
            if tier == QualityTier::Premium || (mv <= 3) {
                wincon_count_efficient += 1;
            }
            if roles.len() > 1 {
                pivotability_weighted += weight;
            }
        }
    };

    for card in mainboard { process_card(card); }
    for card in commanders { process_card(card); }

    // Archetype Signals (Before Scoring)
    let turbo_signal = explosive_mana_points + explosive_draw_points;
    let midrange_signal = (consistency_weighted as f32) + (engine_count_weighted as f32) + (draw_count_weighted as f32);
    let stax_signal = stax_signal_weighted;
    let commander_engine_signal = commanders.iter().filter(|c| {
        let r = infer_roles(c);
        r.contains(&Role::ENGINE) || r.contains(&Role::COST_REDUCTION)
    }).count() as f32;

    let archetype = detect_archetype(stax_signal, commander_engine_signal, turbo_signal, midrange_signal);

    let amv = if non_land_count > 0 { total_mv / non_land_count as f32 } else { 0.0 };

    // C — Consistency (0-5)
    let mut consistency_score = match consistency_weighted {
        v if v >= 8.0 => 5,
        v if v >= 5.0 => 4,
        v if v >= 3.0 => 3,
        v if v >= 1.0 => 2,
        _ => 1,
    };

    // Win Package Density Adjustment
    let win_package_density = if non_land_count > 0 { wincon_count_efficient as f32 / non_land_count as f32 } else { 0.0 };
    if win_package_density > 0.08 {
        consistency_score = (consistency_score + 1).min(5);
    }

    if consistency_weighted >= 8.0 && (draw_count_weighted > 5.0 || engine_count_weighted > 3.0) {
        consistency_score = 5;
    }

    // R — Resilience (0-5)
    let mut resilience_score = match resilience_weighted {
        n if n >= 6.0 => 5,
        n if n >= 4.0 => 4,
        n if n >= 2.5 => 3,
        n if n >= 1.2 => 2,
        _ => 1,
    };

    // I — Interaction (0-5)
    let mut interaction_score = match interaction_weighted {
        v if v >= 10.0 => 5,
        v if v >= 7.0 => 4,
        v if v >= 4.0 => 3,
        v if v >= 2.0 => 2,
        _ => 1,
    };

    // S — Speed (0-5)
    // Heuristic for win turn: Base 8, reduced by efficiency metrics
    let mut estimated_win_turn = 8.5;
    if amv <= 2.2 { estimated_win_turn -= 2.0; }
    else if amv <= 2.6 { estimated_win_turn -= 1.0; }
    
    estimated_win_turn -= (fast_mana_count as f32 / 3.0).min(2.5);
    estimated_win_turn -= (premium_tutor_count as f32 / 2.5).min(2.0);
    if wincon_count_efficient >= 2 { estimated_win_turn -= 1.0; }
    
    let mut efficiency_speed_score = match estimated_win_turn {
        t if t <= 3.5 => 5,
        t if t <= 4.5 => 4,
        t if t <= 5.5 => 3,
        t if t <= 6.5 => 2,
        _ => 1,
    };
    let speed_cap = match fast_mana_count {
        0..=3 => 2,
        4..=7 => 3,
        _ => 5,
    };
    efficiency_speed_score = efficiency_speed_score.min(speed_cap);

    let mana_velocity = (fast_mana_count as f32 * 0.8) + explosive_mana_points;
    let draw_velocity = (premium_draw_count as f32 * 0.8) + explosive_draw_points;
    
    let explosive_speed_score = match mana_velocity.max(draw_velocity) {
        v if v >= 18.0 => 5,
        v if v >= 13.0 => 4,
        v if v >= 9.0  => 3,
        v if v >= 5.0  => 2,
        _ => 1,
    };

    let mut speed_score = efficiency_speed_score.max(explosive_speed_score);

    // P — Pivotability (0-5)
    let mut pivotability_score = match pivotability_weighted {
        n if n >= 6.0 => 5,
        n if n >= 4.0 => 4,
        n if n >= 3.0 => 3,
        n if n >= 1.5 => 2,
        _ => 1,
    };

    // APPLY STRUCTURAL FLOORS
    let mut applied_overrides = Vec::new();
    
    // 1. Command zone card advantage engine
    let cz_engine = commanders.iter().any(|c| {
        let roles = infer_roles(c);
        roles.contains(&Role::ENGINE) && (roles.contains(&Role::DRAW) || roles.contains(&Role::TUTOR))
    });
    if cz_engine {
        consistency_score = consistency_score.max(4);
        pivotability_score = pivotability_score.max(4);
        applied_overrides.push("CZ Engine (C>=4, P>=4)");
    }
    
    // 2. 8+ free interaction spells
    if free_interaction_count >= 8 {
        interaction_score = 5;
        resilience_score = resilience_score.max(4);
        applied_overrides.push("8+ Free Interaction (I=5, R>=4)");
    }
    
    // 3. Compact <= 2 card deterministic win package
    if wincon_count_efficient >= 2 {
        speed_score = 5;
        consistency_score = (consistency_score + 1).min(5);
        applied_overrides.push("Compact Wincon (S=5, C+1)");
    }
    
    // 4. 10+ fast mana pieces
    if fast_mana_count >= 10 {
        speed_score = 5;
        applied_overrides.push("10+ Fast Mana (S=5)");
    }
    
    // 5. 5+ premium tutors (<= 2 MV)
    if premium_tutor_count >= 5 {
        consistency_score = 5;
        applied_overrides.push("5+ Premium Tutors (C=5)");
    }
    
    // Archetype-Aware Adjustments (After Floors)
    match archetype {
        DeckArchetype::Turbo => {
            consistency_score = consistency_score.max(4);
            pivotability_score = pivotability_score.max(4);
            applied_overrides.push("Turbo Archetype (C>=4, P>=4)");
        }
        DeckArchetype::Stax => {
            interaction_score = interaction_score.max(4);
            resilience_score = resilience_score.max(4);
            applied_overrides.push("Stax Archetype (I>=4, R>=4)");
        }
        DeckArchetype::CommanderEngine => {
            consistency_score = consistency_score.max(4);
            resilience_score = resilience_score.max(4);
            pivotability_score = pivotability_score.max(4);
            applied_overrides.push("CmdrEngine Archetype (C>=4, R>=4, P>=4)");
        }
        DeckArchetype::Midrange => {}
    }

    let override_text = if !applied_overrides.is_empty() {
        format!(" [Floors: {}]", applied_overrides.join(", "))
    } else {
        "".to_string()
    };

    let consistency = CrispiDimension {
        score: consistency_score,
        justification: format!("Weighted consistency value: {:.2}. Premium tutors: {}.{}", 
            consistency_weighted, premium_tutor_count, override_text),
    };
    
    let resilience = CrispiDimension {
        score: resilience_score,
        justification: format!("Weighted resilience value: {:.2}.{}", resilience_weighted, override_text),
    };
    
    let interaction = CrispiDimension {
        score: interaction_score,
        justification: format!("Weighted interaction value: {:.2} ({} free, {} stax).{}", 
            interaction_weighted, free_interaction_count, stax_count, override_text),
    };
    
    let speed = CrispiDimension {
        score: speed_score,
        justification: format!("Efficiency: {} (Win turn: {:.1}, Cap: {}), Explosive: {} (Mana vel: {:.1}, Draw vel: {:.1}).{}", 
            efficiency_speed_score, estimated_win_turn, speed_cap, explosive_speed_score, mana_velocity, draw_velocity, override_text),
    };

    let pivotability = CrispiDimension {
        score: pivotability_score,
        justification: format!("Weighted pivotability value: {:.2}.{}", pivotability_weighted, override_text),
    };

    let raw_score = (consistency.score + resilience.score + interaction.score + speed.score + pivotability.score) as f32;
    
    // AMV Multiplier (Applied AFTER floors)
    let amv_multiplier = match amv {
        v if v <= 1.3 => 1.24,
        v if v <= 1.6 => 1.16,
        v if v <= 2.0 => 1.06,
        v if v <= 2.4 => 1.02,
        v if v <= 2.8 => 1.00,
        v if v <= 3.0 => 0.94,
        v if v <= 3.4 => 0.88,
        v if v <= 3.8 => 0.72,
        _ => 0.55,
    };
    
    // --- Two-Card Infinite Combo Detection ---
    let mut detected_combos = Vec::new();
    let mut combo_multiplier = 1.0;
    
    let all_deck_cards: Vec<&Card> = mainboard.iter().chain(commanders.iter()).collect();
    let mut card_map: HashMap<String, &Card> = HashMap::new();
    for card in &all_deck_cards {
        card_map.insert(normalize_card_name(card.get_name()), *card);
    }
    
    let mut total_bonus = 0.0;
    let mut any_combo_found = false;
    
    let has_tutor = all_deck_cards.iter().any(|c| {
        let roles = infer_roles(c);
        roles.contains(&Role::TUTOR)
    });
    
    for combo in TWO_CARD_COMBOS {
        let norm_a = normalize_card_name(combo.card_a);
        let norm_b = normalize_card_name(combo.card_b);
        
        if card_map.contains_key(&norm_a) && card_map.contains_key(&norm_b) {
            let card_a = card_map.get(&norm_a).unwrap();
            let card_b = card_map.get(&norm_b).unwrap();
            
            let mut prereqs_met = true;
            if combo.prereqs > 0 {
                if non_land_count < (5 + combo.prereqs as usize) {
                    prereqs_met = false;
                }
            }
            
            if prereqs_met {
                any_combo_found = true;
                let effects_str = if combo.effects.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", combo.effects.join(", "))
                };
                detected_combos.push(format!("{} + {}{}", combo.card_a, combo.card_b, effects_str));
                
                let mut combo_bonus = 0.15; // Base bonus for MV <= 3
                
                if has_tutor {
                    combo_bonus += 0.05;
                }
                
                let total_mv = (card_a.mana_value() + card_b.mana_value()) as f32;
                let mv_penalty = (total_mv - 3.0).max(0.0) * 0.01;
                combo_bonus -= mv_penalty;
                
                // Speed Penalty
                let mut speed_penalty = 0.0;
                let cards = [card_a, card_b];
                for card in cards {
                    let oracle = card.oracle_text().unwrap_or_default().to_lowercase();
                    let is_inst = is_instant_speed(card);
                    
                    // If it's a sorcery, it's slow
                    if !is_inst && card.is_sorcery() {
                        speed_penalty += 0.02;
                    }
                    
                    // Tap ability check on creatures/artifacts
                    if (card.is_creature() || card.is_artifact()) && !has_non_tapping_activation(&oracle) {
                        // If it's a creature and doesn't have haste, it's slow
                        if card.is_creature() && !oracle.contains("haste") {
                            speed_penalty += 0.02;
                        }
                    }
                }
                combo_bonus -= speed_penalty;
                
                // Cap and floor per combo
                combo_bonus = combo_bonus.clamp(0.05, 0.20);
                
                total_bonus += combo_bonus;
            }
        }
    }
    
    if any_combo_found {
        combo_multiplier = (1.0 + total_bonus).min(1.30);
    }
    
    let final_multiplier = amv_multiplier * combo_multiplier;
    let total_score = (raw_score * final_multiplier).min(25.0);

    let commander_mv_penalty = commanders.iter().map(|c| c.mana_value() as f32).sum::<f32>() * 0.05;
    let land_count = mainboard.iter().filter(|c| c.is_land()).count();
    let land_score = (land_count as f32 / 38.0).min(1.0);
    let role_score = raw_score / 25.0;

    let interpretation = match total_score {
        s if s >= 23.0 => "cEDH Optimized",
        s if s >= 19.0 => "Fringe cEDH",
        s if s >= 15.0 => "High Power",
        s if s >= 9.0 => "Focused / Synergistic",
        _ => "Casual / Battlecruiser",
    }.to_string();

    // Bracket Calculation
    let mut bracket = if n_gc == 0 {
        2
    } else if n_gc <= 3 {
        3
    } else {
        4
    };

    if any_combo_found {
        bracket = bracket.max(4);
    }

    if bracket == 2 && total_score <= 8.0 && amv > 3.5 {
        bracket = 1;
    } else if total_score >= 24.0 {
        bracket = 5;
    }

    CrispiEvaluation {
        total_score,
        raw_score,
        amv_multiplier,
        combo_multiplier,
        final_multiplier,
        commander_mv_penalty,
        land_score,
        role_score,
        interpretation,
        bracket,
        archetype,
        detected_combos,
        consistency,
        resilience,
        interaction,
        speed,
        pivotability,
    }
}
