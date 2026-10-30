use bevy::prelude::*;
use crate::combat::CombatVictoryData;
use crate::pokemon::PokemonSprite;
use crate::run::{AppState, CurrentRunState};
use crate::ui::transition_ui::TransitionToEvent;
use crate::ui::{element_color, BorderColor, Button, CleanupUi, CardHoverAnimation};

pub struct VictoryUiPlugin;

impl Plugin for VictoryUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Victory), setup_victory_ui)
            .add_systems(
                Update,
                (
                    animate_victory_celebration,
                    animate_victory_stars,
                    handle_victory_card_click,
                    handle_skip_card_click,
                    handle_continue_click,
                    victory_button_hover,
                )
                    .run_if(in_state(AppState::Victory)),
            )
            .add_systems(OnExit(AppState::Victory), cleanup_victory_ui);
    }
}

#[derive(Component)]
pub struct VictoryCelebrationSprite {
    pub base_y: f32,
    pub elapsed: f32,
}

#[derive(Component)]
pub struct VictoryStarParticle {
    pub speed_y: f32,
    pub phase: f32,
    pub start_y: f32,
    pub start_x: f32,
    pub lifetime: f32,
    pub elapsed: f32,
}

#[derive(Component)]
pub struct VictoryGlowBanner {
    pub elapsed: f32,
}

#[derive(Component)]
pub struct VictoryContinueButton;

#[derive(Component)]
pub struct VictoryRewardCardButton {
    pub index: usize,
}

#[derive(Component)]
pub struct VictorySkipCardButton;

#[derive(Component)]
pub struct VictoryCardFeedbackText;

#[derive(Component)]
pub struct VictoryCardContainer;

