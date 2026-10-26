use bevy::prelude::*;
use crate::pokemon::get_species_data;
use crate::run::{AppState, CurrentRunState, MetaProfile};
use crate::types::ElementType;
use crate::ui::{element_color, BorderColor, Button, CleanupUi};
use crate::ui::transition_ui::TransitionToEvent;

pub struct StarterSelectionUiPlugin;

impl Plugin for StarterSelectionUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::StarterSelection), setup_starter_selection)
            .add_systems(
                Update,
                (
                    handle_pick_pokemon,
                    update_starter_cards_visuals,
                    update_starter_info_label,
                    handle_starter_action_navigation,
                )
                    .run_if(in_state(AppState::StarterSelection)),
            )
            .add_systems(OnExit(AppState::StarterSelection), cleanup_starter_selection);
    }
}

#[derive(Component)]
pub struct StarterCardButton(pub u32);

#[derive(Component)]
pub struct StarterActionNavButton(pub StarterNavAction);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StarterNavAction {
    StartMap,
    BackToMenu,
}

#[derive(Component)]
pub struct StarterActiveBadge(pub u32);

#[derive(Component)]
pub struct StarterActiveText(pub u32);

#[derive(Component)]
pub struct SelectedStarterInfoLabel;

fn setup_starter_selection(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    meta: Res<MetaProfile>,
    run_state: Res<CurrentRunState>,
) {
    commands
        .spawn((
            CleanupUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(24.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.09, 0.13)),
        ))
        .with_children(|root| {
            // === CABECERA ===
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(6.0),
                ..default()
            })
            .with_children(|header| {
                header.spawn((
                    Text::new("SELECCIÓN DE POKÉMON INICIAL"),
                    TextFont::from_font_size(24.0),
                    TextColor(Color::srgb(1.0, 0.85, 0.2)),
                ));
                header.spawn((
                    Text::new("Elige con cuál de tus Pokémon desbloqueados en el Gachapón deseas iniciar la travesía:"),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::srgb(0.8, 0.85, 0.95)),
                ));
            });

            // === REJILLA / LISTA DE POKÉMON DESBLOQUEADOS ===
            root.spawn(Node {
                width: Val::Percent(100.0),
                max_width: Val::Px(1050.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(16.0),
                row_gap: Val::Px(16.0),
                ..default()
            })
            .with_children(|grid| {
                for &spec_id in meta.unlocked_pokemon.iter() {
                    let is_active = run_state.selected_starter_id == spec_id;
                    let sp = get_species_data(spec_id);
                    let name = sp.as_ref().map(|s| s.name.clone()).unwrap_or_else(|| format!("Pokémon #{}", spec_id));
                    let primary_type = sp.as_ref().and_then(|s| s.primary_types.first().copied()).unwrap_or(ElementType::Normal);
                    let type_col = element_color(primary_type);

                    let (hp, atk, def, spatk) = if let Some(s) = &sp {
                        (s.base_hp, s.base_atk, s.base_def, s.base_sp_atk)
                    } else {
                        (40, 50, 40, 50)
                    };

                    let border_col = if is_active {
                        Color::srgb(1.0, 0.85, 0.2)
                    } else {
                        Color::srgb(0.28, 0.32, 0.42)
                    };

                    let card_bg = if is_active {
                        Color::srgb(0.22, 0.20, 0.16)
                    } else {
                        Color::srgb(0.13, 0.15, 0.20)
                    };

                    grid.spawn((
                        Button,
                        StarterCardButton(spec_id),
                        Node {
                            width: Val::Px(190.0),
                            height: Val::Px(260.0),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(10.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(card_bg),
                        BorderColor(border_col),
                    ))
                    .with_children(|card| {
                        // Badge superior (ID y estado)
                        card.spawn(Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|top| {
                            top.spawn((
                                Text::new(format!("#{:03}", spec_id)),
                                TextFont::from_font_size(11.0),
                                TextColor(Color::srgb(0.65, 0.70, 0.80)),
                            ));
                            top.spawn((
                                StarterActiveBadge(spec_id),
                                Text::new(if is_active { "EN EQUIPO" } else { "" }),
                                TextFont::from_font_size(10.0),
                                TextColor(Color::srgb(1.0, 0.85, 0.2)),
                            ));
                        });

                        // Sprite del Pokémon
                        let sprite_path = format!("sprites/pokemon/front/{}.png", spec_id);
                        card.spawn((
                            ImageNode::new(asset_server.load(&sprite_path)),
                            Node {
                                width: Val::Px(96.0),
                                height: Val::Px(96.0),
                                ..default()
                            },
                        ));

                        // Nombre del Pokémon
                        card.spawn((
                            Text::new(name),
                            TextFont::from_font_size(15.0),
                            TextColor(Color::WHITE),
                        ));

                        // Insignia de Tipo
                        card.spawn((
                            Node {
                                padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(type_col),
                            BorderColor(Color::WHITE),
                        ))
                        .with_child((
                            Text::new(format!("{:?}", primary_type).to_uppercase()),
                            TextFont::from_font_size(9.0),
                            TextColor(Color::srgb(0.08, 0.08, 0.08)),
                        ));

                        // Stats base
                        card.spawn((
                            Text::new(format!("HP:{} ATK:{} DEF:{} SP:{}", hp, atk, def, spatk)),
                            TextFont::from_font_size(10.0),
                            TextColor(Color::srgb(0.7, 0.75, 0.85)),
                        ));

                        // Botón de estado / selección
                        let btn_bg = if is_active {
                            Color::srgb(0.35, 0.30, 0.10)
                        } else {
                            Color::srgb(0.18, 0.35, 0.55)
                        };

                        card.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(26.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(btn_bg),
                        ))
                        .with_child((
                            StarterActiveText(spec_id),
                            Text::new(if is_active { "ACTIVO" } else { "ELEGIR" }),
                            TextFont::from_font_size(11.0),
                            TextColor(Color::WHITE),
                        ));
                    });
                }
            });

            // === BARRA INFERIOR DE ACCIÓN ===
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(12.0),
                ..default()
            })
            .with_children(|footer| {
                let current_name = get_species_data(run_state.selected_starter_id)
                    .map(|s| s.name)
                    .unwrap_or_else(|| "Charmander".to_string());

                footer.spawn((
                    SelectedStarterInfoLabel,
                    Text::new(format!(
                        "Pokémon seleccionado: {} #{} (Mazo adaptado de 10 cartas)",
                        current_name, run_state.selected_starter_id
                    )),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::srgb(0.95, 0.9, 0.7)),
                ));

                footer.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(18.0),
                    ..default()
                })
                .with_children(|actions| {
                    // Botón Iniciar Ruta
                    actions
                        .spawn((
                            Button,
                            StarterActionNavButton(StarterNavAction::StartMap),
                            Node {
                                width: Val::Px(240.0),
                                height: Val::Px(46.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.5)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.20, 0.55, 0.85)),
                            BorderColor(Color::WHITE),
                        ))
                        .with_child((
                            Text::new("INICIAR RUTA (MAPA)"),
                            TextFont::from_font_size(14.0),
                            TextColor(Color::WHITE),
                        ));

                    // Botón Volver al Menú
                    actions
                        .spawn((
                            Button,
                            StarterActionNavButton(StarterNavAction::BackToMenu),
                            Node {
                                width: Val::Px(200.0),
                                height: Val::Px(46.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.5)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.25, 0.28, 0.35)),
                            BorderColor(Color::srgb(0.5, 0.55, 0.65)),
                        ))
                        .with_child((
                            Text::new("VOLVER AL MENÚ"),
                            TextFont::from_font_size(14.0),
                            TextColor(Color::WHITE),
                        ));
                });
            });
        });
}

