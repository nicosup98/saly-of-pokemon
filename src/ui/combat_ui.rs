use bevy::prelude::*;
use crate::card::{Card, InHand, InDrawPile, InDiscardPile};
use crate::combat::events::{
    CombatAnimEvent, CombatAnimKind, CombatSide, EndPlayerTurnEvent, PlayCardEvent, UseConsumableEvent,
    DeckShuffleAnimEvent,
};
use crate::combat::{PlayerPokemon, EnemyPokemon};
use crate::combat::ai::{EnemyAI, EnemyIntentType};
use crate::pokemon::PokemonSprite;
use crate::run::{AppState, CurrentRunState};
use crate::stats::PokemonStats;
use crate::ui::{element_color, CleanupUi, Button, BorderColor, CardHoverAnimation};

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
                    trigger_deck_shuffle_animation,
                )
                    .run_if(in_state(AppState::Combat)),
            )
            .add_systems(
                Update,
                (
                    animate_card_discard_flight,
                    animate_deck_shuffle,
                ),
            )
            .add_systems(OnExit(AppState::Combat), cleanup_combat_ui);
    }
}

#[derive(Component)]
pub struct CombatUiRoot;

#[derive(Component)]
pub struct DrawPileWidget;

#[derive(Component)]
pub struct DrawPileCountText;

#[derive(Component)]
pub struct DiscardPileWidget;

#[derive(Component)]
pub struct DiscardPileCountText;

#[derive(Component)]
pub struct CardDiscardFlight {
    pub timer: Timer,
    pub start_pos: Vec2,
    pub target_pos: Vec2,
    pub card_name: String,
}

#[derive(Component)]
pub struct DeckShuffleFlight {
    pub timer: Timer,
    pub delay_timer: Timer,
    pub start_pos: Vec2,
    pub target_pos: Vec2,
    pub arc_height: f32,
    pub rotation_offset: f32,
}

#[derive(Component)]
pub struct DeckShuffleBanner {
    pub timer: Timer,
}

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
pub struct PlayerStatusesText;

#[derive(Component)]
pub struct EnemyHpBarFill;

#[derive(Component)]
pub struct EnemyHpText;

#[derive(Component)]
pub struct EnemyBlockText;

#[derive(Component)]
pub struct EnemyStatusesText;

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

/// Retorna la ruta relativa del fondo correspondiente para la situación de combate
pub fn combat_background_path(
    node_type: Option<crate::map::generator::NodeType>,
    is_boss: bool,
    is_mini_boss: bool,
) -> &'static str {
    use crate::map::generator::NodeType;
    match node_type {
        Some(NodeType::GymLeader) => "fondos/gym_blaine.jpeg",
        Some(NodeType::MiniBoss) => "fondos/gym_misty.jpeg",
        Some(NodeType::Enemy) => "fondos/fondo_pelea.jpeg",
        _ => {
            if is_boss {
                "fondos/gym_blaine.jpeg"
            } else if is_mini_boss {
                "fondos/gym_misty.jpeg"
            } else {
                "fondos/fondo_pelea.jpeg"
            }
        }
    }
}

