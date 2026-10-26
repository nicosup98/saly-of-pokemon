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
    run_state: Res<CurrentRunState>,
    meta: Res<MetaProfile>,
) {
    // Tras el Acto 3 el contador de Acto queda en 4: la run se completó
    let victory = run_state.current_act > 3;
    let (title, title_color) = if victory {
        ("¡VICTORIA! HAS CONQUISTADO LA LIGA", Color::srgb(1.0, 0.85, 0.2))
    } else {
        ("FIN DE LA PARTIDA", Color::srgb(0.95, 0.35, 0.35))
    };
    let progress = if victory {
        "Completaste los 3 Actos".to_string()
    } else {
        format!("Caíste en el Acto {} - Piso {}", run_state.current_act, run_state.current_floor)
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
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.06, 0.06, 0.10)),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new(title),
                TextFont::from_font_size(32.0),
                TextColor(title_color),
            ));
            root.spawn((
                Text::new(progress),
                TextFont::from_font_size(18.0),
                TextColor(Color::WHITE),
            ));
            root.spawn((
                Text::new(format!(
                    "Reliquias: {} | Cartas en el mazo: {} | Saldo final: {} P$",
                    run_state.relics.len(),
                    run_state.player_deck.len(),
                    run_state.poke_dollars
                )),
                TextFont::from_font_size(14.0),
                TextColor(Color::srgb(0.75, 0.75, 0.8)),
            ));
            root.spawn((
                Text::new(format!("Fichas Gachapón acumuladas: {}", meta.gachapon_tokens)),
                TextFont::from_font_size(16.0),
                TextColor(Color::srgb(1.0, 0.85, 0.3)),
            ));

            root.spawn((
                Button,
                SummaryMenuButton,
                Node {
                    width: Val::Px(240.0),
                    height: Val::Px(46.0),
                    margin: UiRect::top(Val::Px(12.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.35, 0.6)),
                BorderColor(Color::srgb(0.4, 0.7, 1.0)),
            )).with_child((
                Text::new("VOLVER AL MENÚ"),
                TextFont::from_font_size(15.0),
                TextColor(Color::WHITE),
            ));
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
