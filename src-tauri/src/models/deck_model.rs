use crate::models::card_model::Card;
use crate::models::crispi_model::{self, Role};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deck {
    id: u64,
    name: String,
    commander: CommanderSelection,
    cards: Vec<Card>,
    /// Number of cards with the Commander "game changer" designation (mainboard + command zone).
    #[serde(default)]
    game_changer_count: u32,
    #[serde(default)]
    bracket: u8,
    #[serde(default)]
    illegal_count: u32,
    mana_value: i16,
    mana_pips: i16,
    black_pips: i16,
    blue_pips: i16,
    white_pips: i16,
    green_pips: i16,
    red_pips: i16,
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommanderSelection {
    None,
    Single(Card),
    Partner(Card, Card),
}


impl Deck {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            name: "New Deck".to_string(),
            commander: CommanderSelection::None,
            cards: Vec::new(),
            game_changer_count: 0,
            bracket: 2,
            illegal_count: 0,
            mana_value: 0,
            mana_pips: 0,
            black_pips: 0,
            blue_pips: 0,
            white_pips: 0,
            green_pips: 0,
            red_pips: 0,
        }
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    pub fn add_commander(&mut self, card: Card) -> Result<(), String> {
        match &mut self.commander {
            CommanderSelection::None => {
                self.commander = CommanderSelection::Single(card);
                Ok(())
            }
            CommanderSelection::Single(existing) => {
                // Here you could check if both have Partner
                let first = existing.clone();
                self.commander = CommanderSelection::Partner(first, card);
                Ok(())
            }
            CommanderSelection::Partner(_, _) => {
                Err("Deck already has two commanders".into())
            }
        }
    }

    pub fn get_cards_mut(&mut self) -> &mut Vec<Card> {
        &mut self.cards
    }

    pub fn get_commander_mut(&mut self) -> &mut CommanderSelection {
        &mut self.commander
    }

    pub fn get_cards(&self) -> &[Card] {
        &self.cards
    }
    pub fn add_card(&mut self, card: Card ) {
        self.cards.push(card);

    }
    pub fn remove_card(&mut self, card: &Card) {
        if let Some(index) = self.cards.iter().position(|c| c == card) {
            self.cards.remove(index);
        }
    }

    pub fn remove_card_at(&mut self, index: usize) -> bool {
        if index < self.cards.len() {
            self.cards.remove(index);
            true
        } else {
            false
        }
    }

    pub fn get_commander(&self) -> &CommanderSelection {
        &self.commander
    }

    pub fn recount_game_changers(&mut self) {
        let mut n_gc: u32 = 0;
        let mut n_illegal: u32 = 0;
        let mut role_counts = HashMap::new();
        let mut total_mv = 0.0;
        let mut non_land_count = 0;
        
        let mut process_card = |c: &Card| {
            if c.is_game_changer() {
                n_gc += 1;
            }
            if !c.is_legal_in_commander() {
                n_illegal += 1;
            }
            
            let roles = crispi_model::infer_roles(c);
            for role in roles {
                *role_counts.entry(role).or_insert(0) += 1;
            }
            
            if !c.is_land() {
                total_mv += c.mana_value() as f32;
                non_land_count += 1;
            }
        };

        for c in &self.cards {
            process_card(c);
        }
        match &self.commander {
            CommanderSelection::None => {}
            CommanderSelection::Single(c) => process_card(c),
            CommanderSelection::Partner(a, b) => {
                process_card(a);
                process_card(b);
            }
        }

        self.game_changer_count = n_gc;
        self.illegal_count = n_illegal;

        let mut bracket = if n_gc == 0 {
            2
        } else if n_gc <= 3 {
            3
        } else {
            4
        };

        let amv = if non_land_count > 0 { total_mv / non_land_count as f32 } else { 0.0 };
        let crispi = crispi_model::calculate_crispi(&role_counts, amv);

        // Additional Bracket Rules
        if bracket == 2 && crispi.total_score <= 8.0 && amv > 3.5 {
            bracket = 1;
        } else if crispi.total_score >= 24.0 {
            bracket = 5;
        }

        self.bracket = bracket;
    }

    pub fn set_single_commander_from_deck(&mut self, card_id: u64) -> Result<(), String> {
        let card_index = self
            .cards
            .iter()
            .position(|card| card.id() == card_id)
            .ok_or_else(|| format!("Card with id {} not found in deck", card_id))?;

        let selected_card = self.cards.remove(card_index);
        if !selected_card.can_be_commander() {
            self.cards.insert(card_index, selected_card);
            return Err("Selected card cannot be a commander".to_string());
        }

        let previous_commander = std::mem::replace(&mut self.commander, CommanderSelection::None);
        match previous_commander {
            CommanderSelection::None => {}
            CommanderSelection::Single(card) => self.cards.push(card),
            CommanderSelection::Partner(first, second) => {
                self.cards.push(first);
                self.cards.push(second);
            }
        }

        self.commander = CommanderSelection::Single(selected_card);
        Ok(())
    }

    pub fn set_partner_commander_from_deck(&mut self, card_id: u64) -> Result<(), String> {
        let existing_commander = match &self.commander {
            CommanderSelection::Single(card) => card.clone(),
            CommanderSelection::None => return Err("Deck must already have a commander before adding a partner".to_string()),
            CommanderSelection::Partner(_, _) => return Err("Deck already has two commanders".to_string()),
        };

        if !existing_commander.can_be_commander() || !existing_commander.has_partner_mechanic() {
            return Err("Current commander cannot have a partner".to_string());
        }

        let card_index = self
            .cards
            .iter()
            .position(|card| card.id() == card_id)
            .ok_or_else(|| format!("Card with id {} not found in deck", card_id))?;

        let selected_card = self.cards.remove(card_index);
        
        // Validation:
        // Both cards must be legal commanders.
        // Both cards must have a partner mechanic.
        // Special case: Backgrounds only work if the other card has "Choose a Background".
        // Special case: "Partner with X" only works if the other card is X.
        // Special case: "Doctor's companion" only works if the other card is "The Doctor".
        // For now, we'll allow any two cards with ANY partner mechanic to be paired,
        // which covers 99% of cases and Rograkh specifically.
        
        if !selected_card.can_be_commander() {
            self.cards.insert(card_index, selected_card);
            return Err("Selected card cannot be a commander".to_string());
        }

        if !selected_card.has_partner_mechanic() {
            self.cards.insert(card_index, selected_card);
            return Err("Selected card does not have partner mechanic".to_string());
        }

        self.commander = CommanderSelection::Partner(existing_commander, selected_card);
        Ok(())
    }

    pub fn remove_partner_commander_from_deck(&mut self, card_id: u64) -> Result<(), String> {
        let previous_commander = std::mem::replace(&mut self.commander, CommanderSelection::None);

        match previous_commander {
            CommanderSelection::Partner(first, second) => {
                if first.id() == card_id {
                    self.cards.push(first);
                    self.commander = CommanderSelection::Single(second);
                    Ok(())
                } else if second.id() == card_id {
                    self.cards.push(second);
                    self.commander = CommanderSelection::Single(first);
                    Ok(())
                } else {
                    self.commander = CommanderSelection::Partner(first,second);
                    Err(format!("Card with id {} is not a partner commander", card_id))
                }
            }
            other => {
                self.commander = other;
                Err("Deck does not have partner commanders".to_string())
            }
        }
    }



    pub fn clear_commander_to_deck(&mut self) -> Result<(), String> {
        let previous_commander = std::mem::replace(&mut self.commander, CommanderSelection::None);

        match previous_commander {
            CommanderSelection::None => Err("Deck does not have a commander".to_string()),
            CommanderSelection::Single(card) => {
                self.cards.push(card);
                Ok(())
            }
            CommanderSelection::Partner(first, second) => {
                self.cards.push(first);
                self.cards.push(second);
                Ok(())
            }
        }
    }

    pub fn remove_commander(&mut self) -> Result<(), String> {
        let previous_commander = std::mem::replace(&mut self.commander, CommanderSelection::None);

        match previous_commander {
            CommanderSelection::None => Err("Deck does not have a commander".to_string()),
            CommanderSelection::Single(_) => Ok(()),
            CommanderSelection::Partner(_, _) => Ok(()),
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::card_model::{CardType, SuperType};

    #[test]
    fn test_partner_commanders() {
        let mut deck = Deck::new(1);
        
        // Rograkh, Son of Rohgahh (Legendary Creature Kobold)
        let rograkh = Card::new(
            1,
            "".to_string(),
            None,
            "Rograkh, Son of Rohgahh".to_string(),
            Some("{0}".to_string()),
            0,
            vec![CardType::Creature],
            vec![SuperType::Legendary],
            vec!["Kobold".to_string(), "Warrior".to_string()],
            Some("First strike, menace, trample\nPartner".to_string()),
            "legal".to_string(),
            true,
            true,
            None,
        );

        // Ardenn, Intrepid Archaeologist (Legendary Creature Kor Scout)
        let ardenn = Card::new(
            2,
            "".to_string(),
            None,
            "Ardenn, Intrepid Archaeologist".to_string(),
            Some("{2}{W}".to_string()),
            3,
            vec![CardType::Creature],
            vec![SuperType::Legendary],
            vec!["Kor".to_string(), "Scout".to_string()],
            Some("Partner".to_string()),
            "legal".to_string(),
            true,
            true,
            None,
        );

        deck.add_card(rograkh.clone());
        deck.add_card(ardenn.clone());

        // Set Rograkh as commander
        deck.set_single_commander_from_deck(1).expect("Failed to set Rograkh as commander");
        
        // Try to set Ardenn as partner
        deck.set_partner_commander_from_deck(2).expect("Failed to set Ardenn as partner");

        if let CommanderSelection::Partner(c1, c2) = deck.get_commander() {
            assert_eq!(c1.get_name(), "Rograkh, Son of Rohgahh");
            assert_eq!(c2.get_name(), "Ardenn, Intrepid Archaeologist");
        } else {
            panic!("Commander selection is not Partner");
        }
    }

    #[test]
    fn test_background_commander() {
        let mut deck = Deck::new(2);
        
        // Burakos, Party Leader (Choose a Background)
        let burakos = Card::new(
            1,
            "".to_string(),
            None,
            "Burakos, Party Leader".to_string(),
            Some("{3}{B}".to_string()),
            4,
            vec![CardType::Creature],
            vec![SuperType::Legendary],
            vec!["Orc".to_string(), "Wizard".to_string(), "Warrior".to_string(), "Cleric".to_string(), "Rogue".to_string()],
            Some("Choose a Background".to_string()),
            "legal".to_string(),
            true,
            true,
            None,
        );

        // Folk Hero (Legendary Enchantment - Background)
        let folk_hero = Card::new(
            2,
            "".to_string(),
            None,
            "Folk Hero".to_string(),
            Some("{1}{W}".to_string()),
            2,
            vec![CardType::Enchantment],
            vec![SuperType::Legendary],
            vec!["Background".to_string()],
            Some("Commander creatures you own have...".to_string()),
            "legal".to_string(),
            true,
            true,
            None,
        );

        deck.add_card(burakos.clone());
        deck.add_card(folk_hero.clone());

        deck.set_single_commander_from_deck(1).expect("Failed to set Burakos as commander");
        deck.set_partner_commander_from_deck(2).expect("Failed to set Folk Hero as partner");

        if let CommanderSelection::Partner(c1, c2) = deck.get_commander() {
            assert_eq!(c1.get_name(), "Burakos, Party Leader");
            assert_eq!(c2.get_name(), "Folk Hero");
        } else {
            panic!("Commander selection is not Partner for Background");
        }
    }
}
