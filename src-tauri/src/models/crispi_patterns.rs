use crate::models::card_model::Card;
use crate::models::crispi_types::Role;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

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
    r"whenever an opponent .* draw a card",
    r"whenever an opponent .* draw .* cards",
    r"look at the top \d+ cards? of your library.*put .* into your hand",
];

static REMOVAL_PATTERNS: &[&str] = &[
    r"destroy target",
    r"exile target",
    r"counter target spell",
    r"return target .* to (its|their) owner's hand",
    r"target .* gets -\d+/-\d+",
    r"fight target",
    r"deals? \d+ damage to target",
    r"sacrifice target",
    r"target player sacrifices",
    r"tap target .*\. it doesn't untap",
    r"target .* loses all abilities",
    r"enchant .* can't attack or block",
    r"exile up to one target .*",
];

static MASS_REMOVAL_PATTERNS: &[&str] = &[
    r"destroy all (creatures|artifacts|enchantments|lands|planeswalkers)",
    r"exile all (creatures|artifacts|enchantments|graveyards|planeswalkers)",
    r"(destroy|exile) all (creatures|artifacts|enchantments|lands|planeswalkers)",
    r"deals? \d+ damage to each creature",
    r"creatures get -\d+/-\d+",
    r"each player sacrifices a creature",
    r"each opponent sacrifices a creature",
    r"each player sacrifices those",
    r"return all .* to (their|its) owner's hand",
    r"sacrifice all (creatures|artifacts|enchantments|lands|planeswalkers)",
    r"deals? \d+ damage to each creature",
    r"creatures get -",
    r"all creatures get -",
    r"sacrifice all",
    r"return all .* to (their|its) owner's hand",
    r"choose a (creature|artifact|enchantment) type.*destroy all",
    r"exile target .* you don't control",
    r"exile all .* you don't control",
    r"(destroy|exile|sacrifice).*for each creature|for each creature.*(destroy|exile|sacrifice)",
];

static TUTOR_PATTERNS: &[&str] = &[
    r"search your library for .* card",
    r"search your library for an artifact",
    r"search your library for an instant",
    r"search your library for an enchantment",
    r"search your library for a creature",
];

static LAND_TUTOR_PATTERNS: &[&str] = &[
    r"search your library for .* land",
    r"search your library for a basic land",
    r"search your library for a plains",
    r"search your library for an island",
    r"search your library for a swamp",
    r"search your library for a mountain",
    r"search your library for a forest",
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
    r"search your library for a plains",
    r"search your library for an island",
    r"search your library for a swamp",
    r"search your library for a mountain",
    r"search your library for a forest",
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
    r"whenever (?:.* )?enters the battlefield",
    r"at the beginning of your upkeep",
    r"each upkeep",
    r"each end step",
    r"whenever an opponent (loses life|sacrifices|discards)",
    r"whenever [^. ]+ deals combat damage",
    r"whenever enchanted creature deals combat damage",
    r"exile .*: .*",
];

static WINCON_PATTERNS: &[&str] = &[
    r"you win the game",
    r"target player loses the game",
    r"infinite",
];

static FAST_MANA_PATTERNS: &[&str] = &[
    // Produces 2+ mana explicitly
    r"add (?:\{[wubrgc]\}){2,}",
    // Ritual wording
    r"add [wubrgc ]{2,}",
    // Sacrifice for mana (LED, Petal, Treasures)
    r"sacrifice .*: add",
    // Treasure creation
    r"create a treasure token",
    // Untap lands burst
    r"untap up to \d+ lands",
];

static RECURSION_PATTERNS: &[&str] = &[
    r"return .* from your graveyard",
    r"put .* from your graveyard",
    r"cast .* from your graveyard",
    r"rebound",
    r"flashback",
    r"(?:persist|undying)",
    r"may return .* to .* from your graveyard",
    r"return .* onto the battlefield from your graveyard",
    r"return .* to the battlefield from your graveyard",
    r"return .* to the battlefield from exile",
    r"return .* to the battlefield from .*",
    r"when (this|that|target) creature dies, return it to the battlefield",
    r"dies, return it to the battlefield",
    r"gains .* when .* dies, return it to the battlefield",
];

static RITUAL_PATTERNS: &[&str] = &[
    r"add \{.\}\{.\}\{.\}",
    r"add .* mana equal to",
    r"add .* for each",
    r"add \{.\}\{.\} mana",
];

static TREASURE_BURST_PATTERNS: &[&str] = &[
    r"create (?:two|three|four|five|x) treasure",
    r"create a treasure token for each",
    r"create (two|three|four|five|six|seven|eight|nine|ten) treasure tokens",
    // Variable X treasures
    r"create x treasure tokens",
    // Scaling treasure burst
    r"for each .* create a treasure token",
    // ETB burst treasures
    r"enters the battlefield.*create .* treasure tokens",
];

static SAC_MANA_PATTERNS: &[&str] = &[
    r"sacrifice .*: add",
    r"discard your hand, sacrifice .*: add",
];

static COST_REDUCTION_PATTERNS: &[&str] = &[
    r"spells you cast cost \{1\} less",
    r"artifact spells you cast cost",
    r"creature spells you cast cost",
    r"instant and sorcery spells you cast cost",
];

