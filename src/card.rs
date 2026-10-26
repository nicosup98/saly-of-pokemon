use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::types::{ElementType, MoveCategory, RarityTier, StatusType};

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub name: String,
    pub element: ElementType,
    pub category: MoveCategory,
    pub rarity: RarityTier,
    pub energy_cost: u8,
    pub multiplier: f32,
    pub upgraded: bool,
    pub secondary_status: Option<(StatusType, i32)>,
    #[serde(default)]
    pub status_chance: Option<f32>,
    pub secondary_block_multiplier: f32,
    pub exhausts: bool,
    pub targets_all: bool,
    pub draw_cards: u32,
    pub retain_block_ratio: f32,
    pub recoil_damage: i32,
    pub hp_cost_percent: f32,
    pub crit_chance: f32,
    pub scry_count: u32,
    pub ignores_block: bool,
    pub strikes_lowest_defense: bool,
    pub extra_energy_next_turn: i8,
}

impl Card {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        element: ElementType,
        category: MoveCategory,
        rarity: RarityTier,
        energy_cost: u8,
        multiplier: f32,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            element,
            category,
            rarity,
            energy_cost,
            multiplier,
            upgraded: false,
            secondary_status: None,
            status_chance: None,
            secondary_block_multiplier: 0.0,
            exhausts: false,
            targets_all: false,
            draw_cards: 0,
            retain_block_ratio: 0.0,
            recoil_damage: 0,
            hp_cost_percent: 0.0,
            crit_chance: 0.0,
            scry_count: 0,
            ignores_block: false,
            strikes_lowest_defense: false,
            extra_energy_next_turn: 0,
        }
    }

    /// Retorna el nombre visible (añadiendo '+' si está entrenada)
    pub fn display_name(&self) -> String {
        if self.upgraded {
            format!("{}+", self.name)
        } else {
            self.name.clone()
        }
    }

    /// Probabilidad de acierto del estado secundario (por defecto: 60% para ataques ofensivos, 75% para movimientos de habilidad/skill)
    pub fn get_status_chance(&self) -> f32 {
        self.status_chance.unwrap_or(match self.category {
            MoveCategory::Skill => 0.75,
            _ => 0.60,
        })
    }

    /// Descripción formateada del efecto de la carta, indicando multiplicador, estado y probabilidad de acierto
    pub fn description(&self) -> String {
        let base = match self.category {
            MoveCategory::Physical | MoveCategory::Special => format!("Daño: {:.1}x", self.multiplier),
            MoveCategory::Defense => format!("Bloqueo: {:.1}x", self.multiplier),
            _ => format!("{:?}", self.category),
        };
        if let Some((status, stacks)) = self.secondary_status {
            if status.is_debuff() {
                format!("{}\n{}: {} ({}%)", base, status.display_name(), stacks, (self.get_status_chance() * 100.0) as u32)
            } else {
                format!("{}\n{}: {}", base, status.display_name(), stacks)
            }
        } else {
            base
        }
    }

    /// Entrena la carta en el Centro Pokémon a su versión '+' según el GDD
    pub fn upgrade(&mut self) {
        if self.upgraded {
            return;
        }
        self.upgraded = true;
        match self.id.as_str() {
            "tackle" => self.multiplier = 1.35,
            "shelter" => self.multiplier = 1.35,
            "quick_attack" => {
                self.multiplier = 0.85;
                self.draw_cards = 1;
            }
            "ember" => {
                self.multiplier = 1.4;
                self.secondary_status = Some((StatusType::Burn, 4));
            }
            "fire_fang" => {
                self.multiplier = 1.5;
                self.secondary_status = Some((StatusType::Burn, 3));
            }
            "water_gun" => {
                self.multiplier = 1.4;
                self.secondary_block_multiplier = 0.6;
            }
            "vine_whip" => {
                self.multiplier = 1.4;
                self.secondary_status = Some((StatusType::LeechSeed, 4));
            }
            "thunder_shock" => {
                self.multiplier = 1.35;
                self.secondary_status = Some((StatusType::Paralysis, 2));
            }
            "double_team" => {
                self.multiplier = 1.4;
                self.draw_cards = 2;
            }
            "fire_spin" => {
                self.multiplier = 1.1;
                self.secondary_status = Some((StatusType::Burn, 5));
            }
            "aqua_jet" => {
                self.multiplier = 1.0;
                self.retain_block_ratio = 0.8;
            }
            "leech_seed" => {
                self.secondary_status = Some((StatusType::LeechSeed, 6));
                self.secondary_block_multiplier = 0.5;
            }
            "thunder_wave" => {
                self.energy_cost = 0;
                self.secondary_status = Some((StatusType::Paralysis, 4));
            }
            "rock_slide" => {
                self.multiplier = 2.1;
                self.secondary_block_multiplier = 0.9;
            }
            "psybeam" => {
                self.multiplier = 1.6;
                self.scry_count = 4;
            }
            "swords_dance" => {
                self.multiplier = 0.50; // +50% Atk
            }
            "nasty_plot" => {
                self.multiplier = 0.50; // +50% Sp. Atk
            }
            "iron_defense" => {
                self.multiplier = 2.8;
            }
            "flamethrower" => {
                self.multiplier = 2.8;
                self.secondary_status = Some((StatusType::Burn, 9));
            }
            "surf" => {
                self.multiplier = 2.4;
                self.secondary_block_multiplier = 1.1;
            }
            "leaf_blade" => {
                self.multiplier = 1.8;
                self.crit_chance = 1.0;
            }
            "thunderbolt" => {
                self.multiplier = 2.6;
            }
            "substitute" => {
                self.hp_cost_percent = 0.15;
            }
            "belly_drum" => {
                self.energy_cost = 1;
                self.hp_cost_percent = 0.25;
            }
            "trick_room" => {
                self.energy_cost = 1;
            }
            "mirror_coat" => {
                self.multiplier = 2.0;
            }
            "flare_blitz" => {
                self.multiplier = 4.0;
                self.recoil_damage = 3;
            }
            "blast_burn" => {
                self.multiplier = 5.2;
                self.secondary_status = Some((StatusType::Burn, 16));
                self.extra_energy_next_turn = 0; // Sin penalización de energía en +
            }
            "hydro_cannon" => {
                self.multiplier = 5.0;
                self.secondary_block_multiplier = 2.8;
            }
            "frenzy_plant" => {
                self.multiplier = 4.8;
                self.secondary_status = Some((StatusType::Poison, 15));
            }
            "draco_meteor" => {
                self.multiplier = 6.0;
            }
            "psystrike" => {
                self.multiplier = 4.4;
                self.draw_cards = 2;
            }
            _ => {
                self.multiplier *= 1.25;
            }
        }
    }
}

