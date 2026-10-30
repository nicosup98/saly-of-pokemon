use bevy::prelude::*;
use crate::gachapon::{pull_gachapon, species_rarity};
use crate::pokemon::get_species_data;
use crate::run::{AppState, CurrentRunState, MetaProfile};
use crate::types::RarityTier;
use crate::ui::{BorderColor, Button, CleanupUi};

pub struct GachaponUiPlugin;

impl Plugin for GachaponUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GachaponUiState>()
            .init_resource::<GachaponAnimation>()
            .add_systems(OnEnter(AppState::GachaponShop), setup_gachapon_ui)
            .add_systems(
                Update,
                (
                    (
                        handle_slot_fragment,
                        handle_single_pull,
                        handle_multi_pull,
                        handle_skip_gachapon_animation,
                        update_gachapon_animation,
                        animate_gachapon_particles,
                    ),
                    (
                        animate_gachapon_sprite_and_capsule,
                        handle_exit_gachapon,
                        handle_gachapon_to_starter_select,
                        handle_gachapon_quick_select_pokemon,
                        update_gachapon_selection_badges,
                        update_gachapon_display,
                    ),
                )
                    .run_if(in_state(AppState::GachaponShop)),
            )
            .add_systems(OnExit(AppState::GachaponShop), cleanup_gachapon_ui);
    }
}

#[derive(Component)]
pub struct GachaponSelectPokemonBadgeButton(pub u32);

#[derive(Component)]
pub struct GachaponToStarterSelectionButton;

