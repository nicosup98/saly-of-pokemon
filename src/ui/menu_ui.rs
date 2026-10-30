use bevy::prelude::*;
use crate::run::{AppState, CurrentRunState, MetaProfile};
use crate::save::{self, ActiveSaveSlot, SaveNotificationEvent, TOTAL_SLOTS};
use crate::ui::{BorderColor, Button, CleanupUi};
use crate::ui::transition_ui::TransitionToEvent;
use bevy_tweening::{
    lens::{TextColorLens, UiBackgroundColorLens, UiTransformScaleLens, UiTransformTranslationPxLens},
    AnimTarget, RepeatCount, RepeatStrategy, Tween, TweenAnim,
};

pub struct MenuUiPlugin;

impl Plugin for MenuUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
            .add_systems(
                Update,
                (
                    handle_menu_navigation,
                    animate_menu_buttons_hover,
                    animate_menu_particles,
                )
                    .run_if(in_state(AppState::MainMenu)),
            )
            .add_systems(OnExit(AppState::MainMenu), cleanup_menu_ui);
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuButtonAction {
    StartRun,
    SelectStarter,
    Gachapon,
    Profiles,
    SaveCurrentGame,
    QuickSelectProfile(u8),
}

/// Partícula flotante decorativa en el fondo del Menú Principal
#[derive(Component)]
pub struct MenuParticle {
    pub x_pct: f32,
    pub y_pct: f32,
    pub speed_y: f32,
    pub sway_amp: f32,
    pub sway_freq: f32,
    pub phase: f32,
}

#[derive(Component)]
pub struct MenuTeamInfoLabel;

#[derive(Component)]
pub struct MenuMetaInfoLabel;

#[derive(Component)]
pub struct MenuStarterSprite;

#[derive(Component)]
pub struct MenuQuickProfileButton(pub u8);

#[derive(Component)]
pub struct MenuQuickProfileText(pub u8);

