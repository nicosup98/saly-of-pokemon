use bevy::prelude::*;
use crate::run::{AppState, CurrentRunState, MetaProfile};
use crate::save::{self, ActiveSaveSlot, SaveNotificationEvent, TOTAL_SLOTS};
use crate::ui::{BorderColor, Button, CleanupUi};
use crate::ui::transition_ui::TransitionToEvent;
use bevy_tweening::{
    lens::{UiBackgroundColorLens, UiTransformScaleLens},
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
    preloaded: Option<Res<crate::ui::PreloadedAssets>>,
    meta: Res<MetaProfile>,
    run_state: Res<CurrentRunState>,
    active_slot: Option<Res<ActiveSaveSlot>>,
) {
    let pre = preloaded.as_deref();
    let srv = Some(&*asset_server);
    let current_slot = active_slot.as_ref().map(|s| s.0).unwrap_or(1);
    let profiles = save::list_profiles();

    let menu_bg_tex = crate::ui::load_ui_image(pre, srv, "ui/menus/menu_main.png");
    let btn_gold_tex = crate::ui::load_ui_image(pre, srv, "ui/buttons/btn_gold.png");
    let btn_primary_tex = crate::ui::load_ui_image(pre, srv, "ui/buttons/btn_primary.png");
    let btn_secondary_tex = crate::ui::load_ui_image(pre, srv, "ui/buttons/btn_secondary.png");
    let hp_bg_tex = crate::ui::load_ui_image(pre, srv, "ui/bars/hp_bar_bg.png");
    let hp_fill_tex = crate::ui::load_ui_image(pre, srv, "ui/bars/hp_fill_player.png");

    commands
        .spawn((
            CleanupUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Relative,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgb(0.03, 0.04, 0.07)),
        ))
        .with_children(|root| {
            // Fondo principal VectorCraft (`ui/menus/menu_main.png` 1280x720) sin velo opaco ni menú viejo superpuesto
            if let Some(tex) = menu_bg_tex {
                root.spawn((
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

            // === CAPA DE PARTÍCULAS ANIMADAS DEL MENÚ (BRASAS Y DESTELLOS) ===
            for i in 0..28 {
                let fi = i as f32;
                let x_pct = (fi * 17.3 + 5.0) % 94.0 + 3.0;
                let y_pct = (fi * 23.7 + 7.0) % 92.0 + 4.0;
                let size = 5.0 + (i % 4) as f32 * 3.0;
                let speed_y = 6.0 + (i % 5) as f32 * 2.5;
                let sway_amp = 1.2 + (i % 3) as f32 * 0.9;
                let sway_freq = 1.2 + (i % 4) as f32 * 0.45;
                let phase = fi * 0.85;

                let (col_start, col_end) = match i % 3 {
                    0 => (
                        Color::srgba(1.0, 0.48, 0.24, 0.65),
                        Color::srgba(0.95, 0.76, 0.30, 0.20),
                    ),
                    1 => (
                        Color::srgba(0.95, 0.76, 0.30, 0.60),
                        Color::srgba(1.0, 0.36, 0.20, 0.18),
                    ),
                    _ => (
                        Color::srgba(0.22, 0.71, 1.0, 0.55),
                        Color::srgba(0.61, 0.30, 1.0, 0.18),
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

            // === SELECTOR RÁPIDO DE PERFILES EN LA ESQUINA SUPERIOR IZQUIERDA ===
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(6.4),
                    top: Val::Percent(5.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                    border: UiRect::all(Val::Px(1.5)),
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.06, 0.09, 0.14, 0.88)),
                BorderColor(Color::srgb(0.23, 0.33, 0.45)),
            ))
            .with_children(|prof_bar| {
                prof_bar.spawn((
                    Text::new("PERFIL:"),
                    TextFont::from_font_size(11.5),
                    TextColor(Color::srgb(0.95, 0.76, 0.30)),
                ));

                for slot in 1..=TOTAL_SLOTS {
                    let is_active = slot == current_slot;
                    let summary = profiles[(slot - 1) as usize].as_ref();
                    let label = if let Some(s) = summary {
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

                    let (bg, border) = if is_active {
                        (
                            Color::srgb(0.22, 0.42, 0.28),
                            Color::srgb(0.95, 0.76, 0.30),
                        )
                    } else {
                        (
                            Color::srgb(0.11, 0.16, 0.24),
                            Color::srgb(0.22, 0.34, 0.50),
                        )
                    };

                    prof_bar
                        .spawn((
                            Button,
                            MenuButtonAction::QuickSelectProfile(slot),
                            MenuQuickProfileButton(slot),
                            Node {
                                padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.5)),
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
                            TextFont::from_font_size(10.5),
                            TextColor(Color::WHITE),
                        ));
                }
            });

            // === COLUMNA IZQUIERDA DE BOTONES VECTORCRAFT (BAJO EL TÍTULO DE `menu_main.png`) ===
            root.spawn(Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(6.4),
                top: Val::Percent(54.0),
                width: Val::Percent(35.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|btn_col| {
                let options: [(MenuButtonAction, &str, Color, Option<Handle<Image>>); 5] = [
                    (
                        MenuButtonAction::StartRun,
                        "INICIAR / CONTINUAR RUN",
                        Color::srgb(0.15, 0.09, 0.02),
                        btn_gold_tex.clone(),
                    ),
                    (
                        MenuButtonAction::SelectStarter,
                        "CAMBIAR POKÉMON INICIAL",
                        Color::WHITE,
                        btn_primary_tex.clone(),
                    ),
                    (
                        MenuButtonAction::Gachapon,
                        "TIENDA GACHAPÓN Y COLECCIÓN",
                        Color::WHITE,
                        btn_primary_tex.clone(),
                    ),
                    (
                        MenuButtonAction::SaveCurrentGame,
                        "GUARDAR PARTIDA ACTUAL",
                        Color::srgb(0.92, 0.95, 1.0),
                        btn_secondary_tex.clone(),
                    ),
                    (
                        MenuButtonAction::Profiles,
                        "GESTIONAR PERFILES (3 RANURAS)",
                        Color::srgb(0.92, 0.95, 1.0),
                        btn_secondary_tex.clone(),
                    ),
                ];

                for (action, label, text_col, btn_tex) in options {
                    btn_col
                        .spawn((
                            Button,
                            action,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(46.0),
                                position_type: PositionType::Relative,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            UiTransform::default(),
                            BackgroundColor(Color::NONE),
                        ))
                        .with_children(|btn| {
                            if let Some(tex) = btn_tex {
                                btn.spawn((
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
                            btn.spawn((
                                Text::new(label),
                                TextFont::from_font_size(14.0),
                                TextColor(text_col),
                            ));
                        });
                }
            });

            // === PANEL DERECHO (`EQUIPO ACTIVO` EN `menu_main.png`: X=60.9%, Y=14.4%, W=34.2%, H=72.2%) ===
            root.spawn(Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(60.9),
                top: Val::Percent(14.4),
                width: Val::Percent(34.2),
                height: Val::Percent(72.2),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(26.0), Val::Px(20.0)),
                ..default()
            })
            .with_children(|team_panel| {
                // Cabecera superior derecha del panel ("ACTIVO #X")
                team_panel
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::FlexEnd,
                        ..default()
                    })
                    .with_child((
                        Text::new(format!("RANURA #{}", current_slot)),
                        TextFont::from_font_size(12.0),
                        TextColor(Color::srgb(0.95, 0.76, 0.30)),
                    ));

                // Escenario central con Entrenador Red y el Pokémon Inicial animado sobre el pedestal de `menu_main.png`
                let sprite_path = format!(
                    "sprites/pokemon/front/{}.png",
                    run_state.selected_starter_id
                );
                team_panel
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(48.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        column_gap: Val::Px(14.0),
                        ..default()
                    })
                    .with_children(|pair| {
                        pair.spawn((
                            ImageNode::new(asset_server.load("sprites/trainers/player_front.png")),
                            Node {
                                width: Val::Px(72.0),
                                height: Val::Px(72.0),
                                margin: UiRect::top(Val::Px(24.0)),
                                ..default()
                            },
                        ));
                        pair.spawn((
                            MenuStarterSprite,
                            ImageNode::new(asset_server.load(&sprite_path)),
                            Node {
                                width: Val::Px(148.0),
                                height: Val::Px(148.0),
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
                            .with_repeat(RepeatCount::Infinite, RepeatStrategy::MirroredRepeat);
                            pkmn.spawn((
                                TweenAnim::new(breathe),
                                AnimTarget::component::<UiTransform>(pid),
                            ));
                        });
                    });

                // Datos dinámicos del Pokémon y del Perfil Activo en la parte inferior del panel
                let current_name = crate::pokemon::get_species_data(run_state.selected_starter_id)
                    .map(|s| s.name)
                    .unwrap_or_else(|| "Charmander".to_string());

                team_panel
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|stats| {
                        stats.spawn((
                            MenuTeamInfoLabel,
                            Text::new(format!(
                                "Equipo: Entrenador Red y {} (#{}) | Perfil Activo: #{}",
                                current_name, run_state.selected_starter_id, current_slot
                            )),
                            TextFont::from_font_size(14.5),
                            TextColor(Color::srgb(0.93, 0.95, 1.0)),
                        ));

                        // Barra de vida VectorCraft (`hp_bar_bg.png` + `hp_fill_player.png`)
                        let player_lvl = run_state.player_level();
                        let max_hp = crate::pokemon::get_species_data(run_state.selected_starter_id)
                            .map(|d| (d.base_hp as i32 * 2) + (player_lvl as i32 * 4))
                            .unwrap_or(110)
                            .max(1);
                        let cur_hp = run_state.player_hp.unwrap_or(max_hp).clamp(0, max_hp);
                        let hp_pct = (cur_hp as f32 / max_hp as f32 * 100.0).clamp(0.0, 100.0);
                        stats
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(20.0),
                                position_type: PositionType::Relative,
                                align_items: AlignItems::Center,
                                padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                                overflow: Overflow::clip(),
                                ..default()
                            })
                            .with_children(|bar| {
                                if let Some(ref bg) = hp_bg_tex {
                                    bar.spawn((
                                        ImageNode::new(bg.clone()),
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
                                let mut fill = bar.spawn((
                                    Node {
                                        width: Val::Percent(hp_pct),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.27, 0.83, 0.49)),
                                ));
                                if let Some(ref ft) = hp_fill_tex {
                                    fill.insert(ImageNode::new(ft.clone()));
                                }
                            });

                        stats.spawn((
                            Text::new(format!(
                                "HP {} / {} · NV. {}",
                                cur_hp, max_hp, player_lvl
                            )),
                            TextFont::from_font_size(11.5),
                            TextColor(Color::srgb(0.58, 0.64, 0.75)),
                        ));

                        stats.spawn((
                            MenuMetaInfoLabel,
                            Text::new(format!(
                                "Perfil Activo: #{} | Fichas Gachapón: {} | Saldo en Run: {} P$ | Acto {} - Piso {}",
                                current_slot,
                                meta.gachapon_tokens,
                                run_state.poke_dollars,
                                run_state.current_act,
                                run_state.current_floor
                            )),
                            TextFont::from_font_size(12.0),
                            TextColor(Color::srgb(0.95, 0.76, 0.30)),
                        ));
                    });
            });
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
        let is_quick = matches!(action, MenuButtonAction::QuickSelectProfile(_));
        let is_quick_active = matches!(action, MenuButtonAction::QuickSelectProfile(s) if *s == current_slot);
        match *interaction {
            Interaction::Hovered => {
                transform.scale = Vec2::splat(1.04);
                if is_quick {
                    bg.0 = if is_quick_active {
                        Color::srgb(0.28, 0.52, 0.35)
                    } else {
                        Color::srgb(0.22, 0.26, 0.36)
                    };
                }
            }
            Interaction::Pressed => {
                transform.scale = Vec2::splat(0.97);
                if is_quick {
                    bg.0 = Color::srgb(0.30, 0.38, 0.52);
                }
            }
            Interaction::None => {
                transform.scale = Vec2::ONE;
                if is_quick {
                    bg.0 = if is_quick_active {
                        Color::srgb(0.22, 0.42, 0.28)
                    } else {
                        Color::srgb(0.11, 0.16, 0.24)
                    };
                } else {
                    bg.0 = Color::NONE;
                }
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
