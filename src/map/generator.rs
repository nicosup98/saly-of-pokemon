use serde::{Deserialize, Serialize};
use rand::Rng;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum NodeType {
    Enemy,
    MiniBoss,
    PokemonCenter,
    Shop,
    Treasure,
    GymLeader,
}

impl NodeType {
    pub fn name(&self) -> &'static str {
        match self {
            NodeType::Enemy => "Entrenador Común",
            NodeType::MiniBoss => "Mini Jefe (Pokémon Salvaje)",
            NodeType::PokemonCenter => "Centro Pokémon",
            NodeType::Shop => "Tienda Poké Mart",
            NodeType::Treasure => "Cápsula de Tesoro",
            NodeType::GymLeader => "Líder de Gimnasio",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            NodeType::Enemy => "VS",
            NodeType::MiniBoss => "ELITE",
            NodeType::PokemonCenter => "CENTRO",
            NodeType::Shop => "TIENDA",
            NodeType::Treasure => "TESORO",
            NodeType::GymLeader => "LÍDER",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapNode {
    pub floor: u8,
    pub path_index: usize, // 0, 1 o 2 (3 caminos)
    pub node_type: NodeType,
    pub connected_to: Vec<usize>, // índices del piso siguiente a los que se puede viajar
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActMap {
    pub act_number: u8,
    pub floors: Vec<Vec<MapNode>>, // 15 pisos, cada uno con hasta 3 caminos
}

/// Genera el mapa procedural de 15 pisos con 3 caminos interconectados
/// siguiendo rigurosamente las reglas del GDD (Sección 6.3):
/// 1. 3 caminos interconectados
/// 2. Máximo 3 Mini Jefes por Acto
/// 3. Separación mínima de 1 nodo entre Mini Jefes
/// 4. Centro Pokémon garantizado en el piso posterior a cada Mini Jefe
/// 5. Convergencia en el piso 15 con el Líder de Gimnasio
pub fn generate_act_map(act: u8) -> ActMap {
    let mut rng = rand::thread_rng();
    let mut floors = Vec::new();
    let mut mini_boss_count = 0;
    let mut last_floor_had_mini_boss = false;

    for floor in 1..=15 {
        let mut floor_nodes = Vec::new();

        if floor == 15 {
            // Piso 15: Convergencia de los 3 caminos en el Líder de Gimnasio
            floor_nodes.push(MapNode {
                floor,
                path_index: 1,
                node_type: NodeType::GymLeader,
                connected_to: Vec::new(),
            });
        } else {
            // Tras un Mini Jefe, se garantiza un Centro Pokémon en exactamente un camino (el central,
            // conectado con todos los caminos 0, 1 y 2), mientras los otros dos caminos ofrecen
            // alternativas reales (Combate, Tienda o Tesoro) para que el jugador tenga variedad de elección.
            let guaranteed_center_path = if last_floor_had_mini_boss { Some(1) } else { None };

            for path_index in 0..3 {
                let node_type = if floor == 1 {
                    NodeType::Enemy // El piso 1 siempre inicia con combate común
                } else if guaranteed_center_path == Some(path_index) {
                    // Centro Pokémon en un único camino accesible tras el Mini Jefe
                    NodeType::PokemonCenter
                } else {
                    let roll = rng.gen_range(0..100);
                    if roll < 20 && mini_boss_count < 3 && floor >= 5 && floor <= 12 && !last_floor_had_mini_boss {
                        mini_boss_count += 1;
                        NodeType::MiniBoss
                    } else if roll < 45 {
                        NodeType::Enemy
                    } else if roll < 65 {
                        NodeType::Shop
                    } else if roll < 80 {
                        NodeType::Treasure
                    } else if !last_floor_had_mini_boss && !floor_nodes.iter().any(|n: &MapNode| n.node_type == NodeType::PokemonCenter) {
                        NodeType::PokemonCenter
                    } else {
                        NodeType::Enemy
                    }
                };

                // Interconexión con el siguiente piso (caminos adyacentes sin cruces caóticos)
                let connected_to = if floor == 14 {
                    vec![0] // Todos convergen al único Líder de Gimnasio en el piso 15
                } else {
                    match path_index {
                        0 => vec![0, 1],
                        1 => vec![0, 1, 2],
                        _ => vec![1, 2],
                    }
                };

                floor_nodes.push(MapNode {
                    floor,
                    path_index,
                    node_type,
                    connected_to,
                });
            }

            last_floor_had_mini_boss = floor_nodes.iter().any(|n| n.node_type == NodeType::MiniBoss);
        }

        floors.push(floor_nodes);
    }

    ActMap {
        act_number: act,
        floors,
    }
}
