use bevy::prelude::*;
use crate::pokemon::create_pokemon_from_species;
use crate::pokemon_center::upgrade_card_at_center;
use crate::run::{AppState, CurrentRunState};
use crate::ui::{element_color, CleanupUi, Button, BorderColor, CardHoverAnimation};

pub struct CenterUiPlugin;

impl Plugin for CenterUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CenterState>()
            .add_systems(OnEnter(AppState::PokemonCenter), setup_center_ui)
            .add_systems(
                Update,
                (
                    handle_heal_action,
                    handle_upgrade_card_click,
                    handle_continue_click,
                )
                    .run_if(in_state(AppState::PokemonCenter)),
            )
            .add_systems(OnExit(AppState::PokemonCenter), cleanup_center_ui);
    }
}

#[derive(Resource, Default)]
pub struct CenterState {
    pub action_taken: bool,
}

#[derive(Component)]
pub struct CenterHealButton;

#[derive(Component)]
pub struct CenterUpgradeCardButton(pub usize);

#[derive(Component)]
pub struct CenterContinueButton;

#[derive(Component)]
pub struct CenterStatusBanner;

#[derive(Component)]
pub struct CenterHpBarFill;

#[derive(Component)]
pub struct CenterHpText;

/// Nombre, nivel y HP máximo del Pokémon del jugador según el estado de la partida.
fn player_vitals(run_state: &CurrentRunState) -> (String, u32, i32) {
    match create_pokemon_from_species(run_state.selected_starter_id, run_state.player_level()) {
        Some(p) => (p.name.clone(), p.level, p.max_hp),
        None => ("Charmander".to_string(), 5, 84),
    }
}

