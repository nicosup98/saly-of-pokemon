pub mod combat_ui;
pub mod map_ui;
pub mod shop_ui;
pub mod center_ui;
pub mod gachapon_ui;
pub mod menu_ui;
pub mod treasure_ui;
pub mod summary_ui;
pub mod victory_ui;
pub mod starter_selection_ui;
pub mod profile_ui;
pub mod transition_ui;

use std::collections::HashMap;
use bevy::prelude::*;
use crate::card::{ALL_CARD_ASSETS, ALL_SITUATION_BACKGROUNDS};
use crate::save::{ActiveSaveSlot, SaveNotificationEvent};
use bevy_tweening::{
    lens::{UiBackgroundColorLens, UiTransformScaleLens},
    AnimTarget, RepeatCount, RepeatStrategy, Tween, TweenAnim,
};

/// Recurso global que retiene handles de assets comunes cargados al iniciar el juego
#[derive(Resource, Default, Debug)]
pub struct PreloadedAssets {
    pub card_textures: HashMap<&'static str, Handle<Image>>,
    pub background_textures: HashMap<&'static str, Handle<Image>>,
    pub font: Option<Handle<Font>>,
}

impl PreloadedAssets {
    pub fn get_card(&self, path: &str) -> Option<Handle<Image>> {
        self.card_textures.get(path).cloned()
    }

    pub fn get_bg(&self, path: &str) -> Option<Handle<Image>> {
        self.background_textures.get(path).cloned()
    }
}

/// Sistema de Startup que precarga en segundo plano todas las plantillas de cartas y fondos comunes
pub fn preload_common_assets(
    asset_server: Option<Res<AssetServer>>,
    mut preloaded: ResMut<PreloadedAssets>,
) {
    let Some(asset_server) = asset_server else { return };

    for &path in ALL_CARD_ASSETS {
        preloaded.card_textures.insert(path, asset_server.load(path));
    }

    for &path in ALL_SITUATION_BACKGROUNDS {
        preloaded.background_textures.insert(path, asset_server.load(path));
    }

    preloaded.font = Some(asset_server.load("fonts/FiraSans-Bold.ttf"));
}

use bevy::asset::LoadState;

/// Recurso para registrar el progreso de la precarga de assets
#[derive(Resource, Debug, Clone)]
pub struct PreloadProgress {
    pub total: usize,
    pub loaded: usize,
    pub current_asset: String,
    pub is_finished: bool,
    pub dismiss_timer: Timer,
}

impl Default for PreloadProgress {
    fn default() -> Self {
        Self {
            total: 0,
            loaded: 0,
            current_asset: "Iniciando carga de recursos...".to_string(),
            is_finished: false,
            dismiss_timer: Timer::from_seconds(0.35, TimerMode::Once),
        }
    }
}

#[derive(Component)]
pub struct PreloadLoadingScreen;

#[derive(Component)]
pub struct PreloadProgressBarFill;

#[derive(Component)]
pub struct PreloadProgressPercentText;

#[derive(Component)]
pub struct PreloadProgressDetailText;

#[derive(Component)]
pub struct PreloadProgressSpinner;