fn setup_combat_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    run_state: Res<CurrentRunState>,
    map: Option<Res<crate::map::CurrentMap>>,
    q_player: Query<(&PokemonStats, &PokemonSprite, Option<&crate::pokemon::TrainerSprite>), With<PlayerPokemon>>,
    q_enemy: Query<(&PokemonStats, &PokemonSprite, Option<&crate::pokemon::TrainerSprite>, Option<&crate::combat::ai::EnemyAI>), With<EnemyPokemon>>,
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
        .iter()
        .next()
        .map(|(s, sp, t, _)| {
            (
                s.clone(),
                sp.clone(),
                t.cloned().unwrap_or_else(|| crate::pokemon::TrainerSprite::new(0, false, "Entrenador Rival")),
            )
        })
        .unwrap_or_else(|| {
            (
                PokemonStats::new(74, "Geodude", 5, vec![], vec![], 40, 80, 100, 30, 30, 20),
                PokemonSprite::new(74, false),
                crate::pokemon::TrainerSprite::new(0, false, "Entrenador Rival"),
            )
        });

    let (is_boss, is_mini_boss) = q_enemy
        .iter()
        .next()
        .and_then(|(_, _, _, maybe_ai)| maybe_ai.map(|ai| (ai.is_boss, ai.is_mini_boss)))
        .unwrap_or((false, false));

    let node_type = map.as_ref().map(|m| m.node_type_at(run_state.current_floor, m.current_path_index));
    let bg_path = combat_background_path(node_type, is_boss, is_mini_boss);

    // Contenedor principal que abarca toda la pantalla (1280x720)
    commands
        .spawn((
            CombatUiRoot,
            CleanupUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Relative,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(12.0)),
                overflow: Overflow::clip(),
                ..default()
            },
        ))
        .with_children(|root| {
            // Fondo contextual de la situación de combate (Común, Mini Jefe o Líder de Gimnasio)
            root.spawn((
                ImageNode::new(asset_server.load(bg_path)),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    ..default()
                },
            ));

            // Filtro de tinte suave para asegurar alto contraste de la interfaz
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.04, 0.05, 0.08, 0.45)),
            ));
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
                            Some(name) => format!("[ITEM] {}", name),
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
                            UiTransform::default(),
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

                    // Estados activos del Pokémon del jugador
                    p_col.spawn((
                        PlayerStatusesText,
                        Text::new(""),
                        TextFont::from_font_size(11.5),
                        TextColor(Color::srgb(1.0, 0.7, 0.3)),
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
                    // Acción Anunciada del Rival (Transparencia Táctica GDD 2)
                    e_col.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(3.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            margin: UiRect::bottom(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.24, 0.12, 0.16, 0.85)),
                        BorderColor(Color::srgb(0.9, 0.35, 0.35)),
                    )).with_child((
                        EnemyIntentText,
                        Text::new("[ACCIÓN RIVAL] Calculando..."),
                        TextFont::from_font_size(13.5),
                        TextColor(Color::srgb(1.0, 0.45, 0.45)),
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
                            UiTransform::default(),
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

                    // Estados activos del Pokémon rival (ubicados abajo de los sprites, separados de la acción)
                    e_col.spawn((
                        EnemyStatusesText,
                        Text::new(""),
                        TextFont::from_font_size(11.5),
                        TextColor(Color::srgb(1.0, 0.75, 0.35)),
                    ));
                });
            });

            // ==========================================
            // 3. ZONA INFERIOR: ENERGÍA, MANO Y FIN DE TURNO
            // ==========================================
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(210.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(4.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.88)),
                BorderColor(Color::srgb(0.25, 0.30, 0.40)),
            )).with_children(|bottom| {
                // Grupo Izquierdo: Mazo de Robo + Orbe de Energía
                bottom.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(12.0),
                    ..default()
                }).with_children(|left_group| {
                    // Pila de Robo (Draw Pile)
                    left_group.spawn((
                        DrawPileWidget,
                        Node {
                            width: Val::Px(64.0),
                            height: Val::Px(88.0),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            padding: UiRect::axes(Val::Px(4.0), Val::Px(6.0)),
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.10, 0.14, 0.22, 0.95)),
                        BorderColor(Color::srgb(0.35, 0.65, 0.95)),
                    )).with_children(|dp| {
                        dp.spawn((
                            Text::new("MAZO"),
                            TextFont::from_font_size(10.0),
                            TextColor(Color::srgb(0.65, 0.85, 1.0)),
                        ));
                        dp.spawn((
                            DrawPileCountText,
                            Text::new("0"),
                            TextFont::from_font_size(20.0),
                            TextColor(Color::WHITE),
                        ));
                    });

                    // Orbe de Energía
                    left_group.spawn((
                        Node {
                            width: Val::Px(88.0),
                            height: Val::Px(88.0),
                            border: UiRect::all(Val::Px(3.0)),
                            border_radius: BorderRadius::all(Val::Px(44.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.18, 0.16, 0.05)),
                        BorderColor(Color::srgb(1.0, 0.85, 0.15)),
                    )).with_child((
                        EnergyOrbText,
                        Text::new(format!("{}/{} EN", run_state.energy, run_state.max_energy)),
                        TextFont::from_font_size(18.0),
                        TextColor(Color::srgb(1.0, 0.9, 0.3)),
                    ));
                });

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

                // Grupo Derecho: Botón Fin de Turno + Pila de Descarte
                bottom.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(12.0),
                    ..default()
                }).with_children(|right_group| {
                    // Botón de Terminar Turno
                    right_group.spawn((
                        Button,
                        EndTurnButton,
                        Node {
                            width: Val::Px(110.0),
                            height: Val::Px(64.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.65, 0.22, 0.22)),
                        BorderColor(Color::srgb(0.9, 0.4, 0.4)),
                    )).with_child((
                        Text::new("TERMINAR\nTURNO [>>]"),
                        TextFont::from_font_size(14.0),
                        TextColor(Color::WHITE),
                    ));

                    // Pila de Descarte (Discard Pile)
                    right_group.spawn((
                        DiscardPileWidget,
                        Node {
                            width: Val::Px(64.0),
                            height: Val::Px(88.0),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            padding: UiRect::axes(Val::Px(4.0), Val::Px(6.0)),
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.20, 0.12, 0.12, 0.95)),
                        BorderColor(Color::srgb(0.90, 0.45, 0.30)),
                    )).with_children(|dp| {
                        dp.spawn((
                            Text::new("DESCARTE"),
                            TextFont::from_font_size(9.0),
                            TextColor(Color::srgb(1.0, 0.75, 0.65)),
                        ));
                        dp.spawn((
                            DiscardPileCountText,
                            Text::new("0"),
                            TextFont::from_font_size(20.0),
                            TextColor(Color::WHITE),
                        ));
                    });
                });
            });
        });
}

