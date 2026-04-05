use crate::models::card_model::Card;
use crate::models::cedh_staples::CEDH_STAPLES;
use crate::models::combos::{THREE_CARD_COMBOS, TWO_CARD_COMBOS};
use crate::models::crispi_archetypes::detect_archetype;
use crate::models::crispi_classify::{has_non_tapping_activation, is_instant_speed};
use crate::models::crispi_patterns::{ANY_TUTOR_REGEX, FREE_SPELL_REGEX, MULTI_MANA_PRODUCER_REGEX};
use crate::models::crispi_probability::{derive_bracket, hypergeometric_at_least, is_etb_tapped_land};
use std::collections::{HashMap, HashSet};

pub use crate::models::crispi_classify::classify_card;
pub use crate::models::crispi_patterns::{infer_roles, normalize_card_name, normalize_text};
pub use crate::models::crispi_types::{CrispiDimension, CrispiEvaluation, DeckArchetype, QualityTier, Role};

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
    let mut infect_signal_weighted = 0.0;

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
        if roles.contains(&Role::REMOVAL) || roles.contains(&Role::MASS_REMOVAL) {
            if is_inst || roles.contains(&Role::MASS_REMOVAL) {
                interaction_weighted += weight;
                // Free interaction check
                let is_free = mv == 0 || (card.oracle_text().map(|t| FREE_SPELL_REGEX.is_match(&normalize_text(t))).unwrap_or_default());
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
                let is_free = mv == 0 || (card.oracle_text().map(|t| FREE_SPELL_REGEX.is_match(&normalize_text(t))).unwrap_or_default());
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

        if roles.contains(&Role::INFECT) {
            infect_signal_weighted += weight;
        }
    };

    for card in mainboard { process_card(card); }
    for card in commanders { process_card(card); }

    // Archetype Signals (Before Scoring)
    let turbo_signal = explosive_mana_points + explosive_draw_points;
    let midrange_signal = (consistency_weighted as f32) + (engine_count_weighted as f32) + (draw_count_weighted as f32) + (interaction_weighted as f32 * 0.5);
    let stax_signal = stax_signal_weighted;
    let voltron_signal = voltron_signal_weighted;
    let group_hug_signal = group_hug_signal_weighted;
    let infect_signal = infect_signal_weighted;
    let commander_engine_signal = commanders.iter().filter(|c| {
        let r = infer_roles(c);
        let is_engine = r.contains(&Role::ENGINE) && (r.contains(&Role::DRAW) || r.contains(&Role::TUTOR));
        let is_cost_reducer = r.contains(&Role::COST_REDUCTION);
        is_engine || is_cost_reducer
    }).count() as f32;

    let archetype = detect_archetype(stax_signal, commander_engine_signal, turbo_signal, midrange_signal, voltron_signal, group_hug_signal, infect_signal);

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
    } else { 0.0 };

    let magnitude = max_signal;

    let coherence_multiplier = match (focus_ratio, magnitude) {
        (r, m) if r >= 0.45 && m >= 35.0 => 1.10,
        (r, m) if r >= 0.40 && m >= 25.0 => 1.04,
        (r, m) if r >= 0.35 && m >= 18.0 => 0.98,
        (r, _) if r < 0.35 => 0.90,
        _ => 1.0,
    };

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
        DeckArchetype::Midrange => {
            consistency_score = consistency_score.max(3);
            resilience_score = resilience_score.max(3);
            interaction_score = interaction_score.max(3);
            pivotability_score = pivotability_score.max(2);
            applied_overrides.push("Midrange Archetype (C>=3, R>=3, I>=3, P>=2)");
        }
        DeckArchetype::Infect => {
            consistency_score = consistency_score.max(4);
            pivotability_score = pivotability_score.max(3);
            applied_overrides.push("Infect Archetype (C>=4, P>=3)");
        }
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
    let tutor_influence = (tutor_count as f32 * 0.01).min(0.15);
    
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

    // --- Three-Card Infinite Combo Detection ---
    for combo in THREE_CARD_COMBOS {
        let norm_a = normalize_card_name(combo.card_a);
        let norm_b = normalize_card_name(combo.card_b);
        let norm_c = normalize_card_name(combo.card_c);
        
        if card_map.contains_key(&norm_a) && card_map.contains_key(&norm_b) && card_map.contains_key(&norm_c) {
            let card_a = card_map.get(&norm_a).unwrap();
            let card_b = card_map.get(&norm_b).unwrap();
            let card_c = card_map.get(&norm_c).unwrap();
            
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
                detected_combos.push(format!("{} + {} + {}{}", combo.card_a, combo.card_b, combo.card_c, effects_str));
                
                let mut combo_bonus = 0.02 + tutor_influence;
                
                let total_mv = (card_a.mana_value() + card_b.mana_value() + card_c.mana_value()) as f32;
                let mv_penalty = (total_mv - 4.0).max(0.0) * 0.01; // MV penalty starts slightly higher for 3-card
                combo_bonus -= mv_penalty;
                
                // Speed Penalty
                let mut speed_penalty = 0.0;
                let cards = [card_a, card_b, card_c];
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
                combo_bonus -= speed_penalty;
                
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

    let mut final_multiplier = amv_multiplier * combo_multiplier;
    final_multiplier *= coherence_multiplier;

    // --- Win Line Density and cEDH Staples Calculation ---
    let mut win_pieces_count = 0;
    let mut staple_hits = 0;
    let mut total_tutors_count = 0;
    let mut total_recursion_count = 0;

    let mut combo_card_names: HashSet<String> = HashSet::new();
    for combo in TWO_CARD_COMBOS {
        let norm_a = normalize_card_name(combo.card_a);
        let norm_b = normalize_card_name(combo.card_b);
        if card_map.contains_key(&norm_a) && card_map.contains_key(&norm_b) {
            combo_card_names.insert(norm_a);
            combo_card_names.insert(norm_b);
        }
    }
    for combo in THREE_CARD_COMBOS {
        let norm_a = normalize_card_name(combo.card_a);
        let norm_b = normalize_card_name(combo.card_b);
        let norm_c = normalize_card_name(combo.card_c);
        if card_map.contains_key(&norm_a) && card_map.contains_key(&norm_b) && card_map.contains_key(&norm_c) {
            combo_card_names.insert(norm_a);
            combo_card_names.insert(norm_b);
            combo_card_names.insert(norm_c);
        }
    }

    for card in &all_deck_cards {
        let roles = infer_roles(card);
        let norm_name = normalize_card_name(card.get_name());

        if roles.contains(&Role::WINCON) || roles.contains(&Role::ENGINE_WIN) || combo_card_names.contains(&norm_name) {
            win_pieces_count += 1;
        }
        if roles.contains(&Role::TUTOR) {
            total_tutors_count += 1;
        }
        if roles.contains(&Role::RECURSION) {
            total_recursion_count += 1;
        }
        if CEDH_STAPLES.contains(&norm_name) {
            staple_hits += 1;
        }
    }

    let win_density = win_pieces_count as f32
        + total_tutors_count as f32 * 0.7
        + total_recursion_count as f32 * 0.5;

    let win_gate = match win_density {
        d if d >= 18.0 => 1.08,
        d if d >= 12.0 => 1.02,
        d if d >= 7.0  => 0.95,
        _              => 0.82,
    };

    final_multiplier *= win_gate;

    if staple_hits >= 6 {
        final_multiplier *= 1.05;
    }

    let land_count = mainboard.iter().filter(|c| c.is_land()).count();
    let tapped_land_count = mainboard.iter().filter(|c| is_etb_tapped_land(c, normalize_text)).count();
    let provisional_total_score = (raw_score * final_multiplier - commander_mv_penalty).min(25.0).max(0.0);
    let provisional_bracket = derive_bracket(n_gc, any_combo_found, provisional_total_score, amv);
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
        .sum::<f32>() / (key_turn - 1) as f32;

    let nonland_mana_source_count = mainboard.iter().filter(|card| {
        if card.is_land() {
            return false;
        }
        let roles = infer_roles(card);
        (card.is_artifact() && (roles.contains(&Role::RAMP) || roles.contains(&Role::FAST_MANA))) ||
        roles.contains(&Role::RITUAL) ||
        roles.contains(&Role::TREASURE_BURST) ||
        roles.contains(&Role::SAC_MANA) ||
        roles.contains(&Role::FAST_MANA_ONE_SHOT)
    }).count();
    let mana_source_count = land_count + nonland_mana_source_count;
    let flood_threshold = key_turn + 2;
    let flood_risk = hypergeometric_at_least(99, mana_source_count, 7 + (key_turn - 1), flood_threshold);

    let commander_selection = commanders.iter().any(|card| {
        let roles = infer_roles(card);
        roles.contains(&Role::DRAW) ||
        roles.contains(&Role::TUTOR) ||
        roles.contains(&Role::LOOTING) ||
        roles.contains(&Role::IMPULSE_DRAW)
    });
    let deck_selection_density = ((tutor_count as f32) + draw_count_weighted).min(12.0) / 12.0;
    let flood_mitigation = (1.0
        - if commander_selection { 0.25 } else { 0.0 }
        - deck_selection_density * 0.45)
        .clamp(0.35, 1.0);
    let flood_penalty = ((flood_risk - 0.35).max(0.0) * 0.30 * flood_mitigation).min(0.12);
    let land_score = land_drop_score + ((flood_risk - 0.35).max(0.0) * flood_mitigation);
    final_multiplier *= 1.0 - flood_penalty;

    let tapped_land_penalty = if land_count > 0 && tapped_land_count > 0 {
        let tapped_share = tapped_land_count as f32 / land_count as f32;
        let first_land_tapped_risk = tapped_share;
        let first_two_tapped_risk = hypergeometric_at_least(land_count, tapped_land_count, 2.min(land_count), 1);
        let first_three_tapped_risk = hypergeometric_at_least(land_count, tapped_land_count, 3.min(land_count), 1);

        let early_penalty = match provisional_bracket {
            5 => first_land_tapped_risk * 0.24 + first_two_tapped_risk * 0.10 + first_three_tapped_risk * 0.06,
            4 => first_land_tapped_risk * 0.18 + first_two_tapped_risk * 0.08 + first_three_tapped_risk * 0.05,
            _ => first_two_tapped_risk * 0.04 + first_three_tapped_risk * 0.03,
        };

        let tolerated_tapped = match provisional_bracket {
            5 => 1,
            4 => 2,
            _ => 3,
        };
        let excess_tapped = tapped_land_count.saturating_sub(tolerated_tapped) as f32;
        let post_three_excess = tapped_land_count.saturating_sub(3) as f32;
        let saturation_penalty = excess_tapped * 0.015 + post_three_excess * post_three_excess * 0.006;

        (early_penalty + saturation_penalty).min(0.30)
    } else {
        0.0
    };
    final_multiplier *= 1.0 - tapped_land_penalty;

    let total_score = (raw_score * final_multiplier - commander_mv_penalty).min(25.0).max(0.0);
    let role_score = raw_score / 25.0;

    let interpretation = match total_score {
        s if s >= 23.0 => "cEDH Optimized",
        s if s >= 19.0 => "Fringe cEDH",
        s if s >= 15.0 => "High Power",
        s if s >= 9.0 => "Focused / Synergistic",
        _ => "Casual / Battlecruiser",
    }.to_string();

    // Bracket Calculation
    let bracket = derive_bracket(n_gc, any_combo_found, total_score, amv);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::card_model::{Card, CardType};

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
    fn test_ancient_tomb_is_not_engine() {
        let tomb = make_card(
            "Ancient Tomb",
            0,
            vec![CardType::Land],
            "{T}: Add {C}{C}. Ancient Tomb deals 2 damage to you."
        );
        
        let roles = infer_roles(&tomb);
        assert!(!roles.contains(&Role::ENGINE), "Ancient Tomb should not be an Engine. Roles found: {:?}", roles);
    }

    #[test]
    fn test_gemstone_caverns_is_not_engine() {
        let gemstone_caverns = make_card(
            "Gemstone Caverns",
            0,
            vec![CardType::Land],
            "If Gemstone Caverns is in your opening hand and you're not the starting player, you may begin the game with Gemstone Caverns on the battlefield with a luck counter on it. If you do, exile a card from your hand. {T}: Add {C}. If Gemstone Caverns has a luck counter on it, instead add one mana of any color."
        );
        
        let roles = infer_roles(&gemstone_caverns);
        assert!(!roles.contains(&Role::ENGINE), "Gemstone Caverns should not be an Engine. Roles found: {:?}", roles);
        assert!(roles.contains(&Role::LAND), "Should be a Land");
    }

    #[test]
    fn test_field_of_the_dead_is_engine() {
        let field = make_card(
            "Field of the Dead",
            0,
            vec![CardType::Land],
            "Field of the Dead enters the battlefield tapped. Whenever Field of the Dead or another land enters the battlefield under your control, if you control seven or more lands with different names, create a 2/2 black Zombie creature token. {T}: Add {C}."
        );
        
        let roles = infer_roles(&field);
        assert!(roles.contains(&Role::ENGINE), "Field of the Dead should be an Engine. Roles found: {:?}", roles);
    }

    #[test]
    fn test_valakut_the_molten_pinnacle_is_engine() {
        let valakut = make_card(
            "Valakut, the Molten Pinnacle",
            0,
            vec![CardType::Land],
            "Valakut, the Molten Pinnacle enters the battlefield tapped. Whenever a Mountain enters the battlefield under your control, if you control at least five other Mountains, you may have Valakut deal 3 damage to any target. {T}: Add {R}."
        );
        
        let roles = infer_roles(&valakut);
        assert!(roles.contains(&Role::ENGINE), "Valakut should be an Engine because it has 'whenever'. Roles found: {:?}", roles);
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

    #[test]
    fn test_infect_archetype() {
        let atraxa = make_card("Atraxa, Praetors' Voice", 4, vec![CardType::Creature], "Flying, vigilance, deathtouch, lifelink. At the beginning of your end step, proliferate.");
        let commanders = vec![atraxa];

        let mut mainboard = vec![];
        // Infect/Toxic/Poison/Proliferate pieces
        mainboard.push(make_card("Glistener Elf", 1, vec![CardType::Creature], "Infect"));
        mainboard.push(make_card("Blighted Agent", 2, vec![CardType::Creature], "Infect. Blighted Agent can't be blocked."));
        mainboard.push(make_card("Skithiryx, the Blight Dragon", 5, vec![CardType::Creature], "Flying. Infect. {B}: Skithiryx gains haste until end of turn."));
        mainboard.push(make_card("Venerated Rotpriest", 1, vec![CardType::Creature], "Toxic 1. Whenever a creature you control becomes the target of a spell, target opponent gets a poison counter."));
        mainboard.push(make_card("Bloated Contaminator", 3, vec![CardType::Creature], "Trample. Toxic 1. Whenever Bloated Contaminator deals combat damage to a player, proliferate."));
        mainboard.push(make_card("Contaminant Grafter", 3, vec![CardType::Creature], "Trample. Toxic 1. Whenever one or more creatures you control deal combat damage to a player, proliferate."));
        mainboard.push(make_card("Evolution Sage", 3, vec![CardType::Creature], "Whenever a land enters the battlefield under your control, proliferate."));
        mainboard.push(make_card("Infectious Bite", 2, vec![CardType::Sorcery], "Target creature you control deals damage equal to its power to target creature you don't control. Each opponent gets a poison counter."));
        mainboard.push(make_card("Flux Channeler", 3, vec![CardType::Creature], "Whenever you cast a noncreature spell, proliferate."));
        mainboard.push(make_card("Vraska's Fall", 2, vec![CardType::Sorcery], "Each opponent sacrifices a creature or planeswalker and gets a poison counter."));

        // Add some lands
        for _ in 0..30 {
            mainboard.push(make_card("Forest", 0, vec![CardType::Land], "{T}: Add {G}."));
        }

        let evaluation = calculate_crispi(&mainboard, &commanders, 0);

        println!("Infect Test - Total Score: {}", evaluation.total_score);
        println!("Infect Test - Signal: {}", evaluation.infect_signal);
        println!("Infect Test - Archetype: {:?}", evaluation.archetype);
        println!("Infect Test - Consistency Score: {}", evaluation.consistency.score);
        println!("Infect Test - Pivotability Score: {}", evaluation.pivotability.score);

        assert_eq!(evaluation.archetype, DeckArchetype::Infect);
        assert!(evaluation.infect_signal >= 8.0);
        // Floors should apply: Consistency >= 4, Pivotability >= 3
        assert!(evaluation.consistency.score >= 4);
        assert!(evaluation.pivotability.score >= 3);
    }

    #[test]
    fn test_three_card_combo() {
        let mut mainboard = vec![];
        mainboard.push(make_card("Underworld Breach", 2, vec![CardType::Enchantment], "Each nonland card in your graveyard has escape."));
        mainboard.push(make_card("Lion's Eye Diamond", 0, vec![CardType::Artifact], "{T}, Discard your hand, Sacrifice Lion's Eye Diamond: Add three mana of any one color. Activate only as a mana ability."));
        mainboard.push(make_card("Brain Freeze", 2, vec![CardType::Instant], "Storm. Target player mills three cards."));
        
        // Add enough non-land cards to meet prereqs (prereqs: 3 for this combo)
        // logic: if non_land_count < (5 + combo.prereqs) { met = false; }
        // 5 + 3 = 8.
        for i in 0..5 {
            mainboard.push(make_card(&format!("Filler {}", i), 1, vec![CardType::Creature], ""));
        }
        
        let commanders = vec![];
        let evaluation = calculate_crispi(&mainboard, &commanders, 0);
        
        println!("Three-Card Combo Test - Detected: {:?}", evaluation.detected_combos);
        assert!(evaluation.detected_combos.iter().any(|c| c.contains("Underworld Breach") && c.contains("Lion's Eye Diamond") && c.contains("Brain Freeze")));
        assert!(evaluation.combo_multiplier > 1.0);
    }

    #[test]
    fn test_hoarding_broodlord_combo() {
        let mut mainboard = vec![];
        mainboard.push(make_card("Hoarding Broodlord", 8, vec![CardType::Creature], "Convoke. When Hoarding Broodlord enters the battlefield, search your library for a card, exile it face down, then shuffle. For as long as that card remains exiled, you may play it. Spells you cast from exile have convoke."));
        mainboard.push(make_card("Saw in Half", 3, vec![CardType::Instant], "Destroy target creature. If that creature is destroyed this way, its controller creates two tokens that are copies of it, except their base power is half that creature's base power and their base toughness is half that creature's base toughness, each rounded up."));
        mainboard.push(make_card("Sacrifice", 1, vec![CardType::Instant], "As an additional cost to cast Sacrifice, sacrifice a creature. Add an amount of {B} equal to the sacrificed creature's mana value."));
        mainboard.push(make_card("Burnt Offering", 1, vec![CardType::Instant], "As an additional cost to cast Burnt Offering, sacrifice a creature. Add an amount of {B} or {R} equal to the sacrificed creature's mana value."));
        mainboard.push(make_card("Culling the Weak", 1, vec![CardType::Instant], "As an additional cost to cast Culling the Weak, sacrifice a creature. Add {B}{B}{B}{B}."));

        // Prereqs: 3. 5 + 3 = 8.
        // We have 5 cards already. We need 3 more to have 8 total.
        for i in 0..3 {
            mainboard.push(make_card(&format!("Filler {}", i), 1, vec![CardType::Creature], ""));
        }

        let commanders = vec![];
        let evaluation = calculate_crispi(&mainboard, &commanders, 0);

        println!("Hoarding Broodlord Test (Mainboard, 8 cards, Prereqs OK) - Detected: {:?}", evaluation.detected_combos);
        assert!(evaluation.detected_combos.iter().any(|c| c.contains("Hoarding Broodlord") && c.contains("Saw in Half") && c.contains("Sacrifice")));
        assert!(evaluation.detected_combos.iter().any(|c| c.contains("Hoarding Broodlord") && c.contains("Saw in Half") && c.contains("Burnt Offering")));
        assert!(evaluation.detected_combos.iter().any(|c| c.contains("Hoarding Broodlord") && c.contains("Saw in Half") && c.contains("Culling the Weak")));

        // Now with Hoarding Broodlord as Commander
        let hb = mainboard.remove(0);
        let commanders_hb = vec![hb];
        let evaluation_cmdr = calculate_crispi(&mainboard, &commanders_hb, 0);
        println!("Hoarding Broodlord Test (Commander) - Detected: {:?}", evaluation_cmdr.detected_combos);
        assert!(evaluation_cmdr.detected_combos.iter().any(|c| c.contains("Hoarding Broodlord") && c.contains("Saw in Half") && c.contains("Sacrifice")));
    }
}
