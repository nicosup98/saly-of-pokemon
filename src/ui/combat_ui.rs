use bevy::prelude::*;
use crate::card::{Card, InHand};
use crate::combat::events::{
    CombatAnimEvent, CombatAnimKind, CombatSide, EndPlayerTurnEvent, PlayCardEvent, UseConsumableEvent,
};
use crate::combat::{PlayerPokemon, EnemyPokemon};
use crate::combat::ai::{EnemyAI, EnemyIntentType};
use crate::pokemon::PokemonSprite;
use crate::run::{AppState, CurrentRunState};
use crate::stats::PokemonStats;
use crate::ui::{element_color, CleanupUi, Button, BorderColor};

pub struct CombatUiPlugin;

impl Plugin for CombatUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CombatAnimEvent>()
            // La UI lee a los combatientes: deben existir ya (los crea `setup_combat`)
            .add_systems(OnEnter(AppState::Combat), setup_combat_ui.after(crate::combat::setup_combat))
            .add_systems(
                Update,
                (
                    update_combat_hud,
                    update_hand_ui,
                    handle_card_click,
                    handle_end_turn_click,
                    handle_consumable_click,
                    queue_combat_animations,
                    animate_sprites,
                    animate_floating_text,
                )
                    .run_if(in_state(AppState::Combat)),
            )
            .add_systems(OnExit(AppState::Combat), cleanup_combat_ui);
    }
}

#[derive(Component)]
pub struct CombatUiRoot;

#[derive(Component)]
pub struct PlayerHpBarFill;

#[derive(Component)]
pub struct PlayerHpText;

#[derive(Component)]
pub struct PlayerBlockText;

#[derive(Component)]
pub struct PlayerSpriteImage;

#[derive(Component)]
pub struct PlayerStatsText;

#[derive(Component)]
pub struct EnemyHpBarFill;

#[derive(Component)]
pub struct EnemyHpText;

#[derive(Component)]
pub struct EnemyBlockText;

#[derive(Component)]
pub struct EnemyIntentText;

#[derive(Component)]
pub struct EnemySpriteImage;

#[derive(Component)]
pub struct EnergyOrbText;

#[derive(Component)]
pub struct HandContainer;

#[derive(Component)]
pub struct CardButton {
    pub index: usize,
}

#[derive(Component)]
pub struct EndTurnButton;

#[derive(Component)]
pub struct ConsumableButton {
    pub slot: usize,
}

#[derive(Component)]
pub struct CombatLogFeed;

