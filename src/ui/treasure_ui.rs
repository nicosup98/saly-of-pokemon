use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;
use crate::relics::{get_all_relics, Relic, RelicTier};
use crate::run::{AppState, CurrentRunState};
use crate::ui::{CleanupUi, Button, BorderColor};

pub struct TreasureUiPlugin;

impl Plugin for TreasureUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TreasureReward>()
            .add_systems(
                OnEnter(AppState::Treasure),
                (roll_treasure, setup_treasure_ui).chain(),
            )
            .add_systems(Update, handle_continue.run_if(in_state(AppState::Treasure)))
            .add_systems(OnExit(AppState::Treasure), cleanup_treasure_ui);
    }
}

/// Recompensa del cofre abierto en el nodo de Tesoro actual (GDD 7.3).
#[derive(Resource, Default, Clone)]
pub struct TreasureReward {
    pub chest_name: String,
    pub chest_color: Color,
    pub gold: u32,
    pub relic: Option<Relic>,
}

#[derive(Component)]
pub struct TreasureContinueButton;

/// Sortea el tipo de cofre y aplica la recompensa a la partida.
fn roll_treasure(mut reward: ResMut<TreasureReward>, mut run_state: ResMut<CurrentRunState>) {
    let mut rng = rand::thread_rng();
    let roll = rng.gen_range(0..100);

    // (nombre, color, prob. de oro, rango de oro, tiers de reliquia con su peso)
    let (name, color, gold_chance, gold_range, tiers): (&str, Color, u32, (u32, u32), &[(RelicTier, u32)]) =
        if roll < 50 {
            (
                "Cápsula Estándar (Azul)",
                Color::srgb(0.3, 0.55, 0.95),
                50,
                (40, 80),
                &[(RelicTier::Common, 75), (RelicTier::Uncommon, 25)],
            )
        } else if roll < 83 {
            (
                "Supercápsula (Amarilla)",
                Color::srgb(0.95, 0.8, 0.2),
                100,
                (70, 110),
                &[(RelicTier::Uncommon, 50), (RelicTier::Rare, 50)],
            )
        } else {
            (
                "Ultracápsula (Dorada)",
                Color::srgb(0.95, 0.6, 0.15),
                100,
                (100, 150),
                &[(RelicTier::Rare, 100)],
            )
        };

    let gold = if rng.gen_range(0..100) < gold_chance {
        rng.gen_range(gold_range.0..=gold_range.1)
    } else {
        0
    };

    // Reliquia garantizada del tier sorteado (que el jugador aún no tenga)
    let tier_roll = rng.gen_range(0..100);
    let mut acc = 0;
    let mut chosen_tier = tiers[0].0;
    for (tier, weight) in tiers {
        acc += weight;
        if tier_roll < acc {
            chosen_tier = *tier;
            break;
        }
    }

    let all = get_all_relics();
    let owned = |r: &&Relic| run_state.has_relic(&r.id);
    let pool_tier: Vec<&Relic> = all.iter().filter(|r| r.tier == chosen_tier && !owned(r)).collect();
    let pool_any: Vec<&Relic> = all
        .iter()
        .filter(|r| r.tier != RelicTier::Starter && r.tier != RelicTier::BossBadge && !owned(r))
        .collect();
    let relic = pool_tier
        .choose(&mut rng)
        .or_else(|| pool_any.choose(&mut rng))
        .map(|r| (*r).clone());

    run_state.poke_dollars += gold;
    run_state.add_log(format!("[TESORO] Abres una {}.", name));
    if gold > 0 {
        run_state.add_log(format!("[TESORO] Encuentras {} P$.", gold));
    }
    if let Some(r) = &relic {
        run_state.relics.push(r.id.clone());
        run_state.add_log(format!("[RELIQUIA] Obtienes la reliquia {}.", r.name));
    }

    *reward = TreasureReward {
        chest_name: name.to_string(),
        chest_color: color,
        gold,
        relic,
    };
}