#[derive(Resource, Default)]
pub struct GachaponUiState {
    pub selected_target: Option<u32>,
    pub fragments_slotted: u32,
    pub last_pull_result: Option<u32>,
    pub last_pull_is_new: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GachaponAnimStep {
    Idle,
    Shaking,
    Burst,
    Revealed,
}

#[derive(Resource)]
pub struct GachaponAnimation {
    pub step: GachaponAnimStep,
    pub timer: Timer,
    pub pulled_pokemon: u32,
    pub rarity: RarityTier,
    pub is_new: bool,
    pub multi_pull_list: Vec<u32>,
    pub shake_elapsed: f32,
    pub reveal_elapsed: f32,
}

impl Default for GachaponAnimation {
    fn default() -> Self {
        Self {
            step: GachaponAnimStep::Idle,
            timer: Timer::from_seconds(0.1, TimerMode::Once),
            pulled_pokemon: 4,
            rarity: RarityTier::Rare,
            is_new: false,
            multi_pull_list: Vec::new(),
            shake_elapsed: 0.0,
            reveal_elapsed: 0.0,
        }
    }
}

#[derive(Component)]
pub struct GachaponSinglePullButton;

#[derive(Component)]
pub struct GachaponMultiPullButton;

#[derive(Component)]
pub struct GachaponExitButton;

#[derive(Component)]
pub struct GachaponSlotFragmentButton(pub u32);

#[derive(Component)]
pub struct GachaponTokensText;

#[derive(Component)]
pub struct GachaponOddsText;

#[derive(Component)]
pub struct GachaponStatusText;

#[derive(Component)]
pub struct GachaponRewardCard;

#[derive(Component)]
pub struct GachaponRewardHeader;

#[derive(Component)]
pub struct GachaponStageArea;

#[derive(Component)]
pub struct GachaponAuraGlow;

#[derive(Component)]
pub struct GachaponCapsuleOrb;

#[derive(Component)]
pub struct GachaponCapsuleTopHalf;

#[derive(Component)]
pub struct GachaponCapsuleButton;

#[derive(Component)]
pub struct GachaponRewardSprite;

#[derive(Component)]
pub struct GachaponRewardName;

#[derive(Component)]
pub struct GachaponRarityBadge;

#[derive(Component)]
pub struct GachaponSkipButton;

#[derive(Component)]
pub struct GachaponMultiTray;

#[derive(Component)]
pub struct GachaponRarityParticle {
    pub radius: f32,
    pub base_speed: f32,
    pub phase: f32,
    pub orbit_tilt: f32,
    pub rarity: RarityTier,
    pub elapsed: f32,
}

fn setup_gachapon_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    preloaded: Option<Res<crate::ui::PreloadedAssets>>,
    mut meta: ResMut<MetaProfile>,
    mut ui_state: ResMut<GachaponUiState>,
    mut anim: ResMut<GachaponAnimation>,
    run_state: Res<CurrentRunState>,
) {
    let pre = preloaded.as_deref();
    let srv = Some(&*asset_server);
    let top_bar_tex = crate::ui::load_ui_image(pre, srv, "ui/hud/hud_top_bar.png");
    let btn_gold_tex = crate::ui::load_ui_image(pre, srv, "ui/buttons/btn_gold.png");
    let btn_primary_tex = crate::ui::load_ui_image(pre, srv, "ui/buttons/btn_primary.png");
    let btn_secondary_tex = crate::ui::load_ui_image(pre, srv, "ui/buttons/btn_secondary.png");

    ui_state.selected_target = None;
    ui_state.fragments_slotted = 0;
    ui_state.last_pull_result = None;
    anim.step = GachaponAnimStep::Idle;
    anim.multi_pull_list.clear();
    anim.shake_elapsed = 0.0;
    anim.reveal_elapsed = 0.0;

    // Asegurar algunos fragmentos de prueba si no hay
    if meta.pokemon_fragments.is_empty() {
        meta.pokemon_fragments.insert(7, 3); // Squirtle
        meta.pokemon_fragments.insert(123, 2); // Scyther
    }

    commands
        .spawn((
            CleanupUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.03, 0.04, 0.07)),
        ))
        .with_children(|root| {
            // === BARRA SUPERIOR (`ui/hud/hud_top_bar.png`) ===
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(60.0),
                    position_type: PositionType::Relative,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(18.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(14.0)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.06, 0.09, 0.14)),
                BorderColor(Color::srgb(0.61, 0.30, 1.0)),
            ))
            .with_children(|header| {
                if let Some(ref tex) = top_bar_tex {
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

                header
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        ..default()
                    })
                    .with_children(|title_box| {
                        title_box.spawn((
                            Text::new("TIENDA GACHAPÓN DE SILPH S.A."),
                            TextFont::from_font_size(22.0),
                            TextColor(Color::srgb(0.95, 0.76, 0.30)),
                        ));
                        title_box.spawn((
                            Text::new("Desbloquea nuevos Pokémon iniciales y potencia tus tiradas con Fragmentos de Mini Jefe."),
                            TextFont::from_font_size(12.0),
                            TextColor(Color::srgb(0.75, 0.82, 0.92)),
                        ));
                    });

                // Fichas disponibles
                header
                    .spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(5.0)),
                            border: UiRect::all(Val::Px(1.5)),
                            border_radius: BorderRadius::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.08, 0.12, 0.19, 0.95)),
                        BorderColor(Color::srgb(0.95, 0.76, 0.30)),
                    ))
                    .with_child((
                        GachaponTokensText,
                        Text::new(format!("Fichas: {} Gachapón", meta.gachapon_tokens)),
                        TextFont::from_font_size(18.0),
                        TextColor(Color::srgb(0.95, 0.76, 0.30)),
                    ));

                // Acciones de cabecera
                header
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(10.0),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|actions| {
                        actions
                            .spawn((
                                Button,
                                GachaponToStarterSelectionButton,
                                Node {
                                    min_width: Val::Px(165.0),
                                    height: Val::Px(38.0),
                                    position_type: PositionType::Relative,
                                    padding: UiRect::horizontal(Val::Px(14.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.5)),
                                    border_radius: BorderRadius::all(Val::Px(10.0)),
                                    overflow: Overflow::clip(),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.20, 0.45, 0.65)),
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
                                    Text::new("ELEGIR POKÉMON ->"),
                                    TextFont::from_font_size(12.5),
                                    TextColor(Color::srgb(0.15, 0.09, 0.02)),
                                ));
                            });

                        actions
                            .spawn((
                                Button,
                                GachaponExitButton,
                                Node {
                                    min_width: Val::Px(120.0),
                                    height: Val::Px(38.0),
                                    position_type: PositionType::Relative,
                                    padding: UiRect::horizontal(Val::Px(14.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.5)),
                                    border_radius: BorderRadius::all(Val::Px(10.0)),
                                    overflow: Overflow::clip(),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.14, 0.19, 0.28)),
                                BorderColor(Color::srgb(0.29, 0.69, 1.0)),
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
                                    Text::new("VOLVER ->"),
                                    TextFont::from_font_size(12.5),
                                    TextColor(Color::WHITE),
                                ));
                            });
                    });
            });

            // === CONTENIDO PRINCIPAL: MÁQUINA DE CÁPSULAS Y RECOMPENSA ===
            root.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(78.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(20.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Stretch,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            })
            .with_children(|content| {
                // COLUMNA IZQUIERDA: RANURAS DE FRAGMENTOS (RATE-UP FOCALIZADO)
                content
                    .spawn((
                        Node {
                            width: Val::Percent(42.0),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceBetween,
                            padding: UiRect::all(Val::Px(16.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            border_radius: BorderRadius::all(Val::Px(16.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.06, 0.09, 0.14, 0.94)),
                        BorderColor(Color::srgb(0.61, 0.30, 1.0)),
                    ))
                    .with_children(|frag_col| {
                        frag_col
                            .spawn(Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(6.0),
                                ..default()
                            })
                            .with_children(|f_header| {
                                f_header.spawn((
                                    Text::new("[*] RANURAS DE FRAGMENTOS (RATE-UP)"),
                                    TextFont::from_font_size(16.0),
                                    TextColor(Color::srgb(0.95, 0.76, 0.30)),
                                ));
                                f_header.spawn((
                                    Text::new("Inserta hasta 3 fragmentos para potenciar la probabilidad de conseguir una especie específica:"),
                                    TextFont::from_font_size(11.5),
                                    TextColor(Color::srgb(0.75, 0.82, 0.92)),
                                ));
                            });

                        // Lista de Fragmentos disponibles
                        frag_col
                            .spawn(Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(8.0),
                                margin: UiRect::vertical(Val::Px(10.0)),
                                ..default()
                            })
                            .with_children(|frag_list| {
                                for (&spec_id, &count) in meta.pokemon_fragments.iter() {
                                    let name = get_species_data(spec_id)
                                        .map(|d| d.name)
                                        .unwrap_or_else(|| "Pokémon".to_string());

                                    frag_list
                                        .spawn((
                                            Node {
                                                width: Val::Percent(100.0),
                                                height: Val::Px(42.0),
                                                flex_direction: FlexDirection::Row,
                                                justify_content: JustifyContent::SpaceBetween,
                                                align_items: AlignItems::Center,
                                                padding: UiRect::horizontal(Val::Px(12.0)),
                                                border: UiRect::all(Val::Px(1.5)),
                                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.09, 0.13, 0.20)),
                                            BorderColor(Color::srgb(0.22, 0.32, 0.48)),
                                        ))
                                        .with_children(|row| {
                                            row.spawn((
                                                Text::new(format!(
                                                    "[FRAG] {} (Disponibles: {})",
                                                    name, count
                                                )),
                                                TextFont::from_font_size(13.0),
                                                TextColor(Color::WHITE),
                                            ));

                                            row.spawn((
                                                Button,
                                                GachaponSlotFragmentButton(spec_id),
                                                Node {
                                                    width: Val::Px(108.0),
                                                    height: Val::Px(28.0),
                                                    position_type: PositionType::Relative,
                                                    justify_content: JustifyContent::Center,
                                                    align_items: AlignItems::Center,
                                                    border_radius: BorderRadius::all(Val::Px(6.0)),
                                                    overflow: Overflow::clip(),
                                                    ..default()
                                                },
                                                BackgroundColor(Color::srgb(0.4, 0.25, 0.65)),
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
                                                    Text::new("+1 INSERTAR"),
                                                    TextFont::from_font_size(11.0),
                                                    TextColor(Color::WHITE),
                                                ));
                                            });
                                        });
                                }
                            });

                        // Indicador de Probabilidad en Tiempo Real
                        frag_col
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::all(Val::Px(10.0)),
                                    border: UiRect::all(Val::Px(1.5)),
                                    border_radius: BorderRadius::all(Val::Px(8.0)),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(4.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.09, 0.13, 0.20)),
                                BorderColor(Color::srgb(0.61, 0.30, 1.0)),
                            ))
                            .with_children(|odds_box| {
                                odds_box.spawn((
                                    GachaponOddsText,
                                    Text::new("[RATE] Probabilidad Focalizada: 0 Fragmentos insertados (Tirada General: 12% Raro, 4.5% Exótico)."),
                                    TextFont::from_font_size(12.0),
                                    TextColor(Color::srgb(0.95, 0.76, 0.30)),
                                ));
                            });

                        // Botones de Tirada (`btn_primary.png` y `btn_gold.png`)
                        frag_col
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(12.0),
                                justify_content: JustifyContent::SpaceBetween,
                                ..default()
                            })
                            .with_children(|pull_buttons| {
                                pull_buttons
                                    .spawn((
                                        Button,
                                        GachaponSinglePullButton,
                                        Node {
                                            width: Val::Percent(48.0),
                                            height: Val::Px(46.0),
                                            position_type: PositionType::Relative,
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(2.0)),
                                            border_radius: BorderRadius::all(Val::Px(10.0)),
                                            overflow: Overflow::clip(),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.85, 0.38, 0.16)),
                                        BorderColor(Color::srgb(1.0, 0.48, 0.24)),
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
                                            Text::new("TIRADA SIMPLE (100)"),
                                            TextFont::from_font_size(12.0),
                                            TextColor(Color::WHITE),
                                        ));
                                    });

                                pull_buttons
                                    .spawn((
                                        Button,
                                        GachaponMultiPullButton,
                                        Node {
                                            width: Val::Percent(48.0),
                                            height: Val::Px(46.0),
                                            position_type: PositionType::Relative,
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(2.0)),
                                            border_radius: BorderRadius::all(Val::Px(10.0)),
                                            overflow: Overflow::clip(),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.65, 0.45, 0.15)),
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
                                            Text::new("PACK x10 (900)"),
                                            TextFont::from_font_size(12.0),
                                            TextColor(Color::srgb(0.15, 0.09, 0.02)),
                                        ));
                                    });
                            });
                    });

                // COLUMNA DERECHA: APARADOR DE RECOMPENSA Y ANIMACIÓN ESPECTACULAR
                content
                    .spawn((
                        GachaponRewardCard,
                        Node {
                            width: Val::Percent(54.0),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(16.0)),
                            border: UiRect::all(Val::Px(2.5)),
                            border_radius: BorderRadius::all(Val::Px(16.0)),
                            ..default()
                        },
                        UiTransform::default(),
                        BackgroundColor(Color::srgba(0.06, 0.09, 0.14, 0.94)),
                        BorderColor(Color::srgb(0.95, 0.76, 0.30)),
                    ))
                    .with_children(|reward_panel| {
                        // Encabezado dinámico de la animación
                        reward_panel.spawn((
                            GachaponRewardHeader,
                            Text::new("[*] CÁPSULA OBTENIDA"),
                            TextFont::from_font_size(17.0),
                            TextColor(Color::srgb(0.95, 0.76, 0.30)),
                        ));

                        // Escenario de Animación de la Cápsula y el Pokémon
                        reward_panel
                            .spawn((
                                GachaponStageArea,
                                Node {
                                    width: Val::Px(300.0),
                                    height: Val::Px(155.0),
                                    position_type: PositionType::Relative,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    overflow: Overflow::clip(),
                                    ..default()
                                },
                            ))
                            .with_children(|stage| {
                                // Resplandor de Aura tras la cápsula o Pokémon
                                stage.spawn((
                                    GachaponAuraGlow,
                                    Node {
                                        width: Val::Px(140.0),
                                        height: Val::Px(140.0),
                                        border_radius: BorderRadius::all(Val::Px(70.0)),
                                        position_type: PositionType::Absolute,
                                        ..default()
                                    },
                                    UiTransform::default(),
                                    BackgroundColor(Color::srgba(0.8, 0.5, 1.0, 0.15)),
                                    Visibility::Inherited,
                                ));

                                // La Cápsula Pokéball animada (visible al girar)
                                stage
                                    .spawn((
                                        GachaponCapsuleOrb,
                                        Node {
                                            width: Val::Px(84.0),
                                            height: Val::Px(84.0),
                                            border_radius: BorderRadius::all(Val::Px(42.0)),
                                            border: UiRect::all(Val::Px(3.0)),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::SpaceBetween,
                                            align_items: AlignItems::Center,
                                            overflow: Overflow::clip(),
                                            position_type: PositionType::Absolute,
                                            ..default()
                                        },
                                        UiTransform::default(),
                                        BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
                                        BorderColor(Color::srgb(0.3, 0.3, 0.4)),
                                        Visibility::Inherited,
                                    ))
                                    .with_children(|ball| {
                                        // Mitad superior roja / coloreada
                                        ball.spawn((
                                            GachaponCapsuleTopHalf,
                                            Node {
                                                width: Val::Percent(100.0),
                                                height: Val::Px(39.0),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.85, 0.22, 0.22)),
                                        ));
                                        // Junta central con botón
                                        ball.spawn((
                                            Node {
                                                width: Val::Percent(100.0),
                                                height: Val::Px(6.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                                        ))
                                        .with_child((
                                            GachaponCapsuleButton,
                                            Node {
                                                width: Val::Px(16.0),
                                                height: Val::Px(16.0),
                                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                                border: UiRect::all(Val::Px(2.0)),
                                                margin: UiRect::all(Val::Px(-5.0)),
                                                ..default()
                                            },
                                            BackgroundColor(Color::WHITE),
                                            BorderColor(Color::srgb(0.2, 0.2, 0.25)),
                                        ));
                                        // Mitad inferior blanca
                                        ball.spawn(Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(39.0),
                                            ..default()
                                        })
                                        .insert(BackgroundColor(Color::srgb(0.92, 0.92, 0.96)));
                                    });

                                // Sprite de Recompensa del Pokémon (inicialmente oculto)
                                let placeholder_texture =
                                    asset_server.load("sprites/pokemon/front/4.png");
                                stage.spawn((
                                    GachaponRewardSprite,
                                    ImageNode::new(placeholder_texture),
                                    Node {
                                        width: Val::Px(130.0),
                                        height: Val::Px(130.0),
                                        position_type: PositionType::Absolute,
                                        ..default()
                                    },
                                    UiTransform::default(),
                                    Visibility::Hidden,
                                ));
                            });

                        // Insignia de Rareza y Botón de Saltar
                        reward_panel
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                align_items: AlignItems::Center,
                                ..default()
                            })
                            .with_children(|badge_row| {
                                badge_row.spawn((
                                    GachaponRarityBadge,
                                    Text::new("CÁPSULA LISTA"),
                                    TextFont::from_font_size(12.5),
                                    TextColor(Color::srgb(0.7, 0.75, 0.85)),
                                ));

                                badge_row
                                    .spawn((
                                        Button,
                                        GachaponSkipButton,
                                        Node {
                                            padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                                            border: UiRect::all(Val::Px(1.0)),
                                            border_radius: BorderRadius::all(Val::Px(4.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.25, 0.22, 0.32)),
                                        BorderColor(Color::srgb(0.5, 0.45, 0.65)),
                                        Visibility::Hidden,
                                    ))
                                    .with_child((
                                        Text::new("SALTAR >>"),
                                        TextFont::from_font_size(10.0),
                                        TextColor(Color::srgb(0.9, 0.85, 1.0)),
                                    ));
                            });

                        // Nombre y Detalle del Pokémon obtenido
                        reward_panel.spawn((
                            GachaponRewardName,
                            Text::new("¡Gira la palanca para revelar tu Pokémon!"),
                            TextFont::from_font_size(15.0),
                            TextColor(Color::WHITE),
                        ));

                        // Bandeja de resultados para el Pack x10
                        reward_panel.spawn((
                            GachaponMultiTray,
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: Val::Px(6.0),
                                row_gap: Val::Px(4.0),
                                justify_content: JustifyContent::Center,
                                min_height: Val::Px(20.0),
                                ..default()
                            },
                        ));

                        // Colección Desbloqueada
                        reward_panel
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(6.0),
                                    padding: UiRect::all(Val::Px(10.0)),
                                    border: UiRect::all(Val::Px(1.5)),
                                    border_radius: BorderRadius::all(Val::Px(10.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.09, 0.13, 0.20)),
                                BorderColor(Color::srgb(0.22, 0.32, 0.48)),
                            ))
                            .with_children(|col_box| {
                                col_box.spawn((
                                    Text::new(format!(
                                        "[COLECCIÓN] Tu Colección Desbloqueada ({} Pokémon) - Haz clic para seleccionar:",
                                        meta.unlocked_pokemon.len()
                                    )),
                                    TextFont::from_font_size(13.0),
                                    TextColor(Color::srgb(0.75, 0.84, 0.96)),
                                ));

                                col_box
                                    .spawn(Node {
                                        flex_direction: FlexDirection::Row,
                                        column_gap: Val::Px(8.0),
                                        row_gap: Val::Px(8.0),
                                        flex_wrap: FlexWrap::Wrap,
                                        ..default()
                                    })
                                    .with_children(|icons_row| {
                                        for &id in meta.unlocked_pokemon.iter() {
                                            let name = get_species_data(id)
                                                .map(|d| d.name)
                                                .unwrap_or_else(|| "?".to_string());
                                            let is_active = run_state.selected_starter_id == id;
                                            let border_color = if is_active {
                                                Color::srgb(0.95, 0.76, 0.30)
                                            } else {
                                                Color::srgb(0.29, 0.69, 1.0)
                                            };
                                            let bg_color = if is_active {
                                                Color::srgb(0.35, 0.28, 0.15)
                                            } else {
                                                Color::srgb(0.14, 0.19, 0.28)
                                            };
                                            let label = if is_active {
                                                format!("#{} {} [ACTIVO]", id, name)
                                            } else {
                                                format!("#{} {}", id, name)
                                            };

                                            icons_row
                                                .spawn((
                                                    Button,
                                                    GachaponSelectPokemonBadgeButton(id),
                                                    Node {
                                                        padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                                                        border: UiRect::all(Val::Px(1.5)),
                                                        border_radius: BorderRadius::all(Val::Px(6.0)),
                                                        ..default()
                                                    },
                                                    BackgroundColor(bg_color),
                                                    BorderColor(border_color),
                                                ))
                                                .with_child((
                                                    Text::new(label),
                                                    TextFont::from_font_size(11.0),
                                                    TextColor(if is_active {
                                                        Color::srgb(0.95, 0.76, 0.30)
                                                    } else {
                                                        Color::WHITE
                                                    }),
                                                ));
                                        }
                                    });
                            });
                    });
            });

            // === BARRA INFERIOR DE MENSAJES ===
            root.spawn((
                GachaponStatusText,
                Text::new("[INFO] Las fichas gachapón se ganan superando pisos, Mini Jefes y Líderes de Gimnasio en cada run."),
                TextFont::from_font_size(13.0),
                TextColor(Color::srgb(0.68, 0.76, 0.88)),
            ));
        });
}

