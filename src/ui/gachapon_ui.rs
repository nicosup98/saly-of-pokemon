use bevy::prelude::*;
use crate::gachapon::pull_gachapon;
use crate::pokemon::get_species_data;
use crate::run::{AppState, CurrentRunState, MetaProfile};
use crate::ui::{CleanupUi, Button, BorderColor};

pub struct GachaponUiPlugin;

impl Plugin for GachaponUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GachaponUiState>()
            .add_systems(OnEnter(AppState::GachaponShop), setup_gachapon_ui)
            .add_systems(
                Update,
                (
                    handle_slot_fragment,
                    handle_single_pull,
                    handle_multi_pull,
                    handle_exit_gachapon,
                    handle_gachapon_to_starter_select,
                    handle_gachapon_quick_select_pokemon,
                    update_gachapon_selection_badges,
                    update_gachapon_display,
                )
                    .run_if(in_state(AppState::GachaponShop)),
            )
            .add_systems(OnExit(AppState::GachaponShop), cleanup_gachapon_ui);
    }
}

#[derive(Component)]
pub struct GachaponSelectPokemonBadgeButton(pub u32);

#[derive(Component)]
pub struct GachaponToStarterSelectionButton;

#[derive(Resource, Default)]
pub struct GachaponUiState {
    pub selected_target: Option<u32>,
    pub fragments_slotted: u32,
    pub last_pull_result: Option<u32>,
    pub last_pull_is_new: bool,
}

#[derive(Component)]
pub struct GachaponSinglePullButton;

#[derive(Component)]
pub struct GachaponMultiPullButton;

#[derive(Component)]
pub struct GachaponExitButton;

#[derive(Component)]
pub struct GachaponSlotFragmentButton(pub u32);

#[derive(Component)]
pub struct GachaponTokensText;

#[derive(Component)]
pub struct GachaponOddsText;

#[derive(Component)]
pub struct GachaponStatusText;

#[derive(Component)]
pub struct GachaponRewardSprite;

#[derive(Component)]
pub struct GachaponRewardName;

