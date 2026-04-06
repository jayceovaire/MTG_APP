use super::*;
use crate::models::card_model::{Card, CardType};
use crate::models::crispi_archetypes::detect_archetype;

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
fn test_turbo_requires_natural_speed_five() {
    let archetype = detect_archetype(0.0, 0.0, 19.0, 8.0, 0.0, 0.0, 0.0, 4);
    assert_eq!(archetype, DeckArchetype::Midrange);

    let archetype = detect_archetype(0.0, 0.0, 19.0, 8.0, 0.0, 0.0, 0.0, 5);
    assert_eq!(archetype, DeckArchetype::Turbo);
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