fn handle_slot_fragment(
    mut q_buttons: Query<(&Interaction, &GachaponSlotFragmentButton), Changed<Interaction>>,
    meta: Res<MetaProfile>,
    mut ui_state: ResMut<GachaponUiState>,
    mut q_odds: Query<&mut Text, With<GachaponOddsText>>,
    mut q_status: Query<&mut Text, (With<GachaponStatusText>, Without<GachaponOddsText>)>,
) {
    for (interaction, btn) in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            let spec_id = btn.0;
            let available = meta.pokemon_fragments.get(&spec_id).copied().unwrap_or(0);

            if available == 0 {
                if let Ok(mut text) = q_status.single_mut() {
                    **text = "[!] No tienes más fragmentos de esta especie.".to_string();
                }
                continue;
            }

            if ui_state.selected_target != Some(spec_id) {
                ui_state.selected_target = Some(spec_id);
                ui_state.fragments_slotted = 1;
            } else if ui_state.fragments_slotted < 3 && ui_state.fragments_slotted < available {
                ui_state.fragments_slotted += 1;
            } else {
                ui_state.fragments_slotted = 1;
            }

            let name = get_species_data(spec_id).map(|d| d.name).unwrap_or_else(|| "Pokémon".to_string());
            let odds = match ui_state.fragments_slotted {
                1 => 20.0,
                2 => 35.0,
                3 => 55.0,
                _ => 12.0,
            };

            if let Ok(mut text) = q_odds.single_mut() {
                **text = format!(
                    "Rate-Up Focalizado en {}: {}/3 Fragmentos insertados (¡{:.0}% probabilidad directa!)",
                    name, ui_state.fragments_slotted, odds
                );
            }

            if let Ok(mut text) = q_status.single_mut() {
                **text = format!("[RATE-UP] ¡Fragmento de {} insertado en la ranura de amplificación!", name);
            }
        }
    }
}