fn setup_combat_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    run_state: Res<CurrentRunState>,
    q_player: Query<(&PokemonStats, &PokemonSprite, Option<&crate::pokemon::TrainerSprite>), With<PlayerPokemon>>,
    q_enemy: Query<(&PokemonStats, &PokemonSprite, Option<&crate::pokemon::TrainerSprite>), With<EnemyPokemon>>,
) {
    let (player_stats, player_sprite, player_trainer) = q_player
        .single()
        .map(|(s, sp, t)| {
            (
                s.clone(),
                sp.clone(),
                t.cloned().unwrap_or_else(crate::pokemon::TrainerSprite::player_default),
            )
        })
        .unwrap_or_else(|_| {
            (
                PokemonStats::new(4, "Charmander", 5, vec![], vec![], 39, 52, 43, 60, 50, 65),
                PokemonSprite::new(4, true),
                crate::pokemon::TrainerSprite::player_default(),
            )
        });

    let (enemy_stats, enemy_sprite, enemy_trainer) = q_enemy
        .single()
        .map(|(s, sp, t)| {
            (
                s.clone(),
                sp.clone(),
                t.cloned().unwrap_or_else(|| crate::pokemon::TrainerSprite::new(0, false, "Entrenador Rival")),
            )
        })
        .unwrap_or_else(|_| {
            (
                PokemonStats::new(74, "Geodude", 5, vec![], vec![], 40, 80, 100, 30, 30, 20),
                PokemonSprite::new(74, false),
                crate::pokemon::TrainerSprite::new(0, false, "Entrenador Rival"),
            )
        });

    // Contenedor principal que abarca toda la pantalla (1280x720)
    commands
        .spawn((
            CombatUiRoot,
            CleanupUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.07, 0.08, 0.11)),
        ))
        .with_children(|root| {
            // ==========================================
            // 1. BARRA SUPERIOR (Sección 9.1 GDD)
            // ==========================================
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(50.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.14, 0.18)),
                BorderColor(Color::srgb(0.25, 0.28, 0.35)),
            ))
            .with_children(|top_bar| {
                // Izquierda: Nombre, Nivel y Dinero
                top_bar.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(16.0),
                    ..default()
                }).with_children(|left| {
                    left.spawn((
                        Text::new(format!("{} (Nv. {})", player_stats.name, player_stats.level)),
                        TextFont::from_font_size(18.0),
                        TextColor(Color::srgb(0.95, 0.95, 0.95)),
                    ));
                    left.spawn((
                        Text::new(format!("P$ {}", run_state.poke_dollars)),
                        TextFont::from_font_size(18.0),
                        TextColor(Color::srgb(1.0, 0.85, 0.2)),
                    ));
                });

                // Centro: Cinturón de Consumibles (3 ranuras interactivas)
                top_bar.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    ..default()
                }).with_children(|belt| {
                    for slot in 0..3 {
                        let item_text = match &run_state.consumables[slot] {
                            Some(name) => format!("🧪 {}", name),
                            None => "[-]".to_string(),
                        };
                        belt.spawn((
                            Button,
                            ConsumableButton { slot },
                            Node {
                                padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.18, 0.22, 0.28)),
                            BorderColor(Color::srgb(0.35, 0.40, 0.50)),
                        )).with_child((
                            Text::new(item_text),
                            TextFont::from_font_size(14.0),
                            TextColor(Color::WHITE),
                        ));
                    }
                });

                // Derecha: Piso del Acto y Reliquias
                top_bar.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(12.0),
                    ..default()
                }).with_children(|right| {
                    right.spawn((
                        Text::new(format!("Piso {}/15 (Acto {})", run_state.current_floor, run_state.current_act)),
                        TextFont::from_font_size(16.0),
                        TextColor(Color::srgb(0.4, 0.8, 1.0)),
                    ));
                });
            });

            // ==========================================
            // 2. ZONA CENTRAL: ARENA DE COMBATE (GDD 9.1)
            // ==========================================
            root.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Px(310.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(40.0), Val::Px(10.0)),
                ..default()
            }).with_children(|arena| {
                // --- LADO IZQUIERDO: JUGADOR ---
                arena.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(8.0),
                    width: Val::Px(280.0),
                    ..default()
                }).with_children(|p_col| {
                    // Barra de Vida Jugador
                    p_col.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(20.0),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                        BorderColor(Color::srgb(0.4, 0.4, 0.4)),
                    )).with_children(|bar| {
                        bar.spawn((
                            PlayerHpBarFill,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.25, 0.75, 0.35)),
                        ));
                    });

                    // Texto HP y Bloqueo
                    p_col.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        width: Val::Percent(100.0),
                        ..default()
                    }).with_children(|sub| {
                        sub.spawn((
                            PlayerHpText,
                            Text::new(format!("HP: {}/{}", player_stats.hp, player_stats.max_hp)),
                            TextFont::from_font_size(14.0),
                            TextColor(Color::WHITE),
                        ));
                        sub.spawn((
                            PlayerBlockText,
                            Text::new(format!("Bloqueo: {}", player_stats.block)),
                            TextFont::from_font_size(14.0),
                            TextColor(Color::srgb(0.4, 0.7, 1.0)),
                        ));
                    });

                    // Insignia identificativa del Entrenador Jugador
                    p_col.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.20, 0.32)),
                        BorderColor(Color::srgb(0.35, 0.65, 0.95)),
                    )).with_child((
                        Text::new(format!("[ {} ]", player_trainer.name)),
                        TextFont::from_font_size(11.0),
                        TextColor(Color::srgb(0.7, 0.9, 1.0)),
                    ));

                    // Fila de Entrenador (trasero) + Pokémon Jugador
                    p_col.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::FlexEnd,
                        justify_content: JustifyContent::Center,
                        column_gap: Val::Px(8.0),
                        ..default()
                    }).with_children(|combatants| {
                        // Sprite del Entrenador Red en combate (trasero)
                        combatants.spawn((
                            ImageNode::new(asset_server.load(player_trainer.relative_path())),
                            Node {
                                width: Val::Px(56.0),
                                height: Val::Px(56.0),
                                margin: UiRect::bottom(Val::Px(12.0)),
                                ..default()
                            },
                        ));

                        // Sprite del Pokémon Jugador (Back Sprite oficial)
                        combatants.spawn((
                            PlayerSpriteImage,
                            SpriteAnimator::default(),
                            ImageNode::new(asset_server.load(player_sprite.relative_path())),
                            Node {
                                width: Val::Px(140.0),
                                height: Val::Px(140.0),
                                ..default()
                            },
                        ));
                    });

                    // Panel compacto de stats RPG
                    p_col.spawn((
                        PlayerStatsText,
                        Text::new(format!(
                            "ATK: {:.0} | DEF: {:.0} | SP.ATK: {:.0}",
                            player_stats.atk, player_stats.def, player_stats.sp_atk
                        )),
                        TextFont::from_font_size(12.0),
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));
                });

                // --- CENTRO: LOG FEED DE ACCIÓN TÁCTICA ---
                arena.spawn(Node {
                    width: Val::Px(320.0),
                    height: Val::Px(140.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                }).with_children(|log_box| {
                    log_box.spawn((
                        CombatLogFeed,
                        Text::new("[COMBATE] ¡Combate iniciado! Selecciona tus cartas."),
                        TextFont::from_font_size(13.0),
                        TextColor(Color::srgb(0.9, 0.85, 0.6)),
                    ));
                });

                // --- LADO DERECHO: RIVAL (ENEMIGO) ---
                arena.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(8.0),
                    width: Val::Px(280.0),
                    ..default()
                }).with_children(|e_col| {
                    // Intención Anunciada del Enemigo (Transparencia Táctica GDD 2)
                    e_col.spawn((
                        EnemyIntentText,
                        Text::new("[INTENCIÓN] Calculando..."),
                        TextFont::from_font_size(15.0),
                        TextColor(Color::srgb(1.0, 0.35, 0.35)),
                    ));

                    // Barra de Vida Enemigo
                    e_col.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(20.0),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                        BorderColor(Color::srgb(0.4, 0.4, 0.4)),
                    )).with_children(|bar| {
                        bar.spawn((
                            EnemyHpBarFill,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.85, 0.25, 0.25)),
                        ));
                    });

                    // Texto HP y Bloqueo Rival
                    e_col.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        width: Val::Percent(100.0),
                        ..default()
                    }).with_children(|sub| {
                        sub.spawn((
                            EnemyHpText,
                            Text::new(format!("{} (Nv. {}) HP: {}/{}", enemy_stats.name, enemy_stats.level, enemy_stats.hp, enemy_stats.max_hp)),
                            TextFont::from_font_size(14.0),
                            TextColor(Color::WHITE),
                        ));
                        sub.spawn((
                            EnemyBlockText,
                            Text::new(format!("Bloqueo: {}", enemy_stats.block)),
                            TextFont::from_font_size(14.0),
                            TextColor(Color::srgb(0.4, 0.7, 1.0)),
                        ));
                    });

                    // Insignia identificativa del Entrenador Rival
                    e_col.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.24, 0.16, 0.18)),
                        BorderColor(Color::srgb(0.9, 0.4, 0.3)),
                    )).with_child((
                        Text::new(format!("[ {} ]", enemy_trainer.name)),
                        TextFont::from_font_size(11.0),
                        TextColor(Color::srgb(1.0, 0.75, 0.7)),
                    ));

                    // Fila de Pokémon Rival + Entrenador Rival (frontal)
                    e_col.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::FlexEnd,
                        justify_content: JustifyContent::Center,
                        column_gap: Val::Px(8.0),
                        ..default()
                    }).with_children(|combatants| {
                        // Sprite del Pokémon Rival (Front Sprite oficial)
                        combatants.spawn((
                            EnemySpriteImage,
                            SpriteAnimator::default(),
                            ImageNode::new(asset_server.load(enemy_sprite.relative_path())),
                            Node {
                                width: Val::Px(140.0),
                                height: Val::Px(140.0),
                                ..default()
                            },
                        ));

                        // Sprite del Entrenador Rival
                        combatants.spawn((
                            ImageNode::new(asset_server.load(enemy_trainer.relative_path())),
                            Node {
                                width: Val::Px(56.0),
                                height: Val::Px(56.0),
                                margin: UiRect::bottom(Val::Px(12.0)),
                                ..default()
                            },
                        ));
                    });
                });
            });

            // ==========================================
            // 3. ZONA INFERIOR: ENERGÍA, MANO Y FIN DE TURNO
            // ==========================================
            root.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Px(210.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(16.0), Val::Px(4.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            }).with_children(|bottom| {
                // Orbe de Energía (Izquierda)
                bottom.spawn((
                    Node {
                        width: Val::Px(90.0),
                        height: Val::Px(90.0),
                        border: UiRect::all(Val::Px(3.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.18, 0.16, 0.05)),
                    BorderColor(Color::srgb(1.0, 0.85, 0.15)),
                )).with_child((
                    EnergyOrbText,
                    Text::new(format!("{}/{} EN", run_state.energy, run_state.max_energy)),
                    TextFont::from_font_size(20.0),
                    TextColor(Color::srgb(1.0, 0.9, 0.3)),
                ));

                // Abanico de Cartas en Mano (Centro)
                bottom.spawn((
                    HandContainer,
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(12.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        height: Val::Percent(100.0),
                        ..default()
                    },
                ));

                // Botón de Terminar Turno (Derecha)
                bottom.spawn((
                    Button,
                    EndTurnButton,
                    Node {
                        width: Val::Px(120.0),
                        height: Val::Px(60.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.65, 0.22, 0.22)),
                    BorderColor(Color::srgb(0.9, 0.4, 0.4)),
                )).with_child((
                    Text::new("TERMINAR\nTURNO ⏳"),
                    TextFont::from_font_size(15.0),
                    TextColor(Color::WHITE),
                ));
            });
        });
}