fn setup_main_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    meta: Res<MetaProfile>,
    run_state: Res<CurrentRunState>,
    active_slot: Option<Res<ActiveSaveSlot>>,
) {
    let current_slot = active_slot.as_ref().map(|s| s.0).unwrap_or(1);
    let profiles = save::list_profiles();

    commands
        .spawn((
            CleanupUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(22.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.07, 0.12, 0.88)),
        ))
        .with_children(|root| {
            // === CAPA DE PARTÍCULAS ANIMADAS DEL MENÚ (BRASAS Y DESTELLOS) ===
            for i in 0..28 {
                let fi = i as f32;
                let x_pct = (fi * 17.3 + 5.0) % 94.0 + 3.0;
                let y_pct = (fi * 23.7 + 7.0) % 92.0 + 4.0;
                let size = 6.0 + (i % 4) as f32 * 4.0;
                let speed_y = 6.0 + (i % 5) as f32 * 2.5;
                let sway_amp = 1.2 + (i % 3) as f32 * 0.9;
                let sway_freq = 1.2 + (i % 4) as f32 * 0.45;
                let phase = fi * 0.85;

                let (col_start, col_end) = match i % 3 {
                    0 => (
                        Color::srgba(1.0, 0.45, 0.15, 0.75),
                        Color::srgba(1.0, 0.85, 0.25, 0.25),
                    ),
                    1 => (
                        Color::srgba(1.0, 0.85, 0.25, 0.70),
                        Color::srgba(0.95, 0.35, 0.15, 0.20),
                    ),
                    _ => (
                        Color::srgba(0.30, 0.75, 1.0, 0.65),
                        Color::srgba(0.65, 0.40, 1.0, 0.20),
                    ),
                };

                root.spawn((
                    MenuParticle {
                        x_pct,
                        y_pct,
                        speed_y,
                        sway_amp,
                        sway_freq,
                        phase,
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(x_pct),
                        top: Val::Px(0.0),
                        width: Val::Px(size),
                        height: Val::Px(size),
                        border_radius: BorderRadius::all(Val::Px(size * 0.5)),
                        ..default()
                    },
                    UiTransform::from_translation(Val2::px(0.0, y_pct * 7.2)),
                    BackgroundColor(col_start),
                ))
                .with_children(|p| {
                    let target_id = p.target_entity();
                    let scale_tween = Tween::new(
                        EaseFunction::SineInOut,
                        std::time::Duration::from_millis(700 + (i as u64 % 5) * 180),
                        UiTransformScaleLens {
                            start: Vec2::splat(0.65),
                            end: Vec2::splat(1.45),
                        },
                    )
                    .with_repeat(RepeatCount::Infinite, RepeatStrategy::MirroredRepeat);
                    p.spawn((
                        TweenAnim::new(scale_tween),
                        AnimTarget::component::<UiTransform>(target_id),
                    ));

                    let color_tween = Tween::new(
                        EaseFunction::SineInOut,
                        std::time::Duration::from_millis(900 + (i as u64 % 4) * 220),
                        UiBackgroundColorLens {
                            start: col_start,
                            end: col_end,
                        },
                    )
                    .with_repeat(RepeatCount::Infinite, RepeatStrategy::MirroredRepeat);
                    p.spawn((
                        TweenAnim::new(color_tween),
                        AnimTarget::component::<BackgroundColor>(target_id),
                    ));
                });
            }

            // === CABECERA Y SELECTOR RÁPIDO DE PERFIL ===
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|title_box| {
                // Título Principal animado con bevy_tweening
                title_box
                    .spawn((
                        Text::new("SLAY OF POKÉMON"),
                        TextFont::from_font_size(40.0),
                        TextColor(Color::srgb(1.0, 0.45, 0.18)),
                        UiTransform::default(),
                    ))
                    .with_children(|t| {
                        let tid = t.target_entity();
                        let col_tween = Tween::new(
                            EaseFunction::SineInOut,
                            std::time::Duration::from_millis(1400),
                            TextColorLens {
                                start: Color::srgb(1.0, 0.42, 0.15),
                                end: Color::srgb(1.0, 0.88, 0.25),
                            },
                        )
                        .with_repeat(RepeatCount::Infinite, RepeatStrategy::MirroredRepeat);
                        t.spawn((
                            TweenAnim::new(col_tween),
                            AnimTarget::component::<TextColor>(tid),
                        ));

                        let scale_tween = Tween::new(
                            EaseFunction::SineInOut,
                            std::time::Duration::from_millis(1800),
                            UiTransformScaleLens {
                                start: Vec2::splat(0.98),
                                end: Vec2::splat(1.05),
                            },
                        )
                        .with_repeat(RepeatCount::Infinite, RepeatStrategy::MirroredRepeat);
                        t.spawn((
                            TweenAnim::new(scale_tween),
                            AnimTarget::component::<UiTransform>(tid),
                        ));
                    });

                title_box.spawn((
                    Text::new("ROGUELIKE DECKBUILDER RPG (RUST + BEVY ENGINE)"),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::srgb(0.8, 0.85, 0.95)),
                ));

                title_box.spawn((
                    MenuMetaInfoLabel,
                    Text::new(format!(
                        "Perfil Activo: #{} | Fichas Gachapón: {} | Saldo en Run: {} P$ | Acto {} - Piso {}",
                        current_slot,
                        meta.gachapon_tokens,
                        run_state.poke_dollars,
                        run_state.current_act,
                        run_state.current_floor
                    )),
                    TextFont::from_font_size(13.5),
                    TextColor(Color::srgb(1.0, 0.85, 0.3)),
                ));

                // Barra de Selección Rápida de los 3 Perfiles en el propio Menú
                title_box
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            column_gap: Val::Px(10.0),
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.5)),
                            border_radius: BorderRadius::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.10, 0.12, 0.18, 0.92)),
                        BorderColor(Color::srgb(0.30, 0.45, 0.65)),
                    ))
                    .with_children(|prof_bar| {
                        prof_bar.spawn((
                            Text::new("ESCOGER PERFIL:"),
                            TextFont::from_font_size(12.5),
                            TextColor(Color::srgb(0.75, 0.85, 1.0)),
                        ));

                        for slot in 1..=TOTAL_SLOTS {
                            let is_active = slot == current_slot;
                            let summary = profiles[(slot - 1) as usize].as_ref();
                            let label = if let Some(s) = summary {
                                if is_active {
                                    format!(
                                        "* PERFIL {} [ACTIVO] (Piso {})",
                                        slot, s.current_floor
                                    )
                                } else {
                                    format!("PERFIL {} (Piso {})", slot, s.current_floor)
                                }
                            } else if is_active {
                                format!("* PERFIL {} [ACTIVO - NUEVO]", slot)
                            } else {
                                format!("PERFIL {} [VACÍO]", slot)
                            };

                            let (bg, border) = if is_active {
                                (
                                    Color::srgb(0.22, 0.42, 0.28),
                                    Color::srgb(1.0, 0.85, 0.25),
                                )
                            } else {
                                (
                                    Color::srgb(0.15, 0.18, 0.26),
                                    Color::srgb(0.35, 0.50, 0.70),
                                )
                            };

                            prof_bar
                                .spawn((
                                    Button,
                                    MenuButtonAction::QuickSelectProfile(slot),
                                    MenuQuickProfileButton(slot),
                                    Node {
                                        padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border: UiRect::all(Val::Px(2.0)),
                                        border_radius: BorderRadius::all(Val::Px(6.0)),
                                        ..default()
                                    },
                                    UiTransform::default(),
                                    BackgroundColor(bg),
                                    BorderColor(border),
                                ))
                                .with_child((
                                    MenuQuickProfileText(slot),
                                    Text::new(label),
                                    TextFont::from_font_size(11.5),
                                    TextColor(Color::WHITE),
                                ));
                        }
                    });
            });

            // === ZONA CENTRAL CON SPRITE ANIMADO DEL STARTER Y BOTONES ===
            root.spawn(Node {
                width: Val::Percent(90.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|center| {
                // Pedestal animado del Entrenador y Compañero Pokémon
                center
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(12.0),
                            padding: UiRect::all(Val::Px(22.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            border_radius: BorderRadius::all(Val::Px(14.0)),
                            ..default()
                        },
                        UiTransform::default(),
                        BackgroundColor(Color::srgba(0.11, 0.13, 0.20, 0.90)),
                        BorderColor(Color::srgb(0.95, 0.55, 0.20)),
                    ))
                    .with_children(|sprite_col| {
                        let card_id = sprite_col.target_entity();
                        // Animación de levitación suave del pedestal con bevy_tweening
                        let float_tween = Tween::new(
                            EaseFunction::SineInOut,
                            std::time::Duration::from_millis(1600),
                            UiTransformTranslationPxLens {
                                start: Vec2::new(0.0, -7.0),
                                end: Vec2::new(0.0, 7.0),
                            },
                        )
                        .with_repeat(RepeatCount::Infinite, RepeatStrategy::MirroredRepeat);
                        sprite_col.spawn((
                            TweenAnim::new(float_tween),
                            AnimTarget::component::<UiTransform>(card_id),
                        ));

                        let sprite_path = format!(
                            "sprites/pokemon/front/{}.png",
                            run_state.selected_starter_id
                        );
                        sprite_col
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::FlexEnd,
                                justify_content: JustifyContent::Center,
                                column_gap: Val::Px(16.0),
                                ..default()
                            })
                            .with_children(|pair| {
                                pair.spawn((
                                    ImageNode::new(
                                        asset_server.load("sprites/trainers/player_front.png"),
                                    ),
                                    Node {
                                        width: Val::Px(76.0),
                                        height: Val::Px(76.0),
                                        margin: UiRect::bottom(Val::Px(10.0)),
                                        ..default()
                                    },
                                ));
                                pair.spawn((
                                    MenuStarterSprite,
                                    ImageNode::new(asset_server.load(&sprite_path)),
                                    Node {
                                        width: Val::Px(150.0),
                                        height: Val::Px(150.0),
                                        ..default()
                                    },
                                    UiTransform::default(),
                                ))
                                .with_children(|pkmn| {
                                    let pid = pkmn.target_entity();
                                    let breathe = Tween::new(
                                        EaseFunction::SineInOut,
                                        std::time::Duration::from_millis(1100),
                                        UiTransformScaleLens {
                                            start: Vec2::splat(0.95),
                                            end: Vec2::splat(1.07),
                                        },
                                    )
                                    .with_repeat(
                                        RepeatCount::Infinite,
                                        RepeatStrategy::MirroredRepeat,
                                    );
                                    pkmn.spawn((
                                        TweenAnim::new(breathe),
                                        AnimTarget::component::<UiTransform>(pid),
                                    ));
                                });
                            });

                        let current_name =
                            crate::pokemon::get_species_data(run_state.selected_starter_id)
                                .map(|s| s.name)
                                .unwrap_or_else(|| "Charmander".to_string());
                        sprite_col.spawn((
                            MenuTeamInfoLabel,
                            Text::new(format!(
                                "Equipo: Entrenador Red y {} (#{}) | Perfil Activo: #{}",
                                current_name, run_state.selected_starter_id, current_slot
                            )),
                            TextFont::from_font_size(14.0),
                            TextColor(Color::srgb(1.0, 0.78, 0.4)),
                        ));
                    });

                // Menú de Botones de Navegación y Guardado
                center
                    .spawn(Node {
                        width: Val::Px(380.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        ..default()
                    })
                    .with_children(|btn_col| {
                        let options = [
                            (
                                MenuButtonAction::StartRun,
                                "INICIAR / CONTINUAR RUTA (MAPA)",
                                Color::srgb(0.20, 0.60, 0.92),
                            ),
                            (
                                MenuButtonAction::SelectStarter,
                                "SELECCIONAR POKÉMON INICIAL",
                                Color::srgb(0.90, 0.55, 0.20),
                            ),
                            (
                                MenuButtonAction::Gachapon,
                                "TIENDA GACHAPÓN Y COLECCIÓN",
                                Color::srgb(0.70, 0.32, 0.88),
                            ),
                            (
                                MenuButtonAction::SaveCurrentGame,
                                "GUARDAR PARTIDA EN PERFIL ACTIVO",
                                Color::srgb(0.25, 0.85, 0.45),
                            ),
                            (
                                MenuButtonAction::Profiles,
                                "GESTIONAR LOS 3 PERFILES DE GUARDADO",
                                Color::srgb(0.35, 0.75, 0.65),
                            ),
                        ];

                        for (action, label, col) in options {
                            btn_col
                                .spawn((
                                    Button,
                                    action,
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Px(44.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border: UiRect::all(Val::Px(2.0)),
                                        border_radius: BorderRadius::all(Val::Px(8.0)),
                                        ..default()
                                    },
                                    UiTransform::default(),
                                    BackgroundColor(Color::srgb(0.14, 0.16, 0.23)),
                                    BorderColor(col),
                                ))
                                .with_child((
                                    Text::new(label),
                                    TextFont::from_font_size(14.0),
                                    TextColor(Color::WHITE),
                                ));
                        }
                    });
            });

            // === PIE DE PÁGINA ===
            root.spawn((
                Text::new(
                    "Puedes cambiar entre los 3 Perfiles arriba o pulsar 'GUARDAR PARTIDA' en cualquier momento.",
                ),
                TextFont::from_font_size(12.0),
                TextColor(Color::srgb(0.65, 0.7, 0.8)),
            ));
        });
}

