use std::fs;
use std::path::PathBuf;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use bevy_save::dir::get_save_file;
use crate::map::CurrentMap;
use crate::run::{CurrentRunState, MetaProfile};

pub const TOTAL_SLOTS: u8 = 3;

/// Ranura de guardado actualmente activa (1, 2 o 3)
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveSaveSlot(pub u8);

impl Default for ActiveSaveSlot {
    fn default() -> Self {
        Self(1)
    }
}

/// Estructura completa de guardado para una ranura
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveProfile {
    pub slot: u8,
    pub name: String,
    pub updated_at: String,
    pub meta: MetaProfile,
    pub run_state: CurrentRunState,
    pub map: Option<CurrentMap>,
}

/// Resumen ligero para listado y selección de perfiles en UI
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileSummary {
    pub slot: u8,
    pub name: String,
    pub updated_at: String,
    pub starter_id: u32,
    pub current_act: u8,
    pub current_floor: u8,
    pub poke_dollars: u32,
    pub gachapon_tokens: u32,
    pub unlocked_count: usize,
}

pub struct SavePlugin;

/// Evento global para mostrar una notificación visual flotante al guardar o cargar partida
#[derive(Message, Debug, Clone)]
pub struct SaveNotificationEvent {
    pub message: String,
    pub is_error: bool,
}

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveSaveSlot>()
            .add_message::<SaveNotificationEvent>();
    }
}

/// Obtiene la ruta del archivo de guardado usando `bevy_save` con fallback local
pub fn get_profile_path(slot: u8) -> PathBuf {
    let filename = format!("profile_{}.json", slot);
    let bevy_save_path = get_save_file(&filename);

    if let Some(parent) = bevy_save_path.parent() {
        if fs::create_dir_all(parent).is_ok() {
            return bevy_save_path;
        }
    }

    // Fallback en directorio local del juego
    let local_dir = PathBuf::from("saves");
    let _ = fs::create_dir_all(&local_dir);
    local_dir.join(filename)
}

/// Guarda la partida actual en la ranura especificada
pub fn save_profile(
    slot: u8,
    meta: &MetaProfile,
    run: &CurrentRunState,
    map: Option<&CurrentMap>,
) -> Result<(), String> {
    if !(1..=TOTAL_SLOTS).contains(&slot) {
        return Err(format!("Ranura {} inválida (debe ser 1, 2 o 3)", slot));
    }

    let path = get_profile_path(slot);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let now_str = {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let mins = (secs / 60) % 60;
        let hours = (secs / 3600) % 24;
        let days = secs / 86400;
        format!("Día {} - {:02}:{:02} UTC", days % 365, hours, mins)
    };

    let profile = SaveProfile {
        slot,
        name: format!("Perfil {}", slot),
        updated_at: now_str,
        meta: meta.clone(),
        run_state: run.clone(),
        map: map.cloned(),
    };

    let json = serde_json::to_string_pretty(&profile)
        .map_err(|e| format!("Error al serializar partida: {}", e))?;

    fs::write(&path, json)
        .map_err(|e| format!("Error al escribir archivo de guardado {:?}: {}", path, e))?;

    Ok(())
}

/// Carga la partida desde la ranura indicada
pub fn load_profile(slot: u8) -> Result<SaveProfile, String> {
    if !(1..=TOTAL_SLOTS).contains(&slot) {
        return Err(format!("Ranura {} inválida (debe ser 1, 2 o 3)", slot));
    }

    let path = get_profile_path(slot);
    if !path.exists() {
        return Err(format!("El archivo de guardado no existe: {:?}", path));
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Error al leer {:?}: {}", path, e))?;

    let profile: SaveProfile = serde_json::from_str(&content)
        .map_err(|e| format!("Error al deserializar {:?}: {}", path, e))?;

    Ok(profile)
}

/// Elimina o reinicia el archivo de guardado de una ranura
pub fn delete_profile(slot: u8) -> Result<(), String> {
    let path = get_profile_path(slot);
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| format!("Error al eliminar {:?}: {}", path, e))?;
    }
    Ok(())
}

/// Devuelve el resumen de las 3 ranuras de guardado
pub fn list_profiles() -> [Option<ProfileSummary>; 3] {
    let mut summaries = [None, None, None];
    for slot in 1..=TOTAL_SLOTS {
        if let Ok(p) = load_profile(slot) {
            summaries[(slot - 1) as usize] = Some(ProfileSummary {
                slot,
                name: p.name,
                updated_at: p.updated_at,
                starter_id: p.run_state.selected_starter_id,
                current_act: p.run_state.current_act,
                current_floor: p.run_state.current_floor,
                poke_dollars: p.run_state.poke_dollars,
                gachapon_tokens: p.meta.gachapon_tokens,
                unlocked_count: p.meta.unlocked_pokemon.len(),
            });
        }
    }
    summaries
}