fn handle_pick_pokemon(
    mut q_buttons: Query<(&Interaction, &StarterCardButton), Changed<Interaction>>,
    mut run_state: ResMut<CurrentRunState>,
) {
    for (interaction, btn) in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            let spec_id = btn.0;
            if run_state.selected_starter_id != spec_id {
                run_state.selected_starter_id = spec_id;
                // Regenera el mazo inicial para este Pokémon según su tipo
                if let Some(sp) = get_species_data(spec_id) {
                    let elem = sp.primary_types.first().copied().unwrap_or(ElementType::Normal);
                    let is_special = sp.base_sp_atk >= sp.base_atk;
                    run_state.player_deck = crate::card::create_starter_deck(elem, is_special);
                    run_state.player_hp = None;
                }
            }
        }
    }
}

fn update_starter_cards_visuals(
    run_state: Res<CurrentRunState>,
    mut q_cards: Query<(&StarterCardButton, &mut BackgroundColor, &mut bevy::ui::BorderColor)>,
    mut q_badges: Query<(&StarterActiveBadge, &mut Text), Without<StarterActiveText>>,
    mut q_texts: Query<(&StarterActiveText, &mut Text), Without<StarterActiveBadge>>,
) {
    if !run_state.is_changed() {
        return;
    }

    // Actualiza tarjetas
    for (card, mut bg, mut border) in q_cards.iter_mut() {
        let is_active = card.0 == run_state.selected_starter_id;
        *border = if is_active {
            BorderColor(Color::srgb(1.0, 0.85, 0.2))
        } else {
            BorderColor(Color::srgb(0.28, 0.32, 0.42))
        };
        *bg = if is_active {
            BackgroundColor(Color::srgb(0.22, 0.20, 0.16))
        } else {
            BackgroundColor(Color::srgb(0.13, 0.15, 0.20))
        };
    }

    // Actualiza badges
    for (badge, mut text) in q_badges.iter_mut() {
        **text = if badge.0 == run_state.selected_starter_id {
            "EN EQUIPO".to_string()
        } else {
            "".to_string()
        };
    }

    // Actualiza textos de botones
    for (btn_txt, mut text) in q_texts.iter_mut() {
        **text = if btn_txt.0 == run_state.selected_starter_id {
            "ACTIVO".to_string()
        } else {
            "ELEGIR".to_string()
        };
    }
}