fn setup_gachapon_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meta: ResMut<MetaProfile>,
    mut ui_state: ResMut<GachaponUiState>,
    run_state: Res<CurrentRunState>,
) {
    ui_state.selected_target = None;
    ui_state.fragments_slotted = 0;
    ui_state.last_pull_result = None;

    // Asegurar algunos fragmentos de prueba si no hay
    if meta.pokemon_fragments.is_empty() {
        meta.pokemon_fragments.insert(7, 3);   // Squirtle
        meta.pokemon_fragments.insert(123, 2); // Scyther
    }

    commands
        .spawn((
            CleanupUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.06, 0.14)),
        ))
        .with_children(|root| {
            // === BARRA SUPERIOR ===
            root.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Px(56.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(16.0)),
                border: UiRect::bottom(Val::Px(2.0)),
                ..default()
            })
            .insert(BackgroundColor(Color::srgb(0.14, 0.10, 0.22)))
            .insert(BorderColor(Color::srgb(0.8, 0.35, 0.95)))
            .with_children(|header| {
                header.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    ..default()
                }).with_children(|title_box| {
                    title_box.spawn((
                        Text::new("TIENDA GACHAPÓN DE SILPH S.A."),
                        TextFont::from_font_size(22.0),
                        TextColor(Color::srgb(0.9, 0.5, 1.0)),
                    ));
                    title_box.spawn((
                        Text::new("Desbloquea nuevos Pokémon iniciales y potencia tus tiradas con Fragmentos de Mini Jefe."),
                        TextFont::from_font_size(12.0),
                        TextColor(Color::srgb(0.75, 0.7, 0.85)),
                    ));
                });

                // Fichas disponibles
                header.spawn((
                    GachaponTokensText,
                    Text::new(format!("Fichas: {} Gachapón", meta.gachapon_tokens)),
                    TextFont::from_font_size(18.0),
                    TextColor(Color::srgb(1.0, 0.85, 0.2)),
                ));

                // Acciones de cabecera
                header.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    ..default()
                }).with_children(|actions| {
                    actions.spawn((
                        Button,
                        GachaponToStarterSelectionButton,
                        Node {
                            height: Val::Px(36.0),
                            padding: UiRect::horizontal(Val::Px(14.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.5)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.20, 0.45, 0.65)),
                        BorderColor(Color::srgb(0.4, 0.7, 0.9)),
                    )).with_child((
                        Text::new("ELEGIR POKÉMON ➔"),
                        TextFont::from_font_size(13.0),
                        TextColor(Color::WHITE),
                    ));

                    actions.spawn((
                        Button,
                        GachaponExitButton,
                        Node {
                            height: Val::Px(36.0),
                            padding: UiRect::horizontal(Val::Px(16.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.5)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.25, 0.18, 0.35)),
                        BorderColor(Color::srgb(0.7, 0.4, 0.9)),
                    )).with_child((
                        Text::new("VOLVER ➔"),
                        TextFont::from_font_size(13.0),
                        TextColor(Color::WHITE),
                    ));
                });
            });

            // === CONTENIDO PRINCIPAL: MÁQUINA DE CÁPSULAS Y RECOMPENSA ===
            root.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(78.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(20.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Stretch,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            }).with_children(|content| {

                // COLUMNA IZQUIERDA: RANURAS DE FRAGMENTOS (RATE-UP FOCALIZADO)
                content.spawn(Node {
                    width: Val::Percent(42.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::all(Val::Px(14.0)),
                    border: UiRect::all(Val::Px(1.5)),
                    ..default()
                })
                .insert(BackgroundColor(Color::srgb(0.12, 0.10, 0.18)))
                .insert(BorderColor(Color::srgb(0.5, 0.3, 0.7)))
                .with_children(|frag_col| {
                    frag_col.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(6.0),
                        ..default()
                    }).with_children(|f_header| {
                        f_header.spawn((
                            Text::new("🔮 RANURAS DE FRAGMENTOS (RATE-UP)"),
                            TextFont::from_font_size(16.0),
                            TextColor(Color::srgb(0.85, 0.6, 1.0)),
                        ));
                        f_header.spawn((
                            Text::new("Inserta hasta 3 fragmentos para potenciar la probabilidad de conseguir una especie específica:"),
                            TextFont::from_font_size(11.5),
                            TextColor(Color::srgb(0.75, 0.75, 0.8)),
                        ));
                    });

                    // Lista de Fragmentos disponibles
                    frag_col.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        margin: UiRect::vertical(Val::Px(10.0)),
                        ..default()
                    }).with_children(|frag_list| {
                        for (&spec_id, &count) in meta.pokemon_fragments.iter() {
                            let name = get_species_data(spec_id)
                                .map(|d| d.name)
                                .unwrap_or_else(|| "Pokémon".to_string());

                            frag_list.spawn(Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(42.0),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                padding: UiRect::horizontal(Val::Px(10.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            })
                            .insert(BackgroundColor(Color::srgb(0.16, 0.14, 0.24)))
                            .insert(BorderColor(Color::srgb(0.4, 0.3, 0.6)))
                            .with_children(|row| {
                                row.spawn((
                                    Text::new(format!("🧩 {} (Disponibles: {})", name, count)),
                                    TextFont::from_font_size(13.0),
                                    TextColor(Color::WHITE),
                                ));

                                row.spawn((
                                    Button,
                                    GachaponSlotFragmentButton(spec_id),
                                    Node {
                                        width: Val::Px(100.0),
                                        height: Val::Px(28.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.4, 0.25, 0.65)),
                                )).with_child((
                                    Text::new("+1 INSERTAR"),
                                    TextFont::from_font_size(11.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                        }
                    });

                    // Indicador de Probabilidad en Tiempo Real
                    frag_col.spawn(Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    })
                    .insert(BackgroundColor(Color::srgb(0.18, 0.14, 0.26)))
                    .insert(BorderColor(Color::srgb(0.7, 0.5, 0.9)))
                    .with_children(|odds_box| {
                        odds_box.spawn((
                            GachaponOddsText,
                            Text::new("🎯 Probabilidad Focalizada: 0 Fragmentos insertados (Tirada General: 12% Raro, 4.5% Exótico)."),
                            TextFont::from_font_size(12.0),
                            TextColor(Color::srgb(1.0, 0.9, 0.4)),
                        ));
                    });

                    // Botones de Tirada
                    frag_col.spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(12.0),
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    }).with_children(|pull_buttons| {
                        pull_buttons.spawn((
                            Button,
                            GachaponSinglePullButton,
                            Node {
                                width: Val::Percent(48.0),
                                height: Val::Px(44.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.5)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.20, 0.60, 0.40)),
                            BorderColor(Color::srgb(0.4, 0.9, 0.6)),
                        )).with_child((
                            Text::new("TIRADA SIMPLE\n(100 Fichas)"),
                            TextFont::from_font_size(12.0),
                            TextColor(Color::WHITE),
                        ));

                        pull_buttons.spawn((
                            Button,
                            GachaponMultiPullButton,
                            Node {
                                width: Val::Percent(48.0),
                                height: Val::Px(44.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.5)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.65, 0.30, 0.70)),
                            BorderColor(Color::srgb(0.9, 0.5, 1.0)),
                        )).with_child((
                            Text::new("PACK x10 PULLS\n(900 Fichas)"),
                            TextFont::from_font_size(12.0),
                            TextColor(Color::WHITE),
                        ));
                    });
                });

                // COLUMNA DERECHA: APARADOR DE RECOMPENSA Y COLECCIÓN
                content.spawn(Node {
                    width: Val::Percent(54.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(16.0)),
                    border: UiRect::all(Val::Px(1.5)),
                    ..default()
                })
                .insert(BackgroundColor(Color::srgb(0.12, 0.10, 0.18)))
                .insert(BorderColor(Color::srgb(0.6, 0.4, 0.8)))
                .with_children(|reward_panel| {
                    reward_panel.spawn((
                        Text::new("🎁 CÁPSULA OBTENIDA"),
                        TextFont::from_font_size(18.0),
                        TextColor(Color::srgb(1.0, 0.85, 0.3)),
                    ));

                    // Sprite de Recompensa (por defecto silueta o Pokéball)
                    let placeholder_texture = asset_server.load("sprites/pokemon/front/4.png");
                    reward_panel.spawn((
                        GachaponRewardSprite,
                        ImageNode::new(placeholder_texture),
                        Node {
                            width: Val::Px(140.0),
                            height: Val::Px(140.0),
                            margin: UiRect::vertical(Val::Px(8.0)),
                            ..default()
                        },
                    ));

                    reward_panel.spawn((
                        GachaponRewardName,
                        Text::new("¡Gira la palanca para revelar tu Pokémon!"),
                        TextFont::from_font_size(16.0),
                        TextColor(Color::WHITE),
                    ));

                    // Colección Desbloqueada
                    reward_panel.spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(6.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    })
                    .insert(BackgroundColor(Color::srgb(0.16, 0.14, 0.22)))
                    .insert(BorderColor(Color::srgb(0.35, 0.3, 0.5)))
                    .with_children(|col_box| {
                        col_box.spawn((
                            Text::new(format!("📖 Tu Colección Desbloqueada ({} Pokémon) - Haz clic para seleccionar:", meta.unlocked_pokemon.len())),
                            TextFont::from_font_size(13.0),
                            TextColor(Color::srgb(0.8, 0.85, 1.0)),
                        ));

                        col_box.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                            row_gap: Val::Px(8.0),
                            flex_wrap: FlexWrap::Wrap,
                            ..default()
                        }).with_children(|icons_row| {
                            for &id in meta.unlocked_pokemon.iter() {
                                let name = get_species_data(id).map(|d| d.name).unwrap_or_else(|| "?".to_string());
                                let is_active = run_state.selected_starter_id == id;
                                let border_color = if is_active {
                                    Color::srgb(1.0, 0.85, 0.2)
                                } else {
                                    Color::srgb(0.5, 0.4, 0.7)
                                };
                                let bg_color = if is_active {
                                    Color::srgb(0.35, 0.28, 0.15)
                                } else {
                                    Color::srgb(0.22, 0.20, 0.32)
                                };
                                let label = if is_active {
                                    format!("#{} {} [ACTIVO]", id, name)
                                } else {
                                    format!("#{} {}", id, name)
                                };

                                icons_row.spawn((
                                    Button,
                                    GachaponSelectPokemonBadgeButton(id),
                                    Node {
                                        padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                                        border: UiRect::all(Val::Px(1.5)),
                                        ..default()
                                    },
                                    BackgroundColor(bg_color),
                                    BorderColor(border_color),
                                ))
                                .with_child((
                                    Text::new(label),
                                    TextFont::from_font_size(11.0),
                                    TextColor(if is_active { Color::srgb(1.0, 0.9, 0.4) } else { Color::WHITE }),
                                ));
                            }
                        });
                    });
                });
            });

            // === BARRA INFERIOR DE MENSAJES ===
            root.spawn((
                GachaponStatusText,
                Text::new("💡 Las fichas gachapón se ganan superando pisos, Mini Jefes y Líderes de Gimnasio en cada run."),
                TextFont::from_font_size(13.0),
                TextColor(Color::srgb(0.7, 0.65, 0.85)),
            ));
        });
}

