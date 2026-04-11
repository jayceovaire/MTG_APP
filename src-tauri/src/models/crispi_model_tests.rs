use crate::models::card_model::{Card, CardType};
use crate::models::crispi_archetypes::detect_archetype;
use crate::models::crispi_patterns::infer_roles;
use crate::models::crispi_types::{Role, DeckArchetype};
use crate::models::crispi_probability::derive_bracket;
use crate::models::crispi_model::{calculate_crispi, combo_piece_names_for_deck, infer_roles_with_combo_context};

fn make_card(name: &str, mv: u8, types: Vec<CardType>, text: &str) -> Card {
    let mut sub_types = vec![];
    if name.to_lowercase().contains("equipment")
        || text.to_lowercase().contains("equipped creature")
    {
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
        None,
    )
}

#[test]
fn test_ancient_tomb_is_not_engine() {
    let tomb = make_card(
        "Ancient Tomb",
        0,
        vec![CardType::Land],
        "{T}: Add {C}{C}. Ancient Tomb deals 2 damage to you.",
    );

    let roles = infer_roles(&tomb);
    assert!(
        !roles.contains(&Role::ENGINE),
        "Ancient Tomb should not be an Engine. Roles found: {:?}",
        roles
    );
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
    assert!(
        !roles.contains(&Role::ENGINE),
        "Gemstone Caverns should not be an Engine. Roles found: {:?}",
        roles
    );
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
    assert!(
        roles.contains(&Role::ENGINE),
        "Field of the Dead should be an Engine. Roles found: {:?}",
        roles
    );
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
    assert!(
        roles.contains(&Role::ENGINE),
        "Valakut should be an Engine because it has 'whenever'. Roles found: {:?}",
        roles
    );
}

#[test]
fn test_pure_proliferate_is_not_infect_role() {
    let evolution_sage = make_card(
        "Evolution Sage",
        3,
        vec![CardType::Creature],
        "Whenever a land enters the battlefield under your control, proliferate.",
    );

    let roles = infer_roles(&evolution_sage);
    assert!(
        roles.contains(&Role::PROLIFERATE),
        "Evolution Sage should be tagged as Proliferate. Roles found: {:?}",
        roles
    );
    assert!(
        !roles.contains(&Role::INFECT),
        "Pure proliferate should not be tagged as Infect. Roles found: {:?}",
        roles
    );
}

#[test]
fn test_ritual_is_not_ramp() {
    let dark_ritual = make_card(
        "Dark Ritual",
        1,
        vec![CardType::Instant],
        "Add {B}{B}{B}."
    );

    let roles = infer_roles(&dark_ritual);
    assert!(
        roles.contains(&Role::RITUAL),
        "Dark Ritual should be a Ritual. Roles: {:?}",
        roles
    );
    assert!(
        !roles.contains(&Role::RAMP),
        "Ritual should NOT be RAMP. Roles: {:?}",
        roles
    );
}

#[test]
fn test_treasure_burst_is_not_ramp() {
    let brasss_bounty = make_card(
        "Brass's Bounty",
        7,
        vec![CardType::Sorcery],
        "For each land you control, create a Treasure token."
    );

    let roles = infer_roles(&brasss_bounty);
    assert!(
        roles.contains(&Role::TREASURE_BURST),
        "Brass's Bounty should be a Treasure Burst. Roles: {:?}",
        roles
    );
    assert!(
        !roles.contains(&Role::RAMP),
        "Treasure Burst should NOT be RAMP. Roles: {:?}",
        roles
    );
}

#[test]
fn test_gaeas_cradle_is_not_ritual() {
    let cradle = make_card(
        "Gaea's Cradle",
        0,
        vec![CardType::Land],
        "{T}: Add {G} for each creature you control."
    );

    let roles = infer_roles(&cradle);
    assert!(
        !roles.contains(&Role::RITUAL),
        "Gaea's Cradle should NOT be a Ritual. Roles: {:?}",
        roles
    );
}

#[test]
fn test_takenuma_is_not_ritual() {
    let takenuma = make_card(
        "Takenuma, Abandoned Mire",
        0,
        vec![CardType::Land],
        "{T}: Add {B}. Channel - {3}{B}, Discard Takenuma, Abandoned Mire: Mill three cards, then return a creature or planeswalker card from your graveyard to your hand. This ability costs {1} less to activate for each legendary creature you control."
    );

    let roles = infer_roles(&takenuma);
    assert!(
        !roles.contains(&Role::RITUAL),
        "Takenuma should NOT be a Ritual. Roles: {:?}",
        roles
    );
}

#[test]
fn test_mana_vault_is_not_ritual() {
    let mana_vault = make_card(
        "Mana Vault",
        1,
        vec![CardType::Artifact],
        "Mana Vault doesn't untap during your untap step. {T}: Add {C}{C}{C}. At the beginning of your upkeep, you may pay {4}. If you do, untap Mana Vault. At the beginning of your draw step, if Mana Vault is tapped, it deals 1 damage to you."
    );

    let roles = infer_roles(&mana_vault);
    assert!(
        !roles.contains(&Role::RITUAL),
        "Mana Vault should NOT be a Ritual. Roles: {:?}",
        roles
    );
}

#[test]
fn test_land_tutor_is_fixing_not_tutor() {
    let farseek = make_card(
            "Farseek",
            2,
            vec![CardType::Sorcery],
            "Search your library for a Plains, Island, Swamp, or Mountain card and put it onto the battlefield tapped, then shuffle."
        );

    let roles = infer_roles(&farseek);
    assert!(
        roles.contains(&Role::FIXING),
        "Land tutors should count as fixing. Roles found: {:?}",
        roles
    );
    assert!(
        !roles.contains(&Role::TUTOR),
        "Land tutors should not count as generic tutors. Roles found: {:?}",
        roles
    );
}

#[test]
fn test_fetch_land_is_fixing_not_ramp() {
    let evolving_wilds = make_card(
        "Evolving Wilds",
        0,
        vec![CardType::Land],
        "{T}, Sacrifice Evolving Wilds: Search your library for a basic land card, put it onto the battlefield tapped, then shuffle.",
    );

    let roles = infer_roles(&evolving_wilds);
    assert!(
        roles.contains(&Role::LAND),
        "Fetch lands should still be tagged as lands. Roles found: {:?}",
        roles
    );
    assert!(
        roles.contains(&Role::FIXING),
        "Fetch lands should count as fixing. Roles found: {:?}",
        roles
    );
    assert!(
        !roles.contains(&Role::RAMP),
        "Fetch lands should not count as ramp. Roles found: {:?}",
        roles
    );
}

#[test]
fn test_basic_typed_fetch_land_is_not_ramp() {
    let flooded_strand = make_card(
        "Flooded Strand",
        0,
        vec![CardType::Land],
        "{T}, Pay 1 life, Sacrifice Flooded Strand: Search your library for a Plains or Island card, put it onto the battlefield, then shuffle.",
    );

    let roles = infer_roles(&flooded_strand);
    assert!(
        roles.contains(&Role::FIXING),
        "Typed fetch lands should count as fixing. Roles found: {:?}",
        roles
    );
    assert!(
        !roles.contains(&Role::RAMP),
        "Typed fetch lands should not count as ramp. Roles found: {:?}",
        roles
    );
}

#[test]
fn test_repeatable_treasure_creature_is_not_fast_mana_one_shot() {
    let grim_hireling = make_card(
        "Grim Hireling",
        4,
        vec![CardType::Creature],
        "Whenever one or more creatures you control deal combat damage to a player, create two Treasure tokens.",
    );

    let roles = infer_roles(&grim_hireling);
    assert!(
        roles.contains(&Role::RAMP),
        "Repeatable treasure creatures should still count as ramp. Roles found: {:?}",
        roles
    );
    assert!(
        roles.contains(&Role::ENGINE),
        "Repeatable treasure creatures should count as engines. Roles found: {:?}",
        roles
    );
    assert!(
        !roles.contains(&Role::FAST_MANA_ONE_SHOT),
        "Repeatable treasure creatures should not count as fast mana one shot. Roles found: {:?}",
        roles
    );
    assert!(
        !roles.contains(&Role::FAST_MANA),
        "Repeatable treasure creatures should not count as fast mana. Roles found: {:?}",
        roles
    );
}

#[test]
fn test_one_shot_treasure_spell_still_counts_as_fast_mana_one_shot() {
    let strike_it_rich = make_card(
        "Strike It Rich",
        1,
        vec![CardType::Sorcery],
        "Create a Treasure token. Flashback {2}{R}.",
    );

    let roles = infer_roles(&strike_it_rich);
    assert!(
        roles.contains(&Role::FAST_MANA_ONE_SHOT),
        "One-shot treasure spells should still count as fast mana one shot. Roles found: {:?}",
        roles
    );
}

