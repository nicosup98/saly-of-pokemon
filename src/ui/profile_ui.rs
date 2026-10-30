use bevy::prelude::*;
use crate::pokemon::get_species_data;
use crate::run::{AppState, CurrentRunState, MetaProfile};
use crate::save::{self, ActiveSaveSlot, SaveNotificationEvent, TOTAL_SLOTS};
use crate::ui::{BorderColor, Button, CleanupUi};
use crate::ui::transition_ui::TransitionToEvent;
use bevy_tweening::{
    lens::UiTransformScaleLens,
    AnimTarget, RepeatCount, RepeatStrategy, Tween, TweenAnim,
};

pub struct ProfileUiPlugin;

impl Plugin for ProfileUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::ProfileSelect), setup_profile_ui)
            .add_systems(
                Update,
                (
                    handle_profile_slot_actions,
                    handle_profile_back,
                )
                    .run_if(in_state(AppState::ProfileSelect)),
            )
            .add_systems(OnExit(AppState::ProfileSelect), cleanup_profile_ui);
    }
}

#[derive(Component)]
pub enum ProfileSlotAction {
    Load(u8),
    Save(u8),
    New(u8),
    Delete(u8),
}

#[derive(Component)]
pub struct ProfileBackButton;

#[derive(Component)]
pub struct ProfileStatusMessage;

