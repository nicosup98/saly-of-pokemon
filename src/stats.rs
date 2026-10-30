use std::collections::HashMap;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::types::{ElementType, MoveCategory, StatusType};

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct PokemonStats {
    pub species_id: u32,
    pub name: String,
    pub level: u32,
    pub exp: u32,
    pub primary_types: Vec<ElementType>,
    pub coverage_types: Vec<ElementType>,
    pub hp: i32,
    pub max_hp: i32,
    pub atk: f32,
    pub def: f32,
    pub sp_atk: f32,
    pub sp_def: f32,
    pub spe: f32,
    pub block: i32,
    pub statuses: HashMap<StatusType, i32>,
}

impl PokemonStats {
    pub fn new(
        species_id: u32,
        name: impl Into<String>,
        level: u32,
        primary_types: Vec<ElementType>,
        coverage_types: Vec<ElementType>,
        base_hp: u32,
        base_atk: u32,
        base_def: u32,
        base_sp_atk: u32,
        base_sp_def: u32,
        base_spe: u32,
    ) -> Self {
        let max_hp = Self::calc_hp(base_hp, level);
        let atk = Self::calc_stat(base_atk, level);
        let def = Self::calc_stat(base_def, level);
        let sp_atk = Self::calc_stat(base_sp_atk, level);
        let sp_def = Self::calc_stat(base_sp_def, level);
        let spe = Self::calc_stat(base_spe, level);

        Self {
            species_id,
            name: name.into(),
            level,
            exp: 0,
            primary_types,
            coverage_types,
            hp: max_hp,
            max_hp,
            atk,
            def,
            sp_atk,
            sp_def,
            spe,
            block: 0,
            statuses: HashMap::new(),
        }
    }

    /// Fórmula de HP según el GDD (Sección 5.3):
    /// Adaptada al formato roguelike (84 HP en Nv. 5 -> 135 HP en Nv. 16 -> 295 HP en Nv. 36)
    pub fn calc_hp(base_hp: u32, level: u32) -> i32 {
        let b = base_hp as f32;
        let l = level as f32;
        (b + 41.0 + (47.0 * l / 341.0) + (45.0 * l * l / 341.0)).round() as i32
    }

    /// Fórmula de Stats de Combate según el GDD: ((Base_Stat * Nivel) / 100) + 5
    pub fn calc_stat(base_stat: u32, level: u32) -> f32 {
        let b = base_stat as f32;
        let l = level as f32;
        ((b * l) / 100.0 + 5.0).round()
    }

    /// Retorna la cantidad de experiencia (XP) requerida para avanzar del nivel actual al siguiente.
    /// Utiliza una progresión cuadrática: a mayor nivel, se requiere más XP y más combates para subir.
    pub fn exp_to_next_level(&self) -> u32 {
        Self::calc_exp_to_next_level(self.level)
    }

    /// Fórmula de progresión cuadrática de experiencia según el nivel:
    /// Requerido(Nivel) = 40 + (8 * Nivel) + (2 * Nivel^2)
    /// Nv. 5 -> 130 XP
    /// Nv. 10 -> 320 XP
    /// Nv. 20 -> 1000 XP
    /// Nv. 30 -> 2080 XP
    pub fn calc_exp_to_next_level(level: u32) -> u32 {
        let l = level;
        40 + 8 * l + 2 * l * l
    }

    /// Añade experiencia al Pokémon y procesa subidas de nivel consecutivas si supera el umbral.
    /// Retorna la cantidad de niveles subidos y los registros generados.
    pub fn gain_exp(&mut self, exp_amount: u32) -> (u32, Vec<String>) {
        self.exp += exp_amount;
        let mut levels_gained = 0;
        let mut logs = Vec::new();

        while self.level < 100 {
            let needed = self.exp_to_next_level();
            if self.exp >= needed {
                self.exp -= needed;
                self.level += 1;
                levels_gained += 1;
                logs.push(format!("[NIVEL] ¡{} ha alcanzado el Nivel {}!", self.name, self.level));
            } else {
                break;
            }
        }

        (levels_gained, logs)
    }

    /// Factor Porcentual de Mitigación por Defensa (Def o Sp. Def):
    /// Factor = 100 / (100 + Def)
    pub fn calc_mitigation(effective_def: f32) -> f32 {
        let d = effective_def.max(0.0);
        100.0 / (100.0 + d)
    }