#[test]
fn test_repeatable_treasure_enchantment_is_not_fast_mana_one_shot() {
    let black_market_connections = make_card(
        "Black Market Connections",
        3,
        vec![CardType::Enchantment],
        "At the beginning of your precombat main phase, choose one or more. Create a Treasure token. You lose 1 life.",
    );

    let roles = infer_roles(&black_market_connections);
    assert!(
        roles.contains(&Role::RAMP),
        "Repeatable treasure enchantments should count as ramp. Roles found: {:?}",
        roles
    );
    assert!(
        roles.contains(&Role::ENGINE),
        "Repeatable treasure enchantments should count as engines. Roles found: {:?}",
        roles
    );
    assert!(
        !roles.contains(&Role::FAST_MANA_ONE_SHOT),
        "Repeatable treasure enchantments should not count as fast mana one shot. Roles found: {:?}",
        roles
    );
    assert!(
        !roles.contains(&Role::FAST_MANA),
        "Repeatable treasure enchantments should not count as fast mana. Roles found: {:?}",
        roles
    );
}

#[test]
fn test_simian_spirit_guide_is_fast_mana_not_engine() {
    let simian_spirit_guide = make_card(
        "Simian Spirit Guide",
        3,
        vec![CardType::Creature],
        "Exile Simian Spirit Guide from your hand: Add {R}.",
    );

    let roles = infer_roles(&simian_spirit_guide);
    assert!(
        roles.contains(&Role::FAST_MANA_ONE_SHOT),
        "Simian Spirit Guide should count as fast mana one shot. Roles found: {:?}",
        roles
    );
    assert!(
        !roles.contains(&Role::ENGINE),
        "Simian Spirit Guide should not count as an engine. Roles found: {:?}",
        roles
    );
}

#[test]
fn test_elvish_spirit_guide_is_fast_mana_not_engine() {
    let elvish_spirit_guide = make_card(
        "Elvish Spirit Guide",
        3,
        vec![CardType::Creature],
        "Exile Elvish Spirit Guide from your hand: Add {G}.",
    );

    let roles = infer_roles(&elvish_spirit_guide);
    assert!(
        roles.contains(&Role::FAST_MANA_ONE_SHOT),
        "Elvish Spirit Guide should count as fast mana one shot. Roles found: {:?}",
        roles
    );
    assert!(
        !roles.contains(&Role::ENGINE),
        "Elvish Spirit Guide should not count as an engine. Roles found: {:?}",
        roles
    );
}

#[test]
fn test_mana_vault_is_fast_mana_not_engine() {
    let mana_vault = make_card(
        "Mana Vault",
        1,
        vec![CardType::Artifact],
        "Mana Vault doesn't untap during your untap step. At the beginning of your upkeep, you may pay {4}. If you do, untap Mana Vault. At the beginning of your draw step, if Mana Vault is tapped, it deals 1 damage to you. {T}: Add {C}{C}{C}.",
    );

    let roles = infer_roles(&mana_vault);
    assert!(
        roles.contains(&Role::RAMP),
        "Mana Vault should count as ramp. Roles found: {:?}",
        roles
    );
    assert!(
        roles.contains(&Role::FAST_MANA),
        "Mana Vault should count as fast mana. Roles found: {:?}",
        roles
    );
    assert!(
        !roles.contains(&Role::ENGINE),
        "Mana Vault should not count as an engine. Roles found: {:?}",
        roles
    );
}

#[test]
fn test_turbo_requires_natural_speed_five() {
    let archetype = detect_archetype(0.0, 0.0, 19.0, 8.0, 0.0, 0.0, 0.0, 0.0, false, 4, 0, 0);
    assert_eq!(archetype, DeckArchetype::Midrange);

    let archetype = detect_archetype(0.0, 0.0, 19.0, 8.0, 0.0, 0.0, 0.0, 0.0, false, 5, 0, 0);
    assert_eq!(archetype, DeckArchetype::Turbo);
}

#[test]
fn test_two_card_combo_floors_bracket_four() {
    let bracket = derive_bracket(0, 4, 10.0, 2.5);
    assert_eq!(bracket, 4);
}

#[test]
fn test_three_card_combo_floors_bracket_three() {
    let bracket = derive_bracket(0, 3, 10.0, 2.5);
    assert_eq!(bracket, 3);
}

