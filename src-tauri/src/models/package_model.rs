use crate::models::card_model::Card;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    id: u64,
    name: String,
    #[serde(default)]
    description: String,
    cards: Vec<Card>,
    #[serde(default)]
    ui_order: u32,
}

impl Package {
    pub fn new(id: u64, name: String) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            cards: Vec::new(),
            ui_order: 0,
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
    pub fn ui_order(&self) -> u32 {
        self.ui_order
    }
    pub fn set_ui_order(&mut self, order: u32) {
        self.ui_order = order;
    }

    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn get_cards(&self) -> &[Card] {
        &self.cards
    }

    pub fn get_cards_mut(&mut self) -> &mut Vec<Card> {
        &mut self.cards
    }

    pub fn remove_card_at(&mut self, index: usize) -> bool {
        if index >= self.cards.len() {
            return false;
        }

        self.cards.remove(index);
        true
    }
}
