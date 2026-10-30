use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use rand::Rng;
use crate::stats::PokemonStats;
use crate::types::{ElementType, MoveCategory, StatusType};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EnemyIntentType {
    Attack { raw_damage: f32, category: MoveCategory },
    Defend { block_amount: i32 },
    Buff { status: StatusType, stacks: i32 },
    Debuff { status: StatusType, stacks: i32 },
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct EnemyAI {
    pub current_intent: Option<EnemyIntentType>,
    pub history: Vec<String>, // Para la regla anti-repetición de 3 turnos
    pub turn_counter: usize,
    pub is_boss: bool,
    pub is_mini_boss: bool,
}

impl EnemyAI {
    pub fn new(is_boss: bool, is_mini_boss: bool) -> Self {
        Self {
            current_intent: None,
            history: Vec::new(),
            turn_counter: 0,
            is_boss,
            is_mini_boss,
        }
    }

    /// Decide la siguiente intención respetando la regla anti-repetición del GDD
    pub fn decide_next_intent(&mut self, stats: &PokemonStats) -> EnemyIntentType {
        let mut rng = rand::thread_rng();
        self.turn_counter += 1;

        // Comprueba si se repitió 2 veces consecutivas la misma acción
        let last_two_same = if self.history.len() >= 2 {
            self.history[self.history.len() - 1] == self.history[self.history.len() - 2]
        } else {
            false
        };

        let last_action = self.history.last().map(|s| s.as_str());

        // Patrón determinista para Jefes de Gimnasio (ej. Onix Insignia)
        let intent = if self.is_boss {
            match self.turn_counter % 3 {
                1 => {
                    // Turno 1: Fortaleza / Buff
                    EnemyIntentType::Defend {
                        block_amount: (stats.effective_def() * 1.5).round() as i32,
                    }
                }
                2 => {
                    // Turno 2: Ataque moderado
                    EnemyIntentType::Attack {
                        raw_damage: stats.effective_atk() * 1.2,
                        category: MoveCategory::Physical,
                    }
                }
                _ => {
                    // Turno 3: Ataque pesado
                    EnemyIntentType::Attack {
                        raw_damage: stats.effective_atk() * 2.0,
                        category: MoveCategory::Physical,
                    }
                }
            }
        } else {
            // Enemigos estándar con tirada ponderada y filtro anti-repetición
            let roll = rng.gen_range(0..100);
            if roll < 55 {
                // Ataque
                if last_two_same && last_action == Some("attack") {
                    // Prohibido repetir 3 veces seguidas: cambia a defensa o debuff
                    EnemyIntentType::Defend {
                        block_amount: (stats.effective_def() * 1.0).round() as i32,
                    }
                } else {
                    EnemyIntentType::Attack {
                        raw_damage: stats.effective_atk() * 1.15,
                        category: MoveCategory::Physical,
                    }
                }
            } else if roll < 85 {
                // Defensa
                if last_two_same && last_action == Some("defend") {
                    EnemyIntentType::Attack {
                        raw_damage: stats.effective_atk() * 1.0,
                        category: MoveCategory::Physical,
                    }
                } else {
                    EnemyIntentType::Defend {
                        block_amount: (stats.effective_def() * 1.1).round() as i32,
                    }
                }
            } else {
                // Debuff o Buff temático según el tipo elemental del Pokémon
                let (status, is_buff) = if stats.primary_types.contains(&ElementType::Fire) {
                    (StatusType::Burn, false)
                } else if stats.primary_types.contains(&ElementType::Electric) {
                    (StatusType::Paralysis, false)
                } else if stats.primary_types.contains(&ElementType::Grass) {
                    (StatusType::LeechSeed, false)
                } else if stats.primary_types.contains(&ElementType::Poison) {
                    (StatusType::Poison, false)
                } else if stats.primary_types.contains(&ElementType::Ice) {
                    (StatusType::Chill, false)
                } else if stats.primary_types.contains(&ElementType::Water) {
                    (StatusType::Chill, false)
                } else if stats.effective_def() >= stats.effective_atk() {
                    (StatusType::IronDefense, true)
                } else {
                    (StatusType::SwordDance, true)
                };

                if is_buff {
                    EnemyIntentType::Buff {
                        status,
                        stacks: 1,
                    }
                } else {
                    EnemyIntentType::Debuff {
                        status,
                        stacks: 2,
                    }
                }
            }
        };

        // Guarda en historial
        let action_name = match &intent {
            EnemyIntentType::Attack { .. } => "attack",
            EnemyIntentType::Defend { .. } => "defend",
            EnemyIntentType::Buff { .. } => "buff",
            EnemyIntentType::Debuff { .. } => "debuff",
        };
        self.history.push(action_name.to_string());
        self.current_intent = Some(intent.clone());
        intent
    }
}
