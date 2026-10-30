use bevy::prelude::*;
use crate::map::generator::NodeType;
use crate::map::CurrentMap;
use crate::run::{AppState, CurrentRunState, MetaProfile};
use crate::save::{self, ActiveSaveSlot, SaveNotificationEvent};
use crate::ui::{CleanupUi, Button, BorderColor};

pub struct MapUiPlugin;

impl Plugin for MapUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::MapNavigation),
            // El mapa debe estar ya preparado (cambio de Acto) antes de dibujarlo.
            setup_map_ui.after(crate::map::on_enter_map),
        )
        .add_systems(
            Update,
            (
                handle_node_selection,
                handle_map_header_actions,
                node_hover_feedback,
            )
                .run_if(in_state(AppState::MapNavigation)),
        )
        .add_systems(OnExit(AppState::MapNavigation), cleanup_map_ui);
    }
}

#[derive(Component)]
pub enum MapHeaderAction {
    SaveGame,
    BackToMenu,
}

#[derive(Component)]
pub struct MapNodeButton {
    /// Posición del nodo dentro de su piso.
    pub index: usize,
    pub node_type: NodeType,
    /// Solo se puede viajar a los nodos conectados con el nodo anterior.
    pub selectable: bool,
}

const CARD_BG: Color = Color::srgb(0.14, 0.16, 0.22);
const CARD_BG_HOVER: Color = Color::srgb(0.23, 0.26, 0.35);
const CARD_BG_LOCKED: Color = Color::srgb(0.08, 0.09, 0.12);

pub fn node_color(node_type: NodeType) -> Color {
    match node_type {
        NodeType::Enemy => Color::srgb(0.65, 0.25, 0.25),
        NodeType::MiniBoss => Color::srgb(0.85, 0.35, 0.15),
        NodeType::Shop => Color::srgb(0.20, 0.55, 0.85),
        NodeType::PokemonCenter => Color::srgb(0.25, 0.75, 0.35),
        NodeType::Treasure => Color::srgb(0.85, 0.75, 0.20),
        NodeType::GymLeader => Color::srgb(0.75, 0.20, 0.85),
    }
}

fn node_description(node_type: NodeType) -> &'static str {
    match node_type {
        NodeType::Enemy => "Combate contra un entrenador. Recompensa en P$ y EXP.",
        NodeType::MiniBoss => "Pokémon salvaje dominante. Suelta un Fragmento para el Gachapón.",
        NodeType::Shop => "Compra cartas, reliquias y consumibles.",
        NodeType::PokemonCenter => "Cura el 35% de tu HP o entrena una carta a '+'.",
        NodeType::Treasure => "Cápsula con P$ y una reliquia.",
        NodeType::GymLeader => "Líder de Gimnasio. Vence para ganar la Medalla.",
    }
}