/// Actualiza un componente `Text` únicamente cuando su contenido cambia realmente,
/// evitando recalcular el layout de glifos (`cosmic-text`) en cada frame durante las animaciones.
#[inline]
fn set_text_if_changed(text: &mut Text, new_val: &str) {
    if text.as_str() != new_val {
        **text = new_val.to_string();
    }
}

fn update_combat_hud(
    run_state: Res<CurrentRunState>,
    q_player: Query<&PokemonStats, (With<PlayerPokemon>, Without<EnemyPokemon>)>,
    q_enemy: Query<(&PokemonStats, &EnemyAI), (With<EnemyPokemon>, Without<PlayerPokemon>)>,
    mut q_p_fill: Query<&mut Node, (With<PlayerHpBarFill>, Without<EnemyHpBarFill>)>,
    mut q_e_fill: Query<&mut Node, (With<EnemyHpBarFill>, Without<PlayerHpBarFill>)>,
    q_draw: Query<(), With<InDrawPile>>,
    q_discard: Query<(), With<InDiscardPile>>,
    mut q_draw_texts: Query<&mut Text, (With<DrawPileCountText>, Without<DiscardPileCountText>)>,
    mut q_discard_texts: Query<&mut Text, (With<DiscardPileCountText>, Without<DrawPileCountText>)>,
    mut q_texts: Query<(
        Entity,
        &mut Text,
        Option<&PlayerHpText>,
        Option<&PlayerBlockText>,
        Option<&PlayerStatsText>,
        Option<&PlayerStatusesText>,
        Option<&EnemyHpText>,
        Option<&EnemyBlockText>,
        Option<&EnemyStatusesText>,
        Option<&EnemyIntentText>,
        Option<&EnergyOrbText>,
        Option<&CombatLogFeed>,
    ), (Without<DrawPileCountText>, Without<DiscardPileCountText>)>,
) {
    let player_data = q_player.single().ok();
    let enemy_data = q_enemy.single().ok();

    let draw_count = q_draw.iter().count();
    let discard_count = q_discard.iter().count();
    let draw_str = draw_count.to_string();
    let discard_str = discard_count.to_string();
    for mut t in q_draw_texts.iter_mut() {
        set_text_if_changed(&mut t, &draw_str);
    }
    for mut t in q_discard_texts.iter_mut() {
        set_text_if_changed(&mut t, &discard_str);
    }

    if let Some(player) = player_data {
        let pct = (player.hp as f32 / player.max_hp as f32 * 100.0).clamp(0.0, 100.0);
        if let Ok(mut fill) = q_p_fill.single_mut() {
            if fill.width != Val::Percent(pct) {
                fill.width = Val::Percent(pct);
            }
        }
    }

    if let Some((enemy, _)) = enemy_data {
        let pct = (enemy.hp as f32 / enemy.max_hp as f32 * 100.0).clamp(0.0, 100.0);
        if let Ok(mut fill) = q_e_fill.single_mut() {
            if fill.width != Val::Percent(pct) {
                fill.width = Val::Percent(pct);
            }
        }
    }

    for (_, mut text, p_hp, p_block, p_stats, p_statuses, e_hp, e_block, e_statuses, e_intent, orb, log) in q_texts.iter_mut() {
        if p_hp.is_some() {
            if let Some(player) = player_data {
                let s = format!("HP: {}/{}", player.hp, player.max_hp);
                set_text_if_changed(&mut text, &s);
            }
        } else if p_block.is_some() {
            if let Some(player) = player_data {
                let s = format!("ESC: {}", player.block);
                set_text_if_changed(&mut text, &s);
            }
        } else if p_stats.is_some() {
            if let Some(player) = player_data {
                let s = format!(
                    "Atk: {:.0} | Def: {:.0} | Sp.Atk: {:.0}",
                    player.effective_atk(), player.effective_def(), player.effective_sp_atk()
                );
                set_text_if_changed(&mut text, &s);
            }
        } else if p_statuses.is_some() {
            if let Some(player) = player_data {
                if player.statuses.is_empty() {
                    set_text_if_changed(&mut text, "");
                } else {
                    let mut parts: Vec<String> = player
                        .statuses
                        .iter()
                        .filter(|(_, stacks)| **stacks > 0)
                        .map(|(s, stacks)| format!("{}: {}", s.display_name(), stacks))
                        .collect();
                    if parts.is_empty() {
                        set_text_if_changed(&mut text, "");
                    } else {
                        parts.sort();
                        let s = format!("[TUS ESTADOS] {}", parts.join(" | "));
                        set_text_if_changed(&mut text, &s);
                    }
                }
            }
        } else if e_hp.is_some() {
            if let Some((enemy, _)) = enemy_data {
                let s = format!("{} (Nv. {}) HP: {}/{}", enemy.name, enemy.level, enemy.hp, enemy.max_hp);
                set_text_if_changed(&mut text, &s);
            }
        } else if e_block.is_some() {
            if let Some((enemy, _)) = enemy_data {
                let s = format!("ESC: {}", enemy.block);
                set_text_if_changed(&mut text, &s);
            }
        } else if e_statuses.is_some() {
            if let Some((enemy, _)) = enemy_data {
                if enemy.statuses.is_empty() {
                    set_text_if_changed(&mut text, "");
                } else {
                    let mut parts: Vec<String> = enemy
                        .statuses
                        .iter()
                        .filter(|(_, stacks)| **stacks > 0)
                        .map(|(s, stacks)| format!("{}: {}", s.display_name(), stacks))
                        .collect();
                    if parts.is_empty() {
                        set_text_if_changed(&mut text, "");
                    } else {
                        parts.sort();
                        let s = format!("[ESTADOS RIVAL] {}", parts.join(" | "));
                        set_text_if_changed(&mut text, &s);
                    }
                }
            }
        } else if e_intent.is_some() {
            if let Some((_, ai)) = enemy_data {
                if let Some(intent) = &ai.current_intent {
                    let s = match intent {
                        EnemyIntentType::Attack { raw_damage, .. } => {
                            let player_def = player_data.map(|p| p.def).unwrap_or(0.0);
                            let dmg = (raw_damage * PokemonStats::calc_mitigation(player_def)).round() as i32;
                            format!("[ACCIÓN RIVAL] Atacará ({} daño)", dmg)
                        }
                        EnemyIntentType::Defend { block_amount } => {
                            format!("[ACCIÓN RIVAL] Se defenderá (+{} Bloqueo)", block_amount)
                        }
                        EnemyIntentType::Buff { status, stacks } => {
                            format!("[ACCIÓN RIVAL] Se potenciará: {} (+{})", status.display_name(), stacks)
                        }
                        EnemyIntentType::Debuff { status, stacks } => {
                            format!("[ACCIÓN RIVAL] Te lanzará: {} (+{})", status.display_name(), stacks)
                        }
                    };
                    set_text_if_changed(&mut text, &s);
                }
            }
        } else if orb.is_some() {
            let s = format!("{}/{} EN", run_state.energy, run_state.max_energy);
            set_text_if_changed(&mut text, &s);
        } else if log.is_some() {
            if let Some(last_log) = run_state.battle_logs.last() {
                set_text_if_changed(&mut text, last_log);
            }
        }
    }
}