fn rarity_shake_duration(rarity: RarityTier) -> f32 {
    match rarity {
        RarityTier::Common => 0.65,
        RarityTier::Uncommon => 0.85,
        RarityTier::Rare => 1.15,
        RarityTier::Exotic => 1.50,
        RarityTier::Legendary => 1.95,
    }
}

fn rarity_shake_amplitude(rarity: RarityTier) -> f32 {
    match rarity {
        RarityTier::Common => 3.5,
        RarityTier::Uncommon => 6.0,
        RarityTier::Rare => 9.0,
        RarityTier::Exotic => 13.0,
        RarityTier::Legendary => 18.0,
    }
}

fn rarity_theme_color(rarity: RarityTier) -> Color {
    match rarity {
        RarityTier::Common => Color::srgb(0.75, 0.80, 0.90),
        RarityTier::Uncommon => Color::srgb(0.28, 0.92, 0.48),
        RarityTier::Rare => Color::srgb(0.30, 0.75, 1.0),
        RarityTier::Exotic => Color::srgb(0.92, 0.38, 1.0),
        RarityTier::Legendary => Color::srgb(1.0, 0.88, 0.25),
    }
}

fn rarity_aura_color(rarity: RarityTier) -> Color {
    match rarity {
        RarityTier::Common => Color::srgba(0.70, 0.75, 0.85, 0.35),
        RarityTier::Uncommon => Color::srgba(0.25, 0.90, 0.45, 0.45),
        RarityTier::Rare => Color::srgba(0.25, 0.68, 1.0, 0.60),
        RarityTier::Exotic => Color::srgba(0.88, 0.30, 0.95, 0.75),
        RarityTier::Legendary => Color::srgba(1.0, 0.85, 0.20, 0.90),
    }
}

fn rarity_particle_count(rarity: RarityTier) -> usize {
    match rarity {
        RarityTier::Common => 8,
        RarityTier::Uncommon => 14,
        RarityTier::Rare => 22,
        RarityTier::Exotic => 32,
        RarityTier::Legendary => 44,
    }
}

fn rarity_symbols(rarity: RarityTier) -> &'static [&'static str] {
    match rarity {
        RarityTier::Common => &["*", "+", "."],
        RarityTier::Uncommon => &["✦", "+", "*"],
        RarityTier::Rare => &["★", "✦", "✧"],
        RarityTier::Exotic => &["★", "✦", "◆", "✧"],
        RarityTier::Legendary => &["★", "✹", "✦", "✶", "◆"],
    }
}