static FAST_MANA_ONE_SHOT_PATTERNS: &[&str] = &[
    // Sacrifice for mana (Lotus Petal, Treasures, LED)
    r"sacrifice .*: add",
    // Exile from hand for mana (Spirit Guides)
    r"exile .* from your hand: add",
    // Treasure creation
    r"create a treasure token",
    // Rituals (instants/sorceries that add mana)
    r"add [wubrgc ]{2,}",
    // ETB burst mana (rare but real)
    r"enters the battlefield.*add",
    // Untap lands burst (Frantic Search / Turnabout style)
    r"untap up to \d+ lands",
];

static BURST_DRAW_PATTERNS: &[&str] = &[r"draw two cards", r"draw three cards", r"draw x cards"];

static MASS_DRAW_PATTERNS: &[&str] = &[
    r"each player draws",
    r"draw seven cards",
    r"draw four cards",
    r"draw five cards",
];

static LOOTING_PATTERNS: &[&str] = &[r"draw .* then discard", r"discard .* then draw", r"loot"];

static IMPULSE_DRAW_PATTERNS: &[&str] = &[
    r"exile the top .* card.* you may play",
    r"until end of turn, you may play",
    r"until your next turn, you may play",
];

static GROUP_HUG_PATTERNS: &[&str] = &[
    r"each player draws",
    r"each player may play an additional land",
    r"target opponent may draw",
    r"each opponent draws",
    r"whenever a player taps a land for mana, that player adds",
];

static WHEEL_PATTERNS: &[&str] = &[
    r"each player discards their hand, then draws",
    r"each player wheels",
];

static VOLTRON_PIECE_PATTERNS: &[&str] = &[
    r"equipped creature gets",
    r"enchanted creature gets",
    r"whenever equipped creature",
];

static MULTI_MANA_PRODUCER_PATTERNS: &[&str] = &[
    r"add \{.\}\{.\}",
    r"add .* (?:two|three|four|five|six|seven|eight|nine|ten) .* mana",
];

static INFECT_PATTERNS: &[&str] = &[r"\binfect\b", r"\btoxic\b", r"poison counter"];
static PROLIFERATE_PATTERNS: &[&str] = &[r"\bproliferate\b"];

static RAMP_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    RAMP_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static DRAW_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    DRAW_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static REMOVAL_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    REMOVAL_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static MASS_REMOVAL_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    MASS_REMOVAL_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static TUTOR_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    TUTOR_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static LAND_TUTOR_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    LAND_TUTOR_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static PROTECTION_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    PROTECTION_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static FIXING_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    FIXING_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static STAX_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    STAX_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static ENGINE_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    ENGINE_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static WINCON_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    WINCON_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static FAST_MANA_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    FAST_MANA_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static RECURSION_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    RECURSION_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static RITUAL_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    RITUAL_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static TREASURE_BURST_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    TREASURE_BURST_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static SAC_MANA_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    SAC_MANA_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static COST_REDUCTION_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    COST_REDUCTION_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static FAST_MANA_ONE_SHOT_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    FAST_MANA_ONE_SHOT_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
pub static MULTI_MANA_PRODUCER_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    MULTI_MANA_PRODUCER_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static BURST_DRAW_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    BURST_DRAW_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static MASS_DRAW_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    MASS_DRAW_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static WHEEL_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    WHEEL_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static VOLTRON_PIECE_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    VOLTRON_PIECE_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static LOOTING_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    LOOTING_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static IMPULSE_DRAW_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    IMPULSE_DRAW_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static GROUP_HUG_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    GROUP_HUG_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static INFECT_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    INFECT_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
static PROLIFERATE_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    PROLIFERATE_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});

pub static MULTI_COLOR_LAND_PATTERNS: &[&str] = &[
    r"add \{.\} or \{.\}",
    r"add one mana of any color",
    r"add one mana of any one color",
];

pub static MULTI_MANA_LAND_PATTERNS: &[&str] = &[r"add \{.\}\{.\}"];

pub static MULTI_COLOR_LAND_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    MULTI_COLOR_LAND_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
pub static MULTI_MANA_LAND_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    MULTI_MANA_LAND_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});

pub static ANY_TUTOR_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"search your library for .* card").unwrap());
pub static FREE_SPELL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"rather than pay this spell's mana cost|without paying its mana cost").unwrap()
});
pub static NON_TAPPING_ACTIVATION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:^|\.)\s*([^:\.]+):").unwrap());

static RE_REMINDER: Lazy<Regex> = Lazy::new(|| Regex::new(r"\(.*?\)").unwrap());
static RE_SPACES: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());
pub static LAND_ENGINE_VERBS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(whenever|for each|you may pay|return|draw|copy|cast|sacrifice).*").unwrap()
});

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

fn is_repeatable_treasure_engine(card: &Card, normalized: &str) -> bool {
    if !card.is_creature() && !card.is_enchantment() {
        return false;
    }

    let creates_treasure = normalized.contains("create a treasure")
        || normalized.contains("create two treasure")
        || normalized.contains("create three treasure")
        || normalized.contains("create four treasure")
        || normalized.contains("create five treasure")
        || normalized.contains("create x treasure");
    let repeat_trigger = normalized.contains("whenever ")
        || normalized.contains("at the beginning of ");

    creates_treasure && repeat_trigger
}

