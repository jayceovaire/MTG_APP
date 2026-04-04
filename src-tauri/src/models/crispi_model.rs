use crate::models::card_model::Card;
use crate::models::combos::TWO_CARD_COMBOS;
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
    VOLTRON_PIECE,
    LOOTING,
    IMPULSE_DRAW,
    GROUP_HUG,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeckArchetype {
    Turbo,
    Midrange,
    Stax,
    CommanderEngine,
    Voltron,
    GroupHug,
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

#[derive(Debug, Clone, Serialize)]
pub struct CrispiEvaluation {
    pub total_score: f32,
    pub raw_score: f32,
    pub turbo_signal: f32,
    pub midrange_signal: f32,
    pub stax_signal: f32,
    pub voltron_signal: f32,
    pub group_hug_signal: f32,
    pub commander_engine_signal: f32,
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
    r"draw .* cards?",
    r"whenever you draw",
    r"whenever an opponent .* draws",
    r"investigate",
    r"connive",
    r"explore",
    r"exile the top .* of your library. .* you may play",
    r"look at the top .* of your library .* put .* into your hand",
    r"reveal the top .* of your library .* put .* into your hand",
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
    r"search your library for .* card",
    r"search your library for an artifact",
    r"search your library for an instant",
    r"search your library for an enchantment",
    r"search your library for a creature",
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
    r"whenever you (draw|gain .* counters|create .* token|untap)",
    r"at the beginning of your upkeep",
    r"each upkeep",
    r"each end step",
    r"whenever an opponent (loses life|sacrifices|discards|draws)",
    r"whenever [^. ]+ deals combat damage",
    r"whenever enchanted creature deals combat damage",
    r"exile .*: .*",  // optional repeatable activated engines
];

static WINCON_PATTERNS: &[&str] = &[
    r"you win the game",
    r"target player loses the game",
    r"infinite",
];

static FAST_MANA_PATTERNS: &[&str] = &[
    r"add (?:\{.\}){2,}", 
    r"add .* (?:two|three|four|five|six|seven|eight|nine|ten) .* mana",
    r"add one mana of any color",
];

static RECURSION_PATTERNS: &[&str] = &[
    r"return .* from your graveyard",
    r"put .* from your graveyard",                       // usually to battlefield
    r"cast .* from your graveyard",
    r"rebound",
    r"flashback",                                       // instants/sorceries
    r"(?:persist|undying)",                             // creatures returning
    r"may return .* to .* from your graveyard",        // optional recursion
    r"return .* onto the battlefield from your graveyard",
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
    // Generic "reduce cost" of other spells or permanents
    r"(?:other )?spells? you cast cost .* less",
    r"target spell costs .* less to cast",
    r"you may pay .* rather than pay .* for another spell",
    r"reduce the cost of target .* by .*",
    r"spells? you control cost .* less",
    r"creature spells? you control cost .* less",
    r"artifact spells? you control cost .* less",
    r"enchantment spells? you control cost .* less",
    r"planeswalker spells? you control cost .* less",
];

static FAST_MANA_ONE_SHOT_PATTERNS: &[&str] = &[
    r"sacrifice .*\s*:\s*add \{.*\}{2,}",
    r"sacrifice .* add .* mana",
    r"sacrifice .*\s*:\s*create .* treasure",
];

static BURST_DRAW_PATTERNS: &[&str] = &[
    r"draw two cards",
    r"draw a card for each",
    r"draw card for each",
    r"draw cards for each",
    r"draw a card, then",            // often phrased as "draw a card, then …"
];

static MASS_DRAW_PATTERNS: &[&str] = &[
    r"draw three or more",
    r"draw cards equal to",
    r"draw \d+ cards",               // numeric draws
    r"draw x cards",
    r"reveal .* and put .* into your hand\. .* repeat this process",
    r"each player draws .* cards?",  // wheel-like effects
    r"draw your entire library",
];

static LOOTING_PATTERNS: &[&str] = &[
    r"draw (\d+) cards? and discard (\d+) cards?",     // Classic loot (e.g., "draw 2, discard 1")
    r"draw (\d+) cards? then discard (\d+) cards?",    // Alternative phrasing
    r"discard (\d+) cards? then draw (\d+) cards?",    // Backwards loot phrasing
    r"draw a card, then discard a card",              // Single card loot
    r"draw two cards, then discard a card",           // Common looting variant
];

static IMPULSE_DRAW_PATTERNS: &[&str] = &[
    r"look at the top (\d+) cards?",                      // e.g., "look at the top 4 cards"
    r"reveal the top (\d+) cards?",                       // reveal + selection
    r"choose (\d+) cards? to put into your hand",         // "choose 1 to draw"
    r"put the rest on the bottom of your library",        // finishing the look-and-draw effect
    r"may put any number of them into your hand",         // selection variant
    r"you may reveal .* and put .* into your hand",       // modern variant (Throne of Eldraine style)
];

static GROUP_HUG_PATTERNS: &[&str] = &[
    // Everyone draws cards
    r"each player draws? (?:.* )?cards?",
    r"each player may draw (?:.* )?cards?",
    r"each player's (?:.* )?step",

    // Everyone gains resources (mana, lands, counters)
    r"each player may play (?:.* )?lands?",
    r"each player adds? (?:.* )?mana",
    r"all players (?:adds?|gets?) (?:.* )?mana",
    r"each player searches (?:.* )?library",
    r"each player puts? (?:.* )?onto the battlefield",
    r"whenever a player (?:.* )?draws?",
    r"whenever a player (?:.* )?taps? (?:.* )?for mana",

    // Everyone gets buffs (counters, creatures)
    r"all creatures get (?:.* )?until end of turn",
    r"each (?:.* )?gets (?:.* )?counter",

    // Explicit phrasing for shared benefits
    r"you and each other player may .*",
    r"for each player, .*",
];
static WHEEL_PATTERNS: &[&str] = &[
    r"each player discards .* hand .* draws",
    r"discard your hand .* draw",
    r"each player shuffles .* hand .* library .* draws",
];

static VOLTRON_PIECE_PATTERNS: &[&str] = &[
    r"(equipped|enchanted) creature gets \+\d{2,}/\+\d{2,}", // only significant boosts
    r"(equipped|enchanted) creature has (double strike|trample|lifelink|deathtouch|flying|unblockable|can't be blocked)",
    r"put (\d+/\d+|2 or more) counters on (equipped|enchanted) creature",
    r"equip\s*\{[^\}]+\}",  // matches 'Equip {1}', 'Equip {2}{W}', etc.
    r"enchanted creature gets \+\d+/\+\d+", // optional for aura support
];

static MULTI_MANA_PRODUCER_PATTERNS: &[&str] = &[
    r"add (?:\{.\}){2,}", 
    r"add .* (?:two|three|four|five|six|seven|eight|nine|ten) .* mana",
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
static MULTI_MANA_PRODUCER_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| MULTI_MANA_PRODUCER_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static BURST_DRAW_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| BURST_DRAW_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static MASS_DRAW_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| MASS_DRAW_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static WHEEL_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| WHEEL_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static VOLTRON_PIECE_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| VOLTRON_PIECE_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static LOOTING_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| LOOTING_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static IMPULSE_DRAW_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| IMPULSE_DRAW_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());
static GROUP_HUG_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| GROUP_HUG_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());

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