    /// Retorna el Ataque efectivo considerando buffs y quemadura
    pub fn effective_atk(&self) -> f32 {
        let mut val = self.atk;
        if let Some(&stacks) = self.statuses.get(&StatusType::SwordDance) {
            val *= 1.0 + (0.30 * stacks as f32);
        }
        if self.statuses.contains_key(&StatusType::Burn) {
            val *= 0.80; // Quemadura reduce el Atk físico un 20%
        }
        val
    }

    /// Retorna el Ataque Especial efectivo considerando buffs
    pub fn effective_sp_atk(&self) -> f32 {
        let mut val = self.sp_atk;
        if let Some(&stacks) = self.statuses.get(&StatusType::NastyPlot) {
            val *= 1.0 + (0.30 * stacks as f32);
        }
        val
    }

    /// Retorna la Defensa efectiva considerando buffs
    pub fn effective_def(&self) -> f32 {
        let mut val = self.def;
        if let Some(&stacks) = self.statuses.get(&StatusType::IronDefense) {
            val *= 1.0 + (0.20 * stacks as f32);
        }
        val
    }

    /// Retorna la Defensa Especial efectiva
    pub fn effective_sp_def(&self) -> f32 {
        self.sp_def
    }

    /// Aplica daño mitigado contra Bloqueo y luego contra HP
    pub fn take_damage(&mut self, raw_damage: f32, category: MoveCategory) -> (i32, i32) {
        let mitigation = match category {
            MoveCategory::Physical => Self::calc_mitigation(self.effective_def()),
            MoveCategory::Special => Self::calc_mitigation(self.effective_sp_def()),
            _ => 1.0,
        };

        // Si el objetivo tiene Parálisis, recibe un +20% de daño adicional
        let paralyze_bonus = if self.statuses.contains_key(&StatusType::Paralysis) { 1.2 } else { 1.0 };
        let mitigated = (raw_damage * mitigation * paralyze_bonus).round() as i32;

        let blocked = mitigated.min(self.block);
        self.block -= blocked;
        let hp_damage = (mitigated - blocked).max(0);
        self.hp = (self.hp - hp_damage).max(0);

        (blocked, hp_damage)
    }

    /// Añade Bloqueo al Pokémon
    pub fn add_block(&mut self, amount: i32) {
        self.block += amount.max(0);
    }

    /// Cura HP al Pokémon hasta su max_hp
    pub fn heal(&mut self, amount: i32) -> i32 {
        let prev = self.hp;
        self.hp = (self.hp + amount.max(0)).min(self.max_hp);
        self.hp - prev
    }

    /// Aplica o suma cargas de un estado alterado
    pub fn apply_status(&mut self, status: StatusType, stacks: i32) {
        let entry = self.statuses.entry(status).or_insert(0);
        *entry += stacks;
    }

    /// Procesa efectos al inicio del turno
    pub fn on_turn_start(&mut self, retain_block_ratio: f32) {
        if retain_block_ratio > 0.0 {
            self.block = (self.block as f32 * retain_block_ratio).round() as i32;
        } else {
            self.block = 0;
        }
    }

    /// Procesa daño de estado al final del turno (Quemadura, Toxina, etc.)
    pub fn on_turn_end(&mut self) -> Vec<String> {
        let mut logs = Vec::new();
        // Quemadura
        if let Some(&burn_stacks) = self.statuses.get(&StatusType::Burn) {
            if burn_stacks > 0 {
                let dmg = (burn_stacks * 3).max(1);
                self.hp = (self.hp - dmg).max(0);
                logs.push(format!("[QUEMADURA] {} recibe {} de daño por Quemadura.", self.name, dmg));
            }
        }
        // Veneno
        if let Some(&poison_stacks) = self.statuses.get(&StatusType::Poison) {
            if poison_stacks > 0 {
                let dmg = (poison_stacks * 4).max(1);
                self.hp = (self.hp - dmg).max(0);
                logs.push(format!("[TOXINA] {} recibe {} de daño por Toxina.", self.name, dmg));
            }
        }
        logs
    }

    /// Comprueba si el Pokémon ha caído
    pub fn is_fainted(&self) -> bool {
        self.hp <= 0
    }
}
