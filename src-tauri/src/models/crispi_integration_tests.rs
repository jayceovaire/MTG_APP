use crate::models::card_model::{Card, CardType};
use crate::models::crispi_integration::{compute_integration, tutor_can_find};
use crate::models::crispi_patterns::infer_roles;
use crate::models::crispi_types::Role;

fn make_card(name: &str, mv: u8, types: Vec<CardType>, text: &str) -> Card {
    Card::new(
        0,
        "".to_string(),
        None,
        name.to_string(),
        None,
        mv,
        types,
        vec![],
        vec![],
        Some(text.to_string()),
        "legal".to_string(),
        true,
        false,
        None,
    )
}

#[test]
fn test_universal_tutor_integration() {
    let demonic_tutor = make_card(
        "Demonic Tutor",
        2,
        vec![CardType::Sorcery],
        "Search your library for a card and put it into your hand. Then shuffle."
    );
    let llanowar_elves = make_card(
        "Llanowar Elves",
        1,
        vec![CardType::Creature],
        "{T}: Add {G}."
    );

    let dt_roles = infer_roles(&demonic_tutor);
    assert!(dt_roles.contains(&Role::TUTOR), "Demonic Tutor should be a TUTOR. Roles: {:?}", dt_roles);

    let can_find = tutor_can_find(&demonic_tutor, &dt_roles, &llanowar_elves);
    assert!(can_find, "Universal tutor should find any card (Llanowar Elves)");
}

#[test]
fn test_specific_tutor_integration() {
    let worldly_tutor = make_card(
        "Worldly Tutor",
        1,
        vec![CardType::Instant],
        "Search your library for a creature card, reveal it, then shuffle and put it on top."
    );
    let llanowar_elves = make_card(
        "Llanowar Elves",
        1,
        vec![CardType::Creature],
        "{T}: Add {G}."
    );
    let sol_ring = make_card(
        "Sol Ring",
        1,
        vec![CardType::Artifact],
        "{T}: Add {C}{C}."
    );

    let wt_roles = infer_roles(&worldly_tutor);
    assert!(wt_roles.contains(&Role::TUTOR), "Worldly Tutor should be a TUTOR");

    assert!(tutor_can_find(&worldly_tutor, &wt_roles, &llanowar_elves), "Worldly Tutor should find a creature");
    assert!(!tutor_can_find(&worldly_tutor, &wt_roles, &sol_ring), "Worldly Tutor should NOT find an artifact");
}

#[test]
fn test_integration_normalization() {
    let dt = make_card("Demonic Tutor", 2, vec![CardType::Sorcery], "Search your library for a card");
    let card1 = make_card("Card 1", 1, vec![CardType::Creature], "Text");
    let card2 = make_card("Card 2", 1, vec![CardType::Creature], "Text");
    
    let cards = vec![dt.clone(), card1.clone(), card2.clone()];
    let heats = compute_integration(&cards, &[]);
    
    let dt_heat = heats.iter().find(|h| h.name == "Demonic Tutor").unwrap();
    // DT finds card1 (+1) and card2 (+1) = 2.0 points
    // card1 finds nothing = 0.0 points
    // card2 finds nothing = 0.0 points
    // max is 2.0. DT integration should be 1.0 (100%)
    
    assert_eq!(dt_heat.integration, 1.0, "Demonic Tutor should have 100% integration");
}