/// Instancia la pantalla modal de carga al iniciar el juego
pub fn spawn_preload_loading_screen(mut commands: Commands) {
    commands
        .spawn((
            PreloadLoadingScreen,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.07, 0.11)),
            GlobalZIndex(999),
        ))
        .with_children(|root| {
            // Título Superior
            root.spawn((
                Text::new("SPIREMON: ROGUELIKE DECKBUILDER"),
                TextFont::from_font_size(24.0),
                TextColor(Color::srgb(1.0, 0.85, 0.2)),
            ));

            root.spawn((
                Text::new("PRECARGANDO CARTAS Y FONDOS COMUNES"),
                TextFont::from_font_size(14.0),
                TextColor(Color::srgb(0.65, 0.75, 0.85)),
            ));

            // Indicador giratorio / Pokéball estilizada
            root.spawn((
                PreloadProgressSpinner,
                Node {
                    width: Val::Px(36.0),
                    height: Val::Px(36.0),
                    border: UiRect::all(Val::Px(3.0)),
                    border_radius: BorderRadius::all(Val::Px(18.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::axes(Val::Px(0.0), Val::Px(8.0)),
                    ..default()
                },
                UiTransform::default(),
                BackgroundColor(Color::srgb(0.12, 0.16, 0.24)),
                BorderColor(Color::srgb(0.9, 0.3, 0.3)),
            ));

            // Contenedor de la Barra de Progreso
            root.spawn((
                Node {
                    width: Val::Px(500.0),
                    height: Val::Px(24.0),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    padding: UiRect::all(Val::Px(2.0)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.14, 0.18)),
                BorderColor(Color::srgb(0.3, 0.5, 0.8)),
            ))
            .with_children(|bar| {
                // Relleno dinámico de la barra
                bar.spawn((
                    PreloadProgressBarFill,
                    Node {
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.25, 0.85, 0.55)),
                ));
            });

            // Texto de porcentaje (ej: "45% - 6 / 14 assets cargados - Faltan 8")
            root.spawn((
                PreloadProgressPercentText,
                Text::new("Cargando... 0% (0 / 14)"),
                TextFont::from_font_size(15.0),
                TextColor(Color::WHITE),
            ));

            // Texto de detalle con el nombre del asset actual
            root.spawn((
                PreloadProgressDetailText,
                Text::new("Iniciando recursos gráficos..."),
                TextFont::from_font_size(12.0),
                TextColor(Color::srgb(0.7, 0.8, 0.9)),
            ));
        });
}

/// Actualiza el porcentaje y barra de carga comprobando el estado de cada asset precargado
pub fn update_preload_loading_screen(
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    preloaded: Res<PreloadedAssets>,
    mut progress: ResMut<PreloadProgress>,
    q_screen: Query<Entity, With<PreloadLoadingScreen>>,
    mut q_fill: Query<&mut Node, With<PreloadProgressBarFill>>,
    mut q_percent_text: Query<&mut Text, (With<PreloadProgressPercentText>, Without<PreloadProgressDetailText>)>,
    mut q_detail_text: Query<&mut Text, (With<PreloadProgressDetailText>, Without<PreloadProgressPercentText>)>,
    mut q_spinner: Query<&mut UiTransform, With<PreloadProgressSpinner>>,
) {
    let Ok(screen_entity) = q_screen.single() else { return };

    // Si ya terminó, avanza el temporizador de cierre suave
    if progress.is_finished {
        progress.dismiss_timer.tick(time.delta());
        if progress.dismiss_timer.is_finished() {
            commands.entity(screen_entity).despawn();
        }
        return;
    }

    // Gira el icono de carga continuamente
    for mut trans in q_spinner.iter_mut() {
        trans.rotation = Rot2::radians(time.elapsed_secs() * 5.0);
    }

    let total = preloaded.card_textures.len() + preloaded.background_textures.len() + if preloaded.font.is_some() { 1 } else { 0 };
    progress.total = total;

    // Si no hay asset server (entorno headless/tests) o total es 0, finaliza de inmediato
    let Some(ref asset_server) = asset_server else {
        progress.loaded = total;
        progress.is_finished = true;
        commands.entity(screen_entity).despawn();
        return;
    };

    if total == 0 {
        progress.is_finished = true;
        commands.entity(screen_entity).despawn();
        return;
    }

    let mut loaded = 0;
    let mut current_loading = "Finalizando precarga...".to_string();

    for (&path, handle) in &preloaded.card_textures {
        let state = asset_server.get_load_state(handle.id());
        if matches!(state, Some(LoadState::Loaded | LoadState::Failed(_))) {
            loaded += 1;
        } else {
            current_loading = format!("Cargando: {}", path);
        }
    }

    for (&path, handle) in &preloaded.background_textures {
        let state = asset_server.get_load_state(handle.id());
        if matches!(state, Some(LoadState::Loaded | LoadState::Failed(_))) {
            loaded += 1;
        } else {
            current_loading = format!("Cargando: {}", path);
        }
    }

    if let Some(ref font_handle) = preloaded.font {
        let state = asset_server.get_load_state(font_handle.id());
        if matches!(state, Some(LoadState::Loaded | LoadState::Failed(_))) {
            loaded += 1;
        } else {
            current_loading = "Cargando: fonts/FiraSans-Bold.ttf".to_string();
        }
    }

    progress.loaded = loaded;
    progress.current_asset = current_loading;

    let pct = if total > 0 { (loaded as f32 / total as f32) * 100.0 } else { 100.0 };
    let clamped_pct = pct.clamp(0.0, 100.0);

    // Actualiza barra
    for mut node in q_fill.iter_mut() {
        node.width = Val::Percent(clamped_pct);
    }

    // Actualiza textos
    for mut text in q_percent_text.iter_mut() {
        if loaded >= total {
            **text = format!("¡Carga completa! 100% ({}/{})", total, total);
        } else {
            let remaining = total - loaded;
            **text = format!("Precargando assets... {:.0}% ({}/{} - Faltan {})", clamped_pct, loaded, total, remaining);
        }
    }

    for mut text in q_detail_text.iter_mut() {
        if loaded >= total {
            **text = "¡Todos los recursos listos! Iniciando...".to_string();
        } else {
            **text = progress.current_asset.clone();
        }
    }

    if loaded >= total {
        progress.is_finished = true;
    }
}

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        if std::env::var("BEVY_ASSET_ROOT").is_err() {
            let root = crate::resolve_asset_root();
            unsafe {
                std::env::set_var("BEVY_ASSET_ROOT", &root);
            }
        }
        if !app.world().contains_resource::<Assets<Font>>() {
            app.init_asset::<Font>();
        }
        if !app.world().contains_resource::<Assets<Image>>() {
            app.init_asset::<Image>();
        }
        app.init_resource::<ActiveSaveSlot>()
            .init_resource::<PreloadedAssets>()
            .init_resource::<PreloadProgress>()
            .add_message::<SaveNotificationEvent>()
            .insert_resource(ClearColor(Color::srgb(0.05, 0.06, 0.10)))
            .add_systems(Startup, (setup_camera, preload_common_assets, spawn_preload_loading_screen))
            .add_systems(
                Update,
                (
                    apply_default_font_to_texts,
                    spawn_save_notification_toast,
                    update_save_notification_toast,
                    animate_card_hover,
                    update_preload_loading_screen,
                ),
            )
            .add_plugins(transition_ui::TransitionUiPlugin)
            .add_plugins(menu_ui::MenuUiPlugin)
            .add_plugins(combat_ui::CombatUiPlugin)
            .add_plugins(victory_ui::VictoryUiPlugin)
            .add_plugins(map_ui::MapUiPlugin)
            .add_plugins(shop_ui::ShopUiPlugin)
            .add_plugins(center_ui::CenterUiPlugin)
            .add_plugins(gachapon_ui::GachaponUiPlugin)
            .add_plugins(treasure_ui::TreasureUiPlugin)
            .add_plugins(summary_ui::SummaryUiPlugin)
            .add_plugins(starter_selection_ui::StarterSelectionUiPlugin)
            .add_plugins(profile_ui::ProfileUiPlugin);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Asigna automáticamente la fuente `FiraSans-Bold.ttf` (con soporte completo de tildes, eñes y signos `¡¿`)
