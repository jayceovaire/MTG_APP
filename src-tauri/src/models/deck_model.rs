use crate::models::card_model::Card;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deck {
    id: u64,
    name: String,
    commander: CommanderSelection,
    cards: Vec<Card>,
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
        Self{
            id,
            name: "New Deck".to_string(),
            commander: CommanderSelection::None,
            cards: Vec::new(),
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
    fn test_add_card() {

        let card = Card::new(
            1,
            "test".to_string(),
            "card name".to_string(),
            Some("2{B}{R}{U}".to_string()),
            5,
            vec![CardType::Creature],
            vec![SuperType::Legendary],
            vec!["Wizard".to_string()],
            Some("oracle text".to_string()),
        );

    }
}
