use bevy::prelude::*;
use rand::Rng;
use crate::run::{AppState, MetaProfile};
pub use crate::pokemon::species_rarity;

pub struct GachaponPlugin;

impl Plugin for GachaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::GachaponShop), on_enter_gachapon);
    }
}

pub fn on_enter_gachapon(meta: Res<MetaProfile>) {
    println!("\n[GACHAPON] === TIENDA GACHAPÓN ===");
    println!("Fichas disponibles: {} Fichas Gachapón", meta.gachapon_tokens);
    println!("Pokémon desbloqueados: {:?}", meta.unlocked_pokemon);
    println!("Fragmentos acumulados: {:?}", meta.pokemon_fragments);
    println!("Tirada Simple: 100 Fichas | Pack x10: 900 Fichas\n");
}

/// Realiza una tirada simple en el Gachapón con soporte de fragmentos según GDD 8.2
pub fn pull_gachapon(meta: &mut MetaProfile, target_species: Option<u32>, fragments_used: u32) -> Option<u32> {
    if meta.gachapon_tokens < 100 {
        return None;
    }
    meta.gachapon_tokens -= 100;

    let mut rng = rand::thread_rng();

    // Si se insertaron fragmentos para un Pokémon objetivo
    if let Some(target_id) = target_species {
        let frags = fragments_used.min(3);
        if let Some(current_frags) = meta.pokemon_fragments.get_mut(&target_id) {
            let used = (*current_frags).min(frags);
            *current_frags -= used;

            // Probabilidad focalizada según la tabla del GDD 8.2 (ej. Raro 12% -> 20% -> 35% -> 55%)
            let targeted_chance = match used {
                1 => 20.0,
                2 => 35.0,
                3 => 55.0,
                _ => 12.0,
            };

            let roll: f32 = rng.gen_range(0.0..100.0);
            if roll < targeted_chance {
                if !meta.unlocked_pokemon.contains(&target_id) {
                    meta.unlocked_pokemon.push(target_id);
                }
                return Some(target_id);
            }
        }
    }

    // Tirada normal general
    let roll: f32 = rng.gen_range(0.0..100.0);
    let pulled_id = if roll < 55.0 {
        // Común (ej. Rattata)
        19
    } else if roll < 83.0 {
        // Poco Común (ej. Geodude)
        74
    } else if roll < 95.0 {
        // Raro (ej. Squirtle o Bulbasaur)
        if meta.unlocked_pokemon.contains(&7) { 1 } else { 7 }
    } else if roll < 99.5 {
        // Exótico (Scyther)
        123
    } else {
        // Legendario (Mewtwo)
        150
    };

    if !meta.unlocked_pokemon.contains(&pulled_id) {
        meta.unlocked_pokemon.push(pulled_id);
    }

    Some(pulled_id)
}