fn update_combat_hud(
    run_state: Res<CurrentRunState>,
    q_player: Query<&PokemonStats, (With<PlayerPokemon>, Without<EnemyPokemon>)>,
    q_enemy: Query<(&PokemonStats, &EnemyAI), (With<EnemyPokemon>, Without<PlayerPokemon>)>,
    mut q_p_fill: Query<&mut Node, (With<PlayerHpBarFill>, Without<EnemyHpBarFill>)>,
    mut q_e_fill: Query<&mut Node, (With<EnemyHpBarFill>, Without<PlayerHpBarFill>)>,
    mut q_texts: Query<(
        Entity,
        &mut Text,
        Option<&PlayerHpText>,
        Option<&PlayerBlockText>,
        Option<&PlayerStatsText>,
        Option<&EnemyHpText>,
        Option<&EnemyBlockText>,
        Option<&EnemyIntentText>,
        Option<&EnergyOrbText>,
        Option<&CombatLogFeed>,
    )>,
) {
    let player_data = q_player.single().ok();
    let enemy_data = q_enemy.single().ok();

    if let Some(player) = player_data {
        let pct = (player.hp as f32 / player.max_hp as f32 * 100.0).clamp(0.0, 100.0);
        if let Ok(mut fill) = q_p_fill.single_mut() {
            fill.width = Val::Percent(pct);
        }
    }

    if let Some((enemy, _)) = enemy_data {
        let pct = (enemy.hp as f32 / enemy.max_hp as f32 * 100.0).clamp(0.0, 100.0);
        if let Ok(mut fill) = q_e_fill.single_mut() {
            fill.width = Val::Percent(pct);
        }
    }

    for (_, mut text, p_hp, p_block, p_stats, e_hp, e_block, e_intent, orb, log) in q_texts.iter_mut() {
        if p_hp.is_some() {
            if let Some(player) = player_data {
                **text = format!("HP: {}/{}", player.hp, player.max_hp);
            }
        } else if p_block.is_some() {
            if let Some(player) = player_data {
                **text = format!("🛡️ {}", player.block);
            }
        } else if p_stats.is_some() {
            if let Some(player) = player_data {
                **text = format!(
                    "⚔️ Atk: {:.0} | 🛡️ Def: {:.0} | ✨ Sp.Atk: {:.0}",
                    player.effective_atk(), player.effective_def(), player.effective_sp_atk()
                );
            }
        } else if e_hp.is_some() {
            if let Some((enemy, _)) = enemy_data {
                **text = format!("{} (Nv. {}) HP: {}/{}", enemy.name, enemy.level, enemy.hp, enemy.max_hp);
            }
        } else if e_block.is_some() {
            if let Some((enemy, _)) = enemy_data {
                **text = format!("🛡️ {}", enemy.block);
            }
        } else if e_intent.is_some() {
            if let Some((_, ai)) = enemy_data {
                if let Some(intent) = &ai.current_intent {
                    match intent {
                        EnemyIntentType::Attack { raw_damage, .. } => {
                            let player_def = player_data.map(|p| p.def).unwrap_or(0.0);
                            let dmg = (raw_damage * PokemonStats::calc_mitigation(player_def)).round() as i32;
                            **text = format!("[ATACAR] Por {} daño", dmg);
                        }
                        EnemyIntentType::Defend { block_amount } => {
                            **text = format!("[DEFENDER] Ganar {} Bloqueo", block_amount);
                        }
                        EnemyIntentType::Buff { status, .. } => {
                            **text = format!("[POTENCIAR] {}", status.display_name());
                        }
                        EnemyIntentType::Debuff { status, .. } => {
                            **text = format!("[ESTADO] {}", status.display_name());
                        }
                    }
                }
            }
        } else if orb.is_some() {
            **text = format!("{}/{} EN", run_state.energy, run_state.max_energy);
        } else if log.is_some() {
            if let Some(last_log) = run_state.battle_logs.last() {
                **text = last_log.clone();
            }
        }
    }
}