// Marcadores de ubicación de la carta en el ECS
#[derive(Component, Debug, Clone, Copy)]
pub struct InDrawPile;

#[derive(Component, Debug, Clone, Copy)]
pub struct InHand;

#[derive(Component, Debug, Clone, Copy)]
pub struct InDiscardPile;

#[derive(Component, Debug, Clone, Copy)]
pub struct Exhausted;

/// Crea el catálogo de las 30 cartas oficiales definidas en la Sección 5.6 del GDD
pub fn create_master_card_catalog() -> Vec<Card> {
    vec![
        // Tier 1: Común (8 cartas)
        Card::new("tackle", "Placaje", ElementType::Normal, MoveCategory::Physical, RarityTier::Common, 1, 1.0),
        Card::new("shelter", "Refugio", ElementType::Normal, MoveCategory::Defense, RarityTier::Common, 1, 1.0),
        {
            let mut c = Card::new("quick_attack", "Ataque Rápido", ElementType::Normal, MoveCategory::Physical, RarityTier::Common, 0, 0.6);
            c.draw_cards = 1;
            c
        },
        {
            let mut c = Card::new("ember", "Ascuas", ElementType::Fire, MoveCategory::Special, RarityTier::Common, 1, 1.1);
            c.secondary_status = Some((StatusType::Burn, 2));
            c
        },
        {
            let mut c = Card::new("fire_fang", "Colmillo Ígneo", ElementType::Fire, MoveCategory::Physical, RarityTier::Common, 1, 1.15);
            c.secondary_status = Some((StatusType::Burn, 2));
            c
        },
        {
            let mut c = Card::new("water_gun", "Pistola Agua", ElementType::Water, MoveCategory::Special, RarityTier::Common, 1, 1.1);
            c.secondary_block_multiplier = 0.4;
            c
        },
        {
            let mut c = Card::new("vine_whip", "Látigo Cepa", ElementType::Grass, MoveCategory::Physical, RarityTier::Common, 1, 1.1);
            c.secondary_status = Some((StatusType::LeechSeed, 2));
            c
        },
        {
            let mut c = Card::new("thunder_shock", "Impactrueno", ElementType::Electric, MoveCategory::Special, RarityTier::Common, 1, 1.0);
            c.secondary_status = Some((StatusType::Paralysis, 1));
            c
        },

        // Tier 2: Poco Común (7 cartas)
        {
            let mut c = Card::new("double_team", "Doble Equipo", ElementType::Normal, MoveCategory::Defense, RarityTier::Uncommon, 1, 1.1);
            c.draw_cards = 1;
            c
        },
        {
            let mut c = Card::new("fire_spin", "Giro Fuego", ElementType::Fire, MoveCategory::Special, RarityTier::Uncommon, 1, 0.8);
            c.targets_all = true;
            c.secondary_status = Some((StatusType::Burn, 3));
            c
        },
        {
            let mut c = Card::new("aqua_jet", "Acua Jet", ElementType::Water, MoveCategory::Physical, RarityTier::Uncommon, 0, 0.75);
            c.retain_block_ratio = 0.5;
            c
        },
        {
            let mut c = Card::new("leech_seed", "Drenadoras", ElementType::Grass, MoveCategory::Skill, RarityTier::Uncommon, 1, 0.0);
            c.secondary_status = Some((StatusType::LeechSeed, 4));
            c.exhausts = true;
            c
        },
        {
            let mut c = Card::new("thunder_wave", "Onda Trueno", ElementType::Electric, MoveCategory::Skill, RarityTier::Uncommon, 1, 0.0);
            c.secondary_status = Some((StatusType::Paralysis, 3));
            c
        },
        {
            let mut c = Card::new("rock_slide", "Avalancha", ElementType::Rock, MoveCategory::Physical, RarityTier::Uncommon, 2, 1.6);
            c.targets_all = true;
            c.secondary_block_multiplier = 0.6;
            c
        },
        {
            let mut c = Card::new("psybeam", "Psicorrayo", ElementType::Psychic, MoveCategory::Special, RarityTier::Uncommon, 1, 1.25);
            c.scry_count = 2;
            c
        },

        // Tier 3: Raro (7 cartas)
        {
            let mut c = Card::new("swords_dance", "Danza Espada", ElementType::Normal, MoveCategory::Power, RarityTier::Rare, 1, 0.30);
            c.secondary_status = Some((StatusType::SwordDance, 1));
            c
        },
        {
            let mut c = Card::new("nasty_plot", "Maquinación", ElementType::Ghost, MoveCategory::Power, RarityTier::Rare, 1, 0.30);
            c.secondary_status = Some((StatusType::NastyPlot, 1));
            c
        },
        {
            let mut c = Card::new("iron_defense", "Defensa Férrea", ElementType::Steel, MoveCategory::Defense, RarityTier::Rare, 2, 2.2);
            c.secondary_status = Some((StatusType::IronDefense, 1));
            c
        },
        {
            let mut c = Card::new("flamethrower", "Lanzallamas", ElementType::Fire, MoveCategory::Special, RarityTier::Rare, 2, 2.2);
            c.secondary_status = Some((StatusType::Burn, 6));
            c
        },
        {
            let mut c = Card::new("surf", "Surf", ElementType::Water, MoveCategory::Special, RarityTier::Rare, 2, 1.9);
            c.targets_all = true;
            c.secondary_block_multiplier = 0.8;
            c
        },
        {
            let mut c = Card::new("leaf_blade", "Hoja Aguda", ElementType::Grass, MoveCategory::Physical, RarityTier::Rare, 1, 1.5);
            c.crit_chance = 0.50;
            c
        },
        Card::new("thunderbolt", "Rayo", ElementType::Electric, MoveCategory::Special, RarityTier::Rare, 2, 2.1),

        // Tier 4: Exótico (5 cartas)
        {
            let mut c = Card::new("substitute", "Sustituto", ElementType::Normal, MoveCategory::Skill, RarityTier::Exotic, 1, 2.0);
            c.hp_cost_percent = 0.20;
            c.exhausts = true;
            c
        },
        {
            let mut c = Card::new("belly_drum", "Tambor", ElementType::Normal, MoveCategory::Power, RarityTier::Exotic, 2, 1.0);
            c.hp_cost_percent = 0.35;
            c.secondary_status = Some((StatusType::SwordDance, 3));
            c.exhausts = true;
            c
        },
        {
            let mut c = Card::new("trick_room", "Espacio Raro", ElementType::Psychic, MoveCategory::Power, RarityTier::Exotic, 2, 0.0);
            c.secondary_status = Some((StatusType::TrickRoom, 3));
            c
        },
        Card::new("mirror_coat", "Manto Espejo", ElementType::Psychic, MoveCategory::Defense, RarityTier::Exotic, 2, 1.5),
        {
            let mut c = Card::new("flare_blitz", "Envite Ígneo", ElementType::Fire, MoveCategory::Physical, RarityTier::Exotic, 2, 3.2);
            c.recoil_damage = 6;
            c
        },

        // Tier 5: Legendario (5 cartas)
        {
            let mut c = Card::new("blast_burn", "Anillo Ígneo", ElementType::Fire, MoveCategory::Special, RarityTier::Legendary, 3, 4.2);
            c.targets_all = true;
            c.secondary_status = Some((StatusType::Burn, 12));
            c.extra_energy_next_turn = -1;
            c
        },
        {
            let mut c = Card::new("hydro_cannon", "Hidrocañón", ElementType::Water, MoveCategory::Special, RarityTier::Legendary, 3, 4.0);
            c.secondary_block_multiplier = 2.0;
            c
        },
        {
            let mut c = Card::new("frenzy_plant", "Planta Feroz", ElementType::Grass, MoveCategory::Special, RarityTier::Legendary, 3, 3.8);
            c.secondary_status = Some((StatusType::Poison, 10));
            c.exhausts = true;
            c
        },
        Card::new("draco_meteor", "Cometa Draco", ElementType::Dragon, MoveCategory::Special, RarityTier::Legendary, 3, 4.5),
        {
            let mut c = Card::new("psystrike", "Onda Mental", ElementType::Psychic, MoveCategory::Special, RarityTier::Legendary, 2, 3.5);
            c.ignores_block = true;
            c.strikes_lowest_defense = true;
            c
        },
    ]
}