fn handle_slot_fragment(
    mut q_buttons: Query<(&Interaction, &GachaponSlotFragmentButton), Changed<Interaction>>,
    meta: Res<MetaProfile>,
    mut ui_state: ResMut<GachaponUiState>,
    mut q_odds: Query<&mut Text, With<GachaponOddsText>>,
    mut q_status: Query<&mut Text, (With<GachaponStatusText>, Without<GachaponOddsText>)>,
) {
    for (interaction, btn) in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            let spec_id = btn.0;
            let available = meta.pokemon_fragments.get(&spec_id).copied().unwrap_or(0);

            if available == 0 {
                if let Ok(mut text) = q_status.single_mut() {
                    **text = "⚠️ No tienes más fragmentos de esta especie.".to_string();
                }
                continue;
            }

            if ui_state.selected_target != Some(spec_id) {
                ui_state.selected_target = Some(spec_id);
                ui_state.fragments_slotted = 1;
            } else if ui_state.fragments_slotted < 3 && ui_state.fragments_slotted < available {
                ui_state.fragments_slotted += 1;
            } else {
                ui_state.fragments_slotted = 1;
            }

            let name = get_species_data(spec_id).map(|d| d.name).unwrap_or_else(|| "Pokémon".to_string());
            let odds = match ui_state.fragments_slotted {
                1 => 20.0,
                2 => 35.0,
                3 => 55.0,
                _ => 12.0,
            };

            if let Ok(mut text) = q_odds.single_mut() {
                **text = format!(
                    "Rate-Up Focalizado en {}: {}/3 Fragmentos insertados (¡{:.0}% probabilidad directa!)",
                    name, ui_state.fragments_slotted, odds
                );
            }

            if let Ok(mut text) = q_status.single_mut() {
                **text = format!("[RATE-UP] ¡Fragmento de {} insertado en la ranura de amplificación!", name);
            }
        }
    }
}

