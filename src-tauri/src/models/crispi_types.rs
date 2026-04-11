use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Role {
    LAND,
    RAMP,
    DRAW,
    REMOVAL,
    TUTOR,
    PROTECTION,
    FIXING,
    ENGINE,
    WINCON,
    STAX,
    RECURSION,
    FAST_MANA,
    RITUAL,
    TREASURE_BURST,
    SAC_MANA,
    COST_REDUCTION,
    FAST_MANA_ONE_SHOT,
    BURST_DRAW,
    MASS_DRAW,
    WHEEL,
    VOLTRON_PIECE,
    LOOTING,
    IMPULSE_DRAW,
    GROUP_HUG,
    MASS_REMOVAL,
    COMBO_PIECE,
    ENGINE_WIN,
    INFECT,
    PROLIFERATE,
    STORM_PAYOFF,
    MAGECRAFT,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeckArchetype {
    Turbo,
    Midrange,
    Stax,
    CommanderEngine,
    Voltron,
    GroupHug,
    Infect,
    Storm,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QualityTier {
    Premium,
    Efficient,
    Slow,
    LowImpact,
}

impl QualityTier {
    pub fn weight(&self) -> f32 {
        match self {
            QualityTier::Premium => 1.5,
            QualityTier::Efficient => 1.0,
            QualityTier::Slow => 0.75,
            QualityTier::LowImpact => 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrispiDimension {
    pub score: u8,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CrispiEvaluation {
    pub total_score: f32,
    pub raw_score: f32,
    pub turbo_signal: f32,
    pub midrange_signal: f32,
    pub stax_signal: f32,
    pub voltron_signal: f32,
    pub infect_signal: f32,
    pub storm_signal: f32,
    pub group_hug_signal: f32,
    pub commander_engine_signal: f32,
    pub amv_multiplier: f32,
    pub combo_multiplier: f32,
    pub final_multiplier: f32,
    pub commander_mv_penalty: f32,
    pub land_score: f32,
    pub role_score: f32,
    pub interpretation: String,
    pub bracket: u8,
    pub archetype: DeckArchetype,
    pub detected_combos: Vec<String>,
    pub consistency: CrispiDimension,
    pub resilience: CrispiDimension,
    pub interaction: CrispiDimension,
    pub speed: CrispiDimension,
    pub pivotability: CrispiDimension,
}
