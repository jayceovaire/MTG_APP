use crate::models::card_model::Card;
use crate::models::crispi_patterns::{
    normalize_text, ANY_TUTOR_REGEX, FREE_SPELL_REGEX, LAND_ENGINE_VERBS_REGEX, MULTI_COLOR_LAND_REGEX,
    MULTI_MANA_LAND_REGEX, NON_TAPPING_ACTIVATION_REGEX,
};
use crate::models::crispi_types::{QualityTier, Role};
use std::collections::HashSet;

pub fn classify_card(card: &Card, roles: &HashSet<Role>) -> QualityTier {
    let oracle_text = card.oracle_text().map(normalize_text).unwrap_or_default();

    if card.is_land() {
        let enters_tapped = oracle_text.contains("enters the battlefield tapped") && !oracle_text.contains("unless");
        let produces_multi_color = MULTI_COLOR_LAND_REGEX.iter().any(|re| re.is_match(&oracle_text));
        let produces_multi_mana = MULTI_MANA_LAND_REGEX.iter().any(|re| re.is_match(&oracle_text));

        let has_utility = roles.contains(&Role::TUTOR)
            || roles.contains(&Role::RECURSION)
            || roles.contains(&Role::REMOVAL)
            || roles.contains(&Role::MASS_REMOVAL)
            || roles.contains(&Role::PROTECTION)
            || roles.contains(&Role::STAX)
            || roles.contains(&Role::DRAW)
            || roles.contains(&Role::ENGINE)
            || roles.contains(&Role::WINCON);

        if !enters_tapped && (produces_multi_mana || produces_multi_color || has_utility) {
            return QualityTier::Premium;
        }

        if enters_tapped && (has_utility || produces_multi_mana) {
            return QualityTier::Efficient;
        }
        if !enters_tapped {
            return QualityTier::Efficient;
        }

        return QualityTier::Slow;
    }

    let mv = card.mana_value();
    let is_inst = is_instant_speed(card);
    let is_creature_artifact = card.is_creature() || card.is_artifact();
    let is_delayed_tutor = roles.contains(&Role::TUTOR) && oracle_text.contains("suspend");

    let mut is_premium = FREE_SPELL_REGEX.is_match(&oracle_text)
        || roles.contains(&Role::FAST_MANA)
        || (roles.contains(&Role::TUTOR) && !is_delayed_tutor && ANY_TUTOR_REGEX.is_match(&oracle_text) && mv <= 2)
        || ((roles.contains(&Role::REMOVAL) || roles.contains(&Role::PROTECTION)) && is_inst && mv <= 1)
        || (roles.contains(&Role::WINCON) && mv <= 2)
        || (roles.contains(&Role::MASS_REMOVAL) && mv <= 3)
        || (roles.contains(&Role::ENGINE) && mv <= 2);

    if is_premium && is_creature_artifact {
        let has_non_tap = has_non_tapping_activation(&oracle_text);
        let impact = oracle_text.contains("enters the battlefield")
            || oracle_text.contains("when you cast")
            || oracle_text.contains("ward")
            || roles.contains(&Role::STAX);

        if !has_non_tap && !impact {
            is_premium = false;
        }
    }

    if is_premium {
        return QualityTier::Premium;
    }

    if is_delayed_tutor {
        return QualityTier::Slow;
    }

    if mv >= 4 && !roles.contains(&Role::WINCON) && !roles.contains(&Role::FAST_MANA) && !roles.contains(&Role::TUTOR) && !roles.contains(&Role::MASS_REMOVAL) {
        return QualityTier::Slow;
    }

    if !roles.is_empty() {
        return QualityTier::Efficient;
    }

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

pub fn has_non_tapping_activation(text: &str) -> bool {
    for cap in NON_TAPPING_ACTIVATION_REGEX.captures_iter(text) {
        let cost = cap.get(1).unwrap().as_str();
        if !cost.contains("{t}") && !cost.contains("tap") {
            return true;
        }
    }
    false
}

pub fn is_land_engine(normalized: &str) -> bool {
    LAND_ENGINE_VERBS_REGEX.is_match(normalized)
}