/// Sincroniza las cartas de la mano con botones interactivos 2D en pantalla
fn update_hand_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    preloaded: Option<Res<crate::ui::PreloadedAssets>>,
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

    // Genera las cartas interactivas según el GDD (Sección 9.1) con el asset de fondo según su tipo
    commands.entity(container_entity).with_children(|parent| {
        for (i, card) in cards.into_iter().enumerate() {
            let border_col = element_color(card.element);
            let desc = card.description();
            let card_path = card.asset_path();
            let tex_handle = preloaded
                .as_ref()
                .and_then(|p| p.get_card(card_path))
                .unwrap_or_else(|| asset_server.load(card_path));

            parent.spawn((
                Button,
                CardButton { index: i },
                Node {
                    width: Val::Px(146.0),
                    height: Val::Px(196.0),
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
                CardHoverAnimation::new(border_col).with_lift(-22.0).with_scale(1.10),
                BackgroundColor(Color::srgba(0.06, 0.08, 0.12, 0.2)),
                BorderColor(border_col),
            )).with_children(|card_box| {
                // Ilustración y marco del tipo de carta (reutilizando textura precargada)
                card_box.spawn((
                    ImageNode::new(tex_handle),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        top: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        ..default()
                    },
                ));

                // Cabecera: Nombre y Coste (encajados en el banner superior)
                card_box.spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(30.0),
                    margin: UiRect::top(Val::Px(12.0)),
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(2.0)),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                }).with_children(|h| {
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

                // Espacio central transparente para contemplar la ilustración del elemento
                card_box.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });

                // Caja de descripción inferior (con margen derecho para el icono elemental grabado en el asset)
                card_box.spawn(Node {
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
}

fn handle_card_click(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    preloaded: Option<Res<crate::ui::PreloadedAssets>>,
    mut q_buttons: Query<(&Interaction, &CardButton), Changed<Interaction>>,
    mut ev_play: MessageWriter<PlayCardEvent>,
    q_enemy: Query<&PokemonStats, With<EnemyPokemon>>,
    q_player: Query<&PokemonStats, With<PlayerPokemon>>,
    run_state: Res<CurrentRunState>,
    q_hand_cards: Query<&Card, With<InHand>>,
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

    let hand_cards: Vec<&Card> = q_hand_cards.iter().collect();

    for (interaction, card_btn) in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            let idx = card_btn.index;
            if idx < hand_cards.len() {
                let card = hand_cards[idx];
                if run_state.energy >= card.energy_cost {
                    // Spawnea la animación de vuelo hacia la pila de descarte (esquina inferior derecha)
                    let border_col = element_color(card.element);
                    let start_x = 350.0 + (idx as f32 * 115.0);
                    let start_y = 510.0;
                    let target_x = 1205.0;
                    let target_y = 615.0;

                    let card_path = card.asset_path();
                    let maybe_tex = preloaded
                        .as_ref()
                        .and_then(|p| p.get_card(card_path))
                        .or_else(|| asset_server.as_ref().map(|a| a.load(card_path)));

                    let mut flight_cmd = commands.spawn((
                        CleanupUi,
                        CardDiscardFlight {
                            timer: Timer::from_seconds(0.38, TimerMode::Once),
                            start_pos: Vec2::new(start_x, start_y),
                            target_pos: Vec2::new(target_x, target_y),
                            card_name: card.display_name(),
                        },
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(start_x),
                            top: Val::Px(start_y),
                            width: Val::Px(136.0),
                            height: Val::Px(182.0),
                            border: UiRect::all(Val::Px(2.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            overflow: Overflow::clip(),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        UiTransform::default(),
                        GlobalZIndex(250),
                        BackgroundColor(Color::srgba(0.08, 0.10, 0.14, 0.95)),
                        BorderColor(border_col),
                    ));

                    if let Some(tex) = maybe_tex {
                        flight_cmd.with_child((
                            ImageNode::new(tex),
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                right: Val::Px(0.0),
                                top: Val::Px(0.0),
                                bottom: Val::Px(0.0),
                                ..default()
                            },
                        ));
                    }
                }
            }

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

fn cleanup_combat_ui(
    mut commands: Commands,
    q_cleanup: Query<
        Entity,
        Or<(
            With<CleanupUi>,
            With<CardDiscardFlight>,
            With<DeckShuffleFlight>,
            With<DeckShuffleBanner>,
        )>,
    >,
) {
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
                        rgb: (c.red, c.green, c.blue),
                    },
                    UiTransform::default(),
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

/// Avanza los efectos de cada sprite y los aplica usando `UiTransform` (GPU-friendly, sin recalcular layout Taffy) y tinte.
fn animate_sprites(
    time: Res<Time>,
    mut q_sprites: Query<(&mut SpriteAnimator, &mut UiTransform, &mut ImageNode)>,
) {
    let dt = time.delta_secs().min(0.05);

    for (mut animator, mut transform, mut image) in q_sprites.iter_mut() {
        if animator.effects.is_empty() {
            if animator.active {
                animator.active = false;
                transform.translation = Val2::px(0.0, 0.0);
                transform.scale = Vec2::ONE;
                image.color = Color::WHITE;
            }
            continue;
        }

        let mut offset_x = 0.0f32;
        let mut offset_y = 0.0f32;
        let mut scale_mult = 1.0f32;
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
                AnimShape::Lunge(dir) => {
                    let wave = (std::f32::consts::PI * p).sin();
                    offset_x += dir * 90.0 * wave;
                    scale_mult = scale_mult.max(1.0 + 0.08 * wave);
                }
                AnimShape::Shake => offset_x += (effect.elapsed * 70.0).sin() * 14.0 * (1.0 - p),
                AnimShape::Hop => {
                    let wave = (std::f32::consts::PI * p).sin();
                    offset_y -= 22.0 * wave;
                    scale_mult = scale_mult.max(1.0 + 0.05 * wave);
                }
                AnimShape::CelebrateJump => {
                    // Saltos repetidos de júbilo
                    offset_y -= (effect.elapsed * 18.0).sin().abs() * 26.0 * (1.0 - p * 0.4);
                }
                AnimShape::Faint => {
                    // Caída y desvanecimiento
                    offset_y += p * 38.0;
                    offset_x += (effect.elapsed * 30.0).sin() * 3.0 * (1.0 - p);
                    scale_mult = (1.0 - p * 0.25).max(0.75);
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

        transform.translation = Val2::px(offset_x, offset_y);
        transform.scale = Vec2::splat(scale_mult);
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

/// Mueve hacia arriba y desvanece los textos flotantes usando `UiTransform` sin invalidar layout; los elimina al terminar.
fn animate_floating_text(
    mut commands: Commands,
    time: Res<Time>,
    mut q_texts: Query<(Entity, &mut FloatingText, &mut UiTransform, &mut TextColor)>,
) {
    let dt = time.delta_secs().min(0.05);

    for (entity, mut floating, mut transform, mut color) in q_texts.iter_mut() {
        if floating.delay > 0.0 {
            floating.delay -= dt;
            continue;
        }

        floating.elapsed += dt;
        let p = (floating.elapsed / floating.duration).clamp(0.0, 1.0);
        // Curva ease-out suave para la elevación
        let ease_out = 1.0 - (1.0 - p).powi(2);

        transform.translation = Val2::px(0.0, -70.0 * ease_out);
        // Opaco durante el primer 55% y luego se desvanece
        let alpha = if p < 0.55 { 1.0 } else { 1.0 - (p - 0.55) / 0.45 };
        let (r, g, b) = floating.rgb;
        color.0 = Color::srgba(r, g, b, alpha);

        if p >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Anima el vuelo curvo de la carta jugada hacia la pila de descarte con rotación y encogimiento fluidos.
/// Si el juego ya salió de `AppState::Combat` (por ejemplo, al derrotar al rival con esa carta),
/// destruye inmediatamente la carta en vuelo para que nunca quede fija en las demás pantallas.
pub fn animate_card_discard_flight(
    time: Res<Time>,
    state: Option<Res<State<AppState>>>,
    mut commands: Commands,
    mut q_flights: Query<(Entity, &mut CardDiscardFlight, &mut Node, &mut UiTransform)>,
) {
    if q_flights.is_empty() {
        return;
    }

    if let Some(ref current_state) = state {
        if *current_state.get() != AppState::Combat {
            for (entity, _, _, _) in q_flights.iter() {
                commands.entity(entity).despawn();
            }
            return;
        }
    }

    let raw_dt = time.delta_secs();
    if raw_dt <= 0.0 {
        return;
    }
    // Limitar delta por frame en animaciones visuales para evitar saltos bruscos si hay un pico de carga
    let clamped_delta = std::time::Duration::from_secs_f32(raw_dt.min(0.055));

    for (entity, mut flight, mut node, mut transform) in q_flights.iter_mut() {
        flight.timer.tick(clamped_delta);
        let t = flight.timer.fraction();
        // Interpolación cúbica smoothstep para aceleración y frenado sedosos
        let smooth_t = t * t * (3.0 - 2.0 * t);

        // Trayectoria parabólica hacia la pila de descarte en la esquina inferior derecha
        let curr_x = flight.start_pos.x + (flight.target_pos.x - flight.start_pos.x) * smooth_t;
        let arc_lift = (std::f32::consts::PI * t).sin() * 110.0;
        let curr_y = flight.start_pos.y + (flight.target_pos.y - flight.start_pos.y) * smooth_t - arc_lift;

        node.left = Val::Px(curr_x);
        node.top = Val::Px(curr_y);

        // Se encoge progresivamente y rota suavemente hacia el descarte
        transform.scale = Vec2::splat((1.0 - smooth_t * 0.72).max(0.25));
        transform.rotation = Rot2::radians(smooth_t * 0.48);

        if flight.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Genera la animación visual cuando se baraja el mazo (al inicio o al reciclar el descarte)
pub fn trigger_deck_shuffle_animation(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    preloaded: Option<Res<crate::ui::PreloadedAssets>>,
    mut ev_shuffle: MessageReader<DeckShuffleAnimEvent>,
    q_existing_banner: Query<Entity, With<DeckShuffleBanner>>,
) {
    for ev in ev_shuffle.read() {
        for b in q_existing_banner.iter() {
            commands.entity(b).despawn();
        }

        // Cartel flotante anunciador de barajado en el centro inferior
        commands.spawn((
            CleanupUi,
            DeckShuffleBanner {
                timer: Timer::from_seconds(1.15, TimerMode::Once),
            },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(35.0),
                bottom: Val::Px(230.0),
                width: Val::Percent(30.0),
                height: Val::Px(44.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                padding: UiRect::axes(Val::Px(16.0), Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.10, 0.14, 0.22, 0.95)),
            BorderColor(Color::srgb(0.35, 0.85, 0.95)),
            GlobalZIndex(260),
            UiTransform::default(),
        )).with_child((
            Text::new(format!("[MAZO] ¡BARAJANDO MAZO! ({} cartas)", ev.card_count)),
            TextFont::from_font_size(14.0),
            TextColor(Color::srgb(1.0, 0.95, 0.45)),
        ));

        let normal_tex = preloaded
            .as_ref()
            .and_then(|p| p.get_card("cartas/normal.jpeg"))
            .or_else(|| asset_server.as_ref().map(|a| a.load("cartas/normal.jpeg")));

        // Ráfaga de 5 cartas volando desde la pila de Descarte (derecha) hacia el Mazo de Robo (izquierda)
        let num_cards = 5.min(ev.card_count.max(3));
        for i in 0..num_cards {
            let delay = i as f32 * 0.07;
            let mut card_cmd = commands.spawn((
                CleanupUi,
                DeckShuffleFlight {
                    timer: Timer::from_seconds(0.52, TimerMode::Once),
                    delay_timer: Timer::from_seconds(delay, TimerMode::Once),
                    start_pos: Vec2::new(1200.0, 615.0), // Pila de Descarte
                    target_pos: Vec2::new(60.0, 615.0),   // Pila de Robo
                    arc_height: 130.0 + (i as f32 * 14.0),
                    rotation_offset: if i % 2 == 0 { 0.4 } else { -0.4 },
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(1200.0),
                    top: Val::Px(615.0),
                    width: Val::Px(56.0),
                    height: Val::Px(78.0),
                    border: UiRect::all(Val::Px(1.5)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                UiTransform::default(),
                GlobalZIndex(255),
                BackgroundColor(Color::srgb(0.12, 0.16, 0.24)),
                BorderColor(Color::srgb(0.35, 0.75, 1.0)),
            ));

            if let Some(ref tex) = normal_tex {
                card_cmd.with_child((
                    ImageNode::new(tex.clone()),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        top: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        ..default()
                    },
                ));
            }
        }
    }
}

/// Anima el aleteo y arco de las cartas que vuelan al barajar el descarte hacia el mazo de robo
/// usando `UiTransform.translation` para cero recálculos de layout Taffy en las 5 cartas simultáneas.
pub fn animate_deck_shuffle(
    time: Res<Time>,
    state: Option<Res<State<AppState>>>,
    mut commands: Commands,
    mut q_flights: Query<(Entity, &mut DeckShuffleFlight, &mut UiTransform)>,
    mut q_banners: Query<(Entity, &mut DeckShuffleBanner, &mut UiTransform), Without<DeckShuffleFlight>>,
) {
    if q_flights.is_empty() && q_banners.is_empty() {
        return;
    }

    if let Some(ref current_state) = state {
        if *current_state.get() != AppState::Combat {
            for (entity, _, _) in q_flights.iter() {
                commands.entity(entity).despawn();
            }
            for (b_ent, _, _) in q_banners.iter() {
                commands.entity(b_ent).despawn();
            }
            return;
        }
    }
    let raw_dt = time.delta_secs();
    if raw_dt <= 0.0 {
        return;
    }
    let dt = std::time::Duration::from_secs_f32(raw_dt.min(0.055));

    for (entity, mut flight, mut transform) in q_flights.iter_mut() {
        if !flight.delay_timer.is_finished() {
            flight.delay_timer.tick(dt);
            continue;
        }

        flight.timer.tick(dt);
        let t = flight.timer.fraction();
        let smooth_t = t * t * (3.0 - 2.0 * t);

        // Desplazamiento relativo respecto a start_pos aplicado directamente en UiTransform
        let dx = (flight.target_pos.x - flight.start_pos.x) * smooth_t;
        let dy = (flight.target_pos.y - flight.start_pos.y) * smooth_t
            - (std::f32::consts::PI * t).sin() * flight.arc_height;

        transform.translation = Val2::px(dx, dy);
        transform.rotation = Rot2::radians((t * std::f32::consts::TAU * 1.5).sin() * flight.rotation_offset);
        transform.scale = Vec2::splat(1.0 + (std::f32::consts::PI * t).sin() * 0.25);

        if flight.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }

    let factor = (1.0 - (-12.0 * raw_dt.min(0.05)).exp()).clamp(0.0, 1.0);
    for (b_ent, mut banner, mut b_trans) in q_banners.iter_mut() {
        banner.timer.tick(dt);
        let pulse = 1.0 + (time.elapsed_secs() * 6.0).sin() * 0.04;
        b_trans.scale = b_trans.scale.lerp(Vec2::splat(pulse), factor);

        if banner.timer.is_finished() {
            commands.entity(b_ent).despawn();
        }
    }
}