fn spawn_rarity_particles(
    commands: &mut Commands,
    stage_entity: Entity,
    rarity: RarityTier,
) {
    let count = rarity_particle_count(rarity);
    let symbols = rarity_symbols(rarity);
    let color = rarity_theme_color(rarity);

    commands.entity(stage_entity).with_children(|stage| {
        for i in 0..count {
            let frac = i as f32 / count as f32;
            let phase = frac * std::f32::consts::TAU;
            let radius = match rarity {
                RarityTier::Common => 38.0 + (i % 3) as f32 * 6.0,
                RarityTier::Uncommon => 44.0 + (i % 4) as f32 * 8.0,
                RarityTier::Rare => 50.0 + (i % 5) as f32 * 10.0,
                RarityTier::Exotic => 56.0 + (i % 6) as f32 * 12.0,
                RarityTier::Legendary => 62.0 + (i % 7) as f32 * 14.0,
            };
            let speed = match rarity {
                RarityTier::Common => 1.8 + (i % 2) as f32 * 0.4,
                RarityTier::Uncommon => 2.2 + (i % 3) as f32 * 0.5,
                RarityTier::Rare => 2.8 + (i % 3) as f32 * 0.7,
                RarityTier::Exotic => 3.4 + (i % 4) as f32 * 0.8,
                RarityTier::Legendary => 4.2 + (i % 5) as f32 * 1.0,
            };
            let tilt = ((i as f32 * 1.37) % 3.14) - 1.57;
            let symbol = symbols[i % symbols.len()];
            let font_size = match rarity {
                RarityTier::Common => 11.0,
                RarityTier::Uncommon => 13.0,
                RarityTier::Rare => 15.0,
                RarityTier::Exotic => 18.0,
                RarityTier::Legendary => 22.0,
            };

            stage.spawn((
                GachaponRarityParticle {
                    radius,
                    base_speed: speed,
                    phase,
                    orbit_tilt: tilt,
                    rarity,
                    elapsed: 0.0,
                },
                Text::new(symbol),
                TextFont::from_font_size(font_size),
                TextColor(color),
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                UiTransform::default(),
            ));
        }
    });
}

fn populate_multi_tray(
    commands: &mut Commands,
    q_tray: &Query<(Entity, Option<&Children>), With<GachaponMultiTray>>,
    pull_list: &[u32],
) {
    if let Ok((tray_entity, maybe_children)) = q_tray.single() {
        if let Some(children) = maybe_children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }

        commands.entity(tray_entity).with_children(|tray| {
            for &id in pull_list.iter() {
                let rarity = species_rarity(id);
                let name = get_species_data(id).map(|d| d.name).unwrap_or_else(|| "Pokémon".to_string());
                let border_c = rarity_theme_color(rarity);

                tray.spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(6.0), Val::Px(2.5)),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        margin: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.14, 0.12, 0.20)),
                    BorderColor(border_c),
                )).with_child((
                    Text::new(format!("#{} {}", id, name)),
                    TextFont::from_font_size(10.5),
                    TextColor(border_c),
                ));
            }
        });
    }
}

type GachaponVisQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Visibility,
        Option<&'static GachaponCapsuleOrb>,
        Option<&'static GachaponRewardSprite>,
        Option<&'static GachaponSkipButton>,
    ),
>;

type GachaponTextQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Text,
        Option<&'static mut TextColor>,
        Option<&'static GachaponRewardHeader>,
        Option<&'static GachaponRewardName>,
        Option<&'static GachaponRarityBadge>,
        Option<&'static GachaponStatusText>,
    ),
>;

fn apply_revealed_ui(
    anim: &GachaponAnimation,
    q_texts: &mut GachaponTextQuery,
) {
    let name = get_species_data(anim.pulled_pokemon)
        .map(|d| d.name)
        .unwrap_or_else(|| "Pokémon".to_string());
    let theme_c = rarity_theme_color(anim.rarity);

    for (mut text, mut opt_color, is_header, is_name, is_badge, is_status) in q_texts.iter_mut() {
        if is_header.is_some() {
            **text = match anim.rarity {
                RarityTier::Legendary => "¡¡¡¡¡APARICIÓN LEGENDARIA DESLUMBRANTE!!!!!".to_string(),
                RarityTier::Exotic => "¡¡¡APARICIÓN EXÓTICA EXTRAORDINARIA!!!".to_string(),
                RarityTier::Rare => "¡¡APARICIÓN RARA BRILLANTE!!".to_string(),
                RarityTier::Uncommon => "¡Aparición Poco Común!".to_string(),
                RarityTier::Common => "Aparición Común".to_string(),
            };
        } else if is_name.is_some() {
            if anim.multi_pull_list.is_empty() {
                let tag = if anim.is_new { "¡NUEVO POKÉMON!" } else { "¡POKÉMON OBTENIDO!" };
                **text = format!("{} #{} {} ({})", tag, anim.pulled_pokemon, name, anim.rarity.name());
            } else {
                **text = format!("★ Pack x10 destacado: #{} {} ({}) ★", anim.pulled_pokemon, name, anim.rarity.name());
            }
        } else if is_badge.is_some() {
            **text = format!("★ RAREZA: {} ★", anim.rarity.name().to_uppercase());
            if let Some(ref mut c) = opt_color {
                c.0 = theme_c;
            }
        } else if is_status.is_some() {
            **text = format!("[GACHAPÓN] ¡Has obtenido a {} ({})!", name, anim.rarity.name());
        }
    }
}

fn handle_single_pull(
    mut q_buttons: Query<&Interaction, (Changed<Interaction>, With<GachaponSinglePullButton>)>,
    mut meta: ResMut<MetaProfile>,
    mut ui_state: ResMut<GachaponUiState>,
    mut anim: ResMut<GachaponAnimation>,
    mut commands: Commands,
    q_particles: Query<Entity, With<GachaponRarityParticle>>,
    q_tray: Query<(Entity, Option<&Children>), With<GachaponMultiTray>>,
    mut q_vis: GachaponVisQuery,
    mut q_texts: GachaponTextQuery,
) {
    for interaction in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            if anim.step == GachaponAnimStep::Shaking || anim.step == GachaponAnimStep::Burst {
                return;
            }

            if meta.gachapon_tokens < 100 {
                for (mut text, _, _, _, _, is_status) in q_texts.iter_mut() {
                    if is_status.is_some() {
                        **text = "[!] ¡Fichas insuficientes! Necesitas 100 Fichas Gachapón para una tirada.".to_string();
                    }
                }
                return;
            }

            let was_unlocked_before = |id: u32, list: &[u32]| list.contains(&id);
            let target = ui_state.selected_target;
            let frags = ui_state.fragments_slotted;
            let prev_unlocked = meta.unlocked_pokemon.clone();

            if let Some(pulled_id) = pull_gachapon(&mut meta, target, frags) {
                let is_new = !was_unlocked_before(pulled_id, &prev_unlocked);
                let rarity = species_rarity(pulled_id);

                ui_state.last_pull_result = Some(pulled_id);
                ui_state.last_pull_is_new = is_new;
                ui_state.fragments_slotted = 0;
                ui_state.selected_target = None;

                let shake_dur = rarity_shake_duration(rarity);
                anim.step = GachaponAnimStep::Shaking;
                anim.timer = Timer::from_seconds(shake_dur, TimerMode::Once);
                anim.pulled_pokemon = pulled_id;
                anim.rarity = rarity;
                anim.is_new = is_new;
                anim.multi_pull_list.clear();
                anim.shake_elapsed = 0.0;
                anim.reveal_elapsed = 0.0;

                for p_entity in q_particles.iter() {
                    commands.entity(p_entity).despawn();
                }
                if let Ok((_tray_entity, maybe_children)) = q_tray.single() {
                    if let Some(children) = maybe_children {
                        for child in children.iter() {
                            commands.entity(child).despawn();
                        }
                    }
                }

                for (mut vis, is_capsule, is_sprite, is_skip) in q_vis.iter_mut() {
                    if is_capsule.is_some() {
                        *vis = Visibility::Inherited;
                    } else if is_sprite.is_some() {
                        *vis = Visibility::Hidden;
                    } else if is_skip.is_some() {
                        *vis = Visibility::Inherited;
                    }
                }

                for (mut text, mut opt_color, is_header, is_name, is_badge, is_status) in q_texts.iter_mut() {
                    if is_header.is_some() {
                        **text = "¡Girando mecanismo gachapón...!".to_string();
                    } else if is_name.is_some() {
                        **text = "Sintiendo la pulsación de la cápsula...".to_string();
                    } else if is_badge.is_some() {
                        **text = "★ SINTONIZANDO ENERGÍA... ★".to_string();
                        if let Some(ref mut c) = opt_color {
                            c.0 = Color::srgb(0.70, 0.75, 0.85);
                        }
                    } else if is_status.is_some() {
                        **text = "[GACHAPÓN] ¡Tirada en curso! La cápsula vibra con intensa energía...".to_string();
                    }
                }
            }
        }
    }
}