/// Sincroniza las cartas de la mano con botones interactivos 2D en pantalla
fn update_hand_ui(
    mut commands: Commands,
    q_hand_container: Query<Entity, With<HandContainer>>,
    q_hand_cards: Query<&Card, With<InHand>>,
    q_existing_buttons: Query<Entity, With<CardButton>>,
) {
    let Ok(container_entity) = q_hand_container.single() else { return };
    let cards: Vec<&Card> = q_hand_cards.iter().collect();

    // Comprueba si la cantidad cambió para redibujar
    let existing_count = q_existing_buttons.iter().count();
    if existing_count == cards.len() {
        return;
    }

    // Limpia botones viejos
    for b in q_existing_buttons.iter() {
        commands.entity(b).despawn();
    }

    // Genera las cartas interactivas según el GDD (Sección 9.1)
    commands.entity(container_entity).with_children(|parent| {
        for (i, card) in cards.into_iter().enumerate() {
            let border_col = element_color(card.element);

            parent.spawn((
                Button,
                CardButton { index: i },
                Node {
                    width: Val::Px(140.0),
                    height: Val::Px(190.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::all(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.14, 0.16, 0.22)),
                BorderColor(border_col),
            )).with_children(|card_box| {
                // Cabecera: Nombre y Coste
                card_box.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                }).with_children(|h| {
                    h.spawn((
                        Text::new(card.display_name()),
                        TextFont::from_font_size(13.0),
                        TextColor(Color::WHITE),
                    ));
                    h.spawn((
                        Text::new(format!("{} EN", card.energy_cost)),
                        TextFont::from_font_size(13.0),
                        TextColor(Color::srgb(1.0, 0.85, 0.2)),
                    ));
                });

                // Insignia de Tipo y Categoría
                card_box.spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                        align_self: AlignSelf::Center,
                        ..default()
                    },
                    BackgroundColor(border_col),
                )).with_child((
                    Text::new(format!("{:?}", card.element).to_uppercase()),
                    TextFont::from_font_size(10.0),
                    TextColor(Color::srgb(0.1, 0.1, 0.1)),
                ));

                // Descripción del efecto con multiplicador y estado
                let desc = card.description();

                card_box.spawn((
                    Text::new(desc),
                    TextFont::from_font_size(10.0),
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                ));

                // Botón "JUGAR"
                card_box.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(24.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.5, 0.25)),
                )).with_child((
                    Text::new("JUGAR"),
                    TextFont::from_font_size(11.0),
                    TextColor(Color::WHITE),
                ));
            });
        }
    });
}

