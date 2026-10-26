use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Reflect)]
pub enum ElementType {
    Normal,
    Fire,
    Water,
    Grass,
    Electric,
    Ice,
    Fighting,
    Poison,
    Ground,
    Flying,
    Psychic,
    Bug,
    Rock,
    Ghost,
    Dragon,
    Dark,
    Steel,
    Fairy,
}

impl ElementType {
    pub fn name(&self) -> &'static str {
        match self {
            ElementType::Normal => "Normal",
            ElementType::Fire => "Fuego",
            ElementType::Water => "Agua",
            ElementType::Grass => "Planta",
            ElementType::Electric => "Eléctrico",
            ElementType::Ice => "Hielo",
            ElementType::Fighting => "Lucha",
            ElementType::Poison => "Veneno",
            ElementType::Ground => "Tierra",
            ElementType::Flying => "Volador",
            ElementType::Psychic => "Psíquico",
            ElementType::Bug => "Bicho",
            ElementType::Rock => "Roca",
            ElementType::Ghost => "Fantasma",
            ElementType::Dragon => "Dragón",
            ElementType::Dark => "Siniestro",
            ElementType::Steel => "Acero",
            ElementType::Fairy => "Hada",
        }
    }

    /// Retorna el multiplicador de efectividad elemental según el GDD:
    /// - Súper eficaz: x1.5
    /// - Neutro: x1.0
    /// - No muy eficaz: x0.65
    /// - Inmunidad clásica adaptada: x0.25 (reducción severa)
    pub fn effectiveness_against(&self, defender: ElementType) -> f32 {
        use ElementType::*;
        match (self, defender) {
            // Inmunidades clásicas adaptadas a x0.25
            (Normal, Ghost) => 0.25,
            (Fighting, Ghost) => 0.25,
            (Ghost, Normal) => 0.25,
            (Electric, Ground) => 0.25,
            (Poison, Steel) => 0.25,
            (Ground, Flying) => 0.25,
            (Psychic, Dark) => 0.25,
            (Dragon, Fairy) => 0.25,

            // Súper eficaces (x1.5)
            (Fire, Grass | Ice | Bug | Steel) => 1.5,
            (Water, Fire | Ground | Rock) => 1.5,
            (Grass, Water | Ground | Rock) => 1.5,
            (Electric, Water | Flying) => 1.5,
            (Ice, Grass | Ground | Flying | Dragon) => 1.5,
            (Fighting, Normal | Ice | Rock | Dark | Steel) => 1.5,
            (Poison, Grass | Fairy) => 1.5,
            (Ground, Fire | Electric | Poison | Rock | Steel) => 1.5,
            (Flying, Grass | Fighting | Bug) => 1.5,
            (Psychic, Fighting | Poison) => 1.5,
            (Bug, Grass | Psychic | Dark) => 1.5,
            (Rock, Fire | Ice | Flying | Bug) => 1.5,
            (Ghost, Psychic | Ghost) => 1.5,
            (Dragon, Dragon) => 1.5,
            (Dark, Psychic | Ghost) => 1.5,
            (Steel, Ice | Rock | Fairy) => 1.5,
            (Fairy, Fighting | Dragon | Dark) => 1.5,

            // No muy eficaces (x0.65)
            (Fire, Fire | Water | Rock | Dragon) => 0.65,
            (Water, Water | Grass | Dragon) => 0.65,
            (Grass, Fire | Grass | Poison | Flying | Bug | Dragon | Steel) => 0.65,
            (Electric, Electric | Grass | Dragon) => 0.65,
            (Ice, Fire | Water | Ice | Steel) => 0.65,
            (Fighting, Poison | Flying | Psychic | Bug | Fairy) => 0.65,
            (Poison, Poison | Ground | Rock | Ghost) => 0.65,
            (Ground, Grass | Bug) => 0.65,
            (Flying, Electric | Rock | Steel) => 0.65,
            (Psychic, Psychic | Steel) => 0.65,
            (Bug, Fire | Fighting | Poison | Flying | Ghost | Steel | Fairy) => 0.65,
            (Rock, Fighting | Ground | Steel) => 0.65,
            (Ghost, Dark) => 0.65,
            (Dragon, Steel) => 0.65,
            (Dark, Fighting | Dark | Fairy) => 0.65,
            (Steel, Fire | Water | Electric | Steel) => 0.65,
            (Fairy, Fire | Poison | Steel) => 0.65,
            (Normal, Rock | Steel) => 0.65,

            // Neutro por defecto
            _ => 1.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Reflect)]
pub enum MoveCategory {
    Physical, // Daño = Atk * Multiplicador vs Def objetivo
    Special,  // Daño = Sp. Atk * Multiplicador vs Sp. Def objetivo
    Defense,  // Bloqueo = Def o Sp. Def * Multiplicador
    Skill,    // Efectos de soporte, robo, estados alterados
    Power,    // Pasivas permanentes activadas por el resto del combate
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, Reflect)]
pub enum RarityTier {
    Common,
    Uncommon,
    Rare,
    Exotic,
    Legendary,
}

impl RarityTier {
    pub fn name(&self) -> &'static str {
        match self {
            RarityTier::Common => "Común",
            RarityTier::Uncommon => "Poco Común",
            RarityTier::Rare => "Raro",
            RarityTier::Exotic => "Exótico",
            RarityTier::Legendary => "Legendario",
        }
    }

    pub fn base_price(&self) -> u32 {
        match self {
            RarityTier::Common => 50,
            RarityTier::Uncommon => 75,
            RarityTier::Rare => 150,
            RarityTier::Exotic => 220,
            RarityTier::Legendary => 320,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Reflect)]
pub enum StatusType {
    Burn,        // Daño al final del turno + -20% Atk
    Chill,       // Congelación: acumulación reduce ataques
    Poison,      // Toxina progresiva ignorando Bloqueo
    LeechSeed,   // Drenadoras: quita vida y cura al atacante
    Paralysis,   // Reduce daño enemigo y +20% daño recibido
    Plating,     // Blindaje: Bloqueo persistente y contraataque
    SwordDance,  // Buff de Atk (+30% o +50%)
    NastyPlot,   // Buff de Sp. Atk (+30% o +50%)
    IronDefense, // Buff de Def (+20% o +30%)
    TrickRoom,   // Reduce coste de cartas caras a 1
}

impl StatusType {
    pub fn is_debuff(&self) -> bool {
        matches!(
            self,
            StatusType::Burn | StatusType::Chill | StatusType::Poison | StatusType::LeechSeed | StatusType::Paralysis
        )
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            StatusType::Burn => "Quemadura",
            StatusType::Chill => "Congelación",
            StatusType::Poison => "Veneno",
            StatusType::LeechSeed => "Drenadoras",
            StatusType::Paralysis => "Parálisis",
            StatusType::Plating => "Blindaje",
            StatusType::SwordDance => "Danza Espada",
            StatusType::NastyPlot => "Maquinación",
            StatusType::IronDefense => "Defensa Férrea",
            StatusType::TrickRoom => "Espacio Raro",
        }
    }
}