fn setup_center_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    run_state: Res<CurrentRunState>,
    mut center_state: ResMut<CenterState>,
) {
    center_state.action_taken = false;

    let (name, level, max_hp) = player_vitals(&run_state);
    let hp = run_state.player_hp.unwrap_or(max_hp).clamp(1, max_hp);

    let sprite_path = format!("sprites/pokemon/front/{}.png", run_state.selected_starter_id);

    commands
        .spawn((
            CleanupUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Relative,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                overflow: Overflow::clip(),
                ..default()
            },
        ))
        .with_children(|root| {
            // Fondo temático del Centro Pokémon
            root.spawn((
                ImageNode::new(asset_server.load("fondos/centro_pokemon.jpeg")),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    ..default()
                },
            ));

            // Tinte de contraste suave
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.04, 0.08, 0.06, 0.40)),
            ));
            // === CABECERA ===
            root.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Px(60.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(16.0)),
                border: UiRect::bottom(Val::Px(2.0)),
                ..default()
            })
            .insert(BackgroundColor(Color::srgb(0.12, 0.20, 0.16)))
            .insert(BorderColor(Color::srgb(0.35, 0.85, 0.55)))
            .with_children(|header| {
                header.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(12.0),
                    ..default()
                }).with_children(|left| {
                    left.spawn((
                        ImageNode::new(asset_server.load("sprites/trainers/nurse_front.png")),
                        Node {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            ..default()
                        },
                    ));
                    left.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    }).with_children(|text_col| {
                        text_col.spawn((
                            Text::new("CENTRO POKÉMON - SALA DE RECUPERACIÓN"),
                            TextFont::from_font_size(20.0),
                            TextColor(Color::srgb(0.4, 0.95, 0.6)),
                        ));
                        text_col.spawn((
                            Text::new("Enfermera Joy: '¡Tu Pokémon se ve exhausto tras la batalla! Elige cómo deseas prepararlo.'"),
                            TextFont::from_font_size(13.0),
                            TextColor(Color::srgb(0.8, 0.85, 0.8)),
                        ));
                    });
                });

                header.spawn((
                    Button,
                    CenterContinueButton,
                    Node {
                        height: Val::Px(38.0),
                        padding: UiRect::horizontal(Val::Px(18.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.5)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.18, 0.35, 0.25)),
                    BorderColor(Color::srgb(0.4, 0.9, 0.6)),
                )).with_child((
                    Text::new("CONTINUAR ASCENSO ->"),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::WHITE),
                ));
            });

            // === PANEL CENTRAL: ESTADO DEL POKÉMON & 2 OPCIONES TÁCTICAS ===
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(24.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Stretch,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            }).with_children(|content| {

                // COLUMNA IZQUIERDA: POKÉMON ACTUAL
                content.spawn(Node {
                    width: Val::Percent(26.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(16.0)),
                    border: UiRect::all(Val::Px(1.5)),
                    ..default()
                })
                .insert(BackgroundColor(Color::srgb(0.11, 0.16, 0.14)))
                .insert(BorderColor(Color::srgb(0.3, 0.6, 0.4)))
                .with_children(|poke_panel| {
                    poke_panel.spawn((
                        Text::new(format!("{} (Nv. {})", name, level)),
                        TextFont::from_font_size(18.0),
                        TextColor(Color::srgb(0.9, 0.95, 0.9)),
                    ));

                    // Sprite del Pokémon
                    let texture_handle = asset_server.load(&sprite_path);
                    poke_panel.spawn((
                        ImageNode::new(texture_handle),
                        Node {
                            width: Val::Px(130.0),
                            height: Val::Px(130.0),
                            margin: UiRect::vertical(Val::Px(10.0)),
                            ..default()
                        },
                    ));

                    // Barra de Vida
                    let hp_ratio = (hp as f32 / max_hp as f32).clamp(0.0, 1.0);
                    poke_panel.spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(18.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    })
                    .insert(BackgroundColor(Color::srgb(0.18, 0.18, 0.18)))
                    .insert(BorderColor(Color::srgb(0.4, 0.4, 0.4)))
                    .with_children(|bar_bg| {
                        bar_bg.spawn((
                            CenterHpBarFill,
                            Node {
                                width: Val::Percent(hp_ratio * 100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.8, 0.3)),
                        ));
                    });

                    poke_panel.spawn((
                        CenterHpText,
                        Text::new(format!("HP: {} / {}", hp, max_hp)),
                        TextFont::from_font_size(13.0),
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));

                    poke_panel.spawn((
                        Text::new(format!("Cartas en mazo: {}", run_state.player_deck.len())),
                        TextFont::from_font_size(12.0),
                        TextColor(Color::srgb(0.7, 0.8, 0.7)),
                    ));
                });

                // COLUMNA DERECHA: LAS 2 OPCIONES (CURACIÓN O ENTRENAMIENTO)
                content.spawn(Node {
                    width: Val::Percent(70.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(14.0),
                    ..default()
                }).with_children(|options_col| {

                    // OPCIÓN 1: DESCANSAR (+35% HP)
                    options_col.spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(16.0)),
                        border: UiRect::all(Val::Px(1.5)),
                        ..default()
                    })
                    .insert(BackgroundColor(Color::srgb(0.12, 0.18, 0.15)))
                    .insert(BorderColor(Color::srgb(0.85, 0.4, 0.5)))
                    .with_children(|rest_row| {
                        rest_row.spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            width: Val::Percent(70.0),
                            ..default()
                        }).with_children(|left| {
                            left.spawn((
                                Text::new("[+] OPCIÓN 1: DESCANSAR Y CURAR (+35% HP)"),
                                TextFont::from_font_size(16.0),
                                TextColor(Color::srgb(1.0, 0.6, 0.7)),
                            ));
                            left.spawn((
                                Text::new("Tu Pokémon toma un baño termal y descansa en una camilla Silph, recuperando un 35% de su vida máxima."),
                                TextFont::from_font_size(12.0),
                                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                            ));
                        });

                        rest_row.spawn((
                            Button,
                            CenterHealButton,
                            Node {
                                width: Val::Px(150.0),
                                height: Val::Px(42.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.5)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.75, 0.25, 0.40)),
                            BorderColor(Color::srgb(1.0, 0.6, 0.7)),
                        )).with_child((
                            Text::new("DESCANSAR (+35%)"),
                            TextFont::from_font_size(13.0),
                            TextColor(Color::WHITE),
                        ));
                    });

                    // OPCIÓN 2: ENTRENAR CARTA A '+' (Listado de cartas mejorables)
                    options_col.spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        padding: UiRect::all(Val::Px(16.0)),
                        border: UiRect::all(Val::Px(1.5)),
                        ..default()
                    })
                    .insert(BackgroundColor(Color::srgb(0.12, 0.18, 0.15)))
                    .insert(BorderColor(Color::srgb(0.3, 0.7, 0.9)))
                    .with_children(|train_box| {
                        train_box.spawn((
                            Text::new("OPCIÓN 2: ENTRENAR MOVIMIENTO (MEJORAR A '+')"),
                            TextFont::from_font_size(16.0),
                            TextColor(Color::srgb(0.4, 0.85, 1.0)),
                        ));
                        train_box.spawn((
                            Text::new("Selecciona una carta de tu mazo para aumentar permanentemente su daño, bloqueo o reducir su coste de energía:"),
                            TextFont::from_font_size(12.0),
                            TextColor(Color::srgb(0.8, 0.8, 0.8)),
                        ));

                        // Fila de cartas entrenables (hasta 6 seleccionables)
                        train_box.spawn(Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(10.0),
                            flex_wrap: FlexWrap::Wrap,
                            row_gap: Val::Px(8.0),
                            ..default()
                        }).with_children(|cards_flow| {
                            let upgradeable: Vec<(usize, &crate::card::Card)> = run_state
                                .player_deck
                                .iter()
                                .enumerate()
                                .filter(|(_, c)| !c.upgraded)
                                .take(6)
                                .collect();

                            if upgradeable.is_empty() {
                                cards_flow.spawn((
                                    Text::new("[COMPLETO] ¡Todas las cartas de tu mazo ya han sido mejoradas a su versión '+'!"),
                                    TextFont::from_font_size(12.5),
                                    TextColor(Color::srgb(1.0, 0.9, 0.4)),
                                ));
                            } else {
                                for (idx, card) in upgradeable {
                                    let col = element_color(card.element);

                                    cards_flow.spawn((
                                        Button,
                                        CenterUpgradeCardButton(idx),
                                        Node {
                                            width: Val::Px(120.0),
                                            height: Val::Px(160.0),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::SpaceBetween,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(1.5)),
                                            border_radius: BorderRadius::all(Val::Px(6.0)),
                                            overflow: Overflow::clip(),
                                            ..default()
                                        },
                                        UiTransform::default(),
                                        GlobalZIndex(10),
                                        CardHoverAnimation::new(col).with_lift(-12.0).with_scale(1.08),
                                        BackgroundColor(Color::srgba(0.06, 0.08, 0.12, 0.2)),
                                        BorderColor(col),
                                    )).with_children(|c_btn| {
                                        // Fondo ilustrado del tipo de carta
                                        c_btn.spawn((
                                            ImageNode::new(asset_server.load(card.asset_path())),
                                            Node {
                                                position_type: PositionType::Absolute,
                                                left: Val::Px(0.0),
                                                right: Val::Px(0.0),
                                                top: Val::Px(0.0),
                                                bottom: Val::Px(0.0),
                                                ..default()
                                            },
                                        ));

                                        // Cabecera: Nombre y Coste (en el banner superior)
                                        c_btn.spawn(Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(26.0),
                                            margin: UiRect::top(Val::Px(8.0)),
                                            padding: UiRect::axes(Val::Px(10.0), Val::Px(2.0)),
                                            flex_direction: FlexDirection::Row,
                                            justify_content: JustifyContent::SpaceBetween,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        }).with_children(|h| {
                                            h.spawn((
                                                Text::new(card.display_name()),
                                                TextFont::from_font_size(10.5),
                                                TextColor(Color::WHITE),
                                            ));
                                            h.spawn((
                                                Text::new(format!("{} EN", card.energy_cost)),
                                                TextFont::from_font_size(10.5),
                                                TextColor(Color::srgb(1.0, 0.85, 0.2)),
                                            ));
                                        });

                                        // Espacio central para el arte elemental
                                        c_btn.spawn(Node {
                                            flex_grow: 1.0,
                                            ..default()
                                        });

                                        // Botón ENTRENAR en banner inferior
                                        c_btn.spawn((
                                            Node {
                                                width: Val::Percent(85.0),
                                                height: Val::Px(24.0),
                                                margin: UiRect::bottom(Val::Px(10.0)),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgba(0.1, 0.4, 0.2, 0.85)),
                                        )).with_child((
                                            Text::new("ENTRENAR ->"),
                                            TextFont::from_font_size(10.0),
                                            TextColor(Color::srgb(0.9, 1.0, 0.9)),
                                        ));
                                    });
                                }
                            }
                        });
                    });
                });
            });

            // === BARRA INFERIOR DE MENSAJES ===
            root.spawn((
                CenterStatusBanner,
                Text::new("[INFO] Elige una opción: Descansar para recuperar vida o Entrenar para mejorar una carta a '+'."),
                TextFont::from_font_size(13.0),
                TextColor(Color::srgb(0.5, 0.85, 0.65)),
            ));
        });
}