fn handle_card_click(
    mut q_buttons: Query<(&Interaction, &CardButton), Changed<Interaction>>,
    mut ev_play: MessageWriter<PlayCardEvent>,
    q_enemy: Query<&PokemonStats, With<EnemyPokemon>>,
    q_player: Query<&PokemonStats, With<PlayerPokemon>>,
) {
    if let Ok(enemy) = q_enemy.single() {
        if enemy.is_fainted() {
            return;
        }
    }
    if let Ok(player) = q_player.single() {
        if player.is_fainted() {
            return;
        }
    }
    for (interaction, card_btn) in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            ev_play.write(PlayCardEvent {
                card_index: card_btn.index,
            });
        }
    }
}

fn handle_end_turn_click(
    mut q_button: Query<&Interaction, (With<EndTurnButton>, Changed<Interaction>)>,
    mut ev_end: MessageWriter<EndPlayerTurnEvent>,
    q_enemy: Query<&PokemonStats, With<EnemyPokemon>>,
    q_player: Query<&PokemonStats, With<PlayerPokemon>>,
) {
    if let Ok(enemy) = q_enemy.single() {
        if enemy.is_fainted() {
            return;
        }
    }
    if let Ok(player) = q_player.single() {
        if player.is_fainted() {
            return;
        }
    }
    for interaction in q_button.iter_mut() {
        if *interaction == Interaction::Pressed {
            ev_end.write(EndPlayerTurnEvent);
        }
    }
}