fn handle_multi_pull(
    mut q_buttons: Query<&Interaction, (Changed<Interaction>, With<GachaponMultiPullButton>)>,
    mut meta: ResMut<MetaProfile>,
    mut ui_state: ResMut<GachaponUiState>,
    mut anim: ResMut<GachaponAnimation>,
    mut commands: Commands,
    q_particles: Query<Entity, With<GachaponRarityParticle>>,
    q_tray: Query<(Entity, Option<&Children>), With<GachaponMultiTray>>,
    mut q_vis: GachaponVisQuery,
    mut q_texts: GachaponTextQuery,
) {
    for interaction in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            if anim.step == GachaponAnimStep::Shaking || anim.step == GachaponAnimStep::Burst {
                return;
            }

            if meta.gachapon_tokens < 900 {
                for (mut text, _, _, _, _, is_status) in q_texts.iter_mut() {
                    if is_status.is_some() {
                        **text = "[AVISO] ¡Fichas insuficientes! Se requieren 900 Fichas para el Pack x10.".to_string();
                    }
                }
                return;
            }

            meta.gachapon_tokens -= 900;
            let prev_unlocked = meta.unlocked_pokemon.clone();
            let mut pulled_ids = Vec::with_capacity(10);

            for i in 0..10 {
                meta.gachapon_tokens += 100;
                let target = if i == 0 { ui_state.selected_target } else { None };
                let frags = if i == 0 { ui_state.fragments_slotted } else { 0 };

                if let Some(id) = pull_gachapon(&mut meta, target, frags) {
                    pulled_ids.push(id);
                }
            }

            let best_id = *pulled_ids.iter().max_by_key(|&&id| (species_rarity(id), id)).unwrap_or(&4);
            let best_rarity = species_rarity(best_id);
            let is_new = !prev_unlocked.contains(&best_id);

            ui_state.last_pull_result = Some(best_id);
            ui_state.last_pull_is_new = is_new;
            ui_state.fragments_slotted = 0;
            ui_state.selected_target = None;

            let shake_dur = rarity_shake_duration(best_rarity) + 0.25;
            anim.step = GachaponAnimStep::Shaking;
            anim.timer = Timer::from_seconds(shake_dur, TimerMode::Once);
            anim.pulled_pokemon = best_id;
            anim.rarity = best_rarity;
            anim.is_new = is_new;
            anim.multi_pull_list = pulled_ids;
            anim.shake_elapsed = 0.0;
            anim.reveal_elapsed = 0.0;

            for p_entity in q_particles.iter() {
                commands.entity(p_entity).despawn();
            }
            if let Ok((_tray_entity, maybe_children)) = q_tray.single() {
                if let Some(children) = maybe_children {
                    for child in children.iter() {
                        commands.entity(child).despawn();
                    }
                }
            }

            for (mut vis, is_capsule, is_sprite, is_skip) in q_vis.iter_mut() {
                if is_capsule.is_some() {
                    *vis = Visibility::Inherited;
                } else if is_sprite.is_some() {
                    *vis = Visibility::Hidden;
                } else if is_skip.is_some() {
                    *vis = Visibility::Inherited;
                }
            }

            for (mut text, mut opt_color, is_header, is_name, is_badge, is_status) in q_texts.iter_mut() {
                if is_header.is_some() {
                    **text = "¡Abriendo Pack x10 de Cápsulas...!".to_string();
                } else if is_name.is_some() {
                    **text = "Buscando la cápsula con mayor resonancia...".to_string();
                } else if is_badge.is_some() {
                    **text = "★ MULTI-TIRADA EN PROCESO ★".to_string();
                    if let Some(ref mut c) = opt_color {
                        c.0 = Color::srgb(0.85, 0.60, 1.0);
                    }
                } else if is_status.is_some() {
                    **text = "[PACK x10] ¡Procesando 10 cápsulas consecutivas!".to_string();
                }
            }
        }
    }
}

fn handle_skip_gachapon_animation(
    mut q_buttons: Query<&Interaction, (Changed<Interaction>, With<GachaponSkipButton>)>,
    mut anim: ResMut<GachaponAnimation>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    q_stage: Query<Entity, With<GachaponStageArea>>,
    q_particles: Query<Entity, With<GachaponRarityParticle>>,
    q_tray: Query<(Entity, Option<&Children>), With<GachaponMultiTray>>,
    mut q_vis: GachaponVisQuery,
    mut q_sprite_img: Query<&mut ImageNode, With<GachaponRewardSprite>>,
    mut q_aura: Query<&mut BackgroundColor, With<GachaponAuraGlow>>,
    mut q_texts: GachaponTextQuery,
) {
    for interaction in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            if anim.step == GachaponAnimStep::Shaking || anim.step == GachaponAnimStep::Burst {
                anim.step = GachaponAnimStep::Revealed;
                anim.reveal_elapsed = 0.0;

                for (mut vis, is_capsule, is_sprite, is_skip) in q_vis.iter_mut() {
                    if is_capsule.is_some() {
                        *vis = Visibility::Hidden;
                    } else if is_sprite.is_some() {
                        *vis = Visibility::Inherited;
                    } else if is_skip.is_some() {
                        *vis = Visibility::Hidden;
                    }
                }

                if let Ok(mut img) = q_sprite_img.single_mut() {
                    img.image = asset_server.load(format!("sprites/pokemon/front/{}.png", anim.pulled_pokemon));
                }
                if let Ok(mut aura) = q_aura.single_mut() {
                    aura.0 = rarity_aura_color(anim.rarity);
                }

                for p_entity in q_particles.iter() {
                    commands.entity(p_entity).despawn();
                }
                if let Ok(stage_ent) = q_stage.single() {
                    spawn_rarity_particles(&mut commands, stage_ent, anim.rarity);
                }

                apply_revealed_ui(&anim, &mut q_texts);

                if !anim.multi_pull_list.is_empty() {
                    populate_multi_tray(&mut commands, &q_tray, &anim.multi_pull_list);
                }
            }
        }
    }
}