fn handle_single_pull(
    mut q_buttons: Query<&Interaction, (Changed<Interaction>, With<GachaponSinglePullButton>)>,
    mut meta: ResMut<MetaProfile>,
    mut ui_state: ResMut<GachaponUiState>,
    asset_server: Res<AssetServer>,
    mut q_sprite: Query<&mut ImageNode, With<GachaponRewardSprite>>,
    mut q_name: Query<&mut Text, (With<GachaponRewardName>, Without<GachaponTokensText>, Without<GachaponStatusText>)>,
    mut q_status: Query<&mut Text, (With<GachaponStatusText>, Without<GachaponTokensText>, Without<GachaponRewardName>)>,
) {
    for interaction in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            if meta.gachapon_tokens < 100 {
                if let Ok(mut text) = q_status.single_mut() {
                    **text = "❌ ¡Fichas insuficientes! Necesitas 100 Fichas Gachapón para una tirada.".to_string();
                }
                return;
            }

            let was_unlocked_before = |id: u32, list: &[u32]| list.contains(&id);

            let target = ui_state.selected_target;
            let frags = ui_state.fragments_slotted;

            let prev_unlocked = meta.unlocked_pokemon.clone();
            if let Some(pulled_id) = pull_gachapon(&mut meta, target, frags) {
                ui_state.last_pull_result = Some(pulled_id);
                let is_new = !was_unlocked_before(pulled_id, &prev_unlocked);
                ui_state.last_pull_is_new = is_new;

                let name = get_species_data(pulled_id).map(|d| d.name).unwrap_or_else(|| "Pokémon".to_string());
                let sprite_path = format!("sprites/pokemon/front/{}.png", pulled_id);

                if let Ok(mut img) = q_sprite.single_mut() {
                    img.image = asset_server.load(&sprite_path);
                }

                if let Ok(mut name_text) = q_name.single_mut() {
                    if is_new {
                        **name_text = format!("¡NUEVO POKÉMON DESBLOQUEADO: #{} {}!", pulled_id, name);
                    } else {
                        **name_text = format!("¡#{} {} obtenido de nuevo! (+IVs de Potencial)", pulled_id, name);
                    }
                }

                if let Ok(mut text) = q_status.single_mut() {
                    **text = format!("[GACHAPÓN] ¡Tirada completada con éxito! Has obtenido a {}.", name);
                }

                // Reset fragment slot after use
                ui_state.fragments_slotted = 0;
                ui_state.selected_target = None;
            }
        }
    }
}