fn handle_consumable_click(
    mut q_buttons: Query<(&Interaction, &ConsumableButton), Changed<Interaction>>,
    mut ev_use: MessageWriter<UseConsumableEvent>,
    q_enemy: Query<&PokemonStats, With<EnemyPokemon>>,
    q_player: Query<&PokemonStats, With<PlayerPokemon>>,
) {
    if let Ok(enemy) = q_enemy.single() {
        if enemy.is_fainted() {
            return;
        }
    }
    if let Ok(player) = q_player.single() {
        if player.is_fainted() {
            return;
        }
    }
    for (interaction, btn) in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            ev_use.write(UseConsumableEvent {
                slot_index: btn.slot,
            });
        }
    }
}

fn cleanup_combat_ui(mut commands: Commands, q_cleanup: Query<Entity, With<CleanupUi>>) {
    for entity in q_cleanup.iter() {
        commands.entity(entity).despawn();
    }
}

// ============================================================
// ANIMACIONES BÁSICAS DE COMBATE (ataque, defensa, daño, curación)
// ============================================================

/// Forma de un efecto visual aplicado a un sprite.
#[derive(Clone, Copy, Debug)]
enum AnimShape {
    /// Embestida horizontal de ida y vuelta (`dir`: +1 hacia la derecha, -1 hacia la izquierda).
    Lunge(f32),
    /// Sacudida horizontal amortiguada (al recibir un golpe).
    Shake,
    /// Salto vertical corto (guardia / potenciador / curación).
    Hop,
    /// Salto de victoria alegre y repetido del Pokémon.
    CelebrateJump,
    /// Desvanecimiento y caída cuando el Pokémon se debilita.
    Faint,
    /// Destello de color que se desvanece.
    Flash(Color),
}

#[derive(Clone, Copy, Debug)]
struct AnimEffect {
    shape: AnimShape,
    /// Segundos que faltan para que el efecto empiece.
    delay: f32,
    elapsed: f32,
    duration: f32,
}

/// Cola de efectos activos de un sprite de combate.
#[derive(Component, Default)]
pub struct SpriteAnimator {
    effects: Vec<AnimEffect>,
    /// `true` mientras el sprite pueda estar desplazado o teñido y haya que restaurarlo.
    active: bool,
}

/// Texto flotante (daño, bloqueo, curación) que sube y se desvanece.
#[derive(Component)]
pub struct FloatingText {
    delay: f32,
    elapsed: f32,
    duration: f32,
    start_top: f32,
    rgb: (f32, f32, f32),
}

const DAMAGE_COLOR: Color = Color::srgb(1.0, 0.30, 0.30);
const BLOCK_COLOR: Color = Color::srgb(0.35, 0.65, 1.0);
const HEAL_COLOR: Color = Color::srgb(0.35, 0.90, 0.45);
const BUFF_COLOR: Color = Color::srgb(1.0, 0.85, 0.30);

