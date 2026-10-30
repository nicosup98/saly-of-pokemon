use bevy::prelude::Component;
use serde::{Deserialize, Serialize};
use crate::stats::PokemonStats;
use crate::types::ElementType;

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct PokemonSprite {
    pub species_id: u32,
    pub is_back: bool,
}

impl PokemonSprite {
    pub fn new(species_id: u32, is_back: bool) -> Self {
        Self { species_id, is_back }
    }

    pub fn relative_path(&self) -> String {
        if self.is_back {
            format!("sprites/pokemon/back/{}.png", self.species_id)
        } else {
            format!("sprites/pokemon/front/{}.png", self.species_id)
        }
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct TrainerSprite {
    pub trainer_id: u32,
    pub is_back: bool,
    pub name: String,
}

impl TrainerSprite {
    pub fn new(trainer_id: u32, is_back: bool, name: impl Into<String>) -> Self {
        Self {
            trainer_id,
            is_back,
            name: name.into(),
        }
    }

    pub fn player_default() -> Self {
        Self::new(3, true, "Entrenador Red")
    }

    pub fn for_node(node_type: crate::map::generator::NodeType) -> Self {
        use crate::map::generator::NodeType;
        match node_type {
            NodeType::GymLeader => Self::new(6, false, "Líder Brock"),
            NodeType::MiniBoss => Self::new(7, false, "Entrenador Veterano"),
            NodeType::Enemy => Self::new(0, false, "Entrenador Rival"),
            _ => Self::new(1, false, "Entrenador"),
        }
    }

    pub fn relative_path(&self) -> String {
        if self.is_back {
            format!("sprites/trainers/trainer_{}_back.png", self.trainer_id)
        } else {
            format!("sprites/trainers/trainer_{}_front.png", self.trainer_id)
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpeciesData {
    pub id: u32,
    pub name: String,
    pub primary_types: Vec<ElementType>,
    pub coverage_types: Vec<ElementType>,
    pub base_hp: u32,
    pub base_atk: u32,
    pub base_def: u32,
    pub base_sp_atk: u32,
    pub base_sp_def: u32,
    pub base_spe: u32,
    pub evolves_at: Option<(u32, u32)>, // (nivel, id_siguiente_especie)
}

impl SpeciesData {
    pub fn front_sprite_path(&self) -> String {
        format!("sprites/pokemon/front/{}.png", self.id)
    }

    pub fn back_sprite_path(&self) -> String {
        format!("sprites/pokemon/back/{}.png", self.id)
    }
}

pub fn get_species_database() -> Vec<SpeciesData> {
    vec![
        // Línea Charmander (Fuego)
        SpeciesData {
            id: 4,
            name: "Charmander".to_string(),
            primary_types: vec![ElementType::Fire],
            coverage_types: vec![ElementType::Normal, ElementType::Rock],
            base_hp: 39,
            base_atk: 52,
            base_def: 43,
            base_sp_atk: 60,
            base_sp_def: 50,
            base_spe: 65,
            evolves_at: Some((16, 5)), // Charmeleon en Nv 16
        },
        SpeciesData {
            id: 5,
            name: "Charmeleon".to_string(),
            primary_types: vec![ElementType::Fire],
            coverage_types: vec![ElementType::Normal, ElementType::Rock],
            base_hp: 58,
            base_atk: 64,
            base_def: 58,
            base_sp_atk: 80,
            base_sp_def: 65,
            base_spe: 80,
            evolves_at: Some((36, 6)), // Charizard en Nv 36
        },
        SpeciesData {
            id: 6,
            name: "Charizard".to_string(),
            primary_types: vec![ElementType::Fire, ElementType::Dragon],
            coverage_types: vec![ElementType::Normal, ElementType::Rock, ElementType::Flying],
            base_hp: 78,
            base_atk: 84,
            base_def: 78,
            base_sp_atk: 109,
            base_sp_def: 85,
            base_spe: 100,
            evolves_at: None,
        },

        // Línea Squirtle (Agua)
        SpeciesData {
            id: 7,
            name: "Squirtle".to_string(),
            primary_types: vec![ElementType::Water],
            coverage_types: vec![ElementType::Normal, ElementType::Ice],
            base_hp: 44,
            base_atk: 48,
            base_def: 65,
            base_sp_atk: 50,
            base_sp_def: 64,
            base_spe: 43,
            evolves_at: Some((16, 8)),
        },
        SpeciesData {
            id: 8,
            name: "Wartortle".to_string(),
            primary_types: vec![ElementType::Water],
            coverage_types: vec![ElementType::Normal, ElementType::Ice],
            base_hp: 59,
            base_atk: 63,
            base_def: 80,
            base_sp_atk: 65,
            base_sp_def: 80,
            base_spe: 58,
            evolves_at: Some((36, 9)),
        },
        SpeciesData {
            id: 9,
            name: "Blastoise".to_string(),
            primary_types: vec![ElementType::Water, ElementType::Steel],
            coverage_types: vec![ElementType::Normal, ElementType::Ice, ElementType::Rock],
            base_hp: 79,
            base_atk: 83,
            base_def: 100,
            base_sp_atk: 85,
            base_sp_def: 105,
            base_spe: 78,
            evolves_at: None,
        },

        // Línea Bulbasaur (Planta / Veneno)
        SpeciesData {
            id: 1,
            name: "Bulbasaur".to_string(),
            primary_types: vec![ElementType::Grass, ElementType::Poison],
            coverage_types: vec![ElementType::Normal, ElementType::Ground],
            base_hp: 45,
            base_atk: 49,
            base_def: 49,
            base_sp_atk: 65,
            base_sp_def: 65,
            base_spe: 45,
            evolves_at: Some((16, 2)),
        },
        SpeciesData {
            id: 2,
            name: "Ivysaur".to_string(),
            primary_types: vec![ElementType::Grass, ElementType::Poison],
            coverage_types: vec![ElementType::Normal, ElementType::Ground],
            base_hp: 60,
            base_atk: 62,
            base_def: 63,
            base_sp_atk: 80,
            base_sp_def: 80,
            base_spe: 60,
            evolves_at: Some((32, 3)),
        },
        SpeciesData {
            id: 3,
            name: "Venusaur".to_string(),
            primary_types: vec![ElementType::Grass, ElementType::Poison],
            coverage_types: vec![ElementType::Normal, ElementType::Ground, ElementType::Rock],
            base_hp: 80,
            base_atk: 82,
            base_def: 83,
            base_sp_atk: 100,
            base_sp_def: 100,
            base_spe: 80,
            evolves_at: None,
        },

        // Pikachu
        SpeciesData {
            id: 25,
            name: "Pikachu".to_string(),
            primary_types: vec![ElementType::Electric],
            coverage_types: vec![ElementType::Normal, ElementType::Steel],
            base_hp: 35,
            base_atk: 55,
            base_def: 40,
            base_sp_atk: 50,
            base_sp_def: 50,
            base_spe: 90,
            evolves_at: Some((28, 26)), // Raichu en Nv 28
        },
        SpeciesData {
            id: 26,
            name: "Raichu".to_string(),
            primary_types: vec![ElementType::Electric],
            coverage_types: vec![ElementType::Normal, ElementType::Steel, ElementType::Fighting],
            base_hp: 60,
            base_atk: 90,
            base_def: 55,
            base_sp_atk: 90,
            base_sp_def: 80,
            base_spe: 110,
            evolves_at: None,
        },

        // Enemigos Comunes de Ruta
        SpeciesData {
            id: 10,
            name: "Caterpie".to_string(),
            primary_types: vec![ElementType::Bug],
            coverage_types: vec![ElementType::Normal],
            base_hp: 45,
            base_atk: 30,
            base_def: 35,
            base_sp_atk: 20,
            base_sp_def: 20,
            base_spe: 45,
            evolves_at: None,
        },
        SpeciesData {
            id: 19,
            name: "Rattata".to_string(),
            primary_types: vec![ElementType::Normal],
            coverage_types: vec![ElementType::Dark],
            base_hp: 30,
            base_atk: 56,
            base_def: 35,
            base_sp_atk: 25,
            base_sp_def: 35,
            base_spe: 72,
            evolves_at: None,
        },
        SpeciesData {
            id: 74,
            name: "Geodude".to_string(),
            primary_types: vec![ElementType::Rock, ElementType::Ground],
            coverage_types: vec![ElementType::Normal],
            base_hp: 40,
            base_atk: 80,
            base_def: 100,
            base_sp_atk: 30,
            base_sp_def: 30,
            base_spe: 20,
            evolves_at: None,
        },

        // Mini Jefes (Pokémon Salvajes Dominantes)
        SpeciesData {
            id: 123,
            name: "Scyther".to_string(),
            primary_types: vec![ElementType::Bug, ElementType::Flying],
            coverage_types: vec![ElementType::Normal, ElementType::Fighting],
            base_hp: 70,
            base_atk: 110,
            base_def: 80,
            base_sp_atk: 55,
            base_sp_def: 80,
            base_spe: 105,
            evolves_at: None,
        },
        SpeciesData {
            id: 143,
            name: "Snorlax".to_string(),
            primary_types: vec![ElementType::Normal],
            coverage_types: vec![ElementType::Fighting, ElementType::Ground],
            base_hp: 160,
            base_atk: 110,
            base_def: 65,
            base_sp_atk: 65,
            base_sp_def: 110,
            base_spe: 30,
            evolves_at: None,
        },

        // Jefes de Gimnasio (Líder Brock)
        SpeciesData {
            id: 95,
            name: "Onix Insignia".to_string(),
            primary_types: vec![ElementType::Rock, ElementType::Ground],
            coverage_types: vec![ElementType::Normal],
            base_hp: 75,
            base_atk: 65,
            base_def: 160,
            base_sp_atk: 40,
            base_sp_def: 65,
            base_spe: 70,
            evolves_at: None,
        },
        // Jefes de Gimnasio (Líder Misty)
        SpeciesData {
            id: 121,
            name: "Starmie Insignia".to_string(),
            primary_types: vec![ElementType::Water, ElementType::Psychic],
            coverage_types: vec![ElementType::Ice, ElementType::Electric],
            base_hp: 65,
            base_atk: 75,
            base_def: 85,
            base_sp_atk: 100,
            base_sp_def: 85,
            base_spe: 115,
            evolves_at: None,
        },
        // Jefe Final / Campeón de Liga
        SpeciesData {
            id: 150,
            name: "Mewtwo Legendario".to_string(),
            primary_types: vec![ElementType::Psychic],
            coverage_types: vec![ElementType::Fighting, ElementType::Ghost],
            base_hp: 106,
            base_atk: 110,
            base_def: 90,
            base_sp_atk: 154,
            base_sp_def: 90,
            base_spe: 130,
            evolves_at: None,
        },
    ]
}

pub fn create_pokemon_from_species(species_id: u32, level: u32) -> Option<PokemonStats> {
    let db = get_species_database();
    let sp = db.iter().find(|s| s.id == species_id)?;
    Some(PokemonStats::new(
        sp.id,
        &sp.name,
        level,
        sp.primary_types.clone(),
        sp.coverage_types.clone(),
        sp.base_hp,
        sp.base_atk,
        sp.base_def,
        sp.base_sp_atk,
        sp.base_sp_def,
        sp.base_spe,
    ))
}

pub fn get_species_data(id: u32) -> Option<SpeciesData> {
    get_species_database().into_iter().find(|s| s.id == id)
}

/// Recalcula las estadísticas de un Pokémon al subir de nivel según las estadísticas base de su especie
pub fn update_pokemon_stats_for_level(stats: &mut PokemonStats) {
    let db = get_species_database();
    if let Some(sp) = db.iter().find(|s| s.id == stats.species_id) {
        let old_max_hp = stats.max_hp;
        stats.max_hp = PokemonStats::calc_hp(sp.base_hp, stats.level);
        stats.hp += (stats.max_hp - old_max_hp).max(0);
        stats.atk = PokemonStats::calc_stat(sp.base_atk, stats.level);
        stats.def = PokemonStats::calc_stat(sp.base_def, stats.level);
        stats.sp_atk = PokemonStats::calc_stat(sp.base_sp_atk, stats.level);
        stats.sp_def = PokemonStats::calc_stat(sp.base_sp_def, stats.level);
        stats.spe = PokemonStats::calc_stat(sp.base_spe, stats.level);
    }
}

/// Comprueba si el Pokémon actual debe evolucionar según su nivel y el GDD
pub fn check_evolution(stats: &mut PokemonStats) -> Option<String> {
    let db = get_species_database();
    let current_sp = db.iter().find(|s| s.id == stats.species_id)?;
    if let Some((evo_level, next_id)) = current_sp.evolves_at {
        if stats.level >= evo_level {
            if let Some(next_sp) = db.iter().find(|s| s.id == next_id) {
                let old_name = stats.name.clone();
                stats.species_id = next_sp.id;
                stats.name = next_sp.name.clone();
                stats.primary_types = next_sp.primary_types.clone();
                stats.coverage_types = next_sp.coverage_types.clone();

                // Recalcula stats con la nueva especie
                let old_max_hp = stats.max_hp;
                stats.max_hp = PokemonStats::calc_hp(next_sp.base_hp, stats.level);
                stats.hp += stats.max_hp - old_max_hp; // Aumento de vida proporcional
                stats.atk = PokemonStats::calc_stat(next_sp.base_atk, stats.level);
                stats.def = PokemonStats::calc_stat(next_sp.base_def, stats.level);
                stats.sp_atk = PokemonStats::calc_stat(next_sp.base_sp_atk, stats.level);
                stats.sp_def = PokemonStats::calc_stat(next_sp.base_sp_def, stats.level);
                stats.spe = PokemonStats::calc_stat(next_sp.base_spe, stats.level);

                return Some(format!("[*] ¡Tu {} ha evolucionado a {}!", old_name, stats.name));
            }
        }
    }
    None
}

/// Determina la categoría de rareza de una especie de Pokémon para el Gachapón y animaciones
pub fn species_rarity(species_id: u32) -> crate::types::RarityTier {
    use crate::types::RarityTier::*;
    match species_id {
        150 | 6 | 9 | 3 => Legendary,
        123 | 143 | 5 | 8 | 2 | 26 => Exotic,
        4 | 7 | 1 | 25 => Rare,
        74 | 95 | 121 => Uncommon,
        _ => Common,
    }
}