fn update_starter_info_label(
    run_state: Res<CurrentRunState>,
    mut q_info: Query<&mut Text, With<SelectedStarterInfoLabel>>,
) {
    if !run_state.is_changed() {
        return;
    }

    // Actualiza label informativo inferior
    if let Ok(mut text) = q_info.single_mut() {
        let current_name = get_species_data(run_state.selected_starter_id)
            .map(|s| s.name)
            .unwrap_or_else(|| "Charmander".to_string());
        **text = format!(
            "Pokémon seleccionado: {} #{} (Mazo adaptado de 10 cartas)",
            current_name, run_state.selected_starter_id
        );
    }
}

fn handle_starter_action_navigation(
    mut q_buttons: Query<(&Interaction, &StarterActionNavButton), Changed<Interaction>>,
    mut ev_trans: MessageWriter<TransitionToEvent>,
) {
    for (interaction, btn) in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            match btn.0 {
                StarterNavAction::StartMap => {
                    ev_trans.write(TransitionToEvent {
                        target: AppState::MapNavigation,
                    });
                }
                StarterNavAction::BackToMenu => {
                    ev_trans.write(TransitionToEvent {
                        target: AppState::MainMenu,
                    });
                }
            }
        }
    }
}

fn cleanup_starter_selection(mut commands: Commands, q_cleanup: Query<Entity, With<CleanupUi>>) {
    for entity in q_cleanup.iter() {
        commands.entity(entity).despawn();
    }
}