fn setup_map_ui(
    mut commands: Commands,
    run_state: Res<CurrentRunState>,
    map: Res<CurrentMap>,
    active_slot: Option<Res<ActiveSaveSlot>>,
) {
    let current_slot = active_slot.as_ref().map(|s| s.0).unwrap_or(1);
    let current_floor = (run_state.current_floor as usize).clamp(1, 15);
    let floor_nodes = &map.act_map.floors[current_floor - 1];
    let reachable = map.reachable_indices(current_floor as u8);

    commands
        .spawn((
            CleanupUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.10, 0.14)),
        ))
        .with_children(|root| {
            // Cabecera con botones de Guardado y Menú
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|top_bar| {
                // Botón Menú Principal
                top_bar
                    .spawn((
                        Button,
                        MapHeaderAction::BackToMenu,
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                            border: UiRect::all(Val::Px(1.5)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.20, 0.22, 0.30)),
                        BorderColor(Color::srgb(0.5, 0.55, 0.7)),
                    ))
                    .with_child((
                        Text::new("<- MENÚ PRINCIPAL"),
                        TextFont::from_font_size(12.5),
                        TextColor(Color::WHITE),
                    ));

                // Títulos centrales
                top_bar
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(4.0),
                        ..default()
                    })
                    .with_children(|header| {
                        header.spawn((
                            Text::new(format!(
                                "MAPA DE RUTA - ACTO {} (Perfil #{})",
                                run_state.current_act, current_slot
                            )),
                            TextFont::from_font_size(24.0),
                            TextColor(Color::srgb(0.4, 0.8, 1.0)),
                        ));
                        header.spawn((
                            Text::new(format!(
                                "Piso {}/15 | Elige tu próximo destino ({} camino(s) disponible(s))",
                                current_floor,
                                reachable.len()
                            )),
                            TextFont::from_font_size(14.5),
                            TextColor(Color::srgb(0.8, 0.8, 0.8)),
                        ));
                    });

                // Botón Guardar Partida
                top_bar
                    .spawn((
                        Button,
                        MapHeaderAction::SaveGame,
                        Node {
                            padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.18, 0.45, 0.28)),
                        BorderColor(Color::srgb(0.35, 0.90, 0.55)),
                    ))
                    .with_child((
                        Text::new(format!("GUARDAR EN PERFIL #{}", current_slot)),
                        TextFont::from_font_size(12.5),
                        TextColor(Color::WHITE),
                    ));
            });

            // Progreso del Acto: 15 pisos (recorridos, actual y futuros)
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(6.0),
                justify_content: JustifyContent::Center,
                ..default()
            }).with_children(|strip| {
                for floor in 1..=15usize {
                    let (bg, border, text_col) = if floor < current_floor {
                        let chosen = map.path_history.get(floor - 1).copied().unwrap_or(0);
                        let node_type = map.node_type_at(floor as u8, chosen);
                        (node_color(node_type), Color::srgb(0.1, 0.1, 0.1), Color::WHITE)
                    } else if floor == current_floor {
                        (Color::srgb(0.20, 0.24, 0.32), Color::WHITE, Color::WHITE)
                    } else if floor == 15 {
                        (Color::srgb(0.16, 0.10, 0.20), Color::srgb(0.75, 0.20, 0.85), Color::srgb(0.8, 0.6, 0.9))
                    } else {
                        (Color::srgb(0.11, 0.12, 0.16), Color::srgb(0.2, 0.22, 0.28), Color::srgb(0.5, 0.5, 0.55))
                    };

                    strip.spawn((
                        Node {
                            width: Val::Px(52.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(bg),
                        BorderColor(border),
                    )).with_child((
                        Text::new(format!("{}", floor)),
                        TextFont::from_font_size(14.0),
                        TextColor(text_col),
                    ));
                }
            });

            // Caminos disponibles para el piso actual
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(24.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            }).with_children(|paths_row| {
                for (index, node) in floor_nodes.iter().enumerate() {
                    let selectable = reachable.contains(&index);
                    let col = node_color(node.node_type);
                    let (bg, border, text_col) = if selectable {
                        (CARD_BG, col, Color::WHITE)
                    } else {
                        (CARD_BG_LOCKED, Color::srgb(0.2, 0.2, 0.24), Color::srgb(0.4, 0.4, 0.45))
                    };

                    paths_row.spawn((
                        Button,
                        MapNodeButton {
                            index,
                            node_type: node.node_type,
                            selectable,
                        },
                        Node {
                            width: Val::Px(210.0),
                            height: Val::Px(250.0),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(16.0)),
                            border: UiRect::all(Val::Px(3.0)),
                            ..default()
                        },
                        BackgroundColor(bg),
                        BorderColor(border),
                    )).with_children(|card| {
                        card.spawn((
                            Node {
                                padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(if selectable { col.with_alpha(0.2) } else { Color::srgba(0.2, 0.2, 0.2, 0.2) }),
                            BorderColor(border),
                        )).with_child((
                            Text::new(format!("[ {} ]", node.node_type.icon())),
                            TextFont::from_font_size(20.0),
                            TextColor(text_col),
                        ));

                        card.spawn((
                            Text::new(node.node_type.name()),
                            TextFont::from_font_size(16.0),
                            TextColor(text_col),
                        ));

                        card.spawn((
                            Text::new(node_description(node.node_type)),
                            TextFont::from_font_size(11.5),
                            TextColor(if selectable { Color::srgb(0.75, 0.78, 0.85) } else { text_col }),
                            Node {
                                width: Val::Px(170.0),
                                ..default()
                            },
                        ));

                        card.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(32.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(if selectable { col } else { Color::srgb(0.15, 0.15, 0.18) }),
                        )).with_child((
                            Text::new(if selectable { "SELECCIONAR" } else { "SIN CONEXION" }),
                            TextFont::from_font_size(12.0),
                            TextColor(text_col),
                        ));
                    });
                }
            });

            // Pie de página
            root.spawn((
                Text::new("Tras cada Mini Jefe hay un Centro Pokémon garantizado en una de las rutas. Solo puedes avanzar a nodos conectados con tu posición actual."),
                TextFont::from_font_size(13.0),
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

use crate::ui::transition_ui::TransitionToEvent;

fn handle_node_selection(
    q_buttons: Query<(&Interaction, &MapNodeButton), Changed<Interaction>>,
    mut map: ResMut<CurrentMap>,
    mut run_state: ResMut<CurrentRunState>,
    mut ev_trans: MessageWriter<TransitionToEvent>,
) {
    for (interaction, btn) in q_buttons.iter() {
        if *interaction != Interaction::Pressed || !btn.selectable {
            continue;
        }

        map.current_path_index = btn.index;
        map.path_history.push(btn.index);
        let floor = run_state.current_floor;
        run_state.add_log(format!(
            "[RUTA] Piso {}: eliges {} {}.",
            floor,
            btn.node_type.icon(),
            btn.node_type.name()
        ));

        let target = match btn.node_type {
            NodeType::Enemy | NodeType::MiniBoss | NodeType::GymLeader => AppState::Combat,
            NodeType::Shop => AppState::Shop,
            NodeType::PokemonCenter => AppState::PokemonCenter,
            NodeType::Treasure => AppState::Treasure,
        };
        ev_trans.write(TransitionToEvent { target });
        break;
    }
}

fn handle_map_header_actions(
    q_actions: Query<(&Interaction, &MapHeaderAction), Changed<Interaction>>,
    active_slot: Option<Res<ActiveSaveSlot>>,
    meta: Res<MetaProfile>,
    run_state: Res<CurrentRunState>,
    map: Res<CurrentMap>,
    mut ev_notify: MessageWriter<SaveNotificationEvent>,
    mut ev_trans: MessageWriter<TransitionToEvent>,
) {
    let slot = active_slot.as_ref().map(|s| s.0).unwrap_or(1);
    for (interaction, action) in q_actions.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            MapHeaderAction::SaveGame => {
                match save::save_profile(slot, &meta, &run_state, Some(&map)) {
                    Ok(_) => {
                        ev_notify.write(SaveNotificationEvent {
                            message: format!(
                                "[GUARDADO] Partida guardada en el Perfil #{} (Acto {} - Piso {})",
                                slot, run_state.current_act, run_state.current_floor
                            ),
                            is_error: false,
                        });
                    }
                    Err(e) => {
                        ev_notify.write(SaveNotificationEvent {
                            message: format!("[ERROR] Error al guardar: {}", e),
                            is_error: true,
                        });
                    }
                }
            }
            MapHeaderAction::BackToMenu => {
                ev_trans.write(TransitionToEvent {
                    target: AppState::MainMenu,
                });
            }
        }
    }
}

/// Resalta el nodo bajo el cursor (solo si se puede elegir).
fn node_hover_feedback(
    mut q_buttons: Query<(&Interaction, &MapNodeButton, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (interaction, btn, mut bg) in q_buttons.iter_mut() {
        if !btn.selectable {
            continue;
        }
        bg.0 = match interaction {
            Interaction::Hovered | Interaction::Pressed => CARD_BG_HOVER,
            Interaction::None => CARD_BG,
        };
    }
}

fn cleanup_map_ui(mut commands: Commands, q_cleanup: Query<Entity, With<CleanupUi>>) {
    for entity in q_cleanup.iter() {
        commands.entity(entity).despawn();
    }
}
