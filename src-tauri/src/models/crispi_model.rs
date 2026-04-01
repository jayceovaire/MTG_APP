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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrispiDimension {
    pub score: u8,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrispiEvaluation {
    pub total_score: f32,
    pub raw_score: u8,
    pub amv_multiplier: f32,
    pub interpretation: String,
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
    r"enters the battlefield tapped",
    r"spells cost .* more to cast",
    r"players can't",
    r"skip their",
    r"whenever an opponent casts .* spell, .* unless they pay",
    r"whenever an opponent taps a land for mana",
    r"unless they pay \{",
    r"unless that player pays \{",
    r"may pay \{",
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

static RECURSION_PATTERNS: &[&str] = &[
    r"return .* from your graveyard",
    r"put .* from your graveyard",
    r"cast .* from your graveyard",
    r"exile .* from your graveyard",
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
static RECURSION_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| RECURSION_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());

static RE_REMINDER: Lazy<Regex> = Lazy::new(|| Regex::new(r"\(.*?\)").unwrap());
static RE_SPACES: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

pub fn normalize_text(text: &str) -> String {
    let mut normalized = text.to_lowercase();
    normalized = RE_REMINDER.replace_all(&normalized, "").to_string();
    normalized = normalized.replace('\n', " ");
    normalized = RE_SPACES.replace_all(&normalized, " ").to_string();
    normalized.trim().to_string()
}

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
        if STAX_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::STAX);
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
    }

    roles
}

pub fn calculate_crispi(role_counts: &HashMap<Role, usize>, amv: f32) -> CrispiEvaluation {
    // C — Consistency: Tutors, Draw, Engine
    let draw_count = *role_counts.get(&Role::DRAW).unwrap_or(&0);
    let tutor_count = *role_counts.get(&Role::TUTOR).unwrap_or(&0);
    let engine_count = *role_counts.get(&Role::ENGINE).unwrap_or(&0);
    let consistency_score = match draw_count + tutor_count * 2 + engine_count {
        n if n >= 20 => 5,
        n if n >= 15 => 4,
        n if n >= 10 => 3,
        n if n >= 5 => 2,
        n if n >= 2 => 1,
        _ => 0,
    };
    let consistency = CrispiDimension {
        score: consistency_score,
        justification: format!("Found {} draw, {} tutors, and {} engine pieces.", draw_count, tutor_count, engine_count),
    };

    // R — Resilience: Protection, Recursion, Engine
    let protection_count = *role_counts.get(&Role::PROTECTION).unwrap_or(&0);
    let recursion_count = *role_counts.get(&Role::RECURSION).unwrap_or(&0);
    let resilience_score = match protection_count + recursion_count + engine_count / 2 {
        n if n >= 12 => 5,
        n if n >= 8 => 4,
        n if n >= 5 => 3,
        n if n >= 3 => 2,
        n if n >= 1 => 1,
        _ => 0,
    };
    let resilience = CrispiDimension {
        score: resilience_score,
        justification: format!("Includes {} protection and {} recursion cards.", protection_count, recursion_count),
    };

    // I — Interaction: Removal, Stax
    let removal_count = *role_counts.get(&Role::REMOVAL).unwrap_or(&0);
    let stax_count = *role_counts.get(&Role::STAX).unwrap_or(&0);
    let interaction_score = match removal_count + stax_count * 2 {
        n if n >= 15 => 5,
        n if n >= 10 => 4,
        n if n >= 6 => 3,
        n if n >= 3 => 2,
        n if n >= 1 => 1,
        _ => 0,
    };
    let interaction = CrispiDimension {
        score: interaction_score,
        justification: format!("Equipped with {} removal and {} stax/tax pieces.", removal_count, stax_count),
    };

    // S — Speed: Ramp, AMV
    let ramp_count = *role_counts.get(&Role::RAMP).unwrap_or(&0);
    let mut speed_score = match ramp_count {
        n if n >= 12 => 4,
        n if n >= 8 => 3,
        n if n >= 5 => 2,
        n if n >= 2 => 1,
        _ => 0,
    };
    if amv < 2.0 { speed_score += 1; }
    else if amv > 3.5 && speed_score > 0 { speed_score -= 1; }
    speed_score = speed_score.min(5);
    let speed = CrispiDimension {
        score: speed_score,
        justification: format!("Ramp density is {} with an Average Mana Value (AMV) of {:.2}.", ramp_count, amv),
    };

    // P — Pivotability: Wincons, Engine
    let wincon_count = *role_counts.get(&Role::WINCON).unwrap_or(&0);
    let pivotability_score = match wincon_count + engine_count / 2 {
        n if n >= 5 => 5,
        n if n >= 4 => 4,
        n if n >= 3 => 3,
        n if n >= 2 => 2,
        n if n >= 1 => 1,
        _ => 0,
    };
    let pivotability = CrispiDimension {
        score: pivotability_score,
        justification: format!("Found {} distinct win-conditions and {} engines.", wincon_count, engine_count),
    };

    let raw_score = consistency.score + resilience.score + interaction.score + speed.score + pivotability.score;
    
    // AMV Adjustment
    let max_target_amv = 5.0;
    let amv_multiplier = 1.0 + ((max_target_amv - amv) / max_target_amv) * 0.10;
    let amv_multiplier = amv_multiplier.clamp(0.90, 1.10);
    
    let total_score = raw_score as f32 * amv_multiplier;

    let interpretation = match total_score {
        s if s >= 24.0 => "cEDH Optimized",
        s if s >= 20.0 => "Fringe cEDH",
        s if s >= 15.0 => "High Power",
        s if s >= 9.0 => "Focused / Synergistic",
        _ => "Casual / Battlecruiser",
    }.to_string();

    CrispiEvaluation {
        total_score,
        raw_score,
        amv_multiplier,
        interpretation,
        consistency,
        resilience,
        interaction,
        speed,
        pivotability,
    }
}
