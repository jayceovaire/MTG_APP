use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    id: u64,
    image: String, 
    #[serde(default)]
    image_uri: Option<String>, 
    name: String,
    mana_cost: Option<String>,
    mana_value: u8,
    card_type: Vec<CardType>,
    super_type: Vec<SuperType>,
    sub_type: Vec<String>,
    oracle_text: Option<String>,
    /// Commander-format legality from Scryfall (`legal`, `banned`, `not_legal`, `restricted`, …).
    #[serde(default = "default_commander_legality")]
    commander_legality: String,
    #[serde(default)]
    legal_in_commander: bool,
    #[serde(default)]
    game_changer: bool,
    #[serde(default)]
    scryfall_id: Option<String>,
}

fn default_commander_legality() -> String {
    "legal".to_string()
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
        image_uri: Option<String>,
        name: String,
        mana_cost: Option<String>,
        mana_value: u8,
        card_type: Vec<CardType>,
        super_type: Vec<SuperType>,
        sub_type: Vec<String>,
        oracle_text: Option<String>,
        commander_legality: String,
        legal_in_commander: bool,
        game_changer: bool,
        scryfall_id: Option<String>,
    ) -> Self {
        Self {
            id,
            image,
            image_uri,
            name,
            mana_cost,
            mana_value,
            card_type,
            super_type,
            sub_type,
            oracle_text,
            commander_legality,
            legal_in_commander,
            game_changer,
            scryfall_id,
        }
    }

    pub fn image_uri(&self) -> Option<&str> {
        self.image_uri.as_deref()
    }

    pub fn get_image(&self) -> &str {
        &self.image
    }

    pub fn set_image(&mut self, image: String) {
        self.image = image;
    }

    /// True if this card is legal for Commander.
    pub fn is_legal_in_commander(&self) -> bool {
        self.legal_in_commander
    }

    /// True if this card is not legal for Commander (banned, not legal in format, restricted, etc.).
    pub fn is_illegal_in_commander(&self) -> bool {
        !self.legal_in_commander
    }

    pub fn is_game_changer(&self) -> bool {
        self.game_changer
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

    pub fn is_spacecraft(&self) -> bool {
        self.sub_type
            .iter()
            .any(|subtype| subtype.eq_ignore_ascii_case("Spacecraft"))
    }

    pub fn is_planeswalker(&self) -> bool {
        self.card_type.contains(&CardType::Planeswalker)
    }

    /// Checks if this card has the Partner mechanic.
    /// This includes:
    /// - "Partner" (generic partner)
    /// - "Partner with [name]" (specific partner)
    /// - "Friends forever" (from Doctor Who set, works like partner)
    /// - "Choose a Background" (partners with Background cards)
    /// - "Doctor's companion" (partners with The Doctor)
    pub fn has_partner_mechanic(&self) -> bool {
        // Special case: Backgrounds are partners to "Choose a Background" cards
        if self.is_enchantment() && self.sub_type.iter().any(|s| s.eq_ignore_ascii_case("Background")) {
            return true;
        }

        self.oracle_text
            .as_deref()
            .map(|text| {
                let lower = text.to_ascii_lowercase();
                // Standard partner mechanics often appear on their own line or at the start/end of a line
                text.lines().any(|line| {
                    let trimmed = line.trim().to_ascii_lowercase();
                    trimmed == "partner"
                        || trimmed.starts_with("partner with ")
                        || trimmed == "friends forever"
                        || trimmed == "choose a background"
                        || trimmed == "doctor's companion"
                }) ||
                // Also check for them anywhere in the text as a standalone word (to be safe)
                lower.contains("partner") ||
                lower.contains("friends forever") ||
                lower.contains("choose a background") ||
                lower.contains("doctor's companion")
            })
            .unwrap_or(false)
    }

    /// Checks if this card can legally be a commander.
    /// A card can be a commander if it's:
    /// - A Legendary Creature
    /// - A Legendary Vehicle (e.g., Shorikai)
    /// - A Legendary Spacecraft
    /// - A Legendary Planeswalker with "can be your commander" text
    /// - A card with Partner, Partner with, or "can be your commander" in oracle text
    /// - A Background (Legendary Enchantment that can be a commander)
    pub fn can_be_commander(&self) -> bool {
        // Standard case: Legendary Creature, Legendary Vehicle, or Legendary Spacecraft
        if self.is_legendary() && (self.is_creature() || self.is_vehicle() || self.is_spacecraft()) {
            return true;
        }

        // Backgrounds: Legendary Enchantments with "Choose a Background" text (if they are the commander)
        // Actually, Backgrounds themselves can be commanders if you have a "Choose a Background" creature.
        // In Commander, a Background is a legal commander if the other commander has "Choose a Background".
        if self.is_legendary() && self.is_enchantment() {
            if self.sub_type.iter().any(|s| s.eq_ignore_ascii_case("Background")) {
                return true;
            }
        }

        // Cards with "can be your commander" text (includes some planeswalkers)
        if let Some(text) = &self.oracle_text {
            let lower = text.to_ascii_lowercase();
            if lower.contains("can be your commander") {
                return true;
            }
        }

        false
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

    pub fn scryfall_id(&self) -> Option<&str> {
        self.scryfall_id.as_deref()
    }

    pub fn set_scryfall_id(&mut self, scryfall_id: String) {
        self.scryfall_id = Some(scryfall_id);
    }

    pub fn oracle_text(&self) -> Option<&str> {
        self.oracle_text.as_deref()
    }

    pub fn mana_value(&self) -> u8 {
        self.mana_value
    }
}
