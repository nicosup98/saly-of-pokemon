use std::collections::HashMap;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum AppState {
    #[default]
    ProfileSelect,
    MainMenu,
    StarterSelection,
    GachaponShop,
    MapNavigation,
    Combat,
    Victory,
    PokemonCenter,
    Shop,
    Treasure,
    RunSummary,
}

#[derive(SubStates, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[source(AppState = AppState::Combat)]
pub enum CombatPhase {
    #[default]
    CombatSetup,
    PlayerTurnStart,
    PlayerAction,
    CardResolution,
    PlayerTurnEnd,
    EnemyTurnResolution,
    CheckCombatOutcome,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct CurrentRunState {
    pub poke_dollars: u32,
    pub current_act: u8,
    pub current_floor: u8,
    pub relics: Vec<String>,
    pub consumables: [Option<String>; 3],
    pub card_removal_cost: u32,
    pub energy: u8,
    pub max_energy: u8,
    pub player_deck: Vec<crate::card::Card>,
    pub battle_logs: Vec<String>,
    pub selected_starter_id: u32,
    /// HP actual del Pokémon del jugador entre combates (`None` = HP completo, partida nueva).
    pub player_hp: Option<i32>,
    /// Nivel persistente del Pokémon del jugador (`None` = Nivel inicial 5).
    #[serde(default)]
    pub player_level: Option<u32>,
    /// EXP acumulada en el nivel actual (`None` = 0).
    #[serde(default)]
    pub player_exp: Option<u32>,
}

impl Default for CurrentRunState {
    fn default() -> Self {
        Self {
            poke_dollars: 100, // 100 P$ de bolsillo inicial según GDD 7.1
            current_act: 1,
            current_floor: 1,
            relics: Vec::new(),
            consumables: [Some("potion".to_string()), None, None],
            card_removal_cost: 75,
            energy: 3,
            max_energy: 3,
            player_deck: Vec::new(),
            battle_logs: Vec::new(),
            selected_starter_id: 4, // Charmander por defecto
            player_hp: None,
            player_level: None,
            player_exp: None,
        }
    }
}

impl CurrentRunState {
    /// Nivel del Pokémon del jugador según su progreso acumulado en la partida (base Nv. 5)
    pub fn player_level(&self) -> u32 {
        self.player_level.unwrap_or(5)
    }

    /// Experiencia acumulada en el nivel actual
    pub fn player_exp(&self) -> u32 {
        self.player_exp.unwrap_or(0)
    }

    pub fn add_log(&mut self, msg: impl Into<String>) {
        self.battle_logs.push(msg.into());
        if self.battle_logs.len() > 30 {
            self.battle_logs.remove(0);
        }
    }

    pub fn has_relic(&self, relic_id: &str) -> bool {
        self.relics.iter().any(|r| r == relic_id)
    }
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct MetaProfile {
    pub gachapon_tokens: u32,
    pub unlocked_pokemon: Vec<u32>,
    pub pokemon_fragments: HashMap<u32, u32>,
    pub starter_chosen: bool,
    pub ascension_level: u32,
}

impl Default for MetaProfile {
    fn default() -> Self {
        Self {
            gachapon_tokens: 300,
            unlocked_pokemon: vec![4], // Solo Charmander desbloqueado inicialmente
            pokemon_fragments: HashMap::new(),
            starter_chosen: false,
            ascension_level: 0,
        }
    }
}