fn setup_victory_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    victory_data: Option<Res<CombatVictoryData>>,
    run_state: Res<CurrentRunState>,
) {
    let victory = victory_data.map(|v| v.clone()).unwrap_or_default();

    // Sprite frontal del Pokémon del jugador para la celebración
    let sprite_path = PokemonSprite::new(victory.player_species_id, false).relative_path();
    let sprite_handle: Handle<Image> = asset_server.load(sprite_path);

    commands
        .spawn((
            CleanupUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(32.0), Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.06, 0.07, 0.11)),
        ))
        .with_children(|root| {
            // ========================================================
            // CABECERA SUPERIOR: BANNER DE VICTORIA
            // ========================================================
            root.spawn((
                VictoryGlowBanner { elapsed: 0.0 },
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(24.0), Val::Px(10.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.14, 0.22)),
                BorderColor(Color::srgb(1.0, 0.85, 0.2)),
            ))
            .with_children(|banner| {
                banner.spawn((
                    Text::new("* ¡VICTORIA SOBRE EL ENTRENADOR! *"),
                    TextFont::from_font_size(26.0),
                    TextColor(Color::srgb(1.0, 0.86, 0.2)),
                ));
                let sub_title = if victory.is_boss {
                    format!("¡Has vencido al Líder de Gimnasio {}!", victory.enemy_trainer_name)
                } else if victory.is_mini_boss {
                    format!("¡Has derrotado al {} salvaje!", victory.enemy_name)
                } else {
                    format!("¡Has superado al {}!", victory.enemy_trainer_name)
                };
                banner.spawn((
                    Text::new(sub_title),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::srgb(0.85, 0.88, 0.95)),
                ));
            });

            // ========================================================
            // SECCIÓN CENTRAL: PEDESTAL DE CELEBRACIÓN Y RECOMPENSAS
            // ========================================================
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                margin: UiRect::axes(Val::Px(0.0), Val::Px(8.0)),
                ..default()
            })
            .with_children(|mid_row| {
                // COLUMNA IZQUIERDA: PEDESTAL CON ANIMACIÓN DE VICTORIA DEL POKÉMON
                mid_row
                    .spawn(Node {
                        width: Val::Px(320.0),
                        height: Val::Px(230.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::all(Val::Px(12.0)),
                        position_type: PositionType::Relative,
                        ..default()
                    })
                    .insert(BackgroundColor(Color::srgb(0.09, 0.11, 0.17)))
                    .insert(BorderColor(Color::srgb(0.3, 0.6, 0.9)))
                    .with_children(|pedestal| {
                        // Partículas de estrellas de victoria orbitando
                        for i in 0..6 {
                            let (symbol, start_x, start_y, speed) = match i {
                                0 => ("*", 30.0, 160.0, 35.0),
                                1 => ("+", 260.0, 170.0, 40.0),
                                2 => ("*", 70.0, 190.0, 30.0),
                                3 => ("+", 220.0, 150.0, 45.0),
                                4 => ("*", 160.0, 200.0, 32.0),
                                _ => ("+", 120.0, 180.0, 38.0),
                            };
                            pedestal.spawn((
                                VictoryStarParticle {
                                    speed_y: speed,
                                    phase: i as f32 * 1.1,
                                    start_x,
                                    start_y,
                                    lifetime: 2.2,
                                    elapsed: (i as f32) * 0.35,
                                },
                                UiTransform::default(),
                                Text::new(symbol),
                                TextFont::from_font_size(18.0),
                                TextColor(Color::srgba(1.0, 0.85, 0.2, 0.8)),
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(start_x),
                                    top: Val::Px(start_y),
                                    ..default()
                                },
                            ));
                        }

                        // Fila con Entrenador Red y su Pokémon celebrando juntos
                        pedestal.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::FlexEnd,
                            justify_content: JustifyContent::Center,
                            column_gap: Val::Px(12.0),
                            ..default()
                        }).with_children(|pair| {
                            // Sprite animado del Entrenador Red
                            pair.spawn((
                                VictoryCelebrationSprite {
                                    base_y: 0.0,
                                    elapsed: 0.2,
                                },
                                UiTransform::default(),
                                ImageNode::new(asset_server.load("sprites/trainers/player_front.png")),
                                Node {
                                    width: Val::Px(64.0),
                                    height: Val::Px(64.0),
                                    margin: UiRect::bottom(Val::Px(10.0)),
                                    ..default()
                                },
                            ));

                            // Sprite animado del Pokémon del jugador
                            pair.spawn((
                                VictoryCelebrationSprite {
                                    base_y: 0.0,
                                    elapsed: 0.0,
                                },
                                UiTransform::default(),
                                ImageNode::new(sprite_handle),
                                Node {
                                    width: Val::Px(110.0),
                                    height: Val::Px(110.0),
                                    ..default()
                                },
                            ));
                        });

                        // Placa identificativa del Entrenador y su Pokémon
                        pedestal.spawn(Node {
                            margin: UiRect::top(Val::Px(8.0)),
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(4.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        })
                        .insert(BackgroundColor(Color::srgb(0.15, 0.20, 0.30)))
                        .insert(BorderColor(Color::srgb(1.0, 0.85, 0.2)))
                        .with_child((
                            Text::new(format!("Entrenador Red & {} (Nv. {})", victory.player_name, victory.player_level)),
                            TextFont::from_font_size(13.0),
                            TextColor(Color::srgb(1.0, 0.9, 0.4)),
                        ));
                    });

                // COLUMNA DERECHA: DESGLOSE DE RECOMPENSAS
                mid_row
                    .spawn(Node {
                        width: Val::Px(560.0),
                        height: Val::Px(230.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceEvenly,
                        padding: UiRect::all(Val::Px(16.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    })
                    .insert(BackgroundColor(Color::srgb(0.09, 0.11, 0.17)))
                    .insert(BorderColor(Color::srgb(0.4, 0.7, 0.4)))
                    .with_children(|rewards_box| {
                        rewards_box.spawn((
                            Text::new("DESGLOSE DE RECOMPENSAS:"),
                            TextFont::from_font_size(15.0),
                            TextColor(Color::srgb(1.0, 0.85, 0.2)),
                        ));

                        // Fila de EXP
                        rewards_box
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(10.0),
                                ..default()
                            })
                            .with_children(|r| {
                                r.spawn((
                                    Node {
                                        padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.2, 0.65, 0.3)),
                                ))
                                .with_child((
                                    Text::new("[ EXP ]"),
                                    TextFont::from_font_size(11.0),
                                    TextColor(Color::WHITE),
                                ));
                                let exp_text = if victory.levels_gained > 0 {
                                    format!(
                                        "+{} EXP ganados (¡SUBES DE NIVEL! Nv. {} -> Nv. {} [{}/{} XP])",
                                        victory.exp_gained,
                                        victory.player_level.saturating_sub(victory.levels_gained),
                                        victory.player_level,
                                        victory.player_exp,
                                        victory.exp_to_next
                                    )
                                } else {
                                    format!(
                                        "+{} EXP ganados (Nivel actual: Nv. {} [{}/{} XP])",
                                        victory.exp_gained,
                                        victory.player_level,
                                        victory.player_exp,
                                        victory.exp_to_next
                                    )
                                };
                                r.spawn((
                                    Text::new(exp_text),
                                    TextFont::from_font_size(13.0),
                                    TextColor(Color::WHITE),
                                ));
                            });

                        // Fila de Poké-Dólares
                        rewards_box
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(10.0),
                                ..default()
                            })
                            .with_children(|r| {
                                r.spawn((
                                    Node {
                                        padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.85, 0.70, 0.15)),
                                ))
                                .with_child((
                                    Text::new("[ P$ ]"),
                                    TextFont::from_font_size(11.0),
                                    TextColor(Color::srgb(0.1, 0.1, 0.1)),
                                ));
                                r.spawn((
                                    Text::new(format!(
                                        "+{} P$ de recompensa en metálico (Saldo total: {} P$)",
                                        victory.money_gained, run_state.poke_dollars
                                    )),
                                    TextFont::from_font_size(13.0),
                                    TextColor(Color::WHITE),
                                ));
                            });

                        // Fila de Fichas de Gachapón
                        rewards_box
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(10.0),
                                ..default()
                            })
                            .with_children(|r| {
                                r.spawn((
                                    Node {
                                        padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.25, 0.55, 0.85)),
                                ))
                                .with_child((
                                    Text::new("[ FICHAS ]"),
                                    TextFont::from_font_size(11.0),
                                    TextColor(Color::WHITE),
                                ));
                                r.spawn((
                                    Text::new(format!(
                                        "+{} Fichas de Gachapón obtenidas para el Menú Principal",
                                        victory.tokens_gained
                                    )),
                                    TextFont::from_font_size(13.0),
                                    TextColor(Color::WHITE),
                                ));
                            });

                        // Recompensas especiales adicionales (Fragmentos, Medallas o Evolución)
                        if let Some(frag) = &victory.fragment_reward {
                            rewards_box
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(10.0),
                                    ..default()
                                })
                                .with_children(|r| {
                                    r.spawn((
                                        Node {
                                            padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.65, 0.35, 0.85)),
                                    ))
                                    .with_child((
                                        Text::new("[ FRAGMENTO ]"),
                                        TextFont::from_font_size(11.0),
                                        TextColor(Color::WHITE),
                                    ));
                                    r.spawn((
                                        Text::new(format!("¡Obtienes {}!", frag)),
                                        TextFont::from_font_size(13.0),
                                        TextColor(Color::srgb(0.9, 0.8, 1.0)),
                                    ));
                                });
                        }

                        if let Some(badge) = &victory.badge_reward {
                            rewards_box
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(10.0),
                                    ..default()
                                })
                                .with_children(|r| {
                                    r.spawn((
                                        Node {
                                            padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.95, 0.45, 0.15)),
                                    ))
                                    .with_child((
                                        Text::new("[ MEDALLA ]"),
                                        TextFont::from_font_size(11.0),
                                        TextColor(Color::WHITE),
                                    ));
                                    r.spawn((
                                        Text::new(format!("¡Conseguida: {}!", badge)),
                                        TextFont::from_font_size(13.0),
                                        TextColor(Color::srgb(1.0, 0.85, 0.3)),
                                    ));
                                });
                        }

                        if let Some(evo) = &victory.evolution_msg {
                            rewards_box
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(10.0),
                                    ..default()
                                })
                                .with_children(|r| {
                                    r.spawn((
                                        Node {
                                            padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.95, 0.35, 0.2)),
                                    ))
                                    .with_child((
                                        Text::new("[ ¡EVOLUCIÓN! ]"),
                                        TextFont::from_font_size(11.0),
                                        TextColor(Color::WHITE),
                                    ));
                                    r.spawn((
                                        Text::new(evo.clone()),
                                        TextFont::from_font_size(13.0),
                                        TextColor(Color::srgb(1.0, 0.9, 0.4)),
                                    ));
                                });
                        }
                    });
            });

            // ========================================================
            // SECCIÓN DE DRAFT: ELIGE 1 CARTA DE RECOMPENSA PARA TU MAZO
            // ========================================================
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                margin: UiRect::axes(Val::Px(0.0), Val::Px(6.0)),
                ..default()
            })
            .with_children(|draft_section| {
                draft_section.spawn((
                    VictoryCardFeedbackText,
                    Text::new("RECOMPENSA DE CARTA: ELIGE UNA PARA AÑADIR A TU MAZO"),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::srgb(1.0, 0.85, 0.2)),
                ));

                // Contenedor de las 3 cartas
                draft_section
                    .spawn((
                        VictoryCardContainer,
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(20.0),
                            margin: UiRect::axes(Val::Px(0.0), Val::Px(8.0)),
                            ..default()
                        },
                    ))
                    .with_children(|cards_row| {
                        for (i, card) in victory.reward_cards.iter().enumerate() {
                            let border_col = element_color(card.element);
                            let desc = card.description();

                            cards_row
                                .spawn((
                                    Button,
                                    VictoryRewardCardButton { index: i },
                                    Node {
                                        width: Val::Px(148.0),
                                        height: Val::Px(198.0),
                                        flex_direction: FlexDirection::Column,
                                        justify_content: JustifyContent::SpaceBetween,
                                        align_items: AlignItems::Center,
                                        border: UiRect::all(Val::Px(2.0)),
                                        border_radius: BorderRadius::all(Val::Px(6.0)),
                                        overflow: Overflow::clip(),
                                        ..default()
                                    },
                                    UiTransform::default(),
                                    GlobalZIndex(10),
                                    CardHoverAnimation::new(border_col).with_lift(-16.0).with_scale(1.08),
                                    BackgroundColor(Color::srgba(0.06, 0.08, 0.12, 0.2)),
                                    BorderColor(border_col),
                                ))
                                .with_children(|c_box| {
                                    // Fondo con el asset ilustrado del tipo de carta
                                    c_box.spawn((
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

                                    // Cabecera: Nombre y Coste EN
                                    c_box
                                        .spawn(Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(30.0),
                                            margin: UiRect::top(Val::Px(12.0)),
                                            padding: UiRect::axes(Val::Px(14.0), Val::Px(2.0)),
                                            flex_direction: FlexDirection::Row,
                                            justify_content: JustifyContent::SpaceBetween,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        })
                                        .with_children(|h| {
                                            h.spawn((
                                                Text::new(card.display_name()),
                                                TextFont::from_font_size(12.0),
                                                TextColor(Color::WHITE),
                                            ));
                                            h.spawn((
                                                Text::new(format!("{} EN", card.energy_cost)),
                                                TextFont::from_font_size(12.0),
                                                TextColor(Color::srgb(1.0, 0.85, 0.2)),
                                            ));
                                        });

                                    // Espacio central para el arte elemental
                                    c_box.spawn(Node {
                                        flex_grow: 1.0,
                                        ..default()
                                    });

                                    // Descripción en banner inferior
                                    c_box.spawn(Node {
                                        width: Val::Percent(86.0),
                                        min_height: Val::Px(36.0),
                                        margin: UiRect::bottom(Val::Px(14.0)),
                                        padding: UiRect::new(Val::Px(8.0), Val::Px(26.0), Val::Px(4.0), Val::Px(4.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::FlexStart,
                                        ..default()
                                    }).with_child((
                                        Text::new(desc),
                                        TextFont::from_font_size(9.5),
                                        TextColor(Color::srgb(0.95, 0.95, 0.95)),
                                    ));
                                });
                        }
                    });

                // Botón para saltar la carta si no desea diluir su mazo
                draft_section
                    .spawn((
                        Button,
                        VictorySkipCardButton,
                        Node {
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(4.0)),
                            margin: UiRect::top(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.18, 0.20, 0.26)),
                        BorderColor(Color::srgb(0.4, 0.45, 0.55)),
                    ))
                    .with_child((
                        Text::new("SALTAR RECOMPENSA DE CARTA"),
                        TextFont::from_font_size(11.0),
                        TextColor(Color::srgb(0.75, 0.78, 0.85)),
                    ));
            });

            // ========================================================
            // BOTÓN PRINCIPAL DE CONTINUAR AL MAPA
            // ========================================================
            let is_run_completed = run_state.current_floor > 45;
            let (btn_text, btn_bg, btn_border) = if is_run_completed {
                (
                    "VER SALÓN DE LA FAMA (LIGA CONQUISTADA)",
                    Color::srgb(0.8, 0.65, 0.1),
                    Color::srgb(1.0, 0.85, 0.3),
                )
            } else {
                (
                    "CONTINUAR AL MAPA",
                    Color::srgb(0.18, 0.48, 0.28),
                    Color::srgb(0.35, 0.80, 0.45),
                )
            };

            root.spawn((
                Button,
                VictoryContinueButton,
                Node {
                    width: Val::Px(280.0),
                    height: Val::Px(44.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(btn_bg),
                BorderColor(btn_border),
            ))
            .with_child((
                Text::new(btn_text),
                TextFont::from_font_size(14.0),
                TextColor(Color::WHITE),
            ));
        });
}