fn is_one_shot_exile_mana_effect(normalized: &str) -> bool {
    normalized.contains("exile ")
        && normalized.contains(": add ")
        && (normalized.contains("from your hand") || normalized.contains("this card"))
}

fn is_mana_artifact_maintenance_text(card: &Card, normalized: &str) -> bool {
    if !card.is_artifact() {
        return false;
    }

    let has_upkeep_or_drawstep_maintenance = normalized.contains("at the beginning of your upkeep")
        || normalized.contains("at the beginning of your draw step");
    let has_self_untap_or_downside = normalized.contains("doesn't untap during your untap step")
        || normalized.contains("if ")
            && (normalized.contains("untap ")
                || normalized.contains("deals ")
                || normalized.contains("pay {"));
    let is_mana_source = normalized.contains(": add ");

    has_upkeep_or_drawstep_maintenance && has_self_untap_or_downside && is_mana_source
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
        let repeatable_treasure_engine = is_repeatable_treasure_engine(card, &normalized);

        if repeatable_treasure_engine
            || (!card.is_land()
                && RAMP_REGEX.iter().any(|re| re.is_match(&normalized))
                && !((card.is_instant() || card.is_sorcery())
                    && (RITUAL_REGEX.iter().any(|re| re.is_match(&normalized))
                        || TREASURE_BURST_REGEX.iter().any(|re| re.is_match(&normalized))
                        || SAC_MANA_REGEX.iter().any(|re| re.is_match(&normalized))
                        || FAST_MANA_ONE_SHOT_REGEX.iter().any(|re| re.is_match(&normalized)))))
        {
            roles.insert(Role::RAMP);
        }
        if DRAW_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::DRAW);
        }
        if REMOVAL_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::REMOVAL);
        }
        let is_land_tutor = LAND_TUTOR_REGEX.iter().any(|re| re.is_match(&normalized));

        if !is_land_tutor && TUTOR_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::TUTOR);
        }
        if PROTECTION_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::PROTECTION);
        }
        if FIXING_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::FIXING);
        }

        let phrases: Vec<&str> = normalized.split('.').collect();
        for phrase in phrases {
            let trimmed = phrase.trim();
            if !trimmed.is_empty() && !trimmed.contains("you may pay") {
                if STAX_REGEX.iter().any(|re| re.is_match(trimmed))
                    && !(roles.contains(&Role::LAND)
                        && !roles.contains(&Role::TUTOR)
                        && !roles.contains(&Role::RECURSION))
                {
                    roles.insert(Role::STAX);
                    break;
                }
            }
        }

        if repeatable_treasure_engine {
            roles.insert(Role::ENGINE);
        } else if ENGINE_REGEX.iter().any(|re| re.is_match(&normalized))
            && !is_one_shot_exile_mana_effect(&normalized)
            && !is_mana_artifact_maintenance_text(card, &normalized)
        {
            if card.is_land() {
                if LAND_ENGINE_VERBS_REGEX.is_match(&normalized) {
                    roles.insert(Role::ENGINE);
                }
            } else {
                roles.insert(Role::ENGINE);
            }
        }
        if WINCON_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::WINCON);
        }
        if RECURSION_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::RECURSION);
        }

        if (card.is_instant() || card.is_sorcery())
            && RITUAL_REGEX.iter().any(|re| re.is_match(&normalized))
        {
            roles.insert(Role::RITUAL);
        }
        if TREASURE_BURST_REGEX
            .iter()
            .any(|re| re.is_match(&normalized))
        {
            roles.insert(Role::TREASURE_BURST);
        }
        if SAC_MANA_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::SAC_MANA);
        }
        if COST_REDUCTION_REGEX
            .iter()
            .any(|re| re.is_match(&normalized))
        {
            roles.insert(Role::COST_REDUCTION);
        }
        if FAST_MANA_ONE_SHOT_REGEX
            .iter()
            .any(|re| re.is_match(&normalized))
            && !repeatable_treasure_engine
        {
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

        if VOLTRON_PIECE_REGEX
            .iter()
            .any(|re| re.is_match(&normalized))
        {
            roles.insert(Role::VOLTRON_PIECE);
        }

        if MASS_REMOVAL_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::MASS_REMOVAL);
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
        if INFECT_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::INFECT);
        }
        if PROLIFERATE_REGEX.iter().any(|re| re.is_match(&normalized)) {
            roles.insert(Role::PROLIFERATE);
        }

        if FAST_MANA_REGEX.iter().any(|re| re.is_match(&normalized))
            && !repeatable_treasure_engine
        {
            roles.insert(Role::FAST_MANA);
        } else if card.mana_value() == 0
            && RAMP_REGEX.iter().any(|re| re.is_match(&normalized))
            && !card.is_land()
            && !repeatable_treasure_engine
        {
            roles.insert(Role::FAST_MANA);
        }
    }

    roles
}
