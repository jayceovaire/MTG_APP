use crate::models::card_model::Card;
use serde::{Deserialize, Serialize};
use regex::Regex;
use once_cell::sync::Lazy;
use std::collections::HashSet;

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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QualityTier {
    Premium,
    Efficient,
    Slow,
}

impl QualityTier {
    pub fn weight(&self) -> f32 {
        match self {
            QualityTier::Premium => 1.0,
            QualityTier::Efficient => 0.6,
            QualityTier::Slow => 0.2,
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
    pub interpretation: String,
    pub bracket: u8,
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
    r"unless an opponent pays",
    r"each opponent must pay",
    r"opponents can't",
    r"unless they pay \{",
    r"unless that player pays \{",
    r"pay \{.*\} for each",
    r"players must pay",
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
static FAST_MANA_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| FAST_MANA_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());

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
        
        // STAX check with "you may pay" guard clause
        let phrases: Vec<&str> = normalized.split('.').collect();
        for phrase in phrases {
            let trimmed = phrase.trim();
            if !trimmed.is_empty() && !trimmed.contains("you may pay") {
                if STAX_REGEX.iter().any(|re| re.is_match(trimmed)) {
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

        // Fast Mana Check
        if FAST_MANA_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::FAST_MANA);
        } else if card.mana_value() <= 1 && roles.contains(&Role::RAMP) && !card.is_land() {
            roles.insert(Role::FAST_MANA);
        }
    }

    roles
}

pub fn classify_card(card: &Card, roles: &HashSet<Role>) -> QualityTier {
    let oracle_text = card.oracle_text().map(|t| normalize_text(t)).unwrap_or_default();

    // Land Quality Classification (CRITICAL)
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
        
        // Efficient: Enters tapped BUT has a strong effect OR Enters untapped and only produces a single color
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

    // Premium criteria
    let is_premium = {
        // 1. MV <= 2 and has any role (heuristic for "commonly played")
        (mv <= 2 && !roles.is_empty()) ||
        // 2. Free spell
        FREE_SPELL_REGEX.is_match(&oracle_text) ||
        // 3. Fast mana / Produces mana >= its cost
        roles.contains(&Role::FAST_MANA) ||
        // 4. Unrestricted tutor <= 2 MV
        (roles.contains(&Role::TUTOR) && ANY_TUTOR_REGEX.is_match(&oracle_text) && mv <= 2) ||
        // 5. <= 2 MV instant-speed premium interaction
        ((roles.contains(&Role::REMOVAL) || roles.contains(&Role::PROTECTION)) && is_inst && mv <= 2) ||
        // 6. Compact combo piece <= 3 MV
        (roles.contains(&Role::WINCON) && mv <= 3)
    };

    if is_premium {
        return QualityTier::Premium;
    }

    // Slow criteria
    if mv >= 4 {
        return QualityTier::Slow;
    }

    // Default
    QualityTier::Efficient
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
        if roles.contains(&Role::DRAW) { draw_count_weighted += weight; }
        if roles.contains(&Role::ENGINE) { engine_count_weighted += weight; }

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

    let amv = if non_land_count > 0 { total_mv / non_land_count as f32 } else { 0.0 };

    // C — Consistency (0-5)
    let mut consistency_score = match consistency_weighted {
        v if v >= 8.0 => 5,
        v if v >= 5.0 => 4,
        v if v >= 3.0 => 3,
        v if v >= 1.0 => 2,
        _ => 1,
    };
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
    
    let mut speed_score = match estimated_win_turn {
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
    speed_score = speed_score.min(speed_cap);

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
        justification: format!("Estimated win turn: {:.1} (Fast mana: {}, Cap: {}).{}", 
            estimated_win_turn, fast_mana_count, speed_cap, override_text),
    };

    let pivotability = CrispiDimension {
        score: pivotability_score,
        justification: format!("Weighted pivotability value: {:.2}.{}", pivotability_weighted, override_text),
    };

    let raw_score = (consistency.score + resilience.score + interaction.score + speed.score + pivotability.score) as f32;
    
    // AMV Multiplier (Applied AFTER floors)
    let amv_multiplier = match amv {
        v if v <= 2.2 => 1.12,
        v if v <= 2.6 => 1.06,
        v if v <= 3.0 => 1.00,
        v if v <= 3.4 => 0.88,
        v if v <= 3.8 => 0.72,
        _ => 0.55,
    };
    
    let total_score = raw_score * amv_multiplier;

    let interpretation = match total_score {
        s if s >= 24.0 => "cEDH Optimized",
        s if s >= 20.0 => "Fringe cEDH",
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

    if bracket == 2 && total_score <= 8.0 && amv > 3.5 {
        bracket = 1;
    } else if total_score >= 24.0 {
        bracket = 5;
    }

    CrispiEvaluation {
        total_score,
        raw_score,
        amv_multiplier,
        interpretation,
        bracket,
        consistency,
        resilience,
        interaction,
        speed,
        pivotability,
    }
}