fn handle_menu_navigation(
    mut q_buttons: Query<(&Interaction, &MenuButtonAction), Changed<Interaction>>,
    mut active_slot: ResMut<ActiveSaveSlot>,
    mut meta: ResMut<MetaProfile>,
    mut run_state: ResMut<CurrentRunState>,
    mut map_res: Option<ResMut<crate::map::CurrentMap>>,
    asset_server: Option<Res<AssetServer>>,
    mut q_starter_img: Query<&mut ImageNode, With<MenuStarterSprite>>,
    mut q_team_label: Query<
        &mut Text,
        (
            With<MenuTeamInfoLabel>,
            Without<MenuMetaInfoLabel>,
            Without<MenuQuickProfileText>,
        ),
    >,
    mut q_meta_label: Query<
        &mut Text,
        (
            With<MenuMetaInfoLabel>,
            Without<MenuTeamInfoLabel>,
            Without<MenuQuickProfileText>,
        ),
    >,
    mut q_prof_btns: Query<(&MenuQuickProfileButton, &mut BackgroundColor)>,
    mut q_prof_texts: Query<
        (&MenuQuickProfileText, &mut Text),
        (Without<MenuTeamInfoLabel>, Without<MenuMetaInfoLabel>),
    >,
    mut ev_trans: MessageWriter<TransitionToEvent>,
    mut ev_notify: MessageWriter<SaveNotificationEvent>,
) {
    let mut refresh_ui = false;

    for (interaction, action) in q_buttons.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *action {
            MenuButtonAction::StartRun => {
                ev_trans.write(TransitionToEvent {
                    target: AppState::MapNavigation,
                });
            }
            MenuButtonAction::SelectStarter => {
                ev_trans.write(TransitionToEvent {
                    target: AppState::StarterSelection,
                });
            }
            MenuButtonAction::Gachapon => {
                ev_trans.write(TransitionToEvent {
                    target: AppState::GachaponShop,
                });
            }
            MenuButtonAction::Profiles => {
                ev_trans.write(TransitionToEvent {
                    target: AppState::ProfileSelect,
                });
            }
            MenuButtonAction::SaveCurrentGame => {
                let slot = active_slot.0;
                let map_ref = map_res.as_ref().map(|m| &**m);
                match save::save_profile(slot, &meta, &run_state, map_ref) {
                    Ok(_) => {
                        ev_notify.write(SaveNotificationEvent {
                            message: format!(
                                "[GUARDADO] ¡Partida guardada con éxito en el Perfil #{}! (Acto {} - Piso {})",
                                slot, run_state.current_act, run_state.current_floor
                            ),
                            is_error: false,
                        });
                        refresh_ui = true;
                    }
                    Err(e) => {
                        ev_notify.write(SaveNotificationEvent {
                            message: format!("[ERROR] No se pudo guardar en el Perfil #{}: {}", slot, e),
                            is_error: true,
                        });
                    }
                }
            }
            MenuButtonAction::QuickSelectProfile(slot) => {
                active_slot.0 = slot;
                match save::load_profile(slot) {
                    Ok(p) => {
                        *meta = p.meta;
                        *run_state = p.run_state;
                        if let Some(ref mut map) = map_res {
                            if let Some(saved_map) = p.map {
                                **map = saved_map;
                            }
                        }
                        let pkmn_name = crate::pokemon::get_species_data(run_state.selected_starter_id)
                            .map(|s| s.name)
                            .unwrap_or_else(|| "Pokémon".to_string());
                        ev_notify.write(SaveNotificationEvent {
                            message: format!(
                                "[PERFIL #{}] Partida cargada: {} (#{}) | Acto {} - Piso {}",
                                slot, pkmn_name, run_state.selected_starter_id, run_state.current_act, run_state.current_floor
                            ),
                            is_error: false,
                        });
                    }
                    Err(_) => {
                        // Ranura nueva: inicializamos y guardamos automáticamente el nuevo perfil
                        let new_meta = MetaProfile::default();
                        let new_run = CurrentRunState::default();
                        let _ = save::save_profile(slot, &new_meta, &new_run, None);
                        *meta = new_meta;
                        *run_state = new_run;
                        ev_notify.write(SaveNotificationEvent {
                            message: format!(
                                "[PERFIL #{}] Nuevo perfil activado y guardado (Charmander #4)",
                                slot
                            ),
                            is_error: false,
                        });
                    }
                }
                refresh_ui = true;
            }
        }
    }

    if refresh_ui {
        let current_slot = active_slot.0;
        let current_name = crate::pokemon::get_species_data(run_state.selected_starter_id)
            .map(|s| s.name)
            .unwrap_or_else(|| "Charmander".to_string());

        if let Some(asset_server) = asset_server {
            let sprite_path = format!("sprites/pokemon/front/{}.png", run_state.selected_starter_id);
            for mut img in q_starter_img.iter_mut() {
                img.image = asset_server.load(&sprite_path);
            }
        }

        for mut text in q_team_label.iter_mut() {
            **text = format!(
                "Equipo: Entrenador Red y {} (#{}) | Perfil Activo: #{}",
                current_name, run_state.selected_starter_id, current_slot
            );
        }

        for mut text in q_meta_label.iter_mut() {
            **text = format!(
                "Perfil Activo: #{} | Fichas Gachapón: {} | Saldo en Run: {} P$ | Acto {} - Piso {}",
                current_slot,
                meta.gachapon_tokens,
                run_state.poke_dollars,
                run_state.current_act,
                run_state.current_floor
            );
        }

        let profiles = save::list_profiles();
        for (btn, mut bg) in q_prof_btns.iter_mut() {
            bg.0 = if btn.0 == current_slot {
                Color::srgb(0.22, 0.42, 0.28)
            } else {
                Color::srgb(0.15, 0.18, 0.26)
            };
        }
        for (txt_slot, mut text) in q_prof_texts.iter_mut() {
            let slot = txt_slot.0;
            let is_active = slot == current_slot;
            let summary = profiles[(slot - 1) as usize].as_ref();
            **text = if let Some(s) = summary {
                if is_active {
                    format!("* PERFIL {} [ACTIVO] (Piso {})", slot, s.current_floor)
                } else {
                    format!("PERFIL {} (Piso {})", slot, s.current_floor)
                }
            } else if is_active {
                format!("* PERFIL {} [ACTIVO - NUEVO]", slot)
            } else {
                format!("PERFIL {} [VACÍO]", slot)
            };
        }
    }
}