/// a todos los textos de la interfaz que usen el handle por defecto.
/// Filtra únicamente entidades recién creadas o modificadas (`Added<TextFont>` / `Changed<TextFont>`)
/// para evitar iteraciones y mutaciones innecesarias en cada frame.
fn apply_default_font_to_texts(
    asset_server: Option<Res<AssetServer>>,
    preloaded: Option<Res<PreloadedAssets>>,
    mut cached_font: Local<Option<Handle<Font>>>,
    mut q_texts: Query<&mut TextFont, Or<(Added<TextFont>, Changed<TextFont>)>>,
) {
    if q_texts.is_empty() {
        return;
    }
    let font_handle = if let Some(ref handle) = *cached_font {
        handle.clone()
    } else if let Some( handle) = preloaded.as_ref().and_then(|p| p.font.clone()) {
        *cached_font = Some(handle.clone());
        handle
    } else if let Some(asset_server) = asset_server {
        let handle = asset_server.load("fonts/FiraSans-Bold.ttf");
        *cached_font = Some(handle.clone());
        handle
    } else {
        return;
    };

    for mut tf in q_texts.iter_mut() {
        if tf.font == bevy::text::FontSource::default() {
            tf.font = bevy::text::FontSource::from(font_handle.clone());
        }
    }
}

/// Componente del cartel flotante de aviso de guardado/carga
#[derive(Component)]
pub struct SaveToastBanner {
    pub timer: Timer,
}