/// Animación procedural de celebración del Pokémon del jugador (saltos alegres repetidos)
/// utilizando `UiTransform` para evitar recalcular el layout Flexbox (Taffy) cada frame.
fn animate_victory_celebration(
    time: Res<Time>,
    mut q_sprite: Query<(&mut VictoryCelebrationSprite, &mut UiTransform)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (mut celebration, mut transform) in q_sprite.iter_mut() {
        celebration.elapsed += dt;
        // Bouncing jubiloso continuo con squash & stretch sutil
        let wave = (celebration.elapsed * 7.5).sin().abs();
        let bounce = wave * 18.0;
        transform.translation = Val2::px(0.0, -bounce);
        transform.scale = Vec2::new(1.0 - wave * 0.03, 1.0 + wave * 0.05);
    }
}

/// Animación de las estrellas y destellos de victoria que orbitan al Pokémon usando `UiTransform`
fn animate_victory_stars(
    time: Res<Time>,
    mut q_stars: Query<(&mut VictoryStarParticle, &mut UiTransform, &mut TextColor)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (mut star, mut transform, mut color) in q_stars.iter_mut() {
        star.elapsed += dt;
        let progress = (star.elapsed % star.lifetime) / star.lifetime;

        // Subida vertical con oscilación sinusoidal horizontal mediante UiTransform
        let dy = -progress * 90.0;
        let dx = (star.elapsed * 4.0 + star.phase).sin() * 12.0;

        transform.translation = Val2::px(dx, dy);

        // Destello de brillo
        let alpha = if progress < 0.2 {
            progress / 0.2
        } else if progress > 0.8 {
            (1.0 - progress) / 0.2
        } else {
            1.0
        };
        color.0 = Color::srgba(1.0, 0.85, 0.2, alpha.clamp(0.0, 1.0));
    }
}