fn setup_profile_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    preloaded: Option<Res<crate::ui::PreloadedAssets>>,
    active_slot: Option<Res<ActiveSaveSlot>>,
) {
    let pre = preloaded.as_deref();
    let srv = Some(&*asset_server);
    let profiles = save::list_profiles();
    let current_active_slot = active_slot.as_ref().map(|s| s.0).unwrap_or(1);
    let mut active_card_id = None;

    let banner_tex = crate::ui::load_ui_image(pre, srv, "ui/hud/hud_turn_banner.png");
    let btn_gold_tex = crate::ui::load_ui_image(pre, srv, "ui/buttons/btn_gold.png");
    let btn_primary_tex = crate::ui::load_ui_image(pre, srv, "ui/buttons/btn_primary.png");
    let btn_secondary_tex = crate::ui::load_ui_image(pre, srv, "ui/buttons/btn_secondary.png");
    let btn_danger_tex = crate::ui::load_ui_image(pre, srv, "ui/buttons/btn_danger.png");

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
            BackgroundColor(Color::srgb(0.03, 0.04, 0.07)),
        ))
        .with_children(|root| {
            // === CABECERA (`ui/hud/hud_turn_banner.png`) ===
            root.spawn(Node {
                min_width: Val::Px(560.0),
                min_height: Val::Px(84.0),
                position_type: PositionType::Relative,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(42.0), Val::Px(12.0)),
                row_gap: Val::Px(2.0),
                ..default()
            })
            .with_children(|header| {
                if let Some(ref tex) = banner_tex {
                    header.spawn((
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
                header.spawn((
                    Text::new("SELECCIÓN DE PERFIL / GUARDADO"),
                    TextFont::from_font_size(22.0),
                    TextColor(Color::srgb(0.95, 0.76, 0.30)),
                ));
                header.spawn((
                    Text::new(format!(
                        "Elige tu archivo de guardado antes de entrar al menú principal (Ranura activa: #{})",
                        current_active_slot
                    )),
                    TextFont::from_font_size(12.5),
                    TextColor(Color::srgb(0.75, 0.84, 0.96)),
                ));
            });

            // === 3 RANURAS DE GUARDADO ===
            root.spawn(Node {
                width: Val::Percent(100.0),
                max_width: Val::Px(1050.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(24.0),
                ..default()
            })
            .with_children(|slots_row| {
                for slot in 1..=TOTAL_SLOTS {
                    let is_active = current_active_slot == slot;
                    let summary = profiles[(slot - 1) as usize].as_ref();

                    let border_col = if is_active {
                        Color::srgb(0.95, 0.76, 0.30)
                    } else if summary.is_some() {
                        Color::srgb(0.22, 0.71, 1.0)
                    } else {
                        Color::srgb(0.16, 0.23, 0.33)
                    };

                    let bg_col = if is_active {
                        Color::srgb(0.09, 0.13, 0.20)
                    } else {
                        Color::srgb(0.06, 0.09, 0.14)
                    };

                    let mut card_entity = slots_row.spawn((
                        Node {
                            width: Val::Px(310.0),
                            height: Val::Px(380.0),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceBetween,
                            padding: UiRect::all(Val::Px(16.0)),
                            border: UiRect::all(Val::Px(2.5)),
                            border_radius: BorderRadius::all(Val::Px(16.0)),
                            ..default()
                        },
                        UiTransform::default(),
                        BackgroundColor(bg_col),
                        BorderColor(border_col),
                    ));

                    if is_active {
                        active_card_id = Some(card_entity.id());
                    }

                    card_entity.with_children(|card| {
                        // Título de la Ranura
                        card.spawn(Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|top| {
                            top.spawn((
                                Text::new(format!("PERFIL / RANURA {}", slot)),
                                TextFont::from_font_size(18.0),
                                TextColor(Color::WHITE),
                            ));
                            if is_active {
                                top.spawn((
                                    Text::new("[* ACTIVO]"),
                                    TextFont::from_font_size(12.0),
                                    TextColor(Color::srgb(0.95, 0.76, 0.30)),
                                ));
                            }
                        });

                        // Contenido: Ocupado vs Vacío
                        if let Some(p) = summary {
                            let starter_name = get_species_data(p.starter_id)
                                .map(|s| s.name)
                                .unwrap_or_else(|| "Pokémon".to_string());

                            let sprite_path = format!("sprites/pokemon/front/{}.png", p.starter_id);

                            card.spawn(Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|info| {
                                info.spawn((
                                    ImageNode::new(asset_server.load(&sprite_path)),
                                    Node {
                                        width: Val::Px(96.0),
                                        height: Val::Px(96.0),
                                        ..default()
                                    },
                                ));

                                info.spawn((
                                    Text::new(format!("{} #{}", starter_name, p.starter_id)),
                                    TextFont::from_font_size(15.0),
                                    TextColor(Color::srgb(0.95, 0.76, 0.30)),
                                ));

                                info.spawn((
                                    Text::new(format!(
                                        "Progreso: Acto {} - Piso {}",
                                        p.current_act, p.current_floor
                                    )),
                                    TextFont::from_font_size(12.0),
                                    TextColor(Color::srgb(0.8, 0.85, 0.9)),
                                ));

                                info.spawn((
                                    Text::new(format!(
                                        "P$: {} | Fichas: {} | Desbloqueados: {}",
                                        p.poke_dollars, p.gachapon_tokens, p.unlocked_count
                                    )),
                                    TextFont::from_font_size(11.0),
                                    TextColor(Color::srgb(0.7, 0.75, 0.8)),
                                ));

                                info.spawn((
                                    Text::new(format!("Guardado: {}", p.updated_at)),
                                    TextFont::from_font_size(10.0),
                                    TextColor(Color::srgb(0.55, 0.6, 0.7)),
                                ));
                            });

                            // Botones de acción para ranura ocupada (`btn_primary.png`, `btn_secondary.png`, `btn_danger.png`)
                            card.spawn(Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(6.0),
                                ..default()
                            })
                            .with_children(|actions| {
                                actions
                                    .spawn((
                                        Button,
                                        ProfileSlotAction::Load(slot),
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(34.0),
                                            position_type: PositionType::Relative,
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border_radius: BorderRadius::all(Val::Px(8.0)),
                                            overflow: Overflow::clip(),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.85, 0.38, 0.16)),
                                    ))
                                    .with_children(|btn| {
                                        if let Some(ref tex) = btn_primary_tex {
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
                                            Text::new("CARGAR PARTIDA"),
                                            TextFont::from_font_size(12.0),
                                            TextColor(Color::WHITE),
                                        ));
                                    });

                                actions
                                    .spawn((
                                        Button,
                                        ProfileSlotAction::Save(slot),
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(32.0),
                                            position_type: PositionType::Relative,
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(Color::NONE),
                                    ))
                                    .with_children(|btn| {
                                        if let Some(ref tex) = btn_secondary_tex {
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
                                            Text::new("GUARDAR PROGRESO AQUÍ"),
                                            TextFont::from_font_size(11.0),
                                            TextColor(Color::WHITE),
                                        ));
                                    });

                                actions
                                    .spawn((
                                        Button,
                                        ProfileSlotAction::Delete(slot),
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(26.0),
                                            position_type: PositionType::Relative,
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(Color::NONE),
                                    ))
                                    .with_children(|btn| {
                                        if let Some(ref tex) = btn_danger_tex {
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
                                            Text::new("BORRAR PARTIDA"),
                                            TextFont::from_font_size(10.0),
                                            TextColor(Color::WHITE),
                                        ));
                                    });
                            });
                        } else {
                            // Ranura Vacía
                            card.spawn(Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                height: Val::Px(140.0),
                                row_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|empty| {
                                empty.spawn((
                                    Text::new("- RANURA VACÍA -"),
                                    TextFont::from_font_size(14.0),
                                    TextColor(Color::srgb(0.5, 0.55, 0.65)),
                                ));
                                empty.spawn((
                                    Text::new("Sin datos de partida guardados"),
                                    TextFont::from_font_size(11.0),
                                    TextColor(Color::srgb(0.4, 0.45, 0.55)),
                                ));
                            });

                            card.spawn(Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|empty_actions| {
                                empty_actions
                                    .spawn((
                                        Button,
                                        ProfileSlotAction::Save(slot),
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(36.0),
                                            position_type: PositionType::Relative,
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(Color::NONE),
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
                                            Text::new("GUARDAR PARTIDA ACTUAL AQUÍ"),
                                            TextFont::from_font_size(11.5),
                                            TextColor(Color::srgb(0.15, 0.09, 0.02)),
                                        ));
                                    });

                                empty_actions
                                    .spawn((
                                        Button,
                                        ProfileSlotAction::New(slot),
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(36.0),
                                            position_type: PositionType::Relative,
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(Color::NONE),
                                    ))
                                    .with_children(|btn| {
                                        if let Some(ref tex) = btn_secondary_tex {
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
                                            Text::new("INICIAR NUEVA PARTIDA"),
                                            TextFont::from_font_size(12.0),
                                            TextColor(Color::WHITE),
                                        ));
                                    });
                            });
                        }
                    });
                }
            });

            // === BARRA INFERIOR ===
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|footer| {
                footer.spawn((
                    ProfileStatusMessage,
                    Text::new(
                        "Selecciona una ranura para cargar o guardar usando la persistencia bevy_save.",
                    ),
                    TextFont::from_font_size(13.0),
                    TextColor(Color::srgb(0.7, 0.75, 0.85)),
                ));

                footer
                    .spawn((
                        Button,
                        ProfileBackButton,
                        Node {
                            width: Val::Px(300.0),
                            height: Val::Px(48.0),
                            position_type: PositionType::Relative,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
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
                            Text::new("CONTINUAR AL MENÚ PRINCIPAL ->"),
                            TextFont::from_font_size(13.5),
                            TextColor(Color::srgb(0.15, 0.09, 0.02)),
                        ));
                    });
            });
        });

    if let Some(card_id) = active_card_id {
        let tween = Tween::new(
            EaseFunction::SineInOut,
            std::time::Duration::from_millis(1200),
            UiTransformScaleLens {
                start: Vec2::splat(0.99),
                end: Vec2::splat(1.03),
            },
        )
        .with_repeat(RepeatCount::Infinite, RepeatStrategy::MirroredRepeat);
        commands.spawn((
            CleanupUi,
            TweenAnim::new(tween),
            AnimTarget::component::<UiTransform>(card_id),
        ));
    }
}

