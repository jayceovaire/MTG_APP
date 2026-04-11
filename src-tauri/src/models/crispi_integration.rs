use crate::models::card_model::{Card, CardType};
use crate::models::combos::{THREE_CARD_COMBOS, TWO_CARD_COMBOS};
use crate::models::crispi_patterns::{infer_roles, normalize_card_name, normalize_text};
use crate::models::crispi_types::Role;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tag {
    GRAVEYARD,
    ARTIFACT,
    ENCHANTMENT,
    DRAW,
    CAST,
    EXILE,
    SACRIFICE,
    TREASURE,
    STORM,
}

static TAG_PATTERNS: Lazy<Vec<(Tag, Regex)>> = Lazy::new(|| {
    vec![
        (Tag::GRAVEYARD, Regex::new(r"(?i)graveyard|from your graveyard").unwrap()),
        (Tag::ARTIFACT, Regex::new(r"(?i)artifact|artifacts you control").unwrap()),
        (Tag::ENCHANTMENT, Regex::new(r"(?i)enchantment|enchantments you control").unwrap()),
        (Tag::DRAW, Regex::new(r"(?i)whenever you draw|draw a card").unwrap()),
        (Tag::CAST, Regex::new(r"(?i)whenever you cast|cast from").unwrap()),
        (Tag::EXILE, Regex::new(r"(?i)from exile|exile").unwrap()),
        (Tag::SACRIFICE, Regex::new(r"(?i)sacrifice").unwrap()),
        (Tag::TREASURE, Regex::new(r"(?i)treasure").unwrap()),
        (Tag::STORM, Regex::new(r"(?i)storm|copy").unwrap()),
    ]
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardNode {
    pub name: String,
    pub tags: HashSet<Tag>,
    pub integration: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Crispir {
    pub g: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardHeat {
    pub name: String,
    pub crispir: Crispir,
    pub integration: f32,
}

pub fn extract_tags(card: &Card) -> HashSet<Tag> {
    let mut tags = HashSet::new();
    if let Some(text) = card.oracle_text() {
        for (tag, re) in TAG_PATTERNS.iter() {
            if re.is_match(text) {
                tags.insert(tag.clone());
            }
        }
    }
    tags
}

pub fn compute_integration(cards: &[Card]) -> Vec<CardHeat> {
    let mut nodes: Vec<CardNode> = cards
        .iter()
        .map(|c| CardNode {
            name: c.get_name().to_string(),
            tags: extract_tags(c),
            integration: 0.0,
        })
        .collect();

    let roles_map: Vec<HashSet<Role>> = cards.iter().map(|c| infer_roles(c)).collect();

    for i in 0..cards.len() {
        for j in 0..cards.len() {
            if i == j {
                continue;
            }

            let a = &cards[i];
            let b = &cards[j];
            let b_roles = &roles_map[j];

            let mut points = 0.0;

            // Rule 1 — Tutor Edge
            if tutor_can_find(b, b_roles, a) {
                points += 1.0;
                nodes[j].integration += 1.0; // The tutor is integrated because it finds a target
            }

            // Rule 2 — Wincon Group Edge
            if same_wincon_group(a, b) {
                points += 2.0;
            }

            // Rule 3 — Engine Tag Edge
            let a_tags = &nodes[i].tags;
            let b_tags = &nodes[j].tags;
            if !a_tags.is_disjoint(b_tags) {
                points += 1.0;
            }

            // Rule 4 — Type Dependency Edge
            if type_dependency(b, a) {
                points += 1.0;
            }

            nodes[i].integration += points;
        }
    }

    let max_integration = nodes
        .iter()
        .map(|n| n.integration)
        .fold(0.0, f32::max);

    if max_integration > 0.0 {
        for node in &mut nodes {
            node.integration /= max_integration;
        }
    }

    nodes
        .into_iter()
        .map(|n| {
            let integration = n.integration;
            CardHeat {
                name: n.name.clone(),
                crispir: Crispir { g: integration },
                integration,
            }
        })
        .collect()
}

pub fn tutor_can_find(b: &Card, b_roles: &HashSet<Role>, a: &Card) -> bool {
    if !b_roles.contains(&Role::TUTOR) {
        return false;
    }
    let b_text = b
        .oracle_text()
        .map(normalize_text)
        .unwrap_or_default();
    let a_name = normalize_card_name(a.get_name());

    // Rule 0: Universal Tutors (e.g. Demonic Tutor, Vampiric Tutor)
    if b_text.contains("search your library for a card")
        || b_text.contains("search your library for any card")
    {
        return true;
    }

    // Name search
    if b_text.contains(&a_name) {
        return true;
    }

    // Type match
    for t in a.card_type() {
        let type_str = match t {
            CardType::Creature => "creature",
            CardType::Artifact => "artifact",
            CardType::Enchantment => "enchantment",
            CardType::Land => "land",
            _ => continue,
        };
        if b_text.contains(type_str) {
            return true;
        }
    }

    false
}

fn same_wincon_group(a: &Card, b: &Card) -> bool {
    let a_name = normalize_card_name(a.get_name());
    let b_name = normalize_card_name(b.get_name());

    for combo in TWO_CARD_COMBOS {
        let combo_a = normalize_card_name(combo.card_a);
        let combo_b = normalize_card_name(combo.card_b);
        if (a_name == combo_a && b_name == combo_b) || (a_name == combo_b && b_name == combo_a) {
            return true;
        }
    }

    for combo in THREE_CARD_COMBOS {
        let names = [
            normalize_card_name(combo.card_a),
            normalize_card_name(combo.card_b),
            normalize_card_name(combo.card_c),
        ];
        if names.contains(&a_name) && names.contains(&b_name) {
            return true;
        }
    }

    false
}

fn type_dependency(b: &Card, a: &Card) -> bool {
    let b_text = b
        .oracle_text()
        .map(normalize_text)
        .unwrap_or_default();
    for t in a.card_type() {
        let type_str = match t {
            CardType::Creature => "creature",
            CardType::Artifact => "artifact",
            CardType::Enchantment => "enchantment",
            CardType::Land => "land",
            CardType::Instant => "instant",
            CardType::Sorcery => "sorcery",
            CardType::Planeswalker => "planeswalker",
            CardType::Battle => "battle",
            CardType::Tribal => "tribal",
        };
        if b_text.contains(type_str) {
            return true;
        }
    }
    false
}
