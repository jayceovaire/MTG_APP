use crate::models::card_model::Card;
use crate::models::cedh_staples::CEDH_STAPLES;
use crate::models::combos::{THREE_CARD_COMBOS, TWO_CARD_COMBOS};
use crate::models::crispi_archetypes::detect_archetype;
use crate::models::crispi_classify::{has_non_tapping_activation, is_instant_speed};
use crate::models::crispi_patterns::{
    ANY_TUTOR_REGEX, FREE_SPELL_REGEX, MULTI_MANA_PRODUCER_REGEX,
};
use crate::models::crispi_probability::{
    derive_bracket, hypergeometric_at_least, is_etb_tapped_land,
};
use std::collections::{HashMap, HashSet};

pub use crate::models::crispi_classify::classify_card;
pub use crate::models::crispi_patterns::{infer_roles, normalize_card_name, normalize_text};
pub use crate::models::crispi_types::{
    CrispiDimension, CrispiEvaluation, DeckArchetype, QualityTier, Role,
};

struct ComboAnalysis {
    detected_combos: Vec<String>,
    combo_piece_names: HashSet<String>,
    combo_multiplier: f32,
    any_combo_found: bool,
}

struct ColorFixingAssessment {
    color_count: usize,
    target_turn: usize,
    fix_rate: f32,
    multiplier: f32,
    perfect_fix_ratio: f32,
    perfect_fix_bonus: f32,
}

struct ColorFixingProfile {
    color_count: usize,
    fix_rates_by_turn: [f32; 7],
    perfect_fix_ratio: f32,
}

struct CachedCard<'a> {
    card: &'a Card,
    roles: HashSet<Role>,
    tier: QualityTier,
    normalized_name: String,
}

pub fn infer_roles_with_combo_context(
    card: &Card,
    combo_piece_names: &HashSet<String>,
) -> HashSet<Role> {
    let mut roles = infer_roles(card);
    if combo_piece_names.contains(&normalize_card_name(card.get_name())) {
        roles.insert(Role::WINCON);
    }
    roles
}

pub fn combo_piece_names_for_deck(mainboard: &[Card], commanders: &[Card]) -> HashSet<String> {
    analyze_combos(mainboard, commanders, deck_non_land_count(mainboard, commanders), 0)
        .combo_piece_names
}