fn handle_profile_slot_actions(
    mut q_buttons: Query<(&Interaction, &ProfileSlotAction), Changed<Interaction>>,
    mut active_slot: ResMut<ActiveSaveSlot>,
    mut meta: ResMut<MetaProfile>,
    mut run_state: ResMut<CurrentRunState>,
    map_res: Option<ResMut<crate::map::CurrentMap>>,
    mut q_msg: Query<&mut Text, With<ProfileStatusMessage>>,
    mut ev_trans: MessageWriter<TransitionToEvent>,
    mut ev_notify: MessageWriter<SaveNotificationEvent>,
) {
    for (interaction, action) in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            match action {
                ProfileSlotAction::Load(slot) => {
                    match save::load_profile(*slot) {
                        Ok(p) => {
                            active_slot.0 = *slot;
                            *meta = p.meta;
                            *run_state = p.run_state;
                            if let Some(mut map) = map_res {
                                if let Some(m) = p.map {
                                    *map = m;
                                }
                            }
                            let msg = format!(
                                "[OK] ¡Ranura #{} cargada con éxito! (Pokémon #{})",
                                slot, run_state.selected_starter_id
                            );
                            if let Ok(mut text) = q_msg.single_mut() {
                                **text = msg.clone();
                            }
                            ev_notify.write(SaveNotificationEvent {
                                message: msg,
                                is_error: false,
                            });
                            ev_trans.write(TransitionToEvent { target: AppState::MainMenu });
                            return;
                        }
                        Err(e) => {
                            let msg = format!("[ERROR] Error al cargar: {}", e);
                            if let Ok(mut text) = q_msg.single_mut() {
                                **text = msg.clone();
                            }
                            ev_notify.write(SaveNotificationEvent {
                                message: msg,
                                is_error: true,
                            });
                        }
                    }
                }
                ProfileSlotAction::Save(slot) => {
                    let map_ref = map_res.as_ref().map(|m| &**m);
                    match save::save_profile(*slot, &meta, &run_state, map_ref) {
                        Ok(_) => {
                            active_slot.0 = *slot;
                            let msg = format!(
                                "[GUARDADO] ¡Progreso guardado con éxito en el Perfil #{}! (Acto {} - Piso {})",
                                slot, run_state.current_act, run_state.current_floor
                            );
                            if let Ok(mut text) = q_msg.single_mut() {
                                **text = msg.clone();
                            }
                            ev_notify.write(SaveNotificationEvent {
                                message: msg,
                                is_error: false,
                            });
                        }
                        Err(e) => {
                            let msg = format!("[ERROR] Error al guardar: {}", e);
                            if let Ok(mut text) = q_msg.single_mut() {
                                **text = msg.clone();
                            }
                            ev_notify.write(SaveNotificationEvent {
                                message: msg,
                                is_error: true,
                            });
                        }
                    }
                }
                ProfileSlotAction::New(slot) => {
                    active_slot.0 = *slot;
                    let new_meta = MetaProfile::default();
                    let new_run = CurrentRunState::default();
                    let _ = save::save_profile(*slot, &new_meta, &new_run, None);
                    *meta = new_meta;
                    *run_state = new_run;
                    ev_notify.write(SaveNotificationEvent {
                        message: format!(
                            "[NUEVA PARTIDA] Perfil #{} seleccionado e inicializado.",
                            slot
                        ),
                        is_error: false,
                    });
                    ev_trans.write(TransitionToEvent { target: AppState::MainMenu });
                    return;
                }
                ProfileSlotAction::Delete(slot) => {
                    let _ = save::delete_profile(*slot);
                    let msg = format!("[BORRADO] Ranura #{} eliminada.", slot);
                    if let Ok(mut text) = q_msg.single_mut() {
                        **text = msg.clone();
                    }
                    ev_notify.write(SaveNotificationEvent {
                        message: msg,
                        is_error: false,
                    });
                }
            }
        }
    }
}

fn handle_profile_back(
    mut q_button: Query<&Interaction, (Changed<Interaction>, With<ProfileBackButton>)>,
    mut ev_trans: MessageWriter<TransitionToEvent>,
) {
    for interaction in q_button.iter_mut() {
        if *interaction == Interaction::Pressed {
            ev_trans.write(TransitionToEvent { target: AppState::MainMenu });
        }
    }
}

fn cleanup_profile_ui(mut commands: Commands, q_cleanup: Query<Entity, With<CleanupUi>>) {
    for entity in q_cleanup.iter() {
        commands.entity(entity).despawn();
    }
}