/// Traduce los eventos de combate en efectos visuales sobre los sprites.
fn queue_combat_animations(
    mut commands: Commands,
    mut ev_anim: MessageReader<CombatAnimEvent>,
    mut q_player: Query<(Entity, &mut SpriteAnimator), (With<PlayerSpriteImage>, Without<EnemySpriteImage>)>,
    mut q_enemy: Query<(Entity, &mut SpriteAnimator), (With<EnemySpriteImage>, Without<PlayerSpriteImage>)>,
    mut spawned: Local<u32>,
) {
    for ev in ev_anim.read() {
        let target = match ev.side {
            CombatSide::Player => q_player.single_mut().ok(),
            CombatSide::Enemy => q_enemy.single_mut().ok(),
        };
        let Some((entity, mut animator)) = target else { continue };

        // Retraso del golpe recibido para que coincida con el final de la embestida
        const HIT_DELAY: f32 = 0.12;

        let (label, color): (Option<String>, Color) = match ev.kind {
            CombatAnimKind::Strike => {
                let dir = if ev.side == CombatSide::Player { 1.0 } else { -1.0 };
                animator.effects.push(AnimEffect { shape: AnimShape::Lunge(dir), delay: 0.0, elapsed: 0.0, duration: 0.30 });
                (None, Color::WHITE)
            }
            CombatAnimKind::Damage(amount) => {
                animator.effects.push(AnimEffect { shape: AnimShape::Shake, delay: HIT_DELAY, elapsed: 0.0, duration: 0.35 });
                animator.effects.push(AnimEffect { shape: AnimShape::Flash(DAMAGE_COLOR), delay: HIT_DELAY, elapsed: 0.0, duration: 0.40 });
                let text = if amount > 0 { format!("-{}", amount) } else { "0".to_string() };
                (Some(text), DAMAGE_COLOR)
            }
            CombatAnimKind::Block(amount) => {
                animator.effects.push(AnimEffect { shape: AnimShape::Hop, delay: 0.0, elapsed: 0.0, duration: 0.30 });
                animator.effects.push(AnimEffect { shape: AnimShape::Flash(BLOCK_COLOR), delay: 0.0, elapsed: 0.0, duration: 0.55 });
                (Some(format!("+{} BLQ", amount)), BLOCK_COLOR)
            }
            CombatAnimKind::Heal(amount) => {
                animator.effects.push(AnimEffect { shape: AnimShape::Hop, delay: 0.0, elapsed: 0.0, duration: 0.30 });
                animator.effects.push(AnimEffect { shape: AnimShape::Flash(HEAL_COLOR), delay: 0.0, elapsed: 0.0, duration: 0.55 });
                (Some(format!("+{} HP", amount)), HEAL_COLOR)
            }
            CombatAnimKind::Buff => {
                animator.effects.push(AnimEffect { shape: AnimShape::Hop, delay: 0.0, elapsed: 0.0, duration: 0.30 });
                animator.effects.push(AnimEffect { shape: AnimShape::Flash(BUFF_COLOR), delay: 0.0, elapsed: 0.0, duration: 0.50 });
                (Some("UP".to_string()), BUFF_COLOR)
            }
            CombatAnimKind::Victory => {
                animator.effects.push(AnimEffect { shape: AnimShape::CelebrateJump, delay: 0.0, elapsed: 0.0, duration: 0.80 });
                animator.effects.push(AnimEffect { shape: AnimShape::Flash(Color::srgb(1.0, 0.85, 0.2)), delay: 0.0, elapsed: 0.0, duration: 0.80 });
                (Some("¡VICTORIA!".to_string()), Color::srgb(1.0, 0.85, 0.2))
            }
            CombatAnimKind::Faint => {
                animator.effects.push(AnimEffect { shape: AnimShape::Faint, delay: 0.0, elapsed: 0.0, duration: 0.80 });
                animator.effects.push(AnimEffect { shape: AnimShape::Flash(Color::srgb(0.4, 0.4, 0.4)), delay: 0.0, elapsed: 0.0, duration: 0.60 });
                (Some("[DEBILITADO]".to_string()), Color::srgb(0.65, 0.65, 0.70))
            }
        };
        animator.active = true;

        if let Some(label) = label {
            // Los textos consecutivos se reparten en horizontal para no solaparse
            let slot = *spawned % 3;
            *spawned = spawned.wrapping_add(1);
            let delay = if matches!(ev.kind, CombatAnimKind::Damage(_)) { HIT_DELAY } else { 0.0 };
            let c = color.to_srgba();

            commands.entity(entity).with_children(|sprite| {
                sprite.spawn((
                    FloatingText {
                        delay,
                        elapsed: 0.0,
                        duration: 0.9,
                        start_top: 30.0,
                        rgb: (c.red, c.green, c.blue),
                    },
                    Text::new(label),
                    TextFont::from_font_size(30.0),
                    TextColor(Color::srgba(c.red, c.green, c.blue, 0.0)),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(30.0 + slot as f32 * 34.0),
                        top: Val::Px(30.0),
                        ..default()
                    },
                ));
            });
        }
    }
}

