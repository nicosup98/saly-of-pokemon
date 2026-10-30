use bevy::prelude::*;
use crate::map::CurrentMap;
use crate::run::{AppState, CurrentRunState, MetaProfile};
use crate::ui::{CleanupUi, Button, BorderColor};

pub struct SummaryUiPlugin;

impl Plugin for SummaryUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::RunSummary), setup_summary_ui)
            .add_systems(Update, handle_back_to_menu.run_if(in_state(AppState::RunSummary)))
            .add_systems(OnExit(AppState::RunSummary), cleanup_summary_ui);
    }
}

#[derive(Component)]
pub struct SummaryMenuButton;

fn setup_summary_ui(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    preloaded: Option<Res<crate::ui::PreloadedAssets>>,
    run_state: Res<CurrentRunState>,
    meta: Res<MetaProfile>,
) {
    let pre = preloaded.as_deref();
    let srv = asset_server.as_deref();
    let banner_tex = crate::ui::load_ui_image(pre, srv, "ui/hud/hud_turn_banner.png");
    let btn_gold_tex = crate::ui::load_ui_image(pre, srv, "ui/buttons/btn_gold.png");

    // Tras el Acto 3 el contador de Acto queda en 4: la run se completó
    let victory = run_state.current_act > 3;
    let (title, title_color) = if victory {
        (
            "¡VICTORIA! HAS CONQUISTADO LA LIGA",
            Color::srgb(0.95, 0.76, 0.30),
        )
    } else {
        ("FIN DE LA PARTIDA", Color::srgb(1.0, 0.36, 0.36))
    };
    let progress = if victory {
        "Completaste los 3 Actos".to_string()
    } else {
        format!(
            "Caíste en el Acto {} - Piso {}",
            run_state.current_act, run_state.current_floor
        )
    };

    commands
        .spawn((
            CleanupUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(18.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.03, 0.04, 0.07)),
        ))
        .with_children(|root| {
            // Banner superior con `ui/hud/hud_turn_banner.png`
            root.spawn(Node {
                min_width: Val::Px(540.0),
                min_height: Val::Px(78.0),
                position_type: PositionType::Relative,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(28.0), Val::Px(12.0)),
                ..default()
            })
            .with_children(|banner| {
                if let Some(ref tex) = banner_tex {
                    banner.spawn((
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
                banner.spawn((
                    Text::new(title),
                    TextFont::from_font_size(24.0),
                    TextColor(title_color),
                ));
            });

            // Panel central de resumen VectorCraft
            root.spawn((
                Node {
                    width: Val::Px(520.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(12.0),
                    padding: UiRect::all(Val::Px(24.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(16.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.06, 0.09, 0.14, 0.94)),
                BorderColor(Color::srgb(0.16, 0.23, 0.33)),
            ))
            .with_children(|card| {
                card.spawn((
                    Text::new(progress),
                    TextFont::from_font_size(18.0),
                    TextColor(Color::WHITE),
                ));
                card.spawn((
                    Text::new(format!(
                        "Reliquias: {} | Cartas en el mazo: {} | Saldo final: {} P$",
                        run_state.relics.len(),
                        run_state.player_deck.len(),
                        run_state.poke_dollars
                    )),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::srgb(0.75, 0.82, 0.92)),
                ));
                card.spawn((
                    Text::new(format!("Fichas Gachapón acumuladas: {}", meta.gachapon_tokens)),
                    TextFont::from_font_size(16.0),
                    TextColor(Color::srgb(0.95, 0.76, 0.30)),
                ));
            });

            root.spawn((
                Button,
                SummaryMenuButton,
                Node {
                    width: Val::Px(250.0),
                    height: Val::Px(48.0),
                    position_type: PositionType::Relative,
                    margin: UiRect::top(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(12.0)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.28, 0.22, 0.08)),
                BorderColor(Color::srgb(0.95, 0.76, 0.30)),
            ))
            .with_children(|btn| {
                if let Some(ref tex) = btn_gold_tex {
                    btn.spawn((
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
                btn.spawn((
                    Text::new("VOLVER AL MENÚ"),
                    TextFont::from_font_size(15.0),
                    TextColor(Color::srgb(0.15, 0.09, 0.02)),
                ));
            });
        });
}

use crate::ui::transition_ui::TransitionToEvent;

fn handle_back_to_menu(
    q_button: Query<&Interaction, (Changed<Interaction>, With<SummaryMenuButton>)>,
    mut run_state: ResMut<CurrentRunState>,
    mut map: ResMut<CurrentMap>,
    mut ev_trans: MessageWriter<TransitionToEvent>,
) {
    for interaction in q_button.iter() {
        if *interaction == Interaction::Pressed {
            // Nueva partida limpia; el MetaProfile (fichas, Pokémon) se conserva
            *run_state = CurrentRunState::default();
            *map = CurrentMap::default();
            ev_trans.write(TransitionToEvent {
                target: AppState::MainMenu,
            });
            break;
        }
    }
}

fn cleanup_summary_ui(mut commands: Commands, q_cleanup: Query<Entity, With<CleanupUi>>) {
    for entity in q_cleanup.iter() {
        commands.entity(entity).despawn();
    }
}
