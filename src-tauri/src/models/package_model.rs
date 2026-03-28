use crate::models::card_model::Card;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    id: u64,
    name: String,
    #[serde(default)]
    description: String,
    cards: Vec<Card>,
}

impl Package {
    pub fn new(id: u64, name: String) -> Self {
        Self {
            id,
            name,
            description: String::new(),
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

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn get_cards(&self) -> &[Card] {
        &self.cards
    }

    pub fn remove_card_at(&mut self, index: usize) -> bool {
        if index >= self.cards.len() {
            return false;
        }

        self.cards.remove(index);
        true
    }
}