pub fn calculate_crispi(mainboard: &[Card], commanders: &[Card], n_gc: u32) -> CrispiEvaluation {
    let cached_mainboard: Vec<CachedCard<'_>> = mainboard
        .iter()
        .map(|card| {
            let roles = infer_roles(card);
            let tier = classify_card(card, &roles);
            CachedCard {
                card,
                roles,
                tier,
                normalized_name: normalize_card_name(card.get_name()),
            }
        })
        .collect();
    let cached_commanders: Vec<CachedCard<'_>> = commanders
        .iter()
        .map(|card| {
            let roles = infer_roles(card);
            let tier = classify_card(card, &roles);
            CachedCard {
                card,
                roles,
                tier,
                normalized_name: normalize_card_name(card.get_name()),
            }
        })
        .collect();

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
    let mut fixing_count = 0;
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
    let mut infect_signal_weighted = 0.0;
    let mut infect_support_count = 0;
    let mut proliferate_signal_weighted = 0.0;

    let mut process_card = |cached: &CachedCard<'_>| {
        let card = cached.card;
        let roles = &cached.roles;
        let tier = &cached.tier;
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
            if *tier == QualityTier::Premium
                && mv <= 2
                && ANY_TUTOR_REGEX.is_match(&normalize_text(card.oracle_text().unwrap_or_default()))
            {
                premium_tutor_count += 1;
            }
        }
        if roles.contains(&Role::FIXING) {
            fixing_count += 1;
        }
        if roles.contains(&Role::DRAW)
            || roles.contains(&Role::LOOTING)
            || roles.contains(&Role::IMPULSE_DRAW)
        {
            draw_count_weighted += weight;
            if *tier == QualityTier::Premium {
                premium_draw_count += 1;
            }
        }
        if roles.contains(&Role::ENGINE) {
            engine_count_weighted += weight;
        }

        let mut card_mana_points: f32 = 0.0;
        if roles.contains(&Role::RITUAL) {
            card_mana_points = card_mana_points.max(2.5);
        }
        if roles.contains(&Role::TREASURE_BURST) {
            let is_permanent = !card.is_instant() && !card.is_sorcery();
            let tb_weight = if !is_permanent { 2.0 } else { 0.5 };
            card_mana_points = card_mana_points.max(tb_weight);
        }
        if roles.contains(&Role::SAC_MANA) {
            let sac_weight = if mv <= 1 { 1.5 } else { 0.5 };
            card_mana_points = card_mana_points.max(sac_weight);
        }
        if roles.contains(&Role::FAST_MANA_ONE_SHOT) {
            card_mana_points = card_mana_points.max(1.5);
        }
        if roles.contains(&Role::COST_REDUCTION) {
            let cr_weight = if mv <= 1 { 1.5 } else { 0.3 };
            card_mana_points = card_mana_points.max(cr_weight);
        }
        if roles.contains(&Role::FAST_MANA) {
            // 0-MV artifact or multi-mana producer
            let is_multi = card
                .oracle_text()
                .map(|t| {
                    MULTI_MANA_PRODUCER_REGEX
                        .iter()
                        .any(|re| re.is_match(&normalize_text(t)))
                })
                .unwrap_or(false);
            let fm_weight = if mv == 0 {
                1.5
            } else if is_multi && mv <= 1 {
                1.0
            } else if is_multi {
                0.5
            } else {
                0.0
            };
            card_mana_points = card_mana_points.max(fm_weight);
        }
        explosive_mana_points += card_mana_points;

        let mut card_draw_points: f32 = 0.0;
        let is_spell = card.is_instant() || card.is_sorcery();
        if roles.contains(&Role::WHEEL) {
            card_draw_points = card_draw_points.max(2.5);
        }
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
        if roles.contains(&Role::REMOVAL) || roles.contains(&Role::MASS_REMOVAL) {
            if is_inst || roles.contains(&Role::MASS_REMOVAL) {
                interaction_weighted += weight;
                // Free interaction check
                let is_free = mv == 0
                    || (card
                        .oracle_text()
                        .map(|t| FREE_SPELL_REGEX.is_match(&normalize_text(t)))
                        .unwrap_or_default());
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
                let is_free = mv == 0
                    || (card
                        .oracle_text()
                        .map(|t| FREE_SPELL_REGEX.is_match(&normalize_text(t)))
                        .unwrap_or_default());
                if is_free {
                    free_interaction_count += 1;
                }
            }
        }
        if roles.contains(&Role::RECURSION) {
            resilience_weighted += weight * 0.8; // Recursion is slightly less reactive than protection
        }

        // Speed: Fast Mana, Efficient Wincons
        if roles.contains(&Role::FAST_MANA) {
            fast_mana_count += 1;
        }

        if roles.contains(&Role::WINCON) {
            if *tier == QualityTier::Premium || (mv <= 3) {
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

        if roles.contains(&Role::INFECT) {
            infect_signal_weighted += weight;
            infect_support_count += 1;
        }

        if roles.contains(&Role::PROLIFERATE) {
            proliferate_signal_weighted += weight;
        }
    };

    for cached in &cached_mainboard {
        process_card(cached);
    }
    for cached in &cached_commanders {
        process_card(cached);
    }

    let combo_analysis = analyze_combos(mainboard, commanders, non_land_count, tutor_count);
    let combo_count = combo_analysis.detected_combos.len() as u8;
    let effective_wincon_count = wincon_count_efficient + combo_count as u32;
    let land_count = mainboard.iter().filter(|c| c.is_land()).count();

    // Archetype Signals (Before Scoring)
    let turbo_signal = explosive_mana_points + explosive_draw_points;
    let midrange_signal = (consistency_weighted as f32)
        + (engine_count_weighted as f32)
        + (draw_count_weighted as f32)
        + (interaction_weighted as f32 * 0.5);
    let stax_signal = stax_signal_weighted;
    let voltron_signal = voltron_signal_weighted;
    let group_hug_signal = group_hug_signal_weighted;
    let infect_signal = if infect_support_count >= 4 {
        infect_signal_weighted + proliferate_signal_weighted
    } else {
        infect_signal_weighted
    };
    let commander_engine_signal = cached_commanders
        .iter()
        .filter(|cached| {
            let r = &cached.roles;
            let is_engine =
                r.contains(&Role::ENGINE) && (r.contains(&Role::DRAW) || r.contains(&Role::TUTOR));
            let is_cost_reducer = r.contains(&Role::COST_REDUCTION);
            is_engine || is_cost_reducer
        })
        .count() as f32;

    // --- Archetype Coherence Calculation ---
    let signals = [
        turbo_signal,
        midrange_signal,
        stax_signal,
        commander_engine_signal,
        infect_signal,
    ];

    let max_signal = signals.iter().cloned().fold(0.0, f32::max);
    let sum_signal: f32 = signals.iter().sum();

    let focus_ratio = if sum_signal > 0.0 {
        max_signal / sum_signal
    } else {
        0.0
    };

    let magnitude = max_signal;

    let coherence_multiplier = match (focus_ratio, magnitude) {
        (r, m) if r >= 0.45 && m >= 35.0 => 1.10,
        (r, m) if r >= 0.40 && m >= 25.0 => 1.04,
        (r, m) if r >= 0.35 && m >= 18.0 => 0.98,
        (r, _) if r < 0.35 => 0.90,
        _ => 1.0,
    };

    let amv = if non_land_count > 0 {
        total_mv / non_land_count as f32
    } else {
        0.0
    };

    // C — Consistency (0-5)
    let mut consistency_score = match consistency_weighted {
        v if v >= 8.0 => 5,
        v if v >= 5.0 => 4,
        v if v >= 3.0 => 3,
        v if v >= 1.2 => 2,
        _ => 1,
    };
    let tutor_density = tutor_count as f32 / 99.0;
    let color_fixing_profile = build_color_fixing_profile(mainboard, commanders);
    let aggressive_color_fixing = assess_color_fixing(&color_fixing_profile, 4);

    // Win Package Density Adjustment
    let win_package_density = if non_land_count > 0 {
        effective_wincon_count as f32 / non_land_count as f32
    } else {
        0.0
    };
    if win_package_density > 0.08 {
        consistency_score = (consistency_score + 1).min(5);
    }

    if consistency_weighted >= 8.0 && (draw_count_weighted > 5.0 || engine_count_weighted > 3.0) {
        consistency_score = 5;
    }

    // R — Resilience (0-5)
    let mut resilience_score = match resilience_weighted {
        n if n >= 10.0 => 5,
        n if n >= 7.0 => 4,
        n if n >= 4.0 => 3,
        n if n >= 1.5 => 2,
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
    if amv <= 2.2 {
        estimated_win_turn -= 2.0;
    } else if amv <= 2.6 {
        estimated_win_turn -= 1.0;
    }

    estimated_win_turn -= (fast_mana_count as f32 / 3.0).min(2.5);
    estimated_win_turn -= (premium_tutor_count as f32 / 2.5).min(2.0);
    estimated_win_turn -= ((tutor_count.saturating_sub(premium_tutor_count)) as f32 / 4.0).min(1.0);
    if effective_wincon_count >= 2 {
        estimated_win_turn -= 1.0;
    }
    let can_chain_three_spells_turn_two = can_chain_three_spells_turn_two(
        amv,
        land_count,
        fast_mana_count,
        explosive_mana_points,
        fixing_count,
    );
    if can_chain_three_spells_turn_two {
        estimated_win_turn -= 1.0;
    }
    let commander_on_curve = can_cast_commander_by_turn_three(
        commanders,
        land_count,
        fast_mana_count,
        explosive_mana_points,
        fixing_count,
    );
    if commander_on_curve {
        estimated_win_turn -= 0.5;
    }

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
    }
    .max(match tutor_count {
        0..=2 => 2,
        3..=5 => 3,
        _ => 4,
    });
    efficiency_speed_score = efficiency_speed_score.min(speed_cap);

    let mana_velocity = (fast_mana_count as f32 * 0.8) + explosive_mana_points;
    let draw_velocity = (premium_draw_count as f32 * 0.8) + explosive_draw_points;

    let explosive_speed_score = match mana_velocity.max(draw_velocity) {
        v if v >= 15.0 => 5,
        v if v >= 10.0 => 4,
        v if v >= 6.0 => 3,
        v if v >= 3.0 => 2,
        _ => 1,
    };

    let mut speed_score = efficiency_speed_score.max(explosive_speed_score);
    if can_chain_three_spells_turn_two {
        speed_score = speed_score.max(4);
    }
    if commander_on_curve {
        speed_score = speed_score.max(3);
    }
    let true_speed_five = explosive_speed_score >= 5
        && (efficiency_speed_score >= 4
            || can_chain_three_spells_turn_two
            || tutor_density >= 0.06
            || aggressive_color_fixing.fix_rate >= 0.70);
    if speed_score >= 5 && !true_speed_five {
        speed_score = 4;
    }
    let turbo_speed_score = if speed_score >= 5
        && (can_chain_three_spells_turn_two
            || estimated_win_turn <= 4.5
            || (tutor_density >= 0.06 && aggressive_color_fixing.fix_rate >= 0.70))
    {
        speed_score
    } else {
        speed_score.min(4)
    };

    let archetype = detect_archetype(
        stax_signal,
        commander_engine_signal,
        turbo_signal,
        midrange_signal,
        voltron_signal,
        group_hug_signal,
        infect_signal,
        turbo_speed_score,
    );

    // P — Pivotability (0-5)
    let mut pivotability_score = match pivotability_weighted {
        n if n >= 8.0 => 5,
        n if n >= 5.0 => 4,
        n if n >= 3.0 => 3,
        n if n >= 1.5 => 2,
        _ => 1,
    };

    // APPLY STRUCTURAL FLOORS
    let mut applied_overrides: Vec<String> = Vec::new();

    // 1. Command zone card advantage engine
    let cz_engine = cached_commanders.iter().any(|cached| {
        let roles = &cached.roles;
        let tier = &cached.tier;
        let oracle = cached.card.oracle_text().unwrap_or_default().to_lowercase();
        let is_combat_engine = oracle.contains("deals combat damage");

        roles.contains(&Role::ENGINE)
            && (roles.contains(&Role::DRAW) || roles.contains(&Role::TUTOR))
            && (*tier == QualityTier::Premium
                || (*tier == QualityTier::Efficient && !is_combat_engine))
    });
    if cz_engine {
        consistency_score = consistency_score.max(4);
        pivotability_score = pivotability_score.max(3);
        applied_overrides.push("CZ Engine (C>=4, P>=3)".to_string());
    }

    // 2. 8+ free interaction spells
    if free_interaction_count >= 8 {
        interaction_score = 5;
        resilience_score = resilience_score.max(4);
        applied_overrides.push("8+ Free Interaction (I=5, R>=4)".to_string());
    }

    // 3. Compact <= 2 card deterministic win package
    if combo_count > 0 {
        speed_score = (speed_score + combo_count).min(5);
        consistency_score = (consistency_score + 1).min(5);
        applied_overrides.push(format!("Detected Combos (S+{}, C+1)", combo_count));
    } else if effective_wincon_count >= 2 {
        speed_score = (speed_score + 1).min(5);
        consistency_score = (consistency_score + 1).min(5);
        applied_overrides.push("Compact Wincon (S+1, C+1)".to_string());
    }

    // 4. 10+ fast mana pieces
    if fast_mana_count >= 10 {
        speed_score = 5;
        applied_overrides.push("10+ Fast Mana (S=5)".to_string());
    }

    // 5. 5+ premium tutors (<= 2 MV)
    if premium_tutor_count >= 5 {
        consistency_score = 5;
        applied_overrides.push("5+ Premium Tutors (C=5)".to_string());
    }

    // Archetype-Aware Adjustments (After Floors)
    match archetype {
        DeckArchetype::Turbo => {
            consistency_score = consistency_score.max(4);
            pivotability_score = pivotability_score.max(3);
            applied_overrides.push("Turbo Archetype (requires natural S=5, C>=4, P>=3)".to_string());
        }
        DeckArchetype::Stax => {
            interaction_score = interaction_score.max(4);
            resilience_score = resilience_score.max(4);
            applied_overrides.push("Stax Archetype (I>=4, R>=4)".to_string());
        }
        DeckArchetype::CommanderEngine => {
            consistency_score = consistency_score.max(4);
            resilience_score = resilience_score.max(3);
            pivotability_score = pivotability_score.max(3);
            applied_overrides.push("CmdrEngine Archetype (C>=4, R>=3, P>=3)".to_string());
        }
        DeckArchetype::Voltron => {
            resilience_score = resilience_score.max(4);
            pivotability_score = pivotability_score.max(2);
            applied_overrides.push("Voltron Archetype (R>=4, P>=2)".to_string());
        }
        DeckArchetype::GroupHug => {
            consistency_score = consistency_score.max(3);
            pivotability_score = pivotability_score.max(3);
            applied_overrides.push("GroupHug Archetype (C>=3, P>=3)".to_string());
        }
        DeckArchetype::Midrange => {
            consistency_score = consistency_score.max(3);
            resilience_score = resilience_score.max(3);
            interaction_score = interaction_score.max(3);
            pivotability_score = pivotability_score.max(3);
            applied_overrides.push("Midrange Archetype (C>=3, R>=3, I>=3, P>=3)".to_string());
        }
        DeckArchetype::Infect => {
            consistency_score = consistency_score.max(4);
            pivotability_score = pivotability_score.max(3);
            applied_overrides.push("Infect Archetype (C>=4, P>=3)".to_string());
        }
    }

    consistency_score = match tutor_density {
        d if d >= 0.10 => 5,
        d if d >= 0.06 => 4,
        d if d >= 0.03 => 3,
        _ => consistency_score.min(2),
    };
    if speed_score >= 5 && !true_speed_five {
        speed_score = 4;
    }

    let override_text = if !applied_overrides.is_empty() {
        format!(" [Floors: {}]", applied_overrides.join(", "))
    } else {
        "".to_string()
    };

    let raw_score = (consistency_score
        + resilience_score
        + interaction_score
        + speed_score
        + pivotability_score) as f32;
    let gc_bonus = (n_gc as f32 * 0.4).min(6.0);
    let raw_score = (raw_score + gc_bonus).min(30.0);

    // AMV Multiplier (Applied AFTER floors)
    let amv_multiplier = match amv {
        v if v <= 1.5 => 1.08,
        v if v <= 1.8 => 1.04,
        v if v <= 2.5 => 1.02,
        v if v <= 2.8 => 1.01,
        v if v <= 3.0 => 1.00,
        v if v <= 3.4 => 0.92,
        v if v <= 3.8 => 0.85,
        v if v <= 4.0 => 0.65,
        _ => 0.50,
    };

    let all_deck_cards: Vec<&Card> = mainboard.iter().chain(commanders.iter()).collect();
    let mut card_map: HashMap<String, &Card> = HashMap::new();
    for card in &all_deck_cards {
        card_map.insert(normalize_card_name(card.get_name()), *card);
    }
    let detected_combos = combo_analysis.detected_combos.clone();
    let combo_multiplier = combo_analysis.combo_multiplier;
    let any_combo_found = combo_analysis.any_combo_found;

    let mut commander_mv_penalty = 0.0;
    if !commanders.is_empty() {
        commander_mv_penalty = commanders
            .iter()
            .map(|c| {
                let mv = c.mana_value() as f32;
                if mv > 3.0 {
                    // Negative weight (penalty): scales up
                    (mv - 3.0) * 0.25
                } else {
                    // Positive weight (bonus): 3 and lower
                    (mv - 3.5) * 0.15
                }
            })
            .sum::<f32>();
    }

    let mut final_multiplier = amv_multiplier * combo_multiplier;
    final_multiplier *= coherence_multiplier;

    // --- Win Line Density and cEDH Staples Calculation ---
    let mut win_pieces_count = 0;
    let mut staple_hits = 0;
    let mut total_tutors_count = 0;
    let mut total_recursion_count = 0;

    let combo_card_names = combo_analysis.combo_piece_names.clone();

    for cached in cached_mainboard.iter().chain(cached_commanders.iter()) {
        let roles = &cached.roles;
        let norm_name = &cached.normalized_name;

        if roles.contains(&Role::WINCON)
            || roles.contains(&Role::ENGINE_WIN)
            || combo_card_names.contains(norm_name)
        {
            win_pieces_count += 1;
        }
        if roles.contains(&Role::TUTOR) {
            total_tutors_count += 1;
        }
        if roles.contains(&Role::RECURSION) {
            total_recursion_count += 1;
        }
        if CEDH_STAPLES.contains(norm_name.as_str()) {
            staple_hits += 1;
        }
    }

    let win_density = win_pieces_count as f32
        + total_tutors_count as f32 * 0.7
        + total_recursion_count as f32 * 0.5;

    let win_gate = match win_density {
        d if d >= 18.0 => 1.08,
        d if d >= 12.0 => 1.02,
        d if d >= 7.0 => 0.95,
        _ => 0.82,
    };

    final_multiplier *= win_gate;

    if staple_hits >= 6 {
        final_multiplier *= 1.05;
    }

    let tapped_land_count = mainboard
        .iter()
        .filter(|c| is_etb_tapped_land(c, normalize_text))
        .count();
    let provisional_total_score = (raw_score * final_multiplier - commander_mv_penalty)
        .min(25.0)
        .max(0.0);
    let provisional_bracket = derive_bracket(n_gc, any_combo_found, provisional_total_score, amv);
    let color_fixing = assess_color_fixing(&color_fixing_profile, provisional_bracket);
    let key_turn = if provisional_bracket >= 5 {
        3
    } else if provisional_bracket >= 4 {
        5
    } else {
        6
    };
    let land_drop_score = (2..=key_turn)
        .map(|turn| {
            let cards_seen = 7 + (turn - 1);
            hypergeometric_at_least(99, land_count, cards_seen, turn)
        })
        .sum::<f32>()
        / (key_turn - 1) as f32;

    let nonland_mana_source_count = cached_mainboard
        .iter()
        .filter(|cached| {
            if cached.card.is_land() {
                return false;
            }
            let roles = &cached.roles;
            (cached.card.is_artifact()
                && (roles.contains(&Role::RAMP) || roles.contains(&Role::FAST_MANA)))
                || roles.contains(&Role::RITUAL)
                || roles.contains(&Role::TREASURE_BURST)
                || roles.contains(&Role::SAC_MANA)
                || roles.contains(&Role::FAST_MANA_ONE_SHOT)
        })
        .count();
    let mana_source_count = land_count + nonland_mana_source_count;
    let flood_threshold = key_turn + 2;
    let flood_risk =
        hypergeometric_at_least(99, mana_source_count, 7 + (key_turn - 1), flood_threshold);

    let commander_selection = cached_commanders.iter().any(|cached| {
        let roles = &cached.roles;
        roles.contains(&Role::DRAW)
            || roles.contains(&Role::TUTOR)
            || roles.contains(&Role::LOOTING)
            || roles.contains(&Role::IMPULSE_DRAW)
    });
    let deck_selection_density = ((tutor_count as f32) + draw_count_weighted).min(12.0) / 12.0;
    let flood_mitigation =
        (1.0 - if commander_selection { 0.25 } else { 0.0 } - deck_selection_density * 0.45)
            .clamp(0.35, 1.0);
    let flood_penalty = ((flood_risk - 0.35).max(0.0) * 0.30 * flood_mitigation).min(0.12);
    let land_score = land_drop_score + ((flood_risk - 0.35).max(0.0) * flood_mitigation);
    final_multiplier *= 1.0 - flood_penalty;

    let tapped_land_penalty = if land_count > 0 && tapped_land_count > 0 {
        let tapped_share = tapped_land_count as f32 / land_count as f32;
        let first_land_tapped_risk = tapped_share;
        let first_two_tapped_risk =
            hypergeometric_at_least(land_count, tapped_land_count, 2.min(land_count), 1);
        let first_three_tapped_risk =
            hypergeometric_at_least(land_count, tapped_land_count, 3.min(land_count), 1);

        let early_penalty = match provisional_bracket {
            5 => {
                first_land_tapped_risk * 0.24
                    + first_two_tapped_risk * 0.10
                    + first_three_tapped_risk * 0.06
            }
            4 => {
                first_land_tapped_risk * 0.18
                    + first_two_tapped_risk * 0.08
                    + first_three_tapped_risk * 0.05
            }
            _ => first_two_tapped_risk * 0.04 + first_three_tapped_risk * 0.03,
        };

        let tolerated_tapped = match provisional_bracket {
            5 => 1,
            4 => 2,
            _ => 3,
        };
        let excess_tapped = tapped_land_count.saturating_sub(tolerated_tapped) as f32;
        let post_three_excess = tapped_land_count.saturating_sub(3) as f32;
        let saturation_penalty =
            excess_tapped * 0.015 + post_three_excess * post_three_excess * 0.006;

        (early_penalty + saturation_penalty).min(0.30)
    } else {
        0.0
    };
    final_multiplier *= 1.0 - tapped_land_penalty;
    final_multiplier *= color_fixing.multiplier;
    final_multiplier *= color_fixing.perfect_fix_bonus;

    let total_score = (raw_score * final_multiplier - commander_mv_penalty)
        .min(25.0)
        .max(0.0);
    let role_score = raw_score / 25.0;

    let interpretation = match total_score {
        s if s >= 23.0 => "cEDH Optimized",
        s if s >= 19.0 => "Fringe cEDH",
        s if s >= 15.0 => "High Power",
        s if s >= 9.0 => "Focused / Synergistic",
        _ => "Casual / Battlecruiser",
    }
    .to_string();

    // Bracket Calculation
    let bracket = derive_bracket(n_gc, any_combo_found, total_score, amv);

    let consistency = CrispiDimension {
        score: consistency_score,
        justification: format!(
            "Weighted consistency value: {:.2}. Tutors: {} ({:.1}%). Premium tutors: {}. Color fixing: {:.0}% by T{} for {} colors. Perfect-fix lands: {:.0}%.{}",
            consistency_weighted,
            tutor_count,
            tutor_density * 100.0,
            premium_tutor_count,
            color_fixing.fix_rate * 100.0,
            color_fixing.target_turn,
            color_fixing.color_count,
            color_fixing.perfect_fix_ratio * 100.0,
            override_text
        ),
    };

    let resilience = CrispiDimension {
        score: resilience_score,
        justification: format!(
            "Weighted resilience value: {:.2}.{}",
            resilience_weighted, override_text
        ),
    };

    let interaction = CrispiDimension {
        score: interaction_score,
        justification: format!(
            "Weighted interaction value: {:.2} ({} free, {} stax).{}",
            interaction_weighted, free_interaction_count, stax_count, override_text
        ),
    };

    let speed = CrispiDimension {
        score: speed_score,
        justification: format!(
            "Efficiency: {} (Win turn: {:.1}, Cap: {}), Explosive: {} (Mana vel: {:.1}, Draw vel: {:.1}), Tutors: {} ({:.1}%), T2 triple-spell: {}, Commander by T3: {}, Fixing: {:.0}%, Perfect-fix: {:.0}%.{}",
            efficiency_speed_score,
            estimated_win_turn,
            speed_cap,
            explosive_speed_score,
            mana_velocity,
            draw_velocity,
            tutor_count,
            tutor_density * 100.0,
            can_chain_three_spells_turn_two,
            commander_on_curve,
            color_fixing.fix_rate * 100.0,
            color_fixing.perfect_fix_ratio * 100.0,
            override_text
        ),
    };

    let pivotability = CrispiDimension {
        score: pivotability_score,
        justification: format!(
            "Weighted pivotability value: {:.2}.{}",
            pivotability_weighted, override_text
        ),
    };

    CrispiEvaluation {
        total_score,
        raw_score,
        turbo_signal,
        midrange_signal,
        stax_signal,
        voltron_signal,
        infect_signal,
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

fn deck_non_land_count(mainboard: &[Card], commanders: &[Card]) -> usize {
    mainboard
        .iter()
        .chain(commanders.iter())
        .filter(|card| !card.is_land())
        .count()
}

fn can_chain_three_spells_turn_two(
    amv: f32,
    land_count: usize,
    fast_mana_count: u32,
    explosive_mana_points: f32,
    fixing_count: u32,
) -> bool {
    if amv <= 0.0 {
        return false;
    }

    let turn_two_mana_budget = 2.0
        + (land_count.min(36) as f32 / 36.0) * 0.3
        + (fast_mana_count as f32 * 0.45).min(2.0)
        + (explosive_mana_points * 0.20).min(1.8)
        + (fixing_count as f32 * 0.08).min(0.8);
    let three_spell_threshold = (amv * 3.0).clamp(2.5, 6.0);

    turn_two_mana_budget >= three_spell_threshold
}

fn can_cast_commander_by_turn_three(
    commanders: &[Card],
    land_count: usize,
    fast_mana_count: u32,
    explosive_mana_points: f32,
    fixing_count: u32,
) -> bool {
    if commanders.is_empty() {
        return false;
    }

    let turn_three_support = 3.0
        + (land_count.min(36) as f32 / 36.0) * 0.25
        + (fast_mana_count as f32 * 0.35).min(2.0)
        + (explosive_mana_points * 0.15).min(1.2)
        + (fixing_count as f32 * 0.10).min(1.0);

    commanders
        .iter()
        .any(|commander| commander.mana_value() as f32 <= turn_three_support)
}

fn analyze_combos(
    mainboard: &[Card],
    commanders: &[Card],
    non_land_count: usize,
    tutor_count: u32,
) -> ComboAnalysis {
    let all_deck_cards: Vec<&Card> = mainboard.iter().chain(commanders.iter()).collect();
    let mut card_map: HashMap<String, &Card> = HashMap::new();
    for card in &all_deck_cards {
        card_map.insert(normalize_card_name(card.get_name()), *card);
    }

    let mut detected_combos = Vec::new();
    let mut combo_piece_names = HashSet::new();
    let mut total_bonus = 0.0;
    let mut any_combo_found = false;
    let tutor_influence = (tutor_count as f32 * 0.01).min(0.15);

    for combo in TWO_CARD_COMBOS {
        let norm_a = normalize_card_name(combo.card_a);
        let norm_b = normalize_card_name(combo.card_b);

        if let (Some(card_a), Some(card_b)) = (card_map.get(&norm_a), card_map.get(&norm_b)) {
            if !combo_prereqs_met(combo.prereqs, non_land_count) {
                continue;
            }

            any_combo_found = true;
            combo_piece_names.insert(norm_a);
            combo_piece_names.insert(norm_b);
            detected_combos.push(format!(
                "{} + {}{}",
                combo.card_a,
                combo.card_b,
                format_combo_effects(combo.effects)
            ));

            let total_mv = (card_a.mana_value() + card_b.mana_value()) as f32;
            total_bonus += combo_bonus(total_mv, [*card_a, *card_b].into_iter(), tutor_influence, 3.0);
        }
    }

    for combo in THREE_CARD_COMBOS {
        let norm_a = normalize_card_name(combo.card_a);
        let norm_b = normalize_card_name(combo.card_b);
        let norm_c = normalize_card_name(combo.card_c);

        if let (Some(card_a), Some(card_b), Some(card_c)) = (
            card_map.get(&norm_a),
            card_map.get(&norm_b),
            card_map.get(&norm_c),
        ) {
            if !combo_prereqs_met(combo.prereqs, non_land_count) {
                continue;
            }

            any_combo_found = true;
            combo_piece_names.insert(norm_a);
            combo_piece_names.insert(norm_b);
            combo_piece_names.insert(norm_c);
            detected_combos.push(format!(
                "{} + {} + {}{}",
                combo.card_a,
                combo.card_b,
                combo.card_c,
                format_combo_effects(combo.effects)
            ));

            let total_mv = (card_a.mana_value() + card_b.mana_value() + card_c.mana_value()) as f32;
            total_bonus += combo_bonus(
                total_mv,
                [*card_a, *card_b, *card_c].into_iter(),
                tutor_influence,
                4.0,
            );
        }
    }

    let combo_multiplier = if any_combo_found {
        (1.0 + total_bonus).min(1.25)
    } else {
        1.0
    };

    ComboAnalysis {
        detected_combos,
        combo_piece_names,
        combo_multiplier,
        any_combo_found,
    }
}

fn combo_prereqs_met(prereqs: u8, non_land_count: usize) -> bool {
    prereqs == 0 || non_land_count >= (5 + prereqs as usize)
}

fn format_combo_effects(effects: &[&str]) -> String {
    if effects.is_empty() {
        String::new()
    } else {
        format!(" ({})", effects.join(", "))
    }
}

fn combo_bonus<'a>(
    total_mv: f32,
    cards: impl Iterator<Item = &'a Card>,
    tutor_influence: f32,
    mv_floor: f32,
) -> f32 {
    let mut combo_bonus = 0.02 + tutor_influence;
    combo_bonus -= (total_mv - mv_floor).max(0.0) * 0.01;

    let mut speed_penalty = 0.0;
    for card in cards {
        let oracle = card.oracle_text().unwrap_or_default().to_lowercase();
        let is_inst = is_instant_speed(card);

        if !is_inst && card.is_sorcery() {
            speed_penalty += 0.02;
        }

        if (card.is_creature() || card.is_artifact()) && !has_non_tapping_activation(&oracle) {
            if card.is_creature() && !oracle.contains("haste") {
                speed_penalty += 0.02;
            }
        }
    }

    (combo_bonus - speed_penalty).clamp(0.02, 0.20)
}

fn assess_color_fixing(
    profile: &ColorFixingProfile,
    bracket_target: u8,
) -> ColorFixingAssessment {
    let color_count = profile.color_count;
    if color_count <= 1 {
        return ColorFixingAssessment {
            color_count: color_count.max(1),
            target_turn: 1,
            fix_rate: 1.0,
            multiplier: 1.0,
            perfect_fix_ratio: profile.perfect_fix_ratio,
            perfect_fix_bonus: 1.0,
        };
    }

    let target_turn = fixing_target_turn(color_count, bracket_target);
    let fix_rate = profile.fix_rates_by_turn[target_turn.min(6)];
    let perfect_fix_ratio = profile.perfect_fix_ratio;

    let multiplier = if fix_rate >= 0.85 {
        1.04
    } else if fix_rate >= 0.70 {
        1.0
    } else if fix_rate >= 0.50 {
        0.88
    } else {
        0.76
    };
    let perfect_fix_bonus = if perfect_fix_ratio > 0.45 { 1.03 } else { 1.0 };

    ColorFixingAssessment {
        color_count,
        target_turn,
        fix_rate,
        multiplier,
        perfect_fix_ratio,
        perfect_fix_bonus,
    }
}

fn fixing_target_turn(color_count: usize, bracket_target: u8) -> usize {
    let offset = if bracket_target >= 5 {
        -2
    } else if bracket_target >= 4 {
        -1
    } else if bracket_target >= 3 {
        0
    } else {
        1
    };

    (color_count as isize + offset).max(1) as usize
}

fn build_color_fixing_profile(
    mainboard: &[Card],
    commanders: &[Card],
 ) -> ColorFixingProfile {
    let commander_colors = commander_color_mask(commanders);
    let color_count = commander_colors.count_ones() as usize;
    let land_count = mainboard.iter().filter(|card| card.is_land()).count();
    let perfect_fix_sources = mainboard
        .iter()
        .filter(|card| is_perfect_fix_source(card, commander_colors))
        .count();
    let perfect_fix_ratio = if land_count > 0 {
        perfect_fix_sources as f32 / land_count as f32
    } else {
        0.0
    };

    if mainboard.is_empty() || color_count <= 1 {
        return ColorFixingProfile {
            color_count,
            fix_rates_by_turn: [1.0; 7],
            perfect_fix_ratio,
        };
    }

    let max_turn = 6usize;
    let simulations = 100usize;
    let mut success_counts = [0usize; 7];

    let mut rng = 0xC0DEC0DEu64
        ^ (mainboard.len() as u64).wrapping_mul(7919)
        ^ (commanders.len() as u64).wrapping_mul(1543)
        ^ commander_colors as u64
        ^ max_turn as u64;

    for _ in 0..simulations {
        let mut deck: Vec<&Card> = mainboard.iter().collect();
        shuffle_cards(&mut deck, &mut rng);

        let draw_count = (7 + max_turn.saturating_sub(1)).min(deck.len());
        let mut hand: Vec<&Card> = deck.into_iter().take(draw_count).collect();
        let mut battlefield: Vec<&Card> = Vec::new();
        let mut summoning_sick_dorks: Vec<&Card> = Vec::new();
        let mut accessible_colors = 0u8;

        for turn in 1..=max_turn {
            if let Some(land_idx) = choose_land_to_play(&hand, commander_colors, accessible_colors) {
                battlefield.push(hand.remove(land_idx));
            }

            let mut turn_colors = battlefield_color_mask(&battlefield, commander_colors, turn)
                | battlefield_color_mask(&summoning_sick_dorks, commander_colors, turn.saturating_sub(1));
            let mut available_mana = battlefield_mana_total(&battlefield, commander_colors, turn)
                + battlefield_mana_total(&summoning_sick_dorks, commander_colors, turn.saturating_sub(1));

            loop {
                let Some(source_idx) = choose_mana_source_to_cast(
                    &hand,
                    commander_colors,
                    turn_colors,
                    available_mana,
                ) else {
                    break;
                };

                let source = hand.remove(source_idx);
                let cost = source.mana_value() as i32;
                if cost > available_mana {
                    continue;
                }
                available_mana -= cost;

                if is_single_use_mana_source(source) {
                    turn_colors |= mana_source_color_mask(source, commander_colors);
                    available_mana += mana_output_amount(source).max(1);
                } else if is_creature_mana_source(source) {
                    summoning_sick_dorks.push(source);
                } else {
                    battlefield.push(source);
                    turn_colors |= mana_source_color_mask(source, commander_colors);
                    available_mana += mana_output_amount(source).max(1);
                }
            }

            accessible_colors = turn_colors;
            if (accessible_colors & commander_colors) == commander_colors {
                success_counts[turn] += 1;
            }
        }
    }

    let mut fix_rates_by_turn = [0.0; 7];
    for turn in 1..=max_turn {
        fix_rates_by_turn[turn] = success_counts[turn] as f32 / simulations as f32;
    }

    ColorFixingProfile {
        color_count,
        fix_rates_by_turn,
        perfect_fix_ratio,
    }
}

fn shuffle_cards(cards: &mut Vec<&Card>, state: &mut u64) {
    for i in (1..cards.len()).rev() {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = (*state % (i as u64 + 1)) as usize;
        cards.swap(i, j);
    }
}

fn choose_land_to_play(hand: &[&Card], commander_colors: u8, current_colors: u8) -> Option<usize> {
    hand.iter()
        .enumerate()
        .filter(|(_, card)| card.is_land())
        .max_by_key(|(_, card)| {
            let source_colors = mana_source_color_mask(card, commander_colors);
            let new_colors = (source_colors & !current_colors).count_ones();
            let untapped = !is_etb_tapped_land(card, normalize_text) as u32;
            (new_colors, untapped, mana_output_amount(card))
        })
        .map(|(idx, _)| idx)
}

fn choose_mana_source_to_cast(
    hand: &[&Card],
    commander_colors: u8,
    current_colors: u8,
    available_mana: i32,
) -> Option<usize> {
    hand.iter()
        .enumerate()
        .filter(|(_, card)| is_relevant_mana_source(card))
        .filter(|(_, card)| card.mana_value() as i32 <= available_mana)
        .filter(|(_, card)| can_pay_colored_cost(card, current_colors))
        .max_by_key(|(_, card)| {
            let source_colors = mana_source_color_mask(card, commander_colors);
            let new_colors = (source_colors & !current_colors).count_ones();
            let immediate = (!is_creature_mana_source(card)) as u32;
            (
                new_colors,
                immediate,
                mana_output_amount(card),
                10u8.saturating_sub(card.mana_value()),
            )
        })
        .map(|(idx, _)| idx)
}

fn battlefield_color_mask(cards: &[&Card], commander_colors: u8, turn: usize) -> u8 {
    if turn == 0 {
        return 0;
    }

    cards.iter()
        .filter(|card| !card.is_land() || !is_etb_tapped_land(card, normalize_text) || turn > 1)
        .fold(0u8, |acc, card| acc | mana_source_color_mask(card, commander_colors))
}

fn battlefield_mana_total(cards: &[&Card], commander_colors: u8, turn: usize) -> i32 {
    if turn == 0 {
        return 0;
    }

    cards.iter()
        .filter(|card| !card.is_land() || !is_etb_tapped_land(card, normalize_text) || turn > 1)
        .map(|card| {
            if mana_source_color_mask(card, commander_colors) == 0 && !produces_colorless(card) {
                0
            } else {
                mana_output_amount(card)
            }
        })
        .sum()
}

fn is_relevant_mana_source(card: &Card) -> bool {
    if card.is_land() {
        return false;
    }

    let roles = infer_roles(card);
    roles.contains(&Role::FAST_MANA)
        || roles.contains(&Role::RAMP)
        || roles.contains(&Role::FIXING)
        || roles.contains(&Role::RITUAL)
        || roles.contains(&Role::TREASURE_BURST)
        || roles.contains(&Role::SAC_MANA)
        || roles.contains(&Role::FAST_MANA_ONE_SHOT)
}

fn is_perfect_fix_source(card: &Card, commander_colors: u8) -> bool {
    if !card.is_land() {
        return false;
    }

    let normalized = normalize_text(card.oracle_text().unwrap_or_default());
    let source_colors = mana_source_color_mask(card, commander_colors);
    let is_rainbow = normalized.contains("add one mana of any color")
        || normalized.contains("add one mana of any color in your commander's color identity")
        || normalized.contains("any color");
    let is_fetch = normalized.contains("search your library")
        && normalized.contains("land")
        && (normalized.contains("sacrifice") || normalized.contains("{t}"));
    let is_untapped = !is_etb_tapped_land(card, normalize_text);
    let is_multi_untapped = is_untapped && source_colors.count_ones() >= 2;

    is_rainbow || is_fetch || is_multi_untapped
}

fn is_single_use_mana_source(card: &Card) -> bool {
    let roles = infer_roles(card);
    roles.contains(&Role::RITUAL)
        || roles.contains(&Role::FAST_MANA_ONE_SHOT)
        || roles.contains(&Role::TREASURE_BURST)
        || roles.contains(&Role::SAC_MANA)
}

fn is_creature_mana_source(card: &Card) -> bool {
    card.is_creature() && is_relevant_mana_source(card)
}

fn can_pay_colored_cost(card: &Card, available_colors: u8) -> bool {
    color_mask_from_card_cost(card) & !available_colors == 0
}

fn mana_source_color_mask(card: &Card, commander_colors: u8) -> u8 {
    let text = normalize_text(card.oracle_text().unwrap_or_default());
    if text.contains("add one mana of any color in your commander's color identity")
        || text.contains("add one mana of any color")
        || text.contains("add two mana of any one color")
        || text.contains("mana of any color")
    {
        return commander_colors;
    }

    let mut colors = color_mask_from_text(card.oracle_text().unwrap_or_default());
    if colors == 0 {
        colors = basic_land_color_mask(card);
    }
    colors
}

fn mana_output_amount(card: &Card) -> i32 {
    let text = card.oracle_text().unwrap_or_default();
    let explicit = text
        .matches("{W}")
        .count()
        + text.matches("{U}").count()
        + text.matches("{B}").count()
        + text.matches("{R}").count()
        + text.matches("{G}").count()
        + text.matches("{C}").count();

    explicit.clamp(1, 3) as i32
}

fn basic_land_color_mask(card: &Card) -> u8 {
    match normalize_card_name(card.get_name()).as_str() {
        "plains" => 0b00001,
        "island" => 0b00010,
        "swamp" => 0b00100,
        "mountain" => 0b01000,
        "forest" => 0b10000,
        _ => 0,
    }
}

fn produces_colorless(card: &Card) -> bool {
    card.oracle_text()
        .map(|text| normalize_text(text).contains("{c}"))
        .unwrap_or(false)
}

fn commander_color_mask(commanders: &[Card]) -> u8 {
    commanders.iter().fold(0u8, |acc, commander| {
        acc | color_mask_from_card_cost(commander) | color_mask_from_text(commander.oracle_text().unwrap_or_default())
    })
}

fn color_mask_from_card_cost(card: &Card) -> u8 {
    color_mask_from_text(card.mana_cost().unwrap_or_default())
}

fn color_mask_from_text(text: &str) -> u8 {
    let mut mask = 0u8;
    let upper = text.to_ascii_uppercase();
    if upper.contains("{W}") {
        mask |= 0b00001;
    }
    if upper.contains("{U}") {
        mask |= 0b00010;
    }
    if upper.contains("{B}") {
        mask |= 0b00100;
    }
    if upper.contains("{R}") {
        mask |= 0b01000;
    }
    if upper.contains("{G}") {
        mask |= 0b10000;
    }
    mask
}

#[cfg(test)]
#[path = "crispi_model_tests.rs"]
mod crispi_model_tests;