/// Permite elegir 1 de las cartas de recompensa para añadirla al mazo del jugador
fn handle_victory_card_click(
    mut commands: Commands,
    mut q_buttons: Query<(&Interaction, &VictoryRewardCardButton), Changed<Interaction>>,
    mut run_state: ResMut<CurrentRunState>,
    mut victory_data: Option<ResMut<CombatVictoryData>>,
    q_card_container: Query<Entity, With<VictoryCardContainer>>,
    mut q_feedback: Query<&mut Text, With<VictoryCardFeedbackText>>,
) {
    let Some(ref mut victory) = victory_data else { return };
    if victory.card_chosen {
        return;
    }

    for (interaction, btn) in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            if btn.index < victory.reward_cards.len() {
                let chosen_card = victory.reward_cards[btn.index].clone();
                run_state.add_log(format!("[CARTA] ¡Has añadido {} a tu mazo de combate!", chosen_card.display_name()));
                run_state.player_deck.push(chosen_card.clone());
                victory.card_chosen = true;

                // Actualiza mensaje de feedback
                if let Ok(mut text) = q_feedback.single_mut() {
                    **text = format!("* ¡Añadida {} a tu mazo de combate con éxito! *", chosen_card.display_name());
                }

                // Despawn container de cartas para que no se puedan escoger más
                if let Ok(container) = q_card_container.single() {
                    commands.entity(container).despawn();
                }
            }
        }
    }
}