fn setup_treasure_ui(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    preloaded: Option<Res<crate::ui::PreloadedAssets>>,
    reward: Res<TreasureReward>,
    run_state: Res<CurrentRunState>,
) {
    let pre = preloaded.as_deref();
    let srv = asset_server.as_deref();
    let banner_tex = crate::ui::load_ui_image(pre, srv, "ui/hud/hud_turn_banner.png");
    let treasure_icon_tex = crate::ui::load_ui_image(pre, srv, "ui/nodes/node_treasure.png");
    let btn_gold_tex = crate::ui::load_ui_image(pre, srv, "ui/buttons/btn_gold.png");

    commands
        .spawn((
            CleanupUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.03, 0.04, 0.07)),
        ))
        .with_children(|root| {
            // Cabecera con `ui/hud/hud_turn_banner.png`
            root.spawn(Node {
                min_width: Val::Px(540.0),
                min_height: Val::Px(74.0),
                position_type: PositionType::Relative,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(24.0), Val::Px(10.0)),
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
                    Text::new("CÁPSULA DE SUMINISTROS DE SILPH S.A."),
                    TextFont::from_font_size(22.0),
                    TextColor(Color::srgb(0.95, 0.76, 0.30)),
                ));
            });

            root.spawn((
                Node {
                    width: Val::Px(540.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(12.0),
                    padding: UiRect::all(Val::Px(24.0)),
                    border: UiRect::all(Val::Px(2.5)),
                    border_radius: BorderRadius::all(Val::Px(18.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.06, 0.09, 0.14, 0.95)),
                BorderColor(reward.chest_color),
            ))
            .with_children(|panel| {
                if let Some(ref icon_tex) = treasure_icon_tex {
                    panel.spawn((
                        ImageNode::new(icon_tex.clone()),
                        Node {
                            width: Val::Px(120.0),
                            height: Val::Px(120.0),
                            ..default()
                        },
                    ));
                }

                panel.spawn((
                    Text::new(reward.chest_name.clone()),
                    TextFont::from_font_size(22.0),
                    TextColor(reward.chest_color),
                ));

                panel.spawn((
                    Text::new(if reward.gold > 0 {
                        format!("+{} P$", reward.gold)
                    } else {
                        "Sin Poké-Dólares esta vez".to_string()
                    }),
                    TextFont::from_font_size(18.0),
                    TextColor(Color::srgb(0.95, 0.76, 0.30)),
                ));

                match &reward.relic {
                    Some(r) => {
                        panel.spawn((
                            Text::new(format!("Reliquia: {}", r.name)),
                            TextFont::from_font_size(20.0),
                            TextColor(Color::srgb(0.35, 0.82, 1.0)),
                        ));
                        panel.spawn((
                            Text::new(r.description.clone()),
                            TextFont::from_font_size(13.0),
                            TextColor(Color::srgb(0.82, 0.86, 0.94)),
                            Node {
                                width: Val::Px(450.0),
                                ..default()
                            },
                        ));
                    }
                    None => {
                        panel.spawn((
                            Text::new("Ya tienes todas las reliquias disponibles."),
                            TextFont::from_font_size(14.0),
                            TextColor(Color::srgb(0.8, 0.8, 0.8)),
                        ));
                    }
                }

                panel.spawn((
                    Text::new(format!("Saldo actual: {} P$", run_state.poke_dollars)),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::srgb(0.7, 0.78, 0.88)),
                ));
            });

            root.spawn((
                Button,
                TreasureContinueButton,
                Node {
                    width: Val::Px(250.0),
                    height: Val::Px(48.0),
                    position_type: PositionType::Relative,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(12.0)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.5, 0.3)),
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
                    Text::new("CONTINUAR ASCENSO ->"),
                    TextFont::from_font_size(15.0),
                    TextColor(Color::srgb(0.15, 0.09, 0.02)),
                ));
            });
        });
}

use crate::ui::transition_ui::TransitionToEvent;

fn handle_continue(
    q_button: Query<&Interaction, (Changed<Interaction>, With<TreasureContinueButton>)>,
    mut run_state: ResMut<CurrentRunState>,
    mut ev_trans: MessageWriter<TransitionToEvent>,
) {
    for interaction in q_button.iter() {
        if *interaction == Interaction::Pressed {
            run_state.current_floor += 1;
            ev_trans.write(TransitionToEvent {
                target: AppState::MapNavigation,
            });
            break;
        }
    }
}

fn cleanup_treasure_ui(mut commands: Commands, q_cleanup: Query<Entity, With<CleanupUi>>) {
    for entity in q_cleanup.iter() {
        commands.entity(entity).despawn();
    }
}