fn spawn_save_notification_toast(
    mut commands: Commands,
    mut ev_notify: MessageReader<SaveNotificationEvent>,
    q_existing: Query<Entity, With<SaveToastBanner>>,
) {
    for ev in ev_notify.read() {
        // Eliminar avisos anteriores para mostrar el más reciente inmediatamente
        for entity in q_existing.iter() {
            commands.entity(entity).despawn();
        }

        let (bg_start, bg_end, border_col, text_col) = if ev.is_error {
            (
                Color::srgba(0.45, 0.12, 0.12, 0.96),
                Color::srgba(0.60, 0.18, 0.18, 0.98),
                Color::srgb(1.0, 0.35, 0.35),
                Color::srgb(1.0, 0.9, 0.9),
            )
        } else {
            (
                Color::srgba(0.10, 0.32, 0.20, 0.96),
                Color::srgba(0.16, 0.48, 0.28, 0.98),
                Color::srgb(0.35, 0.95, 0.55),
                Color::srgb(0.95, 1.0, 0.95),
            )
        };

        let banner_id = commands
            .spawn((
                SaveToastBanner {
                    timer: Timer::from_seconds(3.2, TimerMode::Once),
                },
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(14.0),
                    right: Val::Px(20.0),
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                    border: UiRect::all(Val::Px(2.5)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                UiTransform::from_scale(Vec2::splat(0.85)),
                BackgroundColor(bg_start),
                BorderColor(border_col),
                GlobalZIndex(250),
            ))
            .with_children(|toast| {
                toast.spawn((
                    Text::new(ev.message.clone()),
                    TextFont::from_font_size(14.0),
                    TextColor(text_col),
                ));

                // Animación de pulso de fondo con bevy_tweening
                let bg_tween = Tween::new(
                    EaseFunction::SineInOut,
                    std::time::Duration::from_millis(700),
                    UiBackgroundColorLens {
                        start: bg_start,
                        end: bg_end,
                    },
                )
                .with_repeat(RepeatCount::Infinite, RepeatStrategy::MirroredRepeat);
                toast.spawn((
                    TweenAnim::new(bg_tween),
                    AnimTarget::component::<BackgroundColor>(toast.target_entity()),
                ));

                // Animación de entrada elástica (escala UI) con bevy_tweening
                let scale_tween = Tween::new(
                    EaseFunction::BackOut,
                    std::time::Duration::from_millis(320),
                    UiTransformScaleLens {
                        start: Vec2::splat(0.80),
                        end: Vec2::splat(1.0),
                    },
                );
                toast.spawn((
                    TweenAnim::new(scale_tween),
                    AnimTarget::component::<UiTransform>(toast.target_entity()),
                ));
            })
            .id();
        let _ = banner_id;
    }
}

fn update_save_notification_toast(
    mut commands: Commands,
    time: Res<Time>,
    mut q_toasts: Query<(Entity, &mut SaveToastBanner)>,
) {
    for (entity, mut toast) in q_toasts.iter_mut() {
        toast.timer.tick(time.delta());
        if toast.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Marcador para botones de UI interactivos con soporte de interacción
#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
#[require(Node, Interaction)]
pub struct Button;

/// Helper constructor para BorderColor uniforme en Bevy 0.20
#[allow(non_snake_case)]
pub fn BorderColor(color: Color) -> bevy::ui::BorderColor {
    bevy::ui::BorderColor::all(color)
}

/// Marcador para limpiar UI al salir de un estado
#[derive(Component)]
pub struct CleanupUi;

/// Helper para obtener el color asociado a cada Tipo Elemental
pub fn element_color(element: crate::types::ElementType) -> Color {
    use crate::types::ElementType::*;
    match element {
        Fire => Color::srgb(0.92, 0.32, 0.20),
        Water => Color::srgb(0.22, 0.55, 0.95),
        Grass => Color::srgb(0.28, 0.72, 0.32),
        Electric => Color::srgb(0.95, 0.82, 0.15),
        Ice => Color::srgb(0.40, 0.85, 0.90),
        Fighting => Color::srgb(0.75, 0.25, 0.20),
        Poison => Color::srgb(0.65, 0.25, 0.65),
        Ground => Color::srgb(0.82, 0.68, 0.35),
        Flying => Color::srgb(0.60, 0.70, 0.95),
        Psychic => Color::srgb(0.95, 0.35, 0.60),
        Bug => Color::srgb(0.65, 0.75, 0.20),
        Rock => Color::srgb(0.72, 0.62, 0.35),
        Ghost => Color::srgb(0.45, 0.35, 0.65),
        Dragon => Color::srgb(0.45, 0.25, 0.90),
        Dark => Color::srgb(0.35, 0.30, 0.30),
        Steel => Color::srgb(0.70, 0.72, 0.75),
        Fairy => Color::srgb(0.95, 0.65, 0.80),
        Normal => Color::srgb(0.62, 0.62, 0.55),
    }
}

/// Componente que marca una carta interactiva para animar elevación, escala y brillo de borde en hover
#[derive(Component, Clone, Copy, Debug)]
pub struct CardHoverAnimation {
    pub base_border: Color,
    pub hover_border: Color,
    pub lift_px: f32,
    pub hover_scale: f32,
}

impl Default for CardHoverAnimation {
    fn default() -> Self {
        Self {
            base_border: Color::srgb(0.5, 0.5, 0.5),
            hover_border: Color::srgb(1.0, 0.95, 0.45),
            lift_px: -22.0,
            hover_scale: 1.10,
        }
    }
}

impl CardHoverAnimation {
    pub fn new(base_border: Color) -> Self {
        Self {
            base_border,
            hover_border: Color::srgb(1.0, 0.95, 0.45),
            lift_px: -22.0,
            hover_scale: 1.10,
        }
    }

    pub fn with_lift(mut self, lift: f32) -> Self {
        self.lift_px = lift;
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.hover_scale = scale;
        self
    }
}

/// Sistema continuo que interpola suavemente la elevación y escala de las cartas al pasar el puntero.
/// Utiliza suavizado exponencial independiente del framerate y evita mutar componentes (`UiTransform`,
/// `BorderColor`, `GlobalZIndex`) cuando la carta ya alcanzó su estado de reposo u objetivo.
pub fn animate_card_hover(
    time: Res<Time>,
    mut q_cards: Query<(
        &Interaction,
        &mut UiTransform,
        &mut bevy::ui::BorderColor,
        Option<&mut GlobalZIndex>,
        &CardHoverAnimation,
    )>,
) {
    let raw_dt = time.delta_secs();
    if raw_dt <= 0.0 {
        return;
    }
    let dt = raw_dt.min(0.05);
    // Suavizado exponencial independiente de los FPS (equivalente a ~18Hz de respuesta)
    let factor = (1.0 - (-18.0 * dt).exp()).clamp(0.0, 1.0);

    for (interaction, mut transform, mut border, mut z_index, hover) in q_cards.iter_mut() {
        let (target_scale, target_y, target_border, z) = match *interaction {
            Interaction::Hovered => (hover.hover_scale, hover.lift_px, hover.hover_border, 150),
            Interaction::Pressed => (1.02, hover.lift_px * 0.55, Color::srgb(1.0, 1.0, 0.8), 150),
            Interaction::None => (1.0, 0.0, hover.base_border, 10),
        };

        // Solo mutar BorderColor si cambió realmente
        if border.top != target_border {
            *border = bevy::ui::BorderColor::all(target_border);
        }

        // Solo mutar GlobalZIndex si cambió realmente
        if let Some(ref mut z_idx) = z_index {
            if z_idx.0 != z {
                z_idx.0 = z;
            }
        }

        let curr_y = match transform.translation.y {
            Val::Px(y) => y,
            _ => 0.0,
        };
        let scale_diff = (transform.scale.x - target_scale).abs();
        let y_diff = (curr_y - target_y).abs();

        // Si ya está exactamente en el objetivo, no ensuciar `Changed<UiTransform>`
        if scale_diff == 0.0 && y_diff == 0.0 {
            continue;
        }

        // Si está muy cerca del objetivo, hacer snap final para detener mutaciones en frames futuros
        if scale_diff < 0.0005 && y_diff < 0.05 {
            transform.scale = Vec2::splat(target_scale);
            transform.translation = Val2::px(0.0, target_y);
        } else {
            transform.scale = transform.scale.lerp(Vec2::splat(target_scale), factor);
            let new_y = curr_y + (target_y - curr_y) * factor;
            transform.translation = Val2::px(0.0, new_y);
        }
    }
}
