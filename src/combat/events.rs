use bevy::prelude::*;
use crate::types::{MoveCategory, StatusType};

#[derive(Message, Debug, Clone)]
pub struct PlayCardEvent {
    pub card_index: usize,
}

#[derive(Message, Debug, Clone)]
pub struct DealDamageEvent {
    pub attacker_name: String,
    pub target_is_player: bool,
    pub amount: f32,
    pub category: MoveCategory,
}

#[derive(Message, Debug, Clone)]
pub struct GainBlockEvent {
    pub target_is_player: bool,
    pub amount: i32,
}

#[derive(Message, Debug, Clone)]
pub struct ApplyStatusEvent {
    pub target_is_player: bool,
    pub status: StatusType,
    pub stacks: i32,
}

#[derive(Message, Debug, Clone)]
pub struct DrawCardEvent {
    pub count: u32,
}

#[derive(Message, Debug, Clone)]
pub struct HealEvent {
    pub target_is_player: bool,
    pub amount: i32,
}

/// Lado del campo de batalla sobre el que ocurre una animación.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CombatSide {
    Player,
    Enemy,
}

/// Tipo de animación visual que la UI debe reproducir.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CombatAnimKind {
    /// El Pokémon embiste hacia su oponente (ataque).
    Strike,
    /// El Pokémon recibe daño (sacudida + destello rojo). El valor es el daño total.
    Damage(i32),
    /// El Pokémon gana Bloqueo (destello azul). El valor es el Bloqueo ganado.
    Block(i32),
    /// El Pokémon recupera HP (destello verde).
    Heal(i32),
    /// El Pokémon recibe un potenciador o aplica un estado (destello amarillo).
    Buff,
    /// Animación de victoria alegre con saltos sucesivos y destello dorado.
    Victory,
    /// El Pokémon rival cae debilitado (desvanecimiento y caída).
    Faint,
}

/// Evento puramente visual: la lógica de combate lo emite y la UI lo anima.
#[derive(Message, Debug, Clone, Copy)]
pub struct CombatAnimEvent {
    pub side: CombatSide,
    pub kind: CombatAnimKind,
}

#[derive(Message, Debug, Clone)]
pub struct EndPlayerTurnEvent;

#[derive(Message, Debug, Clone)]
pub struct UseConsumableEvent {
    pub slot_index: usize,
}

/// Evento emitido al barajar el mazo (al inicio o al reciclar descarte en la pila de robo)
#[derive(Message, Debug, Clone)]
pub struct DeckShuffleAnimEvent {
    pub card_count: usize,
}