/// Avanza los efectos de cada sprite y los aplica como desplazamiento y tinte.
fn animate_sprites(
    time: Res<Time>,
    mut q_sprites: Query<(&mut SpriteAnimator, &mut Node, &mut ImageNode)>,
) {
    let dt = time.delta_secs();

    for (mut animator, mut node, mut image) in q_sprites.iter_mut() {
        if animator.effects.is_empty() {
            if animator.active {
                animator.active = false;
                node.left = Val::Px(0.0);
                node.top = Val::Px(0.0);
                image.color = Color::WHITE;
            }
            continue;
        }

        let mut offset_x = 0.0f32;
        let mut offset_y = 0.0f32;
        // Destello dominante: el de mayor intensidad entre los activos
        let mut flash: Option<(Color, f32)> = None;

        for effect in animator.effects.iter_mut() {
            if effect.delay > 0.0 {
                effect.delay -= dt;
                continue;
            }
            effect.elapsed += dt;
            let p = (effect.elapsed / effect.duration).clamp(0.0, 1.0);

            match effect.shape {
                AnimShape::Lunge(dir) => offset_x += dir * 90.0 * (std::f32::consts::PI * p).sin(),
                AnimShape::Shake => offset_x += (effect.elapsed * 70.0).sin() * 14.0 * (1.0 - p),
                AnimShape::Hop => offset_y -= 22.0 * (std::f32::consts::PI * p).sin(),
                AnimShape::CelebrateJump => {
                    // Saltos repetidos de júbilo
                    offset_y -= (effect.elapsed * 18.0).sin().abs() * 26.0 * (1.0 - p * 0.4);
                }
                AnimShape::Faint => {
                    // Caída y desvanecimiento
                    offset_y += p * 38.0;
                    offset_x += (effect.elapsed * 30.0).sin() * 3.0 * (1.0 - p);
                }
                AnimShape::Flash(color) => {
                    let strength = 1.0 - p;
                    if flash.map_or(true, |(_, s)| strength > s) {
                        flash = Some((color, strength));
                    }
                }
            }
        }

        animator.effects.retain(|e| e.delay > 0.0 || e.elapsed < e.duration);

        node.left = Val::Px(offset_x);
        node.top = Val::Px(offset_y);
        image.color = match flash {
            Some((color, strength)) => {
                let c = color.to_srgba();
                // Interpolación blanco -> color según la intensidad del destello
                Color::srgb(
                    1.0 + (c.red - 1.0) * strength,
                    1.0 + (c.green - 1.0) * strength,
                    1.0 + (c.blue - 1.0) * strength,
                )
            }
            None => Color::WHITE,
        };
    }
}

/// Mueve hacia arriba y desvanece los textos flotantes; los elimina al terminar.
fn animate_floating_text(
    mut commands: Commands,
    time: Res<Time>,
    mut q_texts: Query<(Entity, &mut FloatingText, &mut Node, &mut TextColor)>,
) {
    let dt = time.delta_secs();

    for (entity, mut floating, mut node, mut color) in q_texts.iter_mut() {
        if floating.delay > 0.0 {
            floating.delay -= dt;
            continue;
        }

        floating.elapsed += dt;
        let p = (floating.elapsed / floating.duration).clamp(0.0, 1.0);

        node.top = Val::Px(floating.start_top - 70.0 * p);
        // Opaco durante el primer 55% y luego se desvanece
        let alpha = if p < 0.55 { 1.0 } else { 1.0 - (p - 0.55) / 0.45 };
        let (r, g, b) = floating.rgb;
        color.0 = Color::srgba(r, g, b, alpha);

        if p >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}