fn handle_heal_action(
    mut q_buttons: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<CenterHealButton>)>,
    mut center_state: ResMut<CenterState>,
    mut run_state: ResMut<CurrentRunState>,
    mut q_status: Query<&mut Text, With<CenterStatusBanner>>,
    mut q_hp_fill: Query<&mut Node, With<CenterHpBarFill>>,
    mut q_hp_text: Query<&mut Text, (With<CenterHpText>, Without<CenterStatusBanner>)>,
) {
    for (interaction, mut bg_color) in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            if center_state.action_taken {
                if let Ok(mut text) = q_status.single_mut() {
                    **text = "[!] Ya has realizado tu acción de descanso en este Centro Pokémon.".to_string();
                }
                continue;
            }

            center_state.action_taken = true;
            *bg_color = BackgroundColor(Color::srgb(0.25, 0.35, 0.30));

            // Cura el 35% del HP máximo y lo guarda para los próximos combates
            let (_, _, max_h) = player_vitals(&run_state);
            let before = run_state.player_hp.unwrap_or(max_h).clamp(1, max_h);
            let heal_val = ((max_h as f32) * 0.35).round() as i32;
            let cur_hp = (before + heal_val).min(max_h);
            run_state.player_hp = Some(cur_hp);
            let healed_amount = cur_hp - before;

            let msg = format!("[CURACIÓN] ¡Descanso completado! Tu Pokémon recuperó {} HP ({} / {}).", healed_amount, cur_hp, max_h);
            run_state.add_log(msg.clone());

            if let Ok(mut text) = q_status.single_mut() {
                **text = msg;
            }

            for mut fill in q_hp_fill.iter_mut() {
                let ratio = (cur_hp as f32 / max_h as f32).clamp(0.0, 1.0);
                fill.width = Val::Percent(ratio * 100.0);
            }

            for mut text in q_hp_text.iter_mut() {
                **text = format!("HP: {} / {}", cur_hp, max_h);
            }
        }
    }
}