fn handle_multi_pull(
    mut q_buttons: Query<&Interaction, (Changed<Interaction>, With<GachaponMultiPullButton>)>,
    mut meta: ResMut<MetaProfile>,
    mut ui_state: ResMut<GachaponUiState>,
    asset_server: Res<AssetServer>,
    mut q_sprite: Query<&mut ImageNode, With<GachaponRewardSprite>>,
    mut q_name: Query<&mut Text, (With<GachaponRewardName>, Without<GachaponTokensText>, Without<GachaponStatusText>)>,
    mut q_status: Query<&mut Text, (With<GachaponStatusText>, Without<GachaponTokensText>, Without<GachaponRewardName>)>,
) {
    for interaction in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            if meta.gachapon_tokens < 900 {
                if let Ok(mut text) = q_status.single_mut() {
                    **text = "[AVISO] ¡Fichas insuficientes! Se requieren 900 Fichas para el Pack x10.".to_string();
                }
                return;
            }

            meta.gachapon_tokens -= 900;
            // Otorga 100 fichas virtuales para que cada llamada de pull_gachapon tenga 100
            let mut pulled_names = Vec::new();
            let mut last_id = 4;

            for i in 0..10 {
                meta.gachapon_tokens += 100;
                let target = if i == 0 { ui_state.selected_target } else { None };
                let frags = if i == 0 { ui_state.fragments_slotted } else { 0 };

                if let Some(id) = pull_gachapon(&mut meta, target, frags) {
                    last_id = id;
                    let name = get_species_data(id).map(|d| d.name).unwrap_or_else(|| "Pokémon".to_string());
                    pulled_names.push(name);
                }
            }

            let sprite_path = format!("sprites/pokemon/front/{}.png", last_id);
            if let Ok(mut img) = q_sprite.single_mut() {
                img.image = asset_server.load(&sprite_path);
            }

            if let Ok(mut name_text) = q_name.single_mut() {
                **name_text = format!("¡PACK x10 COMPLETADO! Última cápsula: #{}", last_id);
            }

            if let Ok(mut text) = q_status.single_mut() {
                **text = format!("[RECOMPENSAS x10] {}", pulled_names.join(", "));
            }

            ui_state.fragments_slotted = 0;
            ui_state.selected_target = None;
        }
    }
}

