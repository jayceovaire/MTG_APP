use crate::models::card_model::Card;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    id: u64,
    name: String,
    cards: Vec<Card>,
}

impl Package {
    pub fn new(id: u64, name: String) -> Self {
        Self {
            id,
            name,
            cards: Vec::new(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn get_cards(&self) -> &[Card] {
        &self.cards
    }
}