fn handle_upgrade_card_click(
    mut q_buttons: Query<(&Interaction, &CenterUpgradeCardButton, &mut BackgroundColor), Changed<Interaction>>,
    mut center_state: ResMut<CenterState>,
    mut run_state: ResMut<CurrentRunState>,
    mut q_status: Query<&mut Text, With<CenterStatusBanner>>,
) {
    for (interaction, btn, mut bg_color) in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            if center_state.action_taken {
                if let Ok(mut text) = q_status.single_mut() {
                    **text = "[!] Ya has utilizado tu acción de descanso en este Centro Pokémon.".to_string();
                }
                continue;
            }

            let card_idx = btn.0;
            if upgrade_card_at_center(&mut run_state, card_idx) {
                center_state.action_taken = true;
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.4, 0.2));

                let card_name = run_state.player_deck[card_idx].display_name();
                let msg = format!("[ENTRENAMIENTO] ¡Éxito! '{}' ha sido mejorada.", card_name);
                if let Ok(mut text) = q_status.single_mut() {
                    **text = msg;
                }
            }
        }
    }
}

use crate::ui::transition_ui::TransitionToEvent;

fn handle_continue_click(
    mut q_buttons: Query<&Interaction, (Changed<Interaction>, With<CenterContinueButton>)>,
    mut run_state: ResMut<CurrentRunState>,
    mut ev_trans: MessageWriter<TransitionToEvent>,
) {
    for interaction in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            if run_state.current_floor < 15 {
                run_state.current_floor += 1;
            }
            ev_trans.write(TransitionToEvent {
                target: AppState::MapNavigation,
            });
        }
    }
}

fn cleanup_center_ui(mut commands: Commands, q_cleanup: Query<Entity, With<CleanupUi>>) {
    for entity in q_cleanup.iter() {
        commands.entity(entity).despawn();
    }
}
