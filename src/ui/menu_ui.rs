use bevy::prelude::*;
use crate::run::{AppState, CurrentRunState, MetaProfile};
use crate::ui::{CleanupUi, Button, BorderColor};

pub struct MenuUiPlugin;

impl Plugin for MenuUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
            .add_systems(
                Update,
                (
                    handle_menu_navigation,
                )
                    .run_if(in_state(AppState::MainMenu)),
            )
            .add_systems(OnExit(AppState::MainMenu), cleanup_menu_ui);
    }
}

use crate::save::ActiveSaveSlot;

#[derive(Component)]
pub enum MenuButtonAction {
    StartRun,
    SelectStarter,
    Gachapon,
    Profiles,
}

fn setup_main_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    meta: Res<MetaProfile>,
    run_state: Res<CurrentRunState>,
    active_slot: Option<Res<ActiveSaveSlot>>,
) {
    let current_slot = active_slot.as_ref().map(|s| s.0).unwrap_or(1);
    commands
        .spawn((
            CleanupUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(30.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.06, 0.07, 0.11)),
        ))
        .with_children(|root| {
            // === TÍTULO PRINCIPAL ===
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(8.0),
                ..default()
            }).with_children(|title_box| {
                title_box.spawn((
                    Text::new("SLAY OF POKÉMON"),
                    TextFont::from_font_size(42.0),
                    TextColor(Color::srgb(1.0, 0.45, 0.2)),
                ));
                title_box.spawn((
                    Text::new("ROGUELIKE DECKBUILDER RPG (RUST + BEVY ENGINE)"),
                    TextFont::from_font_size(16.0),
                    TextColor(Color::srgb(0.8, 0.85, 0.95)),
                ));
                title_box.spawn((
                    Text::new(format!("Fichas Gachapón: {} | Saldo en Run: {} P$", meta.gachapon_tokens, run_state.poke_dollars)),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::srgb(1.0, 0.85, 0.3)),
                ));
            });

            // === ZONA CENTRAL CON SPRITE DEL STARTER Y BOTONES ===
            root.spawn(Node {
                width: Val::Percent(85.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                ..default()
            }).with_children(|center| {
                // Ilustración / Sprites del Entrenador y Compañero
                center.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(10.0),
                    ..default()
                }).with_children(|sprite_col| {
                    let sprite_path = format!("sprites/pokemon/front/{}.png", run_state.selected_starter_id);
                    sprite_col.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::FlexEnd,
                        justify_content: JustifyContent::Center,
                        column_gap: Val::Px(12.0),
                        ..default()
                    }).with_children(|pair| {
                        pair.spawn((
                            ImageNode::new(asset_server.load("sprites/trainers/player_front.png")),
                            Node {
                                width: Val::Px(72.0),
                                height: Val::Px(72.0),
                                margin: UiRect::bottom(Val::Px(10.0)),
                                ..default()
                            },
                        ));
                        pair.spawn((
                            ImageNode::new(asset_server.load(&sprite_path)),
                            Node {
                                width: Val::Px(150.0),
                                height: Val::Px(150.0),
                                ..default()
                            },
                        ));
                    });
                    let current_name = crate::pokemon::get_species_data(run_state.selected_starter_id)
                        .map(|s| s.name)
                        .unwrap_or_else(|| "Charmander".to_string());
                    sprite_col.spawn((
                        Text::new(format!(
                            "Equipo: Entrenador Red & {} #{} | Partida: Ranura #{}",
                            current_name, run_state.selected_starter_id, current_slot
                        )),
                        TextFont::from_font_size(14.0),
                        TextColor(Color::srgb(1.0, 0.7, 0.4)),
                    ));
                });

                // Menú de Botones de Navegación
                center.spawn(Node {
                    width: Val::Px(360.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(12.0),
                    ..default()
                }).with_children(|btn_col| {
                    let options = [
                        (MenuButtonAction::StartRun, "INICIAR RUTA (MAPA)", Color::srgb(0.20, 0.55, 0.85)),
                        (MenuButtonAction::SelectStarter, "SELECCIONAR POKÉMON", Color::srgb(0.85, 0.55, 0.20)),
                        (MenuButtonAction::Gachapon, "TIENDA GACHAPÓN", Color::srgb(0.65, 0.30, 0.80)),
                        (MenuButtonAction::Profiles, "PERFILES DE GUARDADO", Color::srgb(0.35, 0.65, 0.55)),
                    ];

                    for (action, label, col) in options {
                        btn_col.spawn((
                            Button,
                            action,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(48.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.5)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.14, 0.16, 0.22)),
                            BorderColor(col),
                        )).with_child((
                            Text::new(label),
                            TextFont::from_font_size(15.0),
                            TextColor(Color::WHITE),
                        ));
                    }
                });
            });

            // === PIE DE PÁGINA ===
            root.spawn((
                Text::new("Desarrollado según las especificaciones del GDD oficial (Rust 2024 + Bevy 0.20)"),
                TextFont::from_font_size(12.0),
                TextColor(Color::srgb(0.6, 0.6, 0.7)),
            ));
        });
}

use crate::ui::transition_ui::TransitionToEvent;

fn handle_menu_navigation(
    mut q_buttons: Query<(&Interaction, &MenuButtonAction), Changed<Interaction>>,
    mut ev_trans: MessageWriter<TransitionToEvent>,
) {
    for (interaction, action) in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            let target = match action {
                MenuButtonAction::StartRun => AppState::MapNavigation,
                MenuButtonAction::SelectStarter => AppState::StarterSelection,
                MenuButtonAction::Gachapon => AppState::GachaponShop,
                MenuButtonAction::Profiles => AppState::ProfileSelect,
            };
            ev_trans.write(TransitionToEvent { target });
        }
    }
}

fn cleanup_menu_ui(mut commands: Commands, q_cleanup: Query<Entity, With<CleanupUi>>) {
    for entity in q_cleanup.iter() {
        commands.entity(entity).despawn();
    }
}
