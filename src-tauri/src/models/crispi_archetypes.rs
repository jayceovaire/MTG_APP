use crate::models::crispi_types::DeckArchetype;

pub fn detect_archetype(
    stax_signal: f32,
    commander_engine_signal: f32,
    turbo_signal: f32,
    midrange_signal: f32,
    voltron_signal: f32,
    group_hug_signal: f32,
    infect_signal: f32,
) -> DeckArchetype {
    if infect_signal >= 8.0 && infect_signal > turbo_signal && infect_signal > midrange_signal {
        DeckArchetype::Infect
    } else if stax_signal >= 15.0 && stax_signal > turbo_signal && stax_signal > midrange_signal {
        DeckArchetype::Stax
    } else if group_hug_signal >= 8.0 && group_hug_signal > midrange_signal {
        DeckArchetype::GroupHug
    } else if voltron_signal >= 8.0 && voltron_signal > turbo_signal && voltron_signal > midrange_signal {
        DeckArchetype::Voltron
    } else if commander_engine_signal > 1.0 && turbo_signal > 12.0 && turbo_signal > midrange_signal {
        DeckArchetype::CommanderEngine
    } else if turbo_signal >= 18.0 && turbo_signal >= midrange_signal {
        DeckArchetype::Turbo
    } else if turbo_signal > 12.0 && turbo_signal > midrange_signal {
        DeckArchetype::Turbo
    } else {
        DeckArchetype::Midrange
    }
}