/// Permite saltar la recompensa de cartas sin añadir ninguna al mazo
fn handle_skip_card_click(
    mut commands: Commands,
    mut q_button: Query<&Interaction, (With<VictorySkipCardButton>, Changed<Interaction>)>,
    mut victory_data: Option<ResMut<CombatVictoryData>>,
    q_card_container: Query<Entity, With<VictoryCardContainer>>,
    mut q_feedback: Query<&mut Text, With<VictoryCardFeedbackText>>,
) {
    let Some(ref mut victory) = victory_data else { return };
    if victory.card_chosen {
        return;
    }

    for interaction in q_button.iter_mut() {
        if *interaction == Interaction::Pressed {
            victory.card_chosen = true;
            if let Ok(mut text) = q_feedback.single_mut() {
                **text = "Recompensa de cartas omitida.".to_string();
            }
            if let Ok(container) = q_card_container.single() {
                commands.entity(container).despawn();
            }
        }
    }
}

/// Al pulsar el botón "CONTINUAR AL MAPA", autoguarda la partida y realiza la transición suave de escena
fn handle_continue_click(
    mut q_continue: Query<&Interaction, (With<VictoryContinueButton>, Changed<Interaction>)>,
    mut ev_trans: MessageWriter<TransitionToEvent>,
    mut ev_notify: MessageWriter<crate::save::SaveNotificationEvent>,
    active_slot: Option<Res<crate::save::ActiveSaveSlot>>,
    meta: Res< crate::run::MetaProfile>,
    run_state: Res<CurrentRunState>,
    map_res: Option<Res<crate::map::CurrentMap>>,
) {
    for interaction in q_continue.iter_mut() {
        if *interaction == Interaction::Pressed {
            let slot = active_slot.as_ref().map(|s| s.0).unwrap_or(1);
            let map_ref = map_res.as_ref().map(|m| &**m);
            if crate::save::save_profile(slot, &meta, &run_state, map_ref).is_ok() {
                ev_notify.write(crate::save::SaveNotificationEvent {
                    message: format!(
                        "[AUTO-GUARDADO] Victoria guardada en el Perfil #{} (Acto {} - Piso {})",
                        slot, run_state.current_act, run_state.current_floor
                    ),
                    is_error: false,
                });
            }
            let target = if run_state.current_floor > 45 {
                AppState::RunSummary
            } else {
                AppState::MapNavigation
            };
            ev_trans.write(TransitionToEvent { target });
        }
    }
}

/// Feedback visual al pasar el cursor sobre los botones interactivos
fn victory_button_hover(
    mut q_buttons: Query<(&Interaction, &mut BackgroundColor, Option<&VictoryContinueButton>), Changed<Interaction>>,
) {
    for (interaction, mut bg, is_continue) in q_buttons.iter_mut() {
        match *interaction {
            Interaction::Hovered => {
                if is_continue.is_some() {
                    bg.0 = Color::srgb(0.25, 0.65, 0.38);
                } else {
                    bg.0 = Color::srgb(0.24, 0.28, 0.38);
                }
            }
            Interaction::None => {
                if is_continue.is_some() {
                    bg.0 = Color::srgb(0.18, 0.48, 0.28);
                } else {
                    bg.0 = Color::srgb(0.14, 0.16, 0.24);
                }
            }
            _ => {}
        }
    }
}

fn cleanup_victory_ui(mut commands: Commands, q_cleanup: Query<Entity, With<CleanupUi>>) {
    for entity in q_cleanup.iter() {
        commands.entity(entity).despawn();
    }
}