fn update_gachapon_animation(
    time: Res<Time>,
    mut anim: ResMut<GachaponAnimation>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    q_stage: Query<Entity, With<GachaponStageArea>>,
    q_particles: Query<Entity, With<GachaponRarityParticle>>,
    q_tray: Query<(Entity, Option<&Children>), With<GachaponMultiTray>>,
    mut q_vis: GachaponVisQuery,
    mut q_sprite_img: Query<&mut ImageNode, With<GachaponRewardSprite>>,
    mut q_aura: Query<&mut BackgroundColor, With<GachaponAuraGlow>>,
    mut q_texts: GachaponTextQuery,
) {
    let dt = time.delta_secs();

    match anim.step {
        GachaponAnimStep::Idle => {}
        GachaponAnimStep::Shaking => {
            anim.shake_elapsed += dt;
            anim.timer.tick(time.delta());

            if anim.timer.just_finished() {
                anim.step = GachaponAnimStep::Burst;
                anim.timer = Timer::from_seconds(0.35, TimerMode::Once);

                for (mut vis, is_capsule, is_sprite, _) in q_vis.iter_mut() {
                    if is_capsule.is_some() {
                        *vis = Visibility::Hidden;
                    } else if is_sprite.is_some() {
                        *vis = Visibility::Inherited;
                    }
                }

                if let Ok(mut img) = q_sprite_img.single_mut() {
                    img.image = asset_server.load(format!("sprites/pokemon/front/{}.png", anim.pulled_pokemon));
                }
                if let Ok(mut aura) = q_aura.single_mut() {
                    aura.0 = rarity_aura_color(anim.rarity);
                }

                for p_entity in q_particles.iter() {
                    commands.entity(p_entity).despawn();
                }
                if let Ok(stage_ent) = q_stage.single() {
                    spawn_rarity_particles(&mut commands, stage_ent, anim.rarity);
                }

                for (mut text, mut opt_color, is_header, _, is_badge, _) in q_texts.iter_mut() {
                    if is_header.is_some() {
                        **text = "¡¡BURST!! ¡¡CÁPSULA ABIERTA!!".to_string();
                    } else if is_badge.is_some() {
                        **text = format!("★ RAREZA DETECTADA: {} ★", anim.rarity.name().to_uppercase());
                        if let Some(ref mut c) = opt_color {
                            c.0 = rarity_theme_color(anim.rarity);
                        }
                    }
                }
            }
        }
        GachaponAnimStep::Burst => {
            anim.timer.tick(time.delta());

            if anim.timer.just_finished() {
                anim.step = GachaponAnimStep::Revealed;
                anim.reveal_elapsed = 0.0;

                for (mut vis, _, _, is_skip) in q_vis.iter_mut() {
                    if is_skip.is_some() {
                        *vis = Visibility::Hidden;
                    }
                }

                apply_revealed_ui(&anim, &mut q_texts);

                if !anim.multi_pull_list.is_empty() {
                    populate_multi_tray(&mut commands, &q_tray, &anim.multi_pull_list);
                }
            }
        }
        GachaponAnimStep::Revealed => {
            anim.reveal_elapsed += dt;
        }
    }
}

fn animate_gachapon_particles(
    time: Res<Time>,
    anim: Res<GachaponAnimation>,
    mut q_particles: Query<(&mut GachaponRarityParticle, &mut UiTransform, &mut TextColor)>,
) {
    if anim.step != GachaponAnimStep::Burst && anim.step != GachaponAnimStep::Revealed {
        return;
    }
    let dt = time.delta_secs().min(0.05);
    for (mut part, mut transform, mut text_color) in q_particles.iter_mut() {
        part.elapsed += dt;
        let t = part.elapsed * part.base_speed + part.phase;

        match part.rarity {
            RarityTier::Common => {
                let x = t.cos() * part.radius;
                let y = t.sin() * (part.radius * 0.75);
                transform.translation = Val2::px(x, y);
            }
            RarityTier::Uncommon => {
                let r = part.radius + (part.elapsed * 12.0).min(20.0);
                let x = t.cos() * r;
                let y = t.sin() * (r * 0.7) - (part.elapsed * 14.0).sin() * 8.0;
                transform.translation = Val2::px(x, y);
            }
            RarityTier::Rare => {
                let dir = if (part.phase * 10.0) as i32 % 2 == 0 { 1.0 } else { -1.0 };
                let t_dir = part.elapsed * part.base_speed * dir + part.phase;
                let x = t_dir.cos() * (part.radius + 8.0);
                let y = t_dir.sin() * (part.radius * 0.55);
                transform.translation = Val2::px(x, y);
                transform.scale = Vec2::splat(1.0 + 0.2 * (t * 2.0).sin());
            }
            RarityTier::Exotic => {
                let r_dyn = part.radius + (part.elapsed * 4.5).sin() * 16.0;
                let x = t.cos() * r_dyn;
                let y = t.sin() * (r_dyn * 0.7) + (t * 2.5).cos() * 8.0;
                transform.translation = Val2::px(x, y);
                transform.scale = Vec2::splat(1.0 + 0.3 * (t * 3.0).cos());
                transform.rotation = Rot2::radians(t * 0.5);
            }
            RarityTier::Legendary => {
                let r_dyn = part.radius + (part.elapsed * 6.0).sin() * 22.0;
                let x = t.cos() * r_dyn;
                let y = (t + part.orbit_tilt).sin() * (r_dyn * 0.65) + (t * 3.5).sin() * 10.0;
                transform.translation = Val2::px(x, y);
                let scale_pulse = 1.0 + 0.45 * (t * 4.0).sin().abs();
                transform.scale = Vec2::splat(scale_pulse);
                transform.rotation = Rot2::radians(t * 1.2);

                let base_color = rarity_theme_color(RarityTier::Legendary);
                let alpha = 0.55 + 0.45 * (t * 5.0).sin().abs();
                text_color.0 = base_color.with_alpha(alpha);
            }
        }
    }
}