/// Genera el mazo inicial de 10 cartas según la Sección 5.4 del GDD
pub fn create_starter_deck(primary_type: ElementType, is_special: bool) -> Vec<Card> {
    let mut deck = Vec::new();
    // 4x Ataque Básico Neutral
    for _ in 0..4 {
        deck.push(Card::new("tackle", "Placaje", ElementType::Normal, MoveCategory::Physical, RarityTier::Common, 1, 1.0));
    }
    // 4x Defensa Básica
    for _ in 0..4 {
        deck.push(Card::new("shelter", "Refugio", ElementType::Normal, MoveCategory::Defense, RarityTier::Common, 1, 1.0));
    }
    // 1x Movimiento Elemental Inicial
    match primary_type {
        ElementType::Fire => {
            let mut c = Card::new("ember", "Ascuas", ElementType::Fire, MoveCategory::Special, RarityTier::Common, 1, 1.1);
            c.secondary_status = Some((StatusType::Burn, 2));
            deck.push(c);
        }
        ElementType::Water => {
            let mut c = Card::new("water_gun", "Pistola Agua", ElementType::Water, MoveCategory::Special, RarityTier::Common, 1, 1.1);
            c.secondary_block_multiplier = 0.4;
            deck.push(c);
        }
        ElementType::Grass => {
            let mut c = Card::new("vine_whip", "Látigo Cepa", ElementType::Grass, MoveCategory::Physical, RarityTier::Common, 1, 1.1);
            c.secondary_status = Some((StatusType::LeechSeed, 2));
            deck.push(c);
        }
        ElementType::Electric => {
            let mut c = Card::new("thunder_shock", "Impactrueno", ElementType::Electric, MoveCategory::Special, RarityTier::Common, 1, 1.0);
            c.secondary_status = Some((StatusType::Paralysis, 1));
            deck.push(c);
        }
        _ => {
            deck.push(Card::new("quick_attack", "Ataque Rápido", ElementType::Normal, MoveCategory::Physical, RarityTier::Common, 0, 0.6));
        }
    }
    // 1x Movimiento de Estado / Utilidad Inicial
    if is_special {
        let mut c = Card::new("double_team", "Doble Equipo", ElementType::Normal, MoveCategory::Defense, RarityTier::Uncommon, 1, 1.1);
        c.draw_cards = 1;
        deck.push(c);
    } else {
        deck.push(Card::new("quick_attack", "Ataque Rápido", ElementType::Normal, MoveCategory::Physical, RarityTier::Common, 0, 0.6));
    }

    deck
}
