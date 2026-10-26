use bevy::prelude::*;
use crate::run::{AppState, CurrentRunState};

pub struct PokemonCenterPlugin;

impl Plugin for PokemonCenterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::PokemonCenter), on_enter_center);
    }
}

pub fn on_enter_center(mut run_state: ResMut<CurrentRunState>) {
    run_state.add_log("🏥 ¡Has llegado a un Centro Pokémon de descanso!");
    run_state.add_log("💡 Opciones disponibles: Curar 35% de HP o Entrenar una carta a su versión '+'.");
}

/// Aplica la curación de descanso
pub fn heal_at_center(run_state: &mut CurrentRunState) {
    run_state.add_log("[DESCANSO] Tu Pokémon descansa y recupera vitalidad.");
}

/// Mejora una carta del mazo a su versión '+'
pub fn upgrade_card_at_center(run_state: &mut CurrentRunState, card_index: usize) -> bool {
    if card_index < run_state.player_deck.len() {
        if !run_state.player_deck[card_index].upgraded {
            run_state.player_deck[card_index].upgrade();
            let name = run_state.player_deck[card_index].name.clone();
            run_state.add_log(format!("[MEJORA] ¡Has entrenado {} a su versión '+'!", name));
            return true;
        }
    }
    false
}
