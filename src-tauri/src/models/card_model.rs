use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card{
    id: u64,
    image: String, //TODO this is to be tied to an address in cache somehow
    name: String,
    mana_cost: Option<String>,
    mana_value: u8,
    card_type: Vec<CardType>,
    super_type: Vec<SuperType>,
    sub_type: Vec<String>,
    oracle_text: Option<String>,

}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardType {
    Creature,
    Instant,
    Sorcery,
    Enchantment,
    Artifact,
    Land,
    Planeswalker,
    Battle,
    Tribal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuperType {
    Legendary,
    Basic,
    Snow,
    World,
    Ongoing,
}

impl Card {
    pub fn new(
        id: u64,
        image: String,
        name: String,
        mana_cost: Option<String>,
        mana_value: u8,
        card_type: Vec<CardType>,
        super_type: Vec<SuperType>,
        sub_type: Vec<String>,
        oracle_text: Option<String>,
    ) -> Self {
        Self {
            id,
            image,
            name,
            mana_cost,
            mana_value,
            card_type,
            super_type,
            sub_type,
            oracle_text,
        }
    }

    pub fn is_legendary(&self) -> bool {
        self.super_type.contains(&SuperType::Legendary)
    }

    pub fn is_creature(&self) -> bool {
        self.card_type.contains(&CardType::Creature)
    }

    pub fn is_enchantment(&self) -> bool {
        self.card_type.contains(&CardType::Enchantment)
    }

    pub fn is_battle(&self) -> bool {
        self.card_type.contains(&CardType::Battle)
    }

    pub fn is_tribal(&self) -> bool {
        self.card_type.contains(&CardType::Tribal)
    }
    pub fn is_instant(&self) -> bool {
        self.card_type.contains(&CardType::Instant)
    }

    pub fn is_sorcery(&self) -> bool {
        self.card_type.contains(&CardType::Sorcery)
    }

    pub fn is_land(&self) -> bool {
        self.card_type.contains(&CardType::Land)
    }

    pub fn is_vehicle(&self) -> bool {
        self.sub_type
            .iter()
            .any(|subtype| subtype.eq_ignore_ascii_case("Vehicle"))
    }

    pub fn is_planeswalker(&self) -> bool {
        self.card_type.contains(&CardType::Planeswalker)
    }

    pub fn has_partner_mechanic(&self) -> bool {
        self.oracle_text
            .as_deref()
            .map(|text| {
                text.lines().any(|line| {
                    line.trim().to_ascii_lowercase().starts_with("partner")
                })
            })
            .unwrap_or(false)
    }

    pub fn can_be_commander(&self) -> bool {
        (self.is_legendary() && self.is_creature()) || (self.is_legendary() && self.is_vehicle())
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
}