static ANY_TUTOR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"search your library for .* card").unwrap());
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


pub fn infer_roles(card: &Card) -> HashSet<Role> {
    let mut roles = HashSet::new();

    if card.is_land() {
        roles.insert(Role::LAND);
    }

    if card.is_equipment() || card.is_aura() {
        roles.insert(Role::VOLTRON_PIECE);
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

        if VOLTRON_PIECE_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::VOLTRON_PIECE);
        }

        if LOOTING_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::LOOTING);
        }
        if IMPULSE_DRAW_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::IMPULSE_DRAW);
        }
        if GROUP_HUG_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::GROUP_HUG);
        }

        // Fast Mana Check (0-MV ramp or high-output)
        if FAST_MANA_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::FAST_MANA);
        } else if card.mana_value() == 0 && RAMP_REGEX.iter().any(|re| re.is_match(&normalized)) && !card.is_land() {
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
    let mut is_premium = FREE_SPELL_REGEX.is_match(&oracle_text) ||
        roles.contains(&Role::FAST_MANA) ||
        (roles.contains(&Role::TUTOR) && ANY_TUTOR_REGEX.is_match(&oracle_text) && mv <= 2) ||
        ((roles.contains(&Role::REMOVAL) || roles.contains(&Role::PROTECTION)) && is_inst && mv <= 1) ||
        (roles.contains(&Role::WINCON) && mv <= 2) ||
        (roles.contains(&Role::ENGINE) && mv <= 2);

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
    voltron_signal: f32,
    group_hug_signal: f32,
) -> DeckArchetype {
    if group_hug_signal >= 8.0 {
        DeckArchetype::GroupHug
    } else if stax_signal >= 15.0 {
        DeckArchetype::Stax
    } else if turbo_signal >= 18.0 {
        DeckArchetype::Turbo
    } else if voltron_signal >= 8.0 {
        DeckArchetype::Voltron
    } else if commander_engine_signal > 1.0 && turbo_signal > 12.0 {
        DeckArchetype::CommanderEngine
    } else if turbo_signal > 12.0 && turbo_signal > midrange_signal {
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
    let mut tutor_count = 0;
    let mut stax_count = 0;
    let mut wincon_count_efficient = 0;
    let mut draw_count_weighted = 0.0;
    let mut engine_count_weighted = 0.0;
    let mut premium_draw_count = 0;
    let mut explosive_mana_points = 0.0;
    let mut explosive_draw_points = 0.0;
    let mut stax_signal_weighted = 0.0;
    let mut voltron_signal_weighted = 0.0;
    let mut group_hug_signal_weighted = 0.0;

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
            tutor_count += 1;
            if tier == QualityTier::Premium && mv <= 2 && ANY_TUTOR_REGEX.is_match(&normalize_text(card.oracle_text().unwrap_or_default())) {
                premium_tutor_count += 1;
            }
        }
        if roles.contains(&Role::DRAW) || roles.contains(&Role::LOOTING) || roles.contains(&Role::IMPULSE_DRAW) { 
            draw_count_weighted += weight; 
            if tier == QualityTier::Premium {
                premium_draw_count += 1;
            }
        }
        if roles.contains(&Role::ENGINE) { engine_count_weighted += weight; }

        let mut card_mana_points: f32 = 0.0;
        if roles.contains(&Role::RITUAL) { card_mana_points = card_mana_points.max(2.5); }
        if roles.contains(&Role::TREASURE_BURST) { 
            let is_permanent = !card.is_instant() && !card.is_sorcery();
            let tb_weight = if !is_permanent { 2.0 } else { 0.5 };
            card_mana_points = card_mana_points.max(tb_weight); 
        }
        if roles.contains(&Role::SAC_MANA) { 
            let sac_weight = if mv <= 1 { 1.5 } else { 0.5 };
            card_mana_points = card_mana_points.max(sac_weight); 
        }
        if roles.contains(&Role::FAST_MANA_ONE_SHOT) { card_mana_points = card_mana_points.max(1.5); }
        if roles.contains(&Role::COST_REDUCTION) { 
            let cr_weight = if mv <= 1 { 1.5 } else { 0.3 };
            card_mana_points = card_mana_points.max(cr_weight); 
        }
        if roles.contains(&Role::FAST_MANA) { 
            // 0-MV artifact or multi-mana producer
            let is_multi = card.oracle_text().map(|t| MULTI_MANA_PRODUCER_REGEX.iter().any(|re| re.is_match(&normalize_text(t)))).unwrap_or(false);
            let fm_weight = if mv == 0 { 1.5 } else if is_multi && mv <= 1 { 1.0 } else if is_multi { 0.5 } else { 0.0 };
            card_mana_points = card_mana_points.max(fm_weight); 
        }
        explosive_mana_points += card_mana_points;

        let mut card_draw_points: f32 = 0.0;
        let is_spell = card.is_instant() || card.is_sorcery();
        if roles.contains(&Role::WHEEL) { card_draw_points = card_draw_points.max(2.5); }
        if roles.contains(&Role::MASS_DRAW) { 
            let mass_weight = if is_spell { 2.0 } else { 0.5 };
            card_draw_points = card_draw_points.max(mass_weight); 
        }
        if roles.contains(&Role::BURST_DRAW) { 
            let burst_weight = if is_spell { 1.5 } else { 0.3 };
            card_draw_points = card_draw_points.max(burst_weight); 
        }
        if roles.contains(&Role::LOOTING) {
            let loot_weight = if is_spell { 0.8 } else { 0.2 };
            card_draw_points = card_draw_points.max(loot_weight);
        }
        if roles.contains(&Role::IMPULSE_DRAW) {
            let impulse_weight = if is_spell { 1.2 } else { 0.3 };
            card_draw_points = card_draw_points.max(impulse_weight);
        }
        explosive_draw_points += card_draw_points;

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

        if roles.contains(&Role::VOLTRON_PIECE) {
            voltron_signal_weighted += weight;
        }

        if roles.contains(&Role::GROUP_HUG) {
            group_hug_signal_weighted += weight;
        }
    };

    for card in mainboard { process_card(card); }
    for card in commanders { process_card(card); }

    // Archetype Signals (Before Scoring)
    let turbo_signal = explosive_mana_points + explosive_draw_points;
    let midrange_signal = (consistency_weighted as f32) + (engine_count_weighted as f32) + (draw_count_weighted as f32);
    let stax_signal = stax_signal_weighted;
    let voltron_signal = voltron_signal_weighted;
    let group_hug_signal = group_hug_signal_weighted;
    let commander_engine_signal = commanders.iter().filter(|c| {
        let r = infer_roles(c);
        let is_engine = r.contains(&Role::ENGINE) && (r.contains(&Role::DRAW) || r.contains(&Role::TUTOR));
        let is_cost_reducer = r.contains(&Role::COST_REDUCTION);
        is_engine || is_cost_reducer
    }).count() as f32;

    let archetype = detect_archetype(stax_signal, commander_engine_signal, turbo_signal, midrange_signal, voltron_signal, group_hug_signal);

    let amv = if non_land_count > 0 { total_mv / non_land_count as f32 } else { 0.0 };

    // C — Consistency (0-5)
    let mut consistency_score = match consistency_weighted {
        v if v >= 8.0 => 5,
        v if v >= 5.0 => 4,
        v if v >= 3.0 => 3,
        v if v >= 1.2 => 2,
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
        n if n >= 13.0 => 5,
        n if n >= 9.0 => 4,
        n if n >= 5.0 => 3,
        n if n >= 2.0 => 2,
        _ => 1,
    };

    // I — Interaction (0-5)
    let mut interaction_score = match interaction_weighted {
        v if v >= 12.0 => 5,
        v if v >= 8.0 => 4,
        v if v >= 5.0 => 3,
        v if v >= 2.5 => 2,
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
        v if v >= 15.0 => 5,
        v if v >= 10.0 => 4,
        v if v >= 6.0  => 3,
        v if v >= 3.0  => 2,
        _ => 1,
    };

    let mut speed_score = efficiency_speed_score.max(explosive_speed_score);

    // P — Pivotability (0-5)
    let mut pivotability_score = match pivotability_weighted {
        n if n >= 8.0 => 5,
        n if n >= 5.0 => 4,
        n if n >= 3.0 => 3,
        n if n >= 1.5 => 2,
        _ => 1,
    };

    // APPLY STRUCTURAL FLOORS
    let mut applied_overrides = Vec::new();
    
    // 1. Command zone card advantage engine
    let cz_engine = commanders.iter().any(|c| {
        let roles = infer_roles(c);
        let tier = classify_card(c, &roles);
        let oracle = c.oracle_text().unwrap_or_default().to_lowercase();
        let is_combat_engine = oracle.contains("deals combat damage");
        
        roles.contains(&Role::ENGINE) && (roles.contains(&Role::DRAW) || roles.contains(&Role::TUTOR)) &&
        (tier == QualityTier::Premium || (tier == QualityTier::Efficient && !is_combat_engine))
    });
    if cz_engine {
        consistency_score = consistency_score.max(4);
        pivotability_score = pivotability_score.max(3);
        applied_overrides.push("CZ Engine (C>=4, P>=3)");
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
            pivotability_score = pivotability_score.max(3);
            applied_overrides.push("Turbo Archetype (C>=4, P>=3)");
        }
        DeckArchetype::Stax => {
            interaction_score = interaction_score.max(4);
            resilience_score = resilience_score.max(4);
            applied_overrides.push("Stax Archetype (I>=4, R>=4)");
        }
        DeckArchetype::CommanderEngine => {
            consistency_score = consistency_score.max(4);
            resilience_score = resilience_score.max(3);
            pivotability_score = pivotability_score.max(3);
            applied_overrides.push("CmdrEngine Archetype (C>=4, R>=3, P>=3)");
        }
        DeckArchetype::Voltron => {
            resilience_score = resilience_score.max(4);
            pivotability_score = pivotability_score.max(2);
            applied_overrides.push("Voltron Archetype (R>=4, P>=2)");
        }
        DeckArchetype::GroupHug => {
            consistency_score = consistency_score.max(3);
            pivotability_score = pivotability_score.max(3);
            applied_overrides.push("GroupHug Archetype (C>=3, P>=3)");
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
    let gc_bonus = (n_gc as f32 * 0.4).min(6.0);
    let raw_score = (raw_score + gc_bonus).min(30.0);
    
    // AMV Multiplier (Applied AFTER floors)
    let amv_multiplier = match amv {
        v if v <= 1.3 => 1.12,
        v if v <= 1.6 => 1.06,
        v if v <= 2.0 => 1.02,
        v if v <= 2.4 => 1.01,
        v if v <= 2.8 => 1.00,
        v if v <= 3.0 => 0.92,
        v if v <= 3.4 => 0.85,
        v if v <= 3.8 => 0.65,
        _ => 0.50,
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
    
    // Tutor influence: scales with the number of tutors
    let tutor_influence = (tutor_count as f32 * 0.02).min(0.15);
    
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
                
                let mut combo_bonus = 0.02 + tutor_influence; // Reduced base bonus (0.02) + tutor scaling
                
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
                combo_bonus = combo_bonus.clamp(0.02, 0.20);
                
                total_bonus += combo_bonus;
            }
        }
    }
    
    if any_combo_found {
        combo_multiplier = (1.0 + total_bonus).min(1.25);
    }
    
    let mut commander_mv_penalty = 0.0;
    if !commanders.is_empty() {
        commander_mv_penalty = commanders.iter().map(|c| {
            let mv = c.mana_value() as f32;
            if mv > 3.0 {
                // Negative weight (penalty): scales up
                (mv - 3.0) * 0.25 
            } else {
                // Positive weight (bonus): 3 and lower
                (mv - 3.5) * 0.15
            }
        }).sum::<f32>();
    }

    let final_multiplier = amv_multiplier * combo_multiplier;
    let total_score = (raw_score * final_multiplier - commander_mv_penalty).min(25.0).max(0.0);

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
        turbo_signal,
        midrange_signal,
        stax_signal,
        voltron_signal,
        group_hug_signal,
        commander_engine_signal,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::card_model::{Card, CardType, SuperType};

    fn make_card(name: &str, mv: u8, types: Vec<CardType>, text: &str) -> Card {
        let mut sub_types = vec![];
        if name.to_lowercase().contains("equipment") || text.to_lowercase().contains("equipped creature") {
            sub_types.push("Equipment".to_string());
        }
        if name.to_lowercase().contains("aura") || text.to_lowercase().contains("enchanted creature") {
            sub_types.push("Aura".to_string());
        }
        Card::new(
            0,
            "".to_string(),
            None,
            name.to_string(),
            None,
            mv,
            types,
            vec![],
            sub_types,
            Some(text.to_string()),
            "legal".to_string(),
            true,
            false,
            None
        )
    }

    #[test]
    fn test_rog_si_archetype() {
        let rog = make_card("Rograkh", 0, vec![CardType::Creature], "First strike, menace, trample. Partner.");
        let silas = make_card("Silas Renn", 3, vec![CardType::Creature, CardType::Artifact], "Deathtouch. Whenever Silas Renn deals combat damage to a player, you may cast target artifact card from your graveyard this turn. Partner.");
        
        let commanders = vec![rog, silas];
        
        let mut mainboard = vec![];
        // Fast Mana
        mainboard.push(make_card("Chrome Mox", 0, vec![CardType::Artifact], "Imprint - ... Add one mana of any color."));
        mainboard.push(make_card("Mox Amber", 0, vec![CardType::Artifact], "Add one mana of any color among legendary..."));
        mainboard.push(make_card("Mox Opal", 0, vec![CardType::Artifact], "Metalcraft - Add one mana..."));
        mainboard.push(make_card("Mana Crypt", 0, vec![CardType::Artifact], "Add {C}{C}."));
        mainboard.push(make_card("Mana Vault", 1, vec![CardType::Artifact], "{T}: Add {C}{C}{C}."));
        mainboard.push(make_card("Sol Ring", 1, vec![CardType::Artifact], "{T}: Add {C}{C}."));
        mainboard.push(make_card("Lotus Petal", 0, vec![CardType::Artifact], "Sacrifice Lotus Petal: Add one mana..."));
        mainboard.push(make_card("Dark Ritual", 1, vec![CardType::Sorcery], "Add {B}{B}{B}."));
        mainboard.push(make_card("Lion's Eye Diamond", 0, vec![CardType::Artifact], "Discard your hand, Sacrifice LED: Add {B}{B}{B}."));

        // Tutors (Consistency)
        mainboard.push(make_card("Demonic Tutor", 2, vec![CardType::Sorcery], "Search your library for a card..."));
        mainboard.push(make_card("Vampiric Tutor", 1, vec![CardType::Instant], "Search your library for a card..."));
        mainboard.push(make_card("Mystical Tutor", 1, vec![CardType::Instant], "Search your library for an instant or sorcery..."));
        mainboard.push(make_card("Imperial Seal", 1, vec![CardType::Sorcery], "Search your library for a card..."));
        mainboard.push(make_card("Gambit", 1, vec![CardType::Sorcery], "Search your library..."));
        mainboard.push(make_card("Wishclaw Talisman", 2, vec![CardType::Artifact], "Search..."));

        // Draw (Turbo/Consistency)
        mainboard.push(make_card("Ad Nauseam", 5, vec![CardType::Instant], "Reveal the top card... repeat this process..."));
        mainboard.push(make_card("Windfall", 3, vec![CardType::Sorcery], "Each player discards their hand, then draws cards..."));
        mainboard.push(make_card("Wheel of Fortune", 3, vec![CardType::Sorcery], "Each player discards their hand, then draws seven cards."));
        mainboard.push(make_card("Rhystic Study", 3, vec![CardType::Enchantment], "Whenever an opponent casts... draw a card..."));
        mainboard.push(make_card("Mystic Remora", 1, vec![CardType::Enchantment], "Whenever an opponent casts... draw a card..."));

        // Interaction
        mainboard.push(make_card("Force of Will", 5, vec![CardType::Instant], "Rather than pay this spell's mana cost... counter target spell."));
        mainboard.push(make_card("Pact of Negation", 0, vec![CardType::Instant], "Counter target spell."));
        mainboard.push(make_card("Mental Misstep", 1, vec![CardType::Instant], "Rather than pay... counter target spell with mana value 1."));
        mainboard.push(make_card("Deflecting Swat", 3, vec![CardType::Instant], "If you control a commander, you may cast this spell without paying its mana cost. Change the target..."));
        mainboard.push(make_card("Deadly Rollick", 4, vec![CardType::Instant], "If you control a commander... exile target creature."));
        mainboard.push(make_card("Fierce Guardianship", 3, vec![CardType::Instant], "If you control a commander... counter target noncreature spell."));

        // Wincons
        mainboard.push(make_card("Thassa's Oracle", 2, vec![CardType::Creature], "When Thassa's Oracle enters... win the game."));
        mainboard.push(make_card("Underworld Breach", 2, vec![CardType::Enchantment], "Each nonland card in your graveyard has escape."));
        mainboard.push(make_card("Brain Freeze", 2, vec![CardType::Instant], "Target player mills three cards. Storm."));

        // Lands
        for _ in 0..25 {
            mainboard.push(make_card("Badlands", 0, vec![CardType::Land], "{T}: Add {B} or {R}."));
        }

        let evaluation = calculate_crispi(&mainboard, &commanders, 15);
        
        println!("RogSi Test - Total Score: {}", evaluation.total_score);
        println!("RogSi Test - Raw Score: {}", evaluation.raw_score);
        println!("RogSi Test - Archetype: {:?}", evaluation.archetype);
        println!("RogSi Test - Interpretation: {}", evaluation.interpretation);
        println!("RogSi Test - Speed: {}", evaluation.speed.score);
        println!("RogSi Test - Consistency: {}", evaluation.consistency.score);
        println!("RogSi Test - Interaction: {}", evaluation.interaction.score);

        assert_eq!(evaluation.archetype, DeckArchetype::Turbo);
        assert!(evaluation.total_score >= 23.0, "Rog Si should be cEDH Optimized! Score was {}", evaluation.total_score);
    }

    #[test]
    fn test_midrange_archetype() {
        let thrasios = make_card("Thrasios, Triton Hero", 2, vec![CardType::Creature], "{4}: Scry 1, then reveal the top card of your library. If it's a land card, put it onto the battlefield tapped. Otherwise, draw a card. Partner.");
        let tymna = make_card("Tymna the Weaver", 3, vec![CardType::Creature], "Lifelink. At the beginning of your postcombat main phase, you may pay X life, where X is the number of opponents that were dealt combat damage by creatures this turn. If you do, draw X cards. Partner.");
        
        let commanders = vec![thrasios, tymna];
        
        let mut mainboard = vec![];
        // Typical Midrange pieces (Engines/Efficiency)
        mainboard.push(make_card("Sylvan Library", 2, vec![CardType::Enchantment], "At the beginning of your draw step, you may draw two additional cards..."));
        mainboard.push(make_card("Esper Sentinel", 1, vec![CardType::Creature], "Whenever an opponent casts their first noncreature spell each turn, draw a card unless that player pays {X}..."));
        mainboard.push(make_card("Seedborn Muse", 5, vec![CardType::Creature], "Untap all permanents you control during each other player's untap step."));
        mainboard.push(make_card("Smothering Tithe", 4, vec![CardType::Enchantment], "Whenever an opponent draws a card, that player may pay {2}. If they don't, you create a Treasure token."));
        mainboard.push(make_card("Arcane Signet", 2, vec![CardType::Artifact], "{T}: Add one mana of any color in your commander's color identity."));
        mainboard.push(make_card("Sol Ring", 1, vec![CardType::Artifact], "{T}: Add {C}{C}."));
        mainboard.push(make_card("Birds of Paradise", 1, vec![CardType::Creature], "{T}: Add one mana of any color."));
        mainboard.push(make_card("Demonic Tutor", 2, vec![CardType::Sorcery], "Search your library for a card..."));
        mainboard.push(make_card("Vampiric Tutor", 1, vec![CardType::Instant], "Search your library for a card..."));
        mainboard.push(make_card("Cyclonic Rift", 2, vec![CardType::Instant], "Return target nonland permanent... Overload {6}{u}."));
        mainboard.push(make_card("Assassin's Trophy", 2, vec![CardType::Instant], "Destroy target permanent..."));
        
        let evaluation = calculate_crispi(&mainboard, &commanders, 0);
        assert_eq!(evaluation.archetype, DeckArchetype::Midrange);
    }

    #[test]
    fn test_kosei_deck() {
        let kosei = make_card("Kosei, Penitent Warlord", 4, vec![CardType::Creature], "Whenever Kosei deals combat damage to a player, if it's enchanted, equipped, and has a counter on it, you draw cards equal to its power...");
        let commanders = vec![kosei];
        
        let mut mainboard = vec![];
        // Fast Mana
        mainboard.push(make_card("Chrome Mox", 0, vec![CardType::Artifact], "Imprint - ... Add one mana of any color."));
        mainboard.push(make_card("Lotus Petal", 0, vec![CardType::Artifact], "Sacrifice Lotus Petal: Add one mana..."));
        mainboard.push(make_card("Mana Vault", 1, vec![CardType::Artifact], "Add {C}{C}{C}."));
        mainboard.push(make_card("Sol Ring", 1, vec![CardType::Artifact], "Add {C}{C}."));

        // Protection (Resilience)
        mainboard.push(make_card("Autumn's Veil", 1, vec![CardType::Instant], "Spells you control can't be countered..."));
        mainboard.push(make_card("Gaea's Gift", 2, vec![CardType::Instant], "Target creature gets +1/+1 and gains reach, trample, hexproof, and indestructible..."));
        mainboard.push(make_card("Tamiyo's Safekeeping", 1, vec![CardType::Instant], "Target permanent gains hexproof and indestructible..."));
        mainboard.push(make_card("Vines of Vastwood", 1, vec![CardType::Instant], "Target creature can't be the target of spells..."));
        mainboard.push(make_card("Silkguard", 1, vec![CardType::Instant], "Any number of target creatures you control get a +1/+1 counter and gain hexproof..."));
        mainboard.push(make_card("Savage Summoning", 1, vec![CardType::Instant], "This spell can't be countered. The next creature card you cast... gains flash and enters with a +1/+1 counter and can't be countered."));
        mainboard.push(make_card("Strength of Will", 1, vec![CardType::Instant], "Target creature gains indestructible..."));

        // Removal (Interaction)
        mainboard.push(make_card("Beast Within", 3, vec![CardType::Instant], "Destroy target permanent..."));
        mainboard.push(make_card("Krosan Grip", 3, vec![CardType::Instant], "Split second. Destroy target artifact or enchantment."));
        mainboard.push(make_card("Return to Nature", 2, vec![CardType::Instant], "Destroy target artifact..."));
        mainboard.push(make_card("Collective Resistance", 2, vec![CardType::Instant], "Destroy target artifact..."));

        // Equipment/Auras (Pivotability/Wincon)
        mainboard.push(make_card("Blackblade Reforged", 2, vec![CardType::Artifact], "Equipped creature gets +1/+1 for each land you control."));
        mainboard.push(make_card("Hammer of Nazahn", 4, vec![CardType::Artifact], "Whenever Hammer of Nazahn or another Equipment enters... gain indestructible..."));
        mainboard.push(make_card("Helm of the Gods", 1, vec![CardType::Artifact], "Equipped creature gets +1/+1 for each enchantment you control."));
        mainboard.push(make_card("Lavaspur Boots", 1, vec![CardType::Artifact], "Equipped creature gets +1/+0 and gains haste and ward {1}."));
        mainboard.push(make_card("Swiftfoot Boots", 2, vec![CardType::Artifact], "Equipped creature gets hexproof and haste."));
        mainboard.push(make_card("Sword of the Animist", 2, vec![CardType::Artifact], "Whenever equipped creature attacks... search for a land..."));
        mainboard.push(make_card("Argentum Armor", 6, vec![CardType::Artifact], "Equipped creature gets +6/+6. Whenever equipped creature attacks, destroy target permanent."));
        mainboard.push(make_card("Hydra's Growth", 3, vec![CardType::Enchantment], "Enchanted creature gets +1/+1. At the beginning of your upkeep, double the number of +1/+1 counters on enchanted creature."));
        mainboard.push(make_card("Bear Umbra", 4, vec![CardType::Enchantment], "Enchanted creature gets +2/+2 and has \"Whenever this creature attacks, untap all lands you control.\" Totem armor."));
        
        // Ramp
        mainboard.push(make_card("Fyndhorn Elves", 1, vec![CardType::Creature], "{T}: Add {G}."));
        mainboard.push(make_card("Boreal Druid", 1, vec![CardType::Creature], "{T}: Add {C}."));
        mainboard.push(make_card("Sakura-Tribe Scout", 1, vec![CardType::Creature], "{T}: Put a land..."));
        mainboard.push(make_card("Biophagus", 2, vec![CardType::Creature], "{T}: Add one mana..."));
        mainboard.push(make_card("Emerald Medallion", 2, vec![CardType::Artifact], "Green spells cost {1} less..."));

        // Synergy
        mainboard.push(make_card("Season of Growth", 2, vec![CardType::Enchantment], "Whenever a creature enters... scry 1. Whenever you cast a spell that targets... draw a card."));
        mainboard.push(make_card("Snake Umbra", 3, vec![CardType::Enchantment], "Enchanted creature gets +1/+1... draw a card. Totem armor."));

        // Lands (dummy lands)
        for _ in 0..30 {
            mainboard.push(make_card("Forest", 0, vec![CardType::Land], "{T}: Add {G}."));
        }

        let evaluation = calculate_crispi(&mainboard, &commanders, 0);
        
        println!("Kosei Test - Total Score: {}", evaluation.total_score);
        println!("Kosei Test - Raw Score: {}", evaluation.raw_score);
        println!("Kosei Test - AMV: {:.2}", evaluation.total_score / evaluation.final_multiplier / evaluation.raw_score); // Not exact but give idea
        println!("Kosei Test - Multiplier: {}", evaluation.final_multiplier);
        println!("Kosei Test - Consistency: {}", evaluation.consistency.score);
        println!("Kosei Test - Resilience: {}", evaluation.resilience.score);
        println!("Kosei Test - Interaction: {}", evaluation.interaction.score);
        println!("Kosei Test - Speed: {}", evaluation.speed.score);
        println!("Kosei Test - Pivotability: {}", evaluation.pivotability.score);
        println!("Kosei Test - Interpretation: {}", evaluation.interpretation);
        println!("Kosei Test - Archetype: {:?}", evaluation.archetype);
        println!("Kosei Test - Voltron Signal: {}", evaluation.voltron_signal);

        // Assert Voltron Archetype
        assert_eq!(evaluation.archetype, DeckArchetype::Voltron);
        // This is expected to be under 19.0 now
        assert!(evaluation.total_score < 19.0, "Deck should not be Fringe cEDH! Score was {}", evaluation.total_score);
    }

    #[test]
    fn test_commander_mv_penalty() {
        let rog = make_card("Rograkh", 0, vec![CardType::Creature], "Partner.");
        let silas = make_card("Silas Renn", 3, vec![CardType::Creature], "Partner.");
        let thrasios = make_card("Thrasios", 2, vec![CardType::Creature], "Partner.");
        let kraum = make_card("Kraum", 5, vec![CardType::Creature], "Partner.");
        let etali = make_card("Etali", 7, vec![CardType::Creature], "");
        let kosei = make_card("Kosei", 4, vec![CardType::Creature], "");

        // Rog/Silas (0 + 3 = 3) -> Bonus
        // Penalty = (0 - 3.5)*0.15 + (3 - 3.5)*0.15 = -0.525 - 0.075 = -0.6
        let mainboard = vec![];
        let eval_rog_si = calculate_crispi(&mainboard, &vec![rog, silas.clone()], 0);
        assert!(eval_rog_si.commander_mv_penalty < 0.0);
        assert_eq!(eval_rog_si.commander_mv_penalty, -0.6);

        // Thrasios/Tymna (2 + 3 = 5)
        // Penalty = (2 - 3.5)*0.15 + (3 - 3.5)*0.15 = -0.225 - 0.075 = -0.3
        let tymna = make_card("Tymna", 3, vec![CardType::Creature], "Partner.");
        let eval_thras_tymna = calculate_crispi(&mainboard, &vec![thrasios, tymna], 0);
        assert_eq!(eval_thras_tymna.commander_mv_penalty, -0.3);

        // Kosei (4)
        // Penalty = (4 - 3.0)*0.25 = 0.25
        let eval_kosei = calculate_crispi(&mainboard, &vec![kosei], 0);
        assert_eq!(eval_kosei.commander_mv_penalty, 0.25);

        // Gitrog (5)
        // Penalty = (5 - 3.0)*0.25 = 0.50
        let gitrog = make_card("The Gitrog Monster", 5, vec![CardType::Creature], "");
        let eval_gitrog = calculate_crispi(&mainboard, &vec![gitrog], 0);
        assert_eq!(eval_gitrog.commander_mv_penalty, 0.50);

        // Kraum/Silas (5 + 3 = 8)
        // Penalty = (5 - 3.0)*0.25 + (3 - 3.5)*0.15 = 0.5 - 0.075 = 0.425
        let eval_kraum_silas = calculate_crispi(&mainboard, &vec![kraum.clone(), silas.clone()], 0);
        assert_eq!(eval_kraum_silas.commander_mv_penalty, 0.425);

        // Etali (7)
        // Penalty = (7 - 3.0)*0.25 = 1.0
        let eval_etali = calculate_crispi(&mainboard, &vec![etali], 0);
        assert_eq!(eval_etali.commander_mv_penalty, 1.0);

        // No commander: 0.0
        let eval_none = calculate_crispi(&mainboard, &vec![], 0);
        assert_eq!(eval_none.commander_mv_penalty, 0.0);
    }

    #[test]
    fn test_group_hug_archetype() {
        let phelddagrif = make_card("Phelddagrif", 4, vec![CardType::Creature], "{G}: Target opponent puts a 1/1 green Hippo creature token onto the battlefield. {W}: Target opponent gains 2 life. {U}: Target opponent may draw a card.");
        let commanders = vec![phelddagrif];
        
        let mut mainboard = vec![];
        // Symmetrical resource pieces (Group Hug)
        mainboard.push(make_card("Temple Bell", 3, vec![CardType::Artifact], "{T}: Each player draws a card."));
        mainboard.push(make_card("Dictate of Kruphix", 3, vec![CardType::Enchantment], "At the beginning of each player's draw step, that player draws an additional card."));
        mainboard.push(make_card("Rites of Flourishing", 3, vec![CardType::Enchantment], "Each player may play an additional land on each of their turns. Each player draws an additional card..."));
        mainboard.push(make_card("Collective Voyage", 1, vec![CardType::Sorcery], "Each player searches their library for basic land cards and puts them onto the battlefield..."));
        mainboard.push(make_card("Veteran Explorer", 1, vec![CardType::Creature], "When Veteran Explorer dies, each player searches their library for up to two basic land cards and puts them onto the battlefield..."));
        mainboard.push(make_card("Zhur-Taa Ancient", 5, vec![CardType::Creature], "Whenever a player taps a land for mana, that player adds one mana..."));
        mainboard.push(make_card("Heartbeat of Spring", 3, vec![CardType::Enchantment], "Whenever a player taps a land for mana, that player adds one mana..."));
        mainboard.push(make_card("Minds Aglow", 1, vec![CardType::Sorcery], "Each player may pay {X}. Then each player draws cards equal to the total amount of {X} paid."));
        mainboard.push(make_card("Howling Mine", 2, vec![CardType::Artifact], "Each player draws an additional card...")); // Simplified for test
        
        // Add some lands
        for _ in 0..30 {
            mainboard.push(make_card("Forest", 0, vec![CardType::Land], "{T}: Add {G}."));
        }

        let evaluation = calculate_crispi(&mainboard, &commanders, 0);
        
        println!("Group Hug Test - Total Score: {}", evaluation.total_score);
        println!("Group Hug Test - Signal: {}", evaluation.group_hug_signal);
        println!("Group Hug Test - Archetype: {:?}", evaluation.archetype);
        println!("Group Hug Test - Consistency Score: {}", evaluation.consistency.score);
        println!("Group Hug Test - Pivotability Score: {}", evaluation.pivotability.score);

        assert_eq!(evaluation.archetype, DeckArchetype::GroupHug);
        assert!(evaluation.group_hug_signal >= 8.0);
        // Floors should apply: Consistency >= 3, Pivotability >= 3
        assert!(evaluation.consistency.score >= 3);
        assert!(evaluation.pivotability.score >= 3);
    }
}
