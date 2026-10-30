pub mod generator;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use self::generator::{generate_act_map, ActMap, NodeType};
use crate::run::{AppState, CurrentRunState};

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct CurrentMap {
    pub act_map: ActMap,
    /// Índice (dentro de su piso) del último nodo elegido por el jugador.
    pub current_path_index: usize,
    /// Índice del nodo elegido en cada piso ya recorrido del Acto actual.
    pub path_history: Vec<usize>,
}

impl Default for CurrentMap {
    fn default() -> Self {
        Self {
            act_map: generate_act_map(1),
            current_path_index: 0,
            path_history: Vec::new(),
        }
    }
}

impl CurrentMap {
    /// Tipo del nodo situado en `floor` (1..=15) y posición `index` dentro del piso.
    pub fn node_type_at(&self, floor: u8, index: usize) -> NodeType {
        let nodes = &self.act_map.floors[(floor as usize).saturating_sub(1).min(14)];
        nodes[index.min(nodes.len() - 1)].node_type
    }

    /// Índices de los nodos del piso `floor` a los que el jugador puede viajar.
    ///
    /// En el primer piso del Acto se puede elegir cualquier camino; a partir de ahí
    /// solo los nodos conectados con el nodo elegido en el piso anterior.
    pub fn reachable_indices(&self, floor: u8) -> Vec<usize> {
        let floor_idx = (floor as usize).saturating_sub(1).min(14);
        let count = self.act_map.floors[floor_idx].len();
        if floor_idx == 0 {
            return (0..count).collect();
        }
        let prev_nodes = &self.act_map.floors[floor_idx - 1];
        let prev = &prev_nodes[self.current_path_index.min(prev_nodes.len() - 1)];
        prev.connected_to.iter().copied().filter(|&i| i < count).collect()
    }

    /// Reinicia el recorrido con un mapa nuevo para el Acto indicado.
    pub fn start_act(&mut self, act: u8) {
        self.act_map = generate_act_map(act);
        self.current_path_index = 0;
        self.path_history.clear();
    }
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentMap>()
            .add_systems(OnEnter(AppState::MapNavigation), on_enter_map);
    }
}

/// Prepara el mapa al entrar en la pantalla de navegación.
///
/// No decide el siguiente nodo: la elección corresponde al jugador (ver `ui::map_ui`).
pub fn on_enter_map(
    mut run_state: ResMut<CurrentRunState>,
    mut map: ResMut<CurrentMap>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // Si completó los 15 pisos del acto, avanza al siguiente Acto
    if run_state.current_floor as usize > 15 {
        run_state.current_act += 1;
        run_state.current_floor = 1;
        let act_num = run_state.current_act;
        if act_num > 3 {
            next_state.set(AppState::RunSummary);
            return;
        }
        map.start_act(act_num);
        run_state.add_log(format!("[MAPA] ¡Has alcanzado el Acto {}!", act_num));
    }

    let floor = run_state.current_floor;
    run_state.add_log(format!("[RUTA] Piso {}/15 - Elige tu próximo destino.", floor));
}