fn animate_gachapon_sprite_and_capsule(
    anim: Res<GachaponAnimation>,
    mut q_card: Query<&mut UiTransform, (With<GachaponRewardCard>, Without<GachaponCapsuleOrb>, Without<GachaponRewardSprite>, Without<GachaponAuraGlow>)>,
    mut q_capsule: Query<&mut UiTransform, (With<GachaponCapsuleOrb>, Without<GachaponRewardCard>, Without<GachaponRewardSprite>, Without<GachaponAuraGlow>)>,
    mut q_sprite: Query<&mut UiTransform, (With<GachaponRewardSprite>, Without<GachaponRewardCard>, Without<GachaponCapsuleOrb>, Without<GachaponAuraGlow>)>,
    mut q_aura: Query<&mut UiTransform, (With<GachaponAuraGlow>, Without<GachaponRewardCard>, Without<GachaponCapsuleOrb>, Without<GachaponRewardSprite>)>,
) {
    match anim.step {
        GachaponAnimStep::Idle => {
            if let Ok(mut t) = q_capsule.single_mut() {
                if t.scale != Vec2::ONE || t.translation != Val2::px(0.0, 0.0) {
                    t.translation = Val2::px(0.0, 0.0);
                    t.scale = Vec2::ONE;
                    t.rotation = Rot2::radians(0.0);
                }
            }
            if let Ok(mut t) = q_card.single_mut() {
                if t.translation != Val2::px(0.0, 0.0) {
                    t.translation = Val2::px(0.0, 0.0);
                }
            }
        }
        GachaponAnimStep::Shaking => {
            let amp = rarity_shake_amplitude(anim.rarity);
            let freq = match anim.rarity {
                RarityTier::Common => 22.0,
                RarityTier::Uncommon => 26.0,
                RarityTier::Rare => 32.0,
                RarityTier::Exotic => 38.0,
                RarityTier::Legendary => 46.0,
            };
            let shake_t = anim.shake_elapsed * freq;
            let offset_x = shake_t.sin() * amp;
            let offset_y = (shake_t * 1.4).cos() * (amp * 0.6);
            let rot = (shake_t * 0.8).sin() * (0.04 * amp);

            if let Ok(mut t) = q_capsule.single_mut() {
                t.translation = Val2::px(offset_x, offset_y);
                t.scale = Vec2::splat(1.0 + 0.05 * (shake_t * 0.5).sin().abs());
                t.rotation = Rot2::radians(rot);
            }

            if anim.rarity >= RarityTier::Exotic {
                if let Ok(mut t) = q_card.single_mut() {
                    let card_shake = (shake_t * 0.7).sin() * 2.5;
                    t.translation = Val2::px(card_shake, 0.0);
                }
            }
        }
        GachaponAnimStep::Burst => {
            if let Ok(mut t) = q_card.single_mut() {
                t.translation = Val2::px(0.0, 0.0);
            }
            if let Ok(mut t) = q_sprite.single_mut() {
                let pop_scale = match anim.rarity {
                    RarityTier::Legendary => 1.25,
                    RarityTier::Exotic => 1.18,
                    RarityTier::Rare => 1.10,
                    _ => 1.05,
                };
                t.scale = Vec2::splat(pop_scale);
                t.translation = Val2::px(0.0, 0.0);
            }
        }
        GachaponAnimStep::Revealed => {
            if let Ok(mut t) = q_card.single_mut() {
                t.translation = Val2::px(0.0, 0.0);
            }
            if let Ok(mut t) = q_sprite.single_mut() {
                let float_speed = match anim.rarity {
                    RarityTier::Legendary => 3.2,
                    RarityTier::Exotic => 2.8,
                    _ => 2.2,
                };
                let float_amp = match anim.rarity {
                    RarityTier::Legendary => 9.0,
                    RarityTier::Exotic => 7.0,
                    RarityTier::Rare => 5.5,
                    _ => 4.0,
                };
                let y = (anim.reveal_elapsed * float_speed).sin() * float_amp;
                let scale_pulse = match anim.rarity {
                    RarityTier::Legendary => 1.08 + 0.06 * (anim.reveal_elapsed * 2.5).sin(),
                    RarityTier::Exotic => 1.04 + 0.03 * (anim.reveal_elapsed * 2.0).sin(),
                    _ => 1.0,
                };
                t.translation = Val2::px(0.0, y);
                t.scale = Vec2::splat(scale_pulse);
            }

            if let Ok(mut t) = q_aura.single_mut() {
                let pulse_amp = match anim.rarity {
                    RarityTier::Legendary => 0.20,
                    RarityTier::Exotic => 0.14,
                    _ => 0.08,
                };
                let s = 1.0 + pulse_amp * (anim.reveal_elapsed * 3.5).sin();
                t.scale = Vec2::splat(s);
            }
        }
    }
}

use crate::ui::transition_ui::TransitionToEvent;

fn handle_exit_gachapon(
    mut q_buttons: Query<&Interaction, (Changed<Interaction>, With<GachaponExitButton>)>,
    mut ev_trans: MessageWriter<TransitionToEvent>,
) {
    for interaction in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            ev_trans.write(TransitionToEvent {
                target: AppState::MainMenu,
            });
        }
    }
}

fn update_gachapon_display(
    meta: Res<MetaProfile>,
    mut q_tokens: Query<&mut Text, With<GachaponTokensText>>,
) {
    if meta.is_changed() {
        for mut text in q_tokens.iter_mut() {
            **text = format!("Fichas: {} Gachapón", meta.gachapon_tokens);
        }
    }
}

fn handle_gachapon_to_starter_select(
    mut q_button: Query<&Interaction, (Changed<Interaction>, With<GachaponToStarterSelectionButton>)>,
    mut ev_trans: MessageWriter<TransitionToEvent>,
) {
    for interaction in q_button.iter_mut() {
        if *interaction == Interaction::Pressed {
            ev_trans.write(TransitionToEvent {
                target: AppState::StarterSelection,
            });
        }
    }
}

fn handle_gachapon_quick_select_pokemon(
    mut q_buttons: Query<(&Interaction, &GachaponSelectPokemonBadgeButton), Changed<Interaction>>,
    mut run_state: ResMut<CurrentRunState>,
    mut q_status: Query<&mut Text, (With<GachaponStatusText>, Without<GachaponOddsText>)>,
) {
    for (interaction, btn) in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            let spec_id = btn.0;
            if run_state.selected_starter_id != spec_id {
                run_state.selected_starter_id = spec_id;
                if let Some(sp) = get_species_data(spec_id) {
                    let elem = sp.primary_types.first().copied().unwrap_or(crate::types::ElementType::Normal);
                    let is_special = sp.base_sp_atk >= sp.base_atk;
                    run_state.player_deck = crate::card::create_starter_deck(elem, is_special);
                    run_state.player_hp = None;
                    if let Ok(mut text) = q_status.single_mut() {
                        **text = format!("[OK] ¡Has cambiado tu Pokémon activo a {} (#{}), mazo actualizado!", sp.name, spec_id);
                    }
                }
            }
        }
    }
}

fn update_gachapon_selection_badges(
    run_state: Res<CurrentRunState>,
    mut q_badges: Query<(&GachaponSelectPokemonBadgeButton, &mut BackgroundColor, &mut bevy::ui::BorderColor, &Children)>,
    mut q_text: Query<&mut Text>,
) {
    if !run_state.is_changed() {
        return;
    }
    for (btn, mut bg, mut border, children) in q_badges.iter_mut() {
        let is_active = btn.0 == run_state.selected_starter_id;
        let name = get_species_data(btn.0).map(|d| d.name).unwrap_or_else(|| "?".to_string());
        *bg = if is_active {
            BackgroundColor(Color::srgb(0.35, 0.28, 0.15))
        } else {
            BackgroundColor(Color::srgb(0.22, 0.20, 0.32))
        };
        *border = if is_active {
            BorderColor(Color::srgb(1.0, 0.85, 0.2))
        } else {
            BorderColor(Color::srgb(0.5, 0.4, 0.7))
        };
        for child in children.iter() {
            if let Ok(mut text) = q_text.get_mut(child) {
                **text = if is_active {
                    format!("#{} {} [ACTIVO]", btn.0, name)
                } else {
                    format!("#{} {}", btn.0, name)
                };
            }
        }
    }
}

fn cleanup_gachapon_ui(mut commands: Commands, q_cleanup: Query<Entity, With<CleanupUi>>) {
    for entity in q_cleanup.iter() {
        commands.entity(entity).despawn();
    }
}