use crate::ui::transition_ui::TransitionToEvent;

fn handle_exit_gachapon(
    mut q_buttons: Query<&Interaction, (Changed<Interaction>, With<GachaponExitButton>)>,
    mut ev_trans: MessageWriter<TransitionToEvent>,
) {
    for interaction in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            ev_trans.write(TransitionToEvent {
                target: AppState::MainMenu,
            });
        }
    }
}

fn update_gachapon_display(
    meta: Res<MetaProfile>,
    mut q_tokens: Query<&mut Text, With<GachaponTokensText>>,
) {
    if meta.is_changed() {
        for mut text in q_tokens.iter_mut() {
            **text = format!("Fichas: {} Gachapón", meta.gachapon_tokens);
        }
    }
}

fn handle_gachapon_to_starter_select(
    mut q_button: Query<&Interaction, (Changed<Interaction>, With<GachaponToStarterSelectionButton>)>,
    mut ev_trans: MessageWriter<TransitionToEvent>,
) {
    for interaction in q_button.iter_mut() {
        if *interaction == Interaction::Pressed {
            ev_trans.write(TransitionToEvent {
                target: AppState::StarterSelection,
            });
        }
    }
}

fn handle_gachapon_quick_select_pokemon(
    mut q_buttons: Query<(&Interaction, &GachaponSelectPokemonBadgeButton), Changed<Interaction>>,
    mut run_state: ResMut<CurrentRunState>,
    mut q_status: Query<&mut Text, (With<GachaponStatusText>, Without<GachaponOddsText>)>,
) {
    for (interaction, btn) in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            let spec_id = btn.0;
            if run_state.selected_starter_id != spec_id {
                run_state.selected_starter_id = spec_id;
                if let Some(sp) = get_species_data(spec_id) {
                    let elem = sp.primary_types.first().copied().unwrap_or(crate::types::ElementType::Normal);
                    let is_special = sp.base_sp_atk >= sp.base_atk;
                    run_state.player_deck = crate::card::create_starter_deck(elem, is_special);
                    run_state.player_hp = None;
                    if let Ok(mut text) = q_status.single_mut() {
                        **text = format!("✅ ¡Has cambiado tu Pokémon activo a {} (#{}), mazo actualizado!", sp.name, spec_id);
                    }
                }
            }
        }
    }
}

fn update_gachapon_selection_badges(
    run_state: Res<CurrentRunState>,
    mut q_badges: Query<(&GachaponSelectPokemonBadgeButton, &mut BackgroundColor, &mut bevy::ui::BorderColor, &Children)>,
    mut q_text: Query<&mut Text>,
) {
    if !run_state.is_changed() {
        return;
    }
    for (btn, mut bg, mut border, children) in q_badges.iter_mut() {
        let is_active = btn.0 == run_state.selected_starter_id;
        let name = get_species_data(btn.0).map(|d| d.name).unwrap_or_else(|| "?".to_string());
        *bg = if is_active {
            BackgroundColor(Color::srgb(0.35, 0.28, 0.15))
        } else {
            BackgroundColor(Color::srgb(0.22, 0.20, 0.32))
        };
        *border = if is_active {
            BorderColor(Color::srgb(1.0, 0.85, 0.2))
        } else {
            BorderColor(Color::srgb(0.5, 0.4, 0.7))
        };
        for child in children.iter() {
            if let Ok(mut text) = q_text.get_mut(child) {
                **text = if is_active {
                    format!("#{} {} [ACTIVO]", btn.0, name)
                } else {
                    format!("#{} {}", btn.0, name)
                };
            }
        }
    }
}

fn cleanup_gachapon_ui(mut commands: Commands, q_cleanup: Query<Entity, With<CleanupUi>>) {
    for entity in q_cleanup.iter() {
        commands.entity(entity).despawn();
    }
}