fn animate_menu_buttons_hover(
    mut q_btns: Query<
        (&Interaction, &MenuButtonAction, &mut BackgroundColor, &mut UiTransform),
        Changed<Interaction>,
    >,
    active_slot: Option<Res<ActiveSaveSlot>>,
) {
    let current_slot = active_slot.as_ref().map(|s| s.0).unwrap_or(1);
    for (interaction, action, mut bg, mut transform) in q_btns.iter_mut() {
        let is_quick_active = matches!(action, MenuButtonAction::QuickSelectProfile(s) if *s == current_slot);
        match *interaction {
            Interaction::Hovered => {
                transform.scale = Vec2::splat(1.04);
                bg.0 = if is_quick_active {
                    Color::srgb(0.28, 0.52, 0.35)
                } else {
                    Color::srgb(0.22, 0.26, 0.36)
                };
            }
            Interaction::Pressed => {
                transform.scale = Vec2::splat(0.97);
                bg.0 = Color::srgb(0.30, 0.38, 0.52);
            }
            Interaction::None => {
                transform.scale = Vec2::ONE;
                bg.0 = if is_quick_active {
                    Color::srgb(0.22, 0.42, 0.28)
                } else if matches!(action, MenuButtonAction::QuickSelectProfile(_)) {
                    Color::srgb(0.15, 0.18, 0.26)
                } else {
                    Color::srgb(0.14, 0.16, 0.23)
                };
            }
        }
    }
}

/// Anima las partículas del menú principal directamente mediante `UiTransform`
/// para evitar recalcular el layout Flexbox (Taffy) en las 28 partículas cada frame.
fn animate_menu_particles(
    time: Res<Time>,
    mut q_particles: Query<(&mut MenuParticle, &mut UiTransform)>,
) {
    let dt = time.delta_secs().min(0.05);
    let elapsed = time.elapsed_secs();
    for (mut p, mut transform) in q_particles.iter_mut() {
        p.y_pct -= p.speed_y * dt;
        if p.y_pct < -4.0 {
            p.y_pct = 104.0;
        }
        let sway_px = (elapsed * p.sway_freq + p.phase).sin() * p.sway_amp * 12.8;
        let y_px = p.y_pct * 7.2;
        transform.translation = Val2::px(sway_px, y_px);
    }
}

fn cleanup_menu_ui(mut commands: Commands, q_cleanup: Query<Entity, With<CleanupUi>>) {
    for entity in q_cleanup.iter() {
        commands.entity(entity).despawn();
    }
}