#[test]
fn test_rog_si_archetype() {
    let rog = make_card(
        "Rograkh",
        0,
        vec![CardType::Creature],
        "First strike, menace, trample. Partner.",
    );
    let silas = make_card("Silas Renn", 3, vec![CardType::Creature, CardType::Artifact], "Deathtouch. Whenever Silas Renn deals combat damage to a player, you may cast target artifact card from your graveyard this turn. Partner.");

    let commanders = vec![rog, silas];

    let mut mainboard = vec![];
    // Fast Mana
    mainboard.push(make_card(
        "Chrome Mox",
        0,
        vec![CardType::Artifact],
        "Imprint - ... Add one mana of any color.",
    ));
    mainboard.push(make_card(
        "Mox Amber",
        0,
        vec![CardType::Artifact],
        "Add one mana of any color among legendary...",
    ));
    mainboard.push(make_card(
        "Mox Opal",
        0,
        vec![CardType::Artifact],
        "Metalcraft - Add one mana...",
    ));
    mainboard.push(make_card(
        "Mana Crypt",
        0,
        vec![CardType::Artifact],
        "Add {C}{C}.",
    ));
    mainboard.push(make_card(
        "Mana Vault",
        1,
        vec![CardType::Artifact],
        "{T}: Add {C}{C}{C}.",
    ));
    mainboard.push(make_card(
        "Sol Ring",
        1,
        vec![CardType::Artifact],
        "{T}: Add {C}{C}.",
    ));
    mainboard.push(make_card(
        "Lotus Petal",
        0,
        vec![CardType::Artifact],
        "Sacrifice Lotus Petal: Add one mana...",
    ));
    mainboard.push(make_card(
        "Dark Ritual",
        1,
        vec![CardType::Sorcery],
        "Add {B}{B}{B}.",
    ));
    mainboard.push(make_card(
        "Lion's Eye Diamond",
        0,
        vec![CardType::Artifact],
        "Discard your hand, Sacrifice LED: Add {B}{B}{B}.",
    ));

    // Tutors (Consistency)
    mainboard.push(make_card(
        "Demonic Tutor",
        2,
        vec![CardType::Sorcery],
        "Search your library for a card...",
    ));
    mainboard.push(make_card(
        "Vampiric Tutor",
        1,
        vec![CardType::Instant],
        "Search your library for a card...",
    ));
    mainboard.push(make_card(
        "Mystical Tutor",
        1,
        vec![CardType::Instant],
        "Search your library for an instant or sorcery...",
    ));
    mainboard.push(make_card(
        "Imperial Seal",
        1,
        vec![CardType::Sorcery],
        "Search your library for a card...",
    ));
    mainboard.push(make_card(
        "Gambit",
        1,
        vec![CardType::Sorcery],
        "Search your library...",
    ));
    mainboard.push(make_card(
        "Wishclaw Talisman",
        2,
        vec![CardType::Artifact],
        "Search...",
    ));

    // Draw (Turbo/Consistency)
    mainboard.push(make_card(
        "Ad Nauseam",
        5,
        vec![CardType::Instant],
        "Reveal the top card... repeat this process...",
    ));
    mainboard.push(make_card(
        "Windfall",
        3,
        vec![CardType::Sorcery],
        "Each player discards their hand, then draws cards...",
    ));
    mainboard.push(make_card(
        "Wheel of Fortune",
        3,
        vec![CardType::Sorcery],
        "Each player discards their hand, then draws seven cards.",
    ));
    mainboard.push(make_card(
        "Rhystic Study",
        3,
        vec![CardType::Enchantment],
        "Whenever an opponent casts... draw a card...",
    ));
    mainboard.push(make_card(
        "Mystic Remora",
        1,
        vec![CardType::Enchantment],
        "Whenever an opponent casts... draw a card...",
    ));

    // Interaction
    mainboard.push(make_card(
        "Force of Will",
        5,
        vec![CardType::Instant],
        "Rather than pay this spell's mana cost... counter target spell.",
    ));
    mainboard.push(make_card(
        "Pact of Negation",
        0,
        vec![CardType::Instant],
        "Counter target spell.",
    ));
    mainboard.push(make_card(
        "Mental Misstep",
        1,
        vec![CardType::Instant],
        "Rather than pay... counter target spell with mana value 1.",
    ));
    mainboard.push(make_card("Deflecting Swat", 3, vec![CardType::Instant], "If you control a commander, you may cast this spell without paying its mana cost. Change the target..."));
    mainboard.push(make_card(
        "Deadly Rollick",
        4,
        vec![CardType::Instant],
        "If you control a commander... exile target creature.",
    ));
    mainboard.push(make_card(
        "Fierce Guardianship",
        3,
        vec![CardType::Instant],
        "If you control a commander... counter target noncreature spell.",
    ));

    // Wincons
    mainboard.push(make_card(
        "Thassa's Oracle",
        2,
        vec![CardType::Creature],
        "When Thassa's Oracle enters... win the game.",
    ));
    mainboard.push(make_card(
        "Underworld Breach",
        2,
        vec![CardType::Enchantment],
        "Each nonland card in your graveyard has escape.",
    ));
    mainboard.push(make_card(
        "Brain Freeze",
        2,
        vec![CardType::Instant],
        "Target player mills three cards. Storm.",
    ));

    // Lands
    for _ in 0..25 {
        mainboard.push(make_card(
            "Badlands",
            0,
            vec![CardType::Land],
            "{T}: Add {B} or {R}.",
        ));
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
    assert!(
        evaluation.total_score >= 23.0,
        "Rog Si should be cEDH Optimized! Score was {}",
        evaluation.total_score
    );
}

#[test]
fn test_midrange_archetype() {
    let thrasios = make_card("Thrasios, Triton Hero", 2, vec![CardType::Creature], "{4}: Scry 1, then reveal the top card of your library. If it's a land card, put it onto the battlefield tapped. Otherwise, draw a card. Partner.");
    let tymna = make_card("Tymna the Weaver", 3, vec![CardType::Creature], "Lifelink. At the beginning of your postcombat main phase, you may pay X life, where X is the number of opponents that were dealt combat damage by creatures this turn. If you do, draw X cards. Partner.");

    let commanders = vec![thrasios, tymna];

    let mut mainboard = vec![];
    // Typical Midrange pieces (Engines/Efficiency)
    mainboard.push(make_card(
        "Sylvan Library",
        2,
        vec![CardType::Enchantment],
        "At the beginning of your draw step, you may draw two additional cards...",
    ));
    mainboard.push(make_card("Esper Sentinel", 1, vec![CardType::Creature], "Whenever an opponent casts their first noncreature spell each turn, draw a card unless that player pays {X}..."));
    mainboard.push(make_card(
        "Seedborn Muse",
        5,
        vec![CardType::Creature],
        "Untap all permanents you control during each other player's untap step.",
    ));
    mainboard.push(make_card("Smothering Tithe", 4, vec![CardType::Enchantment], "Whenever an opponent draws a card, that player may pay {2}. If they don't, you create a Treasure token."));
    mainboard.push(make_card(
        "Arcane Signet",
        2,
        vec![CardType::Artifact],
        "{T}: Add one mana of any color in your commander's color identity.",
    ));
    mainboard.push(make_card(
        "Sol Ring",
        1,
        vec![CardType::Artifact],
        "{T}: Add {C}{C}.",
    ));
    mainboard.push(make_card(
        "Birds of Paradise",
        1,
        vec![CardType::Creature],
        "{T}: Add one mana of any color.",
    ));
    mainboard.push(make_card(
        "Demonic Tutor",
        2,
        vec![CardType::Sorcery],
        "Search your library for a card...",
    ));
    mainboard.push(make_card(
        "Vampiric Tutor",
        1,
        vec![CardType::Instant],
        "Search your library for a card...",
    ));
    mainboard.push(make_card(
        "Cyclonic Rift",
        2,
        vec![CardType::Instant],
        "Return target nonland permanent... Overload {6}{u}.",
    ));
    mainboard.push(make_card(
        "Assassin's Trophy",
        2,
        vec![CardType::Instant],
        "Destroy target permanent...",
    ));

    let evaluation = calculate_crispi(&mainboard, &commanders, 0);
    assert_eq!(evaluation.archetype, DeckArchetype::Midrange);
}

#[test]
fn test_kosei_deck() {
    let kosei = make_card("Kosei, Penitent Warlord", 4, vec![CardType::Creature], "Whenever Kosei deals combat damage to a player, if it's enchanted, equipped, and has a counter on it, you draw cards equal to its power...");
    let commanders = vec![kosei];

    let mut mainboard = vec![];
    // Fast Mana
    mainboard.push(make_card(
        "Chrome Mox",
        0,
        vec![CardType::Artifact],
        "Imprint - ... Add one mana of any color.",
    ));
    mainboard.push(make_card(
        "Lotus Petal",
        0,
        vec![CardType::Artifact],
        "Sacrifice Lotus Petal: Add one mana...",
    ));
    mainboard.push(make_card(
        "Mana Vault",
        1,
        vec![CardType::Artifact],
        "Add {C}{C}{C}.",
    ));
    mainboard.push(make_card(
        "Sol Ring",
        1,
        vec![CardType::Artifact],
        "Add {C}{C}.",
    ));

    // Protection (Resilience)
    mainboard.push(make_card(
        "Autumn's Veil",
        1,
        vec![CardType::Instant],
        "Spells you control can't be countered...",
    ));
    mainboard.push(make_card(
        "Gaea's Gift",
        2,
        vec![CardType::Instant],
        "Target creature gets +1/+1 and gains reach, trample, hexproof, and indestructible...",
    ));
    mainboard.push(make_card(
        "Tamiyo's Safekeeping",
        1,
        vec![CardType::Instant],
        "Target permanent gains hexproof and indestructible...",
    ));
    mainboard.push(make_card(
        "Vines of Vastwood",
        1,
        vec![CardType::Instant],
        "Target creature can't be the target of spells...",
    ));
    mainboard.push(make_card(
        "Silkguard",
        1,
        vec![CardType::Instant],
        "Any number of target creatures you control get a +1/+1 counter and gain hexproof...",
    ));
    mainboard.push(make_card("Savage Summoning", 1, vec![CardType::Instant], "This spell can't be countered. The next creature card you cast... gains flash and enters with a +1/+1 counter and can't be countered."));
    mainboard.push(make_card(
        "Strength of Will",
        1,
        vec![CardType::Instant],
        "Target creature gains indestructible...",
    ));

    // Removal (Interaction)
    mainboard.push(make_card(
        "Beast Within",
        3,
        vec![CardType::Instant],
        "Destroy target permanent...",
    ));
    mainboard.push(make_card(
        "Krosan Grip",
        3,
        vec![CardType::Instant],
        "Split second. Destroy target artifact or enchantment.",
    ));
    mainboard.push(make_card(
        "Return to Nature",
        2,
        vec![CardType::Instant],
        "Destroy target artifact...",
    ));
    mainboard.push(make_card(
        "Collective Resistance",
        2,
        vec![CardType::Instant],
        "Destroy target artifact...",
    ));

    // Equipment/Auras (Pivotability/Wincon)
    mainboard.push(make_card(
        "Blackblade Reforged",
        2,
        vec![CardType::Artifact],
        "Equipped creature gets +1/+1 for each land you control.",
    ));
    mainboard.push(make_card(
        "Hammer of Nazahn",
        4,
        vec![CardType::Artifact],
        "Whenever Hammer of Nazahn or another Equipment enters... gain indestructible...",
    ));
    mainboard.push(make_card(
        "Helm of the Gods",
        1,
        vec![CardType::Artifact],
        "Equipped creature gets +1/+1 for each enchantment you control.",
    ));
    mainboard.push(make_card(
        "Lavaspur Boots",
        1,
        vec![CardType::Artifact],
        "Equipped creature gets +1/+0 and gains haste and ward {1}.",
    ));
    mainboard.push(make_card(
        "Swiftfoot Boots",
        2,
        vec![CardType::Artifact],
        "Equipped creature gets hexproof and haste.",
    ));
    mainboard.push(make_card(
        "Sword of the Animist",
        2,
        vec![CardType::Artifact],
        "Whenever equipped creature attacks... search for a land...",
    ));
    mainboard.push(make_card("Argentum Armor", 6, vec![CardType::Artifact], "Equipped creature gets +6/+6. Whenever equipped creature attacks, destroy target permanent."));
    mainboard.push(make_card("Hydra's Growth", 3, vec![CardType::Enchantment], "Enchanted creature gets +1/+1. At the beginning of your upkeep, double the number of +1/+1 counters on enchanted creature."));
    mainboard.push(make_card("Bear Umbra", 4, vec![CardType::Enchantment], "Enchanted creature gets +2/+2 and has \"Whenever this creature attacks, untap all lands you control.\" Totem armor."));

    // Ramp
    mainboard.push(make_card(
        "Fyndhorn Elves",
        1,
        vec![CardType::Creature],
        "{T}: Add {G}.",
    ));
    mainboard.push(make_card(
        "Boreal Druid",
        1,
        vec![CardType::Creature],
        "{T}: Add {C}.",
    ));
    mainboard.push(make_card(
        "Sakura-Tribe Scout",
        1,
        vec![CardType::Creature],
        "{T}: Put a land...",
    ));
    mainboard.push(make_card(
        "Biophagus",
        2,
        vec![CardType::Creature],
        "{T}: Add one mana...",
    ));
    mainboard.push(make_card(
        "Emerald Medallion",
        2,
        vec![CardType::Artifact],
        "Green spells cost {1} less...",
    ));

    // Synergy
    mainboard.push(make_card("Season of Growth", 2, vec![CardType::Enchantment], "Whenever a creature enters... scry 1. Whenever you cast a spell that targets... draw a card."));
    mainboard.push(make_card(
        "Snake Umbra",
        3,
        vec![CardType::Enchantment],
        "Enchanted creature gets +1/+1... draw a card. Totem armor.",
    ));

    // Lands (dummy lands)
    for _ in 0..30 {
        mainboard.push(make_card(
            "Forest",
            0,
            vec![CardType::Land],
            "{T}: Add {G}.",
        ));
    }

    let evaluation = calculate_crispi(&mainboard, &commanders, 0);

    println!("Kosei Test - Total Score: {}", evaluation.total_score);
    println!("Kosei Test - Raw Score: {}", evaluation.raw_score);
    println!(
        "Kosei Test - AMV: {:.2}",
        evaluation.total_score / evaluation.final_multiplier / evaluation.raw_score
    ); // Not exact but give idea
    println!("Kosei Test - Multiplier: {}", evaluation.final_multiplier);
    println!("Kosei Test - Consistency: {}", evaluation.consistency.score);
    println!("Kosei Test - Resilience: {}", evaluation.resilience.score);
    println!("Kosei Test - Interaction: {}", evaluation.interaction.score);
    println!("Kosei Test - Speed: {}", evaluation.speed.score);
    println!(
        "Kosei Test - Pivotability: {}",
        evaluation.pivotability.score
    );
    println!("Kosei Test - Interpretation: {}", evaluation.interpretation);
    println!("Kosei Test - Archetype: {:?}", evaluation.archetype);
    println!("Kosei Test - Voltron Signal: {}", evaluation.voltron_signal);

    // Assert Voltron Archetype
    assert_eq!(evaluation.archetype, DeckArchetype::Voltron);
    // This is expected to be under 19.0 now
    assert!(
        evaluation.total_score < 19.0,
        "Deck should not be Fringe cEDH! Score was {}",
        evaluation.total_score
    );
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
    mainboard.push(make_card(
        "Temple Bell",
        3,
        vec![CardType::Artifact],
        "{T}: Each player draws a card.",
    ));
    mainboard.push(make_card(
        "Dictate of Kruphix",
        3,
        vec![CardType::Enchantment],
        "At the beginning of each player's draw step, that player draws an additional card.",
    ));
    mainboard.push(make_card("Rites of Flourishing", 3, vec![CardType::Enchantment], "Each player may play an additional land on each of their turns. Each player draws an additional card..."));
    mainboard.push(make_card("Collective Voyage", 1, vec![CardType::Sorcery], "Each player searches their library for basic land cards and puts them onto the battlefield..."));
    mainboard.push(make_card("Veteran Explorer", 1, vec![CardType::Creature], "When Veteran Explorer dies, each player searches their library for up to two basic land cards and puts them onto the battlefield..."));
    mainboard.push(make_card(
        "Zhur-Taa Ancient",
        5,
        vec![CardType::Creature],
        "Whenever a player taps a land for mana, that player adds one mana...",
    ));
    mainboard.push(make_card(
        "Heartbeat of Spring",
        3,
        vec![CardType::Enchantment],
        "Whenever a player taps a land for mana, that player adds one mana...",
    ));
    mainboard.push(make_card("Minds Aglow", 1, vec![CardType::Sorcery], "Each player may pay {X}. Then each player draws cards equal to the total amount of {X} paid."));
    mainboard.push(make_card(
        "Howling Mine",
        2,
        vec![CardType::Artifact],
        "Each player draws an additional card...",
    )); // Simplified for test

    // Add some lands
    for _ in 0..30 {
        mainboard.push(make_card(
            "Forest",
            0,
            vec![CardType::Land],
            "{T}: Add {G}.",
        ));
    }

    let evaluation = calculate_crispi(&mainboard, &commanders, 0);

    println!("Group Hug Test - Total Score: {}", evaluation.total_score);
    println!("Group Hug Test - Signal: {}", evaluation.group_hug_signal);
    println!("Group Hug Test - Archetype: {:?}", evaluation.archetype);
    println!(
        "Group Hug Test - Consistency Score: {}",
        evaluation.consistency.score
    );
    println!(
        "Group Hug Test - Pivotability Score: {}",
        evaluation.pivotability.score
    );

    assert_eq!(evaluation.archetype, DeckArchetype::GroupHug);
    assert!(evaluation.group_hug_signal >= 8.0);
    // Floors should apply: Consistency >= 3, Pivotability >= 3
    assert!(evaluation.consistency.score >= 3);
    assert!(evaluation.pivotability.score >= 3);
}

#[test]
fn test_infect_archetype() {
    let atraxa = make_card(
        "Atraxa, Praetors' Voice",
        4,
        vec![CardType::Creature],
        "Flying, vigilance, deathtouch, lifelink. At the beginning of your end step, proliferate.",
    );
    let commanders = vec![atraxa];

    let mut mainboard = vec![];
    // Infect/Toxic/Poison/Proliferate pieces
    mainboard.push(make_card(
        "Glistener Elf",
        1,
        vec![CardType::Creature],
        "Infect",
    ));
    mainboard.push(make_card(
        "Blighted Agent",
        2,
        vec![CardType::Creature],
        "Infect. Blighted Agent can't be blocked.",
    ));
    mainboard.push(make_card(
        "Skithiryx, the Blight Dragon",
        5,
        vec![CardType::Creature],
        "Flying. Infect. {B}: Skithiryx gains haste until end of turn.",
    ));
    mainboard.push(make_card("Venerated Rotpriest", 1, vec![CardType::Creature], "Toxic 1. Whenever a creature you control becomes the target of a spell, target opponent gets a poison counter."));
    mainboard.push(make_card("Bloated Contaminator", 3, vec![CardType::Creature], "Trample. Toxic 1. Whenever Bloated Contaminator deals combat damage to a player, proliferate."));
    mainboard.push(make_card("Contaminant Grafter", 3, vec![CardType::Creature], "Trample. Toxic 1. Whenever one or more creatures you control deal combat damage to a player, proliferate."));
    mainboard.push(make_card(
        "Evolution Sage",
        3,
        vec![CardType::Creature],
        "Whenever a land enters the battlefield under your control, proliferate.",
    ));
    mainboard.push(make_card("Infectious Bite", 2, vec![CardType::Sorcery], "Target creature you control deals damage equal to its power to target creature you don't control. Each opponent gets a poison counter."));
    mainboard.push(make_card(
        "Flux Channeler",
        3,
        vec![CardType::Creature],
        "Whenever you cast a noncreature spell, proliferate.",
    ));
    mainboard.push(make_card(
        "Vraska's Fall",
        2,
        vec![CardType::Sorcery],
        "Each opponent sacrifices a creature or planeswalker and gets a poison counter.",
    ));

    // Add some lands
    for _ in 0..30 {
        mainboard.push(make_card(
            "Forest",
            0,
            vec![CardType::Land],
            "{T}: Add {G}.",
        ));
    }

    let evaluation = calculate_crispi(&mainboard, &commanders, 0);

    println!("Infect Test - Total Score: {}", evaluation.total_score);
    println!("Infect Test - Signal: {}", evaluation.infect_signal);
    println!("Infect Test - Archetype: {:?}", evaluation.archetype);
    println!(
        "Infect Test - Consistency Score: {}",
        evaluation.consistency.score
    );
    println!(
        "Infect Test - Pivotability Score: {}",
        evaluation.pivotability.score
    );

    assert_eq!(evaluation.archetype, DeckArchetype::Infect);
    assert!(evaluation.infect_signal >= 8.0);
    // Floors should apply: Consistency >= 4, Pivotability >= 3
    assert!(evaluation.consistency.score >= 4);
    assert!(evaluation.pivotability.score >= 3);
}

#[test]
fn test_storm_archetype() {
    let commanders = vec![make_card(
        "Kess, Dissident Mage",
        4,
        vec![CardType::Creature],
        "During each of your turns, you may cast an instant or sorcery spell from your graveyard."
    )];

    let mut mainboard = vec![];
    // Rituals
    mainboard.push(make_card("Dark Ritual", 1, vec![CardType::Instant], "Add {B}{B}{B}."));
    mainboard.push(make_card("Cabal Ritual", 2, vec![CardType::Instant], "Add {B}{B}{B}. Threshold - Add {B}{B}{B}{B}{B} instead."));
    mainboard.push(make_card("Pyretic Ritual", 2, vec![CardType::Instant], "Add {R}{R}{R}."));
    mainboard.push(make_card("Desperate Ritual", 2, vec![CardType::Instant], "Add {R}{R}{R}. Splice onto Arcane."));

    // Cantrips / Draw
    mainboard.push(make_card("Brainstorm", 1, vec![CardType::Instant], "Draw three cards, then put two cards from your hand on top of your library."));
    mainboard.push(make_card("Ponder", 1, vec![CardType::Sorcery], "Look at the top three cards of your library, then put them back in any order. You may shuffle. Draw a card."));
    mainboard.push(make_card("Preordain", 1, vec![CardType::Sorcery], "Scry 2, then draw a card."));
    mainboard.push(make_card("Gitaxian Probe", 1, vec![CardType::Sorcery], "Look at target player's hand. Draw a card."));

    // Cost Reduction / Engines
    mainboard.push(make_card("Baral, Chief of Compliance", 2, vec![CardType::Creature], "Instant and sorcery spells you cast cost {1} less to cast. Whenever a spell or ability you control counters a spell, you may draw a card. If you do, discard a card."));
    mainboard.push(make_card("Birgi, God of Storytelling", 3, vec![CardType::Creature], "Whenever you cast a spell, add {R}."));
    mainboard.push(make_card("Storm-Kiln Artist", 4, vec![CardType::Creature], "Magecraft - Whenever you cast or copy an instant or sorcery spell, create a Treasure token."));

    // Wincons
    mainboard.push(make_card("Grapeshot", 2, vec![CardType::Sorcery], "Grapeshot deals 1 damage to any target. Storm."));
    mainboard.push(make_card("Tendrils of Agony", 4, vec![CardType::Sorcery], "Target player loses 2 life and you gain 2 life. Storm."));
    mainboard.push(make_card("Aetherflux Reservoir", 4, vec![CardType::Artifact], "Whenever you cast a spell, you gain 1 life for each spell you've cast this turn. Pay 50 life: Aetherflux Reservoir deals 50 damage to any target."));

    // Add lands
    for _ in 0..30 {
        mainboard.push(make_card("Island", 0, vec![CardType::Land], "{T}: Add {U}."));
    }

    let evaluation = calculate_crispi(&mainboard, &commanders, 0);

    println!("Storm Test - Signal: {}", evaluation.storm_signal);
    println!("Storm Test - Archetype: {:?}", evaluation.archetype);

    assert_eq!(evaluation.archetype, DeckArchetype::Storm);
    assert!(evaluation.storm_signal >= 12.0);
}

#[test]
fn test_high_quality_non_storm_deck_is_not_storm() {
    let commanders = vec![make_card("Thrasios", 2, vec![CardType::Creature], "Partner. {4}: Scry 1, then reveal the top card of your library. If it's a land card, put it onto the battlefield tapped. Otherwise, draw a card.")];
    
    let mut mainboard = vec![];
    // Rituals/Fast Mana (but no payoffs)
    mainboard.push(make_card("Dark Ritual", 1, vec![CardType::Instant], "Add {B}{B}{B}."));
    mainboard.push(make_card("Cabal Ritual", 2, vec![CardType::Instant], "Add {B}{B}{B}. Threshold - Add {B}{B}{B}{B}{B} instead."));
    mainboard.push(make_card("Pyretic Ritual", 2, vec![CardType::Instant], "Add {R}{R}{R}."));
    mainboard.push(make_card("Desperate Ritual", 2, vec![CardType::Instant], "Add {R}{R}{R}. Splice onto Arcane."));
    mainboard.push(make_card("Lotus Petal", 0, vec![CardType::Artifact], "Sacrifice Lotus Petal: Add one mana of any color."));
    mainboard.push(make_card("Mana Vault", 1, vec![CardType::Artifact], "{T}: Add {C}{C}{C}."));
    
    // Cantrips
    mainboard.push(make_card("Brainstorm", 1, vec![CardType::Instant], "Draw three cards..."));
    mainboard.push(make_card("Ponder", 1, vec![CardType::Sorcery], "Look at the top three..."));
    
    // Some engines (but not specifically storm ones)
    mainboard.push(make_card("Rhystic Study", 3, vec![CardType::Enchantment], "Whenever an opponent casts a spell..."));
    mainboard.push(make_card("Mystic Remora", 1, vec![CardType::Enchantment], "Whenever an opponent casts a noncreature spell..."));

    // Signal check: 
    // Dark(2) + Cabal(2) + Pyretic(2) + Desperate(2) + Lotus Petal(1.5) + Mana Vault(0.0 - Roles::FAST_MANA, but not SAC_MANA etc in storm)
    // Actually, FAST_MANA doesn't add to storm signal unless it's FAST_MANA_ONE_SHOT.
    // Lotus Petal is FAST_MANA_ONE_SHOT (+1.5).
    // Brainstorm is BURST_DRAW (+1.0).
    // MV<=1 cantrips (Dark, Brain, Ponder, Lotus) = 4 * 0.5 = 2.0.
    // Total = 2*4 + 1.5 + 1.0 + 2.0 = 8.0 + 4.5 = 12.5.
    // It should cross 12.0 signal, but without payoff it shouldn't be Storm.

    // Add lands
    for _ in 0..30 {
        mainboard.push(make_card("Island", 0, vec![CardType::Land], "{T}: Add {U}."));
    }

    let evaluation = calculate_crispi(&mainboard, &commanders, 0);

    println!("High Quality Test - Signal: {}", evaluation.storm_signal);
    println!("High Quality Test - Archetype: {:?}", evaluation.archetype);

    assert_ne!(evaluation.archetype, DeckArchetype::Storm, "Should not be Storm without a payoff.");
}

#[test]
fn test_gravestorm_payoff() {
    let commanders = vec![make_card("The Gitrog Monster", 5, vec![CardType::Creature], "")];
    let mut mainboard = vec![];
    
    // Gravestorm payoff
    mainboard.push(make_card("Bitter Ordeal", 3, vec![CardType::Sorcery], "Gravestorm. Search target player's library for a card and exile it."));
    
    // Rituals/Cantrips to get to 12.0 signal
    mainboard.push(make_card("Dark Ritual", 1, vec![CardType::Instant], "Add {B}{B}{B}."));
    mainboard.push(make_card("Cabal Ritual", 2, vec![CardType::Instant], "Add {B}{B}{B}."));
    mainboard.push(make_card("Ponder", 1, vec![CardType::Sorcery], "Look at the top three cards..."));
    mainboard.push(make_card("Lotus Petal", 0, vec![CardType::Artifact], "Sacrifice Lotus Petal: Add one mana of any color."));
    mainboard.push(make_card("Lion's Eye Diamond", 0, vec![CardType::Artifact], "Sacrifice LED: Add {B}{B}{B}."));
    mainboard.push(make_card("Rite of Flame", 1, vec![CardType::Sorcery], "Add {R}{R}."));

    // Signal:
    // Bitter Ordeal (Storm Tag + Payoff) = 3.0 + 2.0 = 5.0.
    // Rituals (Dark, Cabal, Rite) = 3 * 2.0 = 6.0.
    // Lotus Petal (Fast Mana One Shot) = 1.5.
    // Lion's Eye Diamond (Fast Mana One Shot) = 1.5.
    // Cantrips/Low MV (Bitter, Dark, Ponder, Lotus, LED, Rite) = 6 * 0.5 = 3.0.
    // Total = 5.0 + 6.0 + 1.5 + 1.5 + 3.0 = 17.0.
    
    for _ in 0..30 {
        mainboard.push(make_card("Swamp", 0, vec![CardType::Land], "{T}: Add {B}."));
    }

    let evaluation = calculate_crispi(&mainboard, &commanders, 0);

    println!("Gravestorm Test - Signal: {}", evaluation.storm_signal);
    println!("Gravestorm Test - Archetype: {:?}", evaluation.archetype);

    assert_eq!(evaluation.archetype, DeckArchetype::Storm, "Gravestorm should count as Storm.");
    assert!(evaluation.storm_signal >= 12.0);
}

#[test]
fn test_specific_storm_payoffs_requested_by_user() {
    // Veyran (vivi ornitier)
    let veyran = make_card("Veyran, Voice of Duality", 3, vec![CardType::Creature], "Whenever you cast or copy an instant or sorcery spell, Veyran, Voice of Duality gets +1/+1 until end of turn. If a triggered ability of a permanent you control triggers as a result of you casting or copying an instant or sorcery spell, that ability triggers an additional time.");
    let mut deck_v = vec![veyran];
    for _ in 0..10 { deck_v.push(make_card("Ritual", 1, vec![CardType::Instant], "Add {B}{B}{B}.")); }
    for _ in 0..30 { deck_v.push(make_card("Island", 0, vec![CardType::Land], "{T}: Add {U}.")); }
    let eval_v = calculate_crispi(&deck_v, &vec![], 0);
    assert_eq!(eval_v.archetype, DeckArchetype::Storm, "Veyran (Vivi) should satisfy payoff requirement.");

    // Ral Monsoon Mage
    let ral = make_card("Ral, Monsoon Mage", 2, vec![CardType::Creature], "Whenever you cast an instant or sorcery spell, Ral, Monsoon Mage deals 1 damage to you. Then flip a coin... (flip logic)");
    let mut deck_r = vec![ral];
    for _ in 0..10 { deck_r.push(make_card("Ritual", 1, vec![CardType::Instant], "Add {B}{B}{B}.")); }
    for _ in 0..30 { deck_r.push(make_card("Island", 0, vec![CardType::Land], "{T}: Add {U}.")); }
    let eval_r = calculate_crispi(&deck_r, &vec![], 0);
    assert_eq!(eval_r.archetype, DeckArchetype::Storm, "Ral Monsoon Mage should satisfy payoff requirement.");
}

#[test]
fn test_blue_farm_is_not_storm() {
    let commanders = vec![
        make_card("Tymna the Weaver", 3, vec![CardType::Creature], "Lifelink. At the beginning of your second main phase..."),
        make_card("Kraum, Ludevic's Opus", 5, vec![CardType::Creature], "Flying, haste. Whenever an opponent casts their second spell each turn, draw a card."),
    ];

    let mut mainboard = vec![];
    // Key Fast Mana / "Stormy" cards
    mainboard.push(make_card("Dark Ritual", 1, vec![CardType::Instant], "Add {B}{B}{B}."));
    mainboard.push(make_card("Lotus Petal", 0, vec![CardType::Artifact], "Sacrifice Lotus Petal: Add one mana of any color."));
    mainboard.push(make_card("Lion's Eye Diamond", 0, vec![CardType::Artifact], "Discard your hand, Sacrifice LED: Add {B}{B}{B}."));
    mainboard.push(make_card("Mana Vault", 1, vec![CardType::Artifact], "{T}: Add {C}{C}{C}."));
    mainboard.push(make_card("Chrome Mox", 0, vec![CardType::Artifact], "Imprint ... {T}: Add one mana of any of the exiled card's colors."));
    mainboard.push(make_card("Mox Diamond", 0, vec![CardType::Artifact], "Discard a land ... {T}: Add one mana of any color."));
    mainboard.push(make_card("Sol Ring", 1, vec![CardType::Artifact], "{T}: Add {C}{C}."));
    mainboard.push(make_card("Mana Crypt", 0, vec![CardType::Artifact], "{T}: Add {C}{C}."));

    // Cantrips & Selection
    mainboard.push(make_card("Brainstorm", 1, vec![CardType::Instant], "Draw three cards..."));
    mainboard.push(make_card("Ponder", 1, vec![CardType::Sorcery], "Look at the top three..."));
    mainboard.push(make_card("Preordain", 1, vec![CardType::Sorcery], "Scry 2, then draw a card."));
    mainboard.push(make_card("Gitaxian Probe", 1, vec![CardType::Sorcery], "Look at target player's hand, draw a card."));
    mainboard.push(make_card("Mystical Tutor", 1, vec![CardType::Instant], "Search your library for an instant or sorcery..."));

    // Tutors
    mainboard.push(make_card("Demonic Tutor", 2, vec![CardType::Sorcery], "Search your library for a card..."));
    mainboard.push(make_card("Vampiric Tutor", 1, vec![CardType::Instant], "Search your library for a card..."));

    // Protection / Interaction (Lots of cheap ones)
    mainboard.push(make_card("Mental Misstep", 1, vec![CardType::Instant], "Counter target spell with mana value 1."));
    mainboard.push(make_card("Silence", 1, vec![CardType::Instant], "Your opponents can't cast spells this turn."));
    mainboard.push(make_card("Chain of Vapor", 1, vec![CardType::Instant], "Return target nonland permanent to its owner's hand."));
    mainboard.push(make_card("Swan Song", 1, vec![CardType::Instant], "Counter target enchantment, instant, or sorcery spell."));
    mainboard.push(make_card("Flusterstorm", 1, vec![CardType::Instant], "Storm. Counter target instant or sorcery spell."));

    // Wincons
    mainboard.push(make_card("Ad Nauseam", 5, vec![CardType::Instant], "Reveal the top card..."));
    mainboard.push(make_card("Underworld Breach", 2, vec![CardType::Enchantment], "Each nonland card in your graveyard has escape."));
    mainboard.push(make_card("Brain Freeze", 2, vec![CardType::Instant], "Storm. Target player puts the top three cards..."));

    // Fill with some lands
    for _ in 0..28 {
        mainboard.push(make_card("City of Brass", 0, vec![CardType::Land], "{T}: Add one mana of any color."));
    }

    let evaluation = calculate_crispi(&mainboard, &commanders, 0);

    println!("Blue Farm Test - Storm Signal: {}", evaluation.storm_signal);
    println!("Blue Farm Test - Turbo Signal: {}", evaluation.turbo_signal);
    println!("Blue Farm Test - Midrange Signal: {}", evaluation.midrange_signal);
    println!("Blue Farm Test - Archetype: {:?}", evaluation.archetype);

    assert_ne!(evaluation.archetype, DeckArchetype::Storm, "Blue Farm should be Turbo or Midrange, not Storm.");
}

#[test]
fn test_proliferate_without_poison_support_does_not_count_as_infect() {
    let atraxa = make_card(
        "Atraxa, Praetors' Voice",
        4,
        vec![CardType::Creature],
        "Flying, vigilance, deathtouch, lifelink. At the beginning of your end step, proliferate.",
    );
    let commanders = vec![atraxa];

    let mut mainboard = vec![];
    mainboard.push(make_card(
        "Evolution Sage",
        3,
        vec![CardType::Creature],
        "Whenever a land enters the battlefield under your control, proliferate.",
    ));
    mainboard.push(make_card(
        "Flux Channeler",
        3,
        vec![CardType::Creature],
        "Whenever you cast a noncreature spell, proliferate.",
    ));
    mainboard.push(make_card("Grateful Apparition", 2, vec![CardType::Creature], "Flying. Whenever Grateful Apparition deals combat damage to a player or planeswalker, proliferate."));
    mainboard.push(make_card("Kami of Whispered Hopes", 3, vec![CardType::Creature], "If one or more +1/+1 counters would be put on a permanent you control, that many plus one +1/+1 counters are put on it instead. {T}: Add X mana of any one color, where X is Kami of Whispered Hopes's power."));
    mainboard.push(make_card("Hardened Scales", 1, vec![CardType::Enchantment], "If one or more +1/+1 counters would be put on a creature you control, that many plus one +1/+1 counters are put on it instead."));
    mainboard.push(make_card("Conclave Mentor", 2, vec![CardType::Creature], "If one or more +1/+1 counters would be put on a creature you control, that many plus one +1/+1 counters are put on it instead."));

    for _ in 0..30 {
        mainboard.push(make_card(
            "Forest",
            0,
            vec![CardType::Land],
            "{T}: Add {G}.",
        ));
    }

    let evaluation = calculate_crispi(&mainboard, &commanders, 0);

    assert_eq!(
        evaluation.infect_signal, 0.0,
        "Proliferate without poison support should not create infect signal."
    );
    assert_ne!(evaluation.archetype, DeckArchetype::Infect);
}

#[test]
fn test_three_card_combo() {
    let mut mainboard = vec![];
    mainboard.push(make_card(
        "Underworld Breach",
        2,
        vec![CardType::Enchantment],
        "Each nonland card in your graveyard has escape.",
    ));
    mainboard.push(make_card("Lion's Eye Diamond", 0, vec![CardType::Artifact], "{T}, Discard your hand, Sacrifice Lion's Eye Diamond: Add three mana of any one color. Activate only as a mana ability."));
    mainboard.push(make_card(
        "Brain Freeze",
        2,
        vec![CardType::Instant],
        "Storm. Target player mills three cards.",
    ));

    // Add enough non-land cards to meet prereqs (prereqs: 3 for this combo)
    // logic: if non_land_count < (5 + combo.prereqs) { met = false; }
    // 5 + 3 = 8.
    for i in 0..5 {
        mainboard.push(make_card(
            &format!("Filler {}", i),
            1,
            vec![CardType::Creature],
            "",
        ));
    }

    let commanders = vec![];
    let evaluation = calculate_crispi(&mainboard, &commanders, 0);

    println!(
        "Three-Card Combo Test - Detected: {:?}",
        evaluation.detected_combos
    );
    assert!(evaluation
        .detected_combos
        .iter()
        .any(|c| c.contains("Underworld Breach")
            && c.contains("Lion's Eye Diamond")
            && c.contains("Brain Freeze")));
    assert!(evaluation.combo_multiplier > 1.0);
}

#[test]
fn test_combo_pieces_are_promoted_to_wincon_role() {
    let mainboard = vec![
        make_card(
            "Demonic Consultation",
            1,
            vec![CardType::Instant],
            "Choose a card name. Exile the top six cards of your library, then reveal cards from the top of your library until you reveal the chosen card. Put that card into your hand and exile all other cards revealed this way.",
        ),
        make_card(
            "Thassa's Oracle",
            2,
            vec![CardType::Creature],
            "When Thassa's Oracle enters the battlefield, look at the top X cards of your library, where X is your devotion to blue. Put up to one of them on top of your library and the rest on the bottom of your library in a random order. If X is greater than or equal to the number of cards in your library, you win the game.",
        ),
    ];
    let commanders = vec![];
    let combo_piece_names = combo_piece_names_for_deck(&mainboard, &commanders);

    let oracle_roles = infer_roles_with_combo_context(&mainboard[1], &combo_piece_names);
    let consultation_roles = infer_roles_with_combo_context(&mainboard[0], &combo_piece_names);

    assert!(oracle_roles.contains(&Role::WINCON));
    assert!(consultation_roles.contains(&Role::WINCON));
}

#[test]
fn test_hoarding_broodlord_combo() {
    let mut mainboard = vec![];
    mainboard.push(make_card("Hoarding Broodlord", 8, vec![CardType::Creature], "Convoke. When Hoarding Broodlord enters the battlefield, search your library for a card, exile it face down, then shuffle. For as long as that card remains exiled, you may play it. Spells you cast from exile have convoke."));
    mainboard.push(make_card("Saw in Half", 3, vec![CardType::Instant], "Destroy target creature. If that creature is destroyed this way, its controller creates two tokens that are copies of it, except their base power is half that creature's base power and their base toughness is half that creature's base toughness, each rounded up."));
    mainboard.push(make_card("Sacrifice", 1, vec![CardType::Instant], "As an additional cost to cast Sacrifice, sacrifice a creature. Add an amount of {B} equal to the sacrificed creature's mana value."));
    mainboard.push(make_card("Burnt Offering", 1, vec![CardType::Instant], "As an additional cost to cast Burnt Offering, sacrifice a creature. Add an amount of {B} or {R} equal to the sacrificed creature's mana value."));
    mainboard.push(make_card(
        "Culling the Weak",
        1,
        vec![CardType::Instant],
        "As an additional cost to cast Culling the Weak, sacrifice a creature. Add {B}{B}{B}{B}.",
    ));

    // Prereqs: 3. 5 + 3 = 8.
    // We have 5 cards already. We need 3 more to have 8 total.
    for i in 0..3 {
        mainboard.push(make_card(
            &format!("Filler {}", i),
            1,
            vec![CardType::Creature],
            "",
        ));
    }

    let commanders = vec![];
    let evaluation = calculate_crispi(&mainboard, &commanders, 0);

    println!(
        "Hoarding Broodlord Test (Mainboard, 8 cards, Prereqs OK) - Detected: {:?}",
        evaluation.detected_combos
    );
    assert!(evaluation
        .detected_combos
        .iter()
        .any(|c| c.contains("Hoarding Broodlord")
            && c.contains("Saw in Half")
            && c.contains("Sacrifice")));
    assert!(evaluation
        .detected_combos
        .iter()
        .any(|c| c.contains("Hoarding Broodlord")
            && c.contains("Saw in Half")
            && c.contains("Burnt Offering")));
    assert!(evaluation
        .detected_combos
        .iter()
        .any(|c| c.contains("Hoarding Broodlord")
            && c.contains("Saw in Half")
            && c.contains("Culling the Weak")));

    // Now with Hoarding Broodlord as Commander
    let hb = mainboard.remove(0);
    let commanders_hb = vec![hb];
    let evaluation_cmdr = calculate_crispi(&mainboard, &commanders_hb, 0);
    println!(
        "Hoarding Broodlord Test (Commander) - Detected: {:?}",
        evaluation_cmdr.detected_combos
    );
    assert!(evaluation_cmdr
        .detected_combos
        .iter()
        .any(|c| c.contains("Hoarding Broodlord")
            && c.contains("Saw in Half")
            && c.contains("Sacrifice")));
}

#[test]
fn test_cormela_deck_score_investigation() {
    let commanders = vec![make_card(
        "Cormela, Glamour Thief",
        4,
        vec![CardType::Creature],
        "Haste. {T}: Add {U}, {B}, or {R}. Spend this mana only to cast instant or sorcery spells. When Cormela dies, return up to one target instant or sorcery card from your graveyard to your hand.",
    )];

    let mut mainboard = vec![];
    let mut push = |name: &str, mv: u8, types: Vec<CardType>, text: &str| {
        mainboard.push(make_card(name, mv, types, text));
    };

    push("Birgi, God of Storytelling", 3, vec![CardType::Creature], "Whenever you cast a spell, add {R}. Until end of turn, you don't lose this mana as steps and phases end.");
    push("Brash Taunter", 5, vec![CardType::Creature], "Indestructible. Whenever Brash Taunter is dealt damage, it deals that much damage to target opponent.");
    push("Dualcaster Mage", 3, vec![CardType::Creature], "Flash. When Dualcaster Mage enters the battlefield, copy target instant or sorcery spell. You may choose new targets for the copy.");
    push("Erebor Flamesmith", 2, vec![CardType::Creature], "Whenever you cast an instant or sorcery spell, Erebor Flamesmith deals 1 damage to each opponent.");
    push("Faerie Mastermind", 2, vec![CardType::Creature], "Flash. Flying. Whenever an opponent draws their second card each turn, you draw a card.");
    push("Firebrand Archer", 2, vec![CardType::Creature], "Whenever you cast a noncreature spell, Firebrand Archer deals 1 damage to each opponent.");
    push("Geyser Drake", 4, vec![CardType::Creature], "Flying. Whenever you cast an instant or sorcery spell, Geyser Drake gets +1/+0 until end of turn.");
    push("Goblin Electromancer", 2, vec![CardType::Creature], "Instant and sorcery spells you cast cost {1} less to cast.");
    push("Guttersnipe", 3, vec![CardType::Creature], "Whenever you cast an instant or sorcery spell, Guttersnipe deals 2 damage to each opponent.");
    push("Hexing Squelcher", 2, vec![CardType::Creature], "");
    push("Nightscape Familiar", 2, vec![CardType::Creature], "Blue spells and red spells you cast cost {1} less to cast.");
    push("Pestilent Spirit", 3, vec![CardType::Creature], "Instant and sorcery spells you control have deathtouch.");
    push("Razorkin Needlehead", 2, vec![CardType::Creature], "");
    push("Silver Myr", 2, vec![CardType::Artifact, CardType::Creature], "{T}: Add {U}.");
    push("Storm-Kiln Artist", 4, vec![CardType::Creature], "Magecraft - Whenever you cast or copy an instant or sorcery spell, create a Treasure token.");
    push("Syr Konrad, the Grim", 5, vec![CardType::Creature], "Whenever another creature dies, Syr Konrad, the Grim deals 1 damage to each opponent.");
    push("Thassa's Oracle", 2, vec![CardType::Creature], "When Thassa's Oracle enters the battlefield, look at the top X cards of your library, where X is your devotion to blue. If X is greater than or equal to the number of cards in your library, you win the game.");
    push("The Warring Triad", 3, vec![CardType::Creature], "");
    push("Third Path Iconoclast", 2, vec![CardType::Creature], "Whenever you cast a noncreature spell, create a 1/1 Soldier artifact creature token.");
    push("Unstoppable Slasher", 3, vec![CardType::Creature], "");
    push("Valley Floodcaller", 2, vec![CardType::Creature], "");

    push("Lotus Petal", 0, vec![CardType::Artifact], "Sacrifice Lotus Petal: Add one mana of any color.");
    push("Sol Ring", 1, vec![CardType::Artifact], "{T}: Add {C}{C}.");

    push("Dictate of the Twin Gods", 5, vec![CardType::Enchantment], "Flash. If a source would deal damage to a permanent or player, it deals double that damage to that permanent or player instead.");
    push("Fiery Inscription", 3, vec![CardType::Enchantment], "Whenever you cast an instant or sorcery spell, Fiery Inscription deals 2 damage to each opponent.");
    push("Goblin Bombardment", 2, vec![CardType::Enchantment], "Sacrifice a creature: Goblin Bombardment deals 1 damage to any target.");
    push("Mystic Remora", 1, vec![CardType::Enchantment], "Cumulative upkeep {1}. Whenever an opponent casts a noncreature spell, you may draw a card unless that player pays {4}.");

    push("Amazing Acrobatics", 5, vec![CardType::Instant], "");
    push("Borne Upon a Wind", 2, vec![CardType::Instant], "You may cast spells as though they had flash this turn. Draw a card.");
    push("Brainstorm", 1, vec![CardType::Instant], "Draw three cards, then put two cards from your hand on top of your library in any order.");
    push("Cabal Ritual", 2, vec![CardType::Instant], "Add {B}{B}{B}. Threshold - Add {B}{B}{B}{B}{B} instead.");
    push("Counterspell", 2, vec![CardType::Instant], "Counter target spell.");
    push("Dark Ritual", 1, vec![CardType::Instant], "Add {B}{B}{B}.");
    push("Deflecting Swat", 3, vec![CardType::Instant], "If you control a commander, you may cast this spell without paying its mana cost. You may choose new targets for target spell or ability.");
    push("Demonic Consultation", 1, vec![CardType::Instant], "Choose a card name. Exile the top six cards of your library, then reveal cards from the top of your library until you reveal the chosen card. Put that card into your hand and exile all other cards revealed this way.");
    push("Fake Your Own Death", 2, vec![CardType::Instant], "Until end of turn, target creature gets +2/+0 and gains \"When this creature dies, return it to the battlefield tapped under its owner's control and create a Treasure token.\"");
    push("Flare of Duplication", 3, vec![CardType::Instant], "You may sacrifice a nontoken red creature rather than pay this spell's mana cost. Copy target instant or sorcery spell you control. You may choose new targets for the copy.");
    push("Flusterstorm", 1, vec![CardType::Instant], "Counter target instant or sorcery spell unless its controller pays {1}. Storm.");
    push("Frantic Search", 3, vec![CardType::Instant], "Draw two cards, then discard two cards. Untap up to three lands.");
    push("Get Out", 2, vec![CardType::Instant], "");
    push("Lightning Bolt", 1, vec![CardType::Instant], "Lightning Bolt deals 3 damage to any target.");
    push("Mana Leak", 2, vec![CardType::Instant], "Counter target spell unless its controller pays {3}.");
    push("Negate", 2, vec![CardType::Instant], "Counter target noncreature spell.");
    push("Not Dead After All", 1, vec![CardType::Instant], "Until end of turn, target creature gains \"When this creature dies, return it to the battlefield tapped under its owner's control with a wicked role token attached to it.\"");
    push("Otherworldly Gaze", 1, vec![CardType::Instant], "Surveil 3. Flashback {1}{U}.");
    push("Pain 101", 2, vec![CardType::Instant], "");
    push("Psychic Strike", 3, vec![CardType::Instant], "Counter target spell. Its controller mills two cards.");
    push("Run Away Together", 2, vec![CardType::Instant], "Choose two target creatures controlled by different players. Return those creatures to their owners' hands.");
    push("Sinister Sabotage", 3, vec![CardType::Instant], "Counter target spell. Surveil 1.");
    push("Stoic Rebuttal", 3, vec![CardType::Instant], "Metalcraft - This spell costs {1} less to cast if you control three or more artifacts. Counter target spell.");
    push("Supernatural Stamina", 1, vec![CardType::Instant], "Until end of turn, target creature gets +2/+0 and gains \"When this creature dies, return it to the battlefield tapped under its owner's control.\"");

    push("Blasphemous Act", 9, vec![CardType::Sorcery], "This spell costs {1} less to cast for each creature on the battlefield. Blasphemous Act deals 13 damage to each creature.");
    push("Freeze in Place", 3, vec![CardType::Sorcery], "");
    push("Grapeshot", 2, vec![CardType::Sorcery], "Grapeshot deals 1 damage to any target. Storm.");
    push("Impede Momentum", 3, vec![CardType::Sorcery], "");
    push("Lava Spike", 1, vec![CardType::Sorcery], "Lava Spike deals 3 damage to target player or planeswalker.");
    push("Lively Dirge", 3, vec![CardType::Sorcery], "");
    push("Profane Tutor", 0, vec![CardType::Sorcery], "Suspend 2 - {1}{B}. Search your library for a card, put that card into your hand, then shuffle.");
    push("Ranger's Firebrand", 2, vec![CardType::Sorcery], "");
    push("Reanimate", 1, vec![CardType::Sorcery], "Put target creature card from a graveyard onto the battlefield under your control. You lose life equal to its mana value.");
    push("Rite of Flame", 1, vec![CardType::Sorcery], "Add {R}{R}, then add {R} for each card named Rite of Flame in each graveyard.");
    push("Singularity Rupture", 2, vec![CardType::Sorcery], "");
    push("Slip Through Space", 1, vec![CardType::Sorcery], "Target creature can't be blocked this turn. Draw a card.");
    push("Thundering Rebuke", 2, vec![CardType::Sorcery], "Thundering Rebuke deals 4 damage to target creature or planeswalker.");
    push("Twinflame", 2, vec![CardType::Sorcery], "Strive. Choose any number of target creatures. For each of them, create a token that's a copy of that creature. That token gains haste. Exile it at the beginning of the next end step.");

    push("Badlands", 0, vec![CardType::Land], "{T}: Add {B} or {R}.");
    push("Command Tower", 0, vec![CardType::Land], "{T}: Add one mana of any color in your commander's color identity.");
    for _ in 0..10 {
        push("Island", 0, vec![CardType::Land], "{T}: Add {U}.");
    }
    for _ in 0..10 {
        push("Mountain", 0, vec![CardType::Land], "{T}: Add {R}.");
    }
    for _ in 0..10 {
        push("Swamp", 0, vec![CardType::Land], "{T}: Add {B}.");
    }
    push("Underground Sea", 0, vec![CardType::Land], "{T}: Add {U} or {B}.");
    push("Volcanic Island", 0, vec![CardType::Land], "{T}: Add {U} or {R}.");

    let evaluation = calculate_crispi(&mainboard, &commanders, 0);

    println!("Cormela Investigation - Total Score: {}", evaluation.total_score);
    println!("Cormela Investigation - Raw Score: {}", evaluation.raw_score);
    println!("Cormela Investigation - Interpretation: {}", evaluation.interpretation);
    println!("Cormela Investigation - Bracket: {}", evaluation.bracket);
    println!("Cormela Investigation - Archetype: {:?}", evaluation.archetype);
    println!(
        "Cormela Investigation - C/R/I/S/P: {}/{}/{}/{}/{}",
        evaluation.consistency.score,
        evaluation.resilience.score,
        evaluation.interaction.score,
        evaluation.speed.score,
        evaluation.pivotability.score
    );
    println!(
        "Cormela Investigation - Multipliers: amv={} combo={} final={}",
        evaluation.amv_multiplier, evaluation.combo_multiplier, evaluation.final_multiplier
    );
    println!(
        "Cormela Investigation - Signals: turbo={} midrange={} stax={} cmdr={} group_hug={} infect={}",
        evaluation.turbo_signal,
        evaluation.midrange_signal,
        evaluation.stax_signal,
        evaluation.commander_engine_signal,
        evaluation.group_hug_signal,
        evaluation.infect_signal
    );
    println!(
        "Cormela Investigation - Justifications:\nC: {}\nR: {}\nI: {}\nS: {}\nP: {}",
        evaluation.consistency.justification,
        evaluation.resilience.justification,
        evaluation.interaction.justification,
        evaluation.speed.justification,
        evaluation.pivotability.justification
    );
    println!(
        "Cormela Investigation - Detected Combos: {:?}",
        evaluation.detected_combos
    );

    assert!(evaluation.total_score < 15.0);
    assert_eq!(evaluation.bracket, 4);
    assert_eq!(evaluation.interpretation, "Focused / Synergistic");
    assert!(evaluation.consistency.score <= 2);
}

#[test]
fn test_gitrog_is_not_storm() {
    let mut mainboard = Vec::new();
    let mut commanders = Vec::new();

    let push = |name: &str, mv: u8, types: Vec<CardType>, text: &str, mb: &mut Vec<Card>| {
        mb.push(make_card(name, mv, types, text));
    };

    // Gitrog Combo Pieces (Non-Storm)
    push("The Gitrog Monster", 5, vec![CardType::Creature], "Whenever a land card is put into your graveyard from anywhere, draw a card.", &mut commanders);
    push("Dakmor Salvage", 0, vec![CardType::Land], "Dredge 2.", &mut mainboard);
    push("Putrid Imp", 1, vec![CardType::Creature], "Discard a card: Putrid Imp gains flying until end of turn.", &mut mainboard);
    push("Noose Constrictor", 2, vec![CardType::Creature], "Discard a card: Noose Constrictor gets +1/+1 until end of turn.", &mut mainboard);
    push("Wild Mongrel", 2, vec![CardType::Creature], "Discard a card: Wild Mongrel gets +1/+1 and becomes the color of your choice until end of turn.", &mut mainboard);

    // Storm-like combo pieces (Witherbloom + Smog)
    push("Witherbloom Apprentice", 2, vec![CardType::Creature], "Magecraft - Whenever you cast or copy an instant or sorcery spell, each opponent loses 1 life and you gain 1 life.", &mut mainboard);
    push("Chain of Smog", 2, vec![CardType::Sorcery], "Target player discards two cards. Then that player may copy this spell and may choose a new target for that copy.", &mut mainboard);

    // High density of quality cards to boost storm signal (Rituals + Tutors)
    push("Dark Ritual", 1, vec![CardType::Instant], "Add {B}{B}{B}.", &mut mainboard);
    push("Cabal Ritual", 2, vec![CardType::Instant], "Add {B}{B}{B}.", &mut mainboard);
    push("Lotus Petal", 0, vec![CardType::Artifact], "Sacrifice Lotus Petal: Add one mana of any color.", &mut mainboard);
    push("Entomb", 1, vec![CardType::Instant], "Search your library for a card and put it into your graveyard.", &mut mainboard);
    push("Vampiric Tutor", 1, vec![CardType::Instant], "Search your library for a card, then shuffle and put that card on top.", &mut mainboard);
    push("Demonic Tutor", 2, vec![CardType::Sorcery], "Search your library for a card, put it into your hand.", &mut mainboard);

    // Fill with basics
    for i in 0..20 {
        push(&format!("Swamp {}", i), 0, vec![CardType::Land], "{T}: Add {B}.", &mut mainboard);
    }
    for i in 0..20 {
        push(&format!("Forest {}", i), 0, vec![CardType::Land], "{T}: Add {G}.", &mut mainboard);
    }

    let evaluation = calculate_crispi(&mainboard, &commanders, 0);

    println!("Gitrog Archetype: {:?}", evaluation.archetype);
    println!("Storm Signal: {}", evaluation.storm_signal);
    println!("Detected Combos: {:?}", evaluation.detected_combos);

    // It should have 3 non-storm combos and 1 storm-related combo.
    // 3 > 1, so it should not be Storm.
    assert_ne!(evaluation.archetype, DeckArchetype::Storm, "Gitrog deck with many non-storm combos should not be classified as Storm");
}

#[test]
fn test_high_storm_signal_above_50_is_storm() {
    // Test the new feature: if storm_signal >= 50, it should be Storm
    // even without has_storm_payoff or other conditions
    let archetype = detect_archetype(0.0, 0.0, 10.0, 5.0, 0.0, 0.0, 0.0, 50.0, false, 2, 0, 0);
    assert_eq!(
        archetype, DeckArchetype::Storm,
        "Deck with storm_signal >= 50 should be classified as Storm"
    );

    // Test edge case: storm_signal just barely above 50
    let archetype = detect_archetype(0.0, 0.0, 10.0, 5.0, 0.0, 0.0, 0.0, 50.1, false, 2, 0, 0);
    assert_eq!(
        archetype, DeckArchetype::Storm,
        "Deck with storm_signal > 50 should be classified as Storm"
    );

    // Test edge case: storm_signal just below 50 (should not automatically be Storm)
    let archetype = detect_archetype(0.0, 0.0, 10.0, 5.0, 0.0, 0.0, 0.0, 49.9, false, 2, 0, 0);
    assert_ne!(
        archetype, DeckArchetype::Storm,
        "Deck with storm_signal < 50 should not automatically be Storm without payoff"
    );
}
