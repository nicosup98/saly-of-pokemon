use bevy::prelude::*;
use crate::run::AppState;
use crate::ui::BorderColor;

#[derive(Message, Debug, Clone)]
pub struct TransitionToEvent {
    pub target: AppState,
}

#[derive(Resource, Debug, Clone)]
pub struct SceneTransition {
    pub phase: TransitionPhase,
    pub progress: f32,
    pub duration: f32,
    pub target_state: Option<AppState>,
    pub banner_title: String,
    pub banner_subtitle: String,
    pub effect: TransitionEffect,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TransitionPhase {
    Idle,
    FadeOut,
    FadeIn,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TransitionEffect {
    FadeToBlack,
    CombatFlash,
}

impl Default for SceneTransition {
    fn default() -> Self {
        Self {
            phase: TransitionPhase::Idle,
            progress: 0.0,
            duration: 0.28,
            target_state: None,
            banner_title: String::new(),
            banner_subtitle: String::new(),
            effect: TransitionEffect::FadeToBlack,
        }
    }
}

impl SceneTransition {
    pub fn trigger(&mut self, target: AppState) {
        let (title, sub) = state_banner_info(target);
        self.banner_title = title;
        self.banner_subtitle = sub;
        self.target_state = Some(target);
        self.phase = TransitionPhase::FadeOut;
        self.progress = 0.0;
        self.effect = match target {
            AppState::Combat => TransitionEffect::CombatFlash,
            _ => TransitionEffect::FadeToBlack,
        };
    }
}

pub fn state_banner_info(state: AppState) -> (String, String) {
    match state {
        AppState::MainMenu => (
            "SLAY OF POKÉMON".to_string(),
            "Menú Principal".to_string(),
        ),
        AppState::ProfileSelect => (
            "PERFILES DE GUARDADO".to_string(),
            "Gestiona tus 3 ranuras de partida y progreso".to_string(),
        ),
        AppState::StarterSelection => (
            "ELIGE TU POKÉMON".to_string(),
            "Selecciona a tu compañero de aventura".to_string(),
        ),
        AppState::MapNavigation => (
            "MAPA DE KANTO".to_string(),
            "Elige el siguiente nodo en tu camino".to_string(),
        ),
        AppState::Combat => (
            "¡COMBATE POKÉMON!".to_string(),
            "¡Un rival salvaje te desafía!".to_string(),
        ),
        AppState::Victory => (
            "¡VICTORIA EN COMBATE!".to_string(),
            "¡Has superado al rival y ganado recompensas!".to_string(),
        ),
        AppState::PokemonCenter => (
            "CENTRO POKÉMON".to_string(),
            "Descanso, recuperación y mejora de cartas".to_string(),
        ),
        AppState::Shop => (
            "TIENDA POKÉMON".to_string(),
            "Cartas, reliquias y objetos útiles".to_string(),
        ),
        AppState::Treasure => (
            "CÁMARA DEL TESORO".to_string(),
            "¡Has descubierto un cofre con recompensas!".to_string(),
        ),
        AppState::GachaponShop => (
            "GACHAPÓN DE ENTRENADOR".to_string(),
            "Canjea fichas por nuevos Pokémon".to_string(),
        ),
        AppState::RunSummary => (
            "RESUMEN DE LA EXPEDICIÓN".to_string(),
            "Registro de tus hazañas en la Liga".to_string(),
        ),
    }
}

#[derive(Component)]
pub struct TransitionOverlayRoot;

#[derive(Component)]
pub struct TransitionBannerTitle;

#[derive(Component)]
pub struct TransitionBannerSubtitle;

#[derive(Component)]
pub struct TransitionPokeballSpinner;

pub struct TransitionUiPlugin;

impl Plugin for TransitionUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneTransition>()
            .add_message::<TransitionToEvent>()
            .add_systems(Startup, setup_transition_ui)
            .add_systems(
                Update,
                (
                    handle_transition_events,
                    detect_direct_state_change,
                    update_transition_animation,
                )
                    .chain(),
            );
    }
}

fn setup_transition_ui(mut commands: Commands) {
    commands.spawn((
        TransitionOverlayRoot,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(14.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.06, 0.07, 0.10, 0.0)),
        GlobalZIndex(9999),
        Visibility::Hidden,
    )).with_children(|parent| {
        // Ícono Pokéball animado
        parent.spawn((
            TransitionPokeballSpinner,
            UiTransform::default(),
            Node {
                width: Val::Px(56.0),
                height: Val::Px(56.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Percent(50.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.9, 0.22, 0.22)),
            BorderColor(Color::srgb(0.12, 0.12, 0.18)),
        )).with_children(|ball| {
            ball.spawn((
                Node {
                    width: Val::Px(20.0),
                    height: Val::Px(20.0),
                    border: UiRect::all(Val::Px(3.0)),
                    border_radius: BorderRadius::all(Val::Percent(50.0)),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                BorderColor(Color::srgb(0.12, 0.12, 0.18)),
            ));
        });

        // Título principal
        parent.spawn((
            TransitionBannerTitle,
            Text::new(""),
            TextFont::from_font_size(26.0),
            TextColor(Color::WHITE),
        ));

        // Subtítulo
        parent.spawn((
            TransitionBannerSubtitle,
            Text::new(""),
            TextFont::from_font_size(14.0),
            TextColor(Color::srgb(0.7, 0.75, 0.85)),
        ));
    });
}

fn handle_transition_events(
    mut ev_trans: MessageReader<TransitionToEvent>,
    mut transition: ResMut<SceneTransition>,
) {
    for ev in ev_trans.read() {
        if transition.phase != TransitionPhase::FadeOut {
            transition.trigger(ev.target);
        }
    }
}

fn detect_direct_state_change(
    state: Res<State<AppState>>,
    mut transition: ResMut<SceneTransition>,
) {
    if state.is_changed() && transition.phase == TransitionPhase::Idle {
        let (title, sub) = state_banner_info(*state.get());
        transition.banner_title = title;
        transition.banner_subtitle = sub;
        transition.phase = TransitionPhase::FadeIn;
        transition.progress = 1.0;
        transition.effect = TransitionEffect::FadeToBlack;
    }
}

fn update_transition_animation(
    time: Res<Time>,
    mut transition: ResMut<SceneTransition>,
    mut next_state: ResMut<NextState<AppState>>,
    mut q_overlay: Query<(&mut BackgroundColor, &mut Visibility), With<TransitionOverlayRoot>>,
    mut q_title: Query<(&mut Text, &mut TextColor), (With<TransitionBannerTitle>, Without<TransitionBannerSubtitle>)>,
    mut q_sub: Query<(&mut Text, &mut TextColor), (With<TransitionBannerSubtitle>, Without<TransitionBannerTitle>)>,
    mut q_spinner: Query<(&mut UiTransform, &mut Visibility), (With<TransitionPokeballSpinner>, Without<TransitionOverlayRoot>)>,
) {
    let Ok((mut bg_col, mut overlay_vis)) = q_overlay.single_mut() else { return };

    if transition.phase == TransitionPhase::Idle {
        if *overlay_vis != Visibility::Hidden {
            *overlay_vis = Visibility::Hidden;
        }
        return;
    }

    let raw_dt = time.delta_secs();

    // Si el tiempo está congelado o en pruebas ECS sin simulación de reloj, aplicar de inmediato
    if raw_dt <= 0.0 {
        if let Some(target) = transition.target_state.take() {
            next_state.set(target);
        }
        transition.phase = TransitionPhase::Idle;
        transition.progress = 0.0;
        if *overlay_vis != Visibility::Hidden {
            *overlay_vis = Visibility::Hidden;
        }
        return;
    }

    // Limitar el delta por frame (máx 33ms) para que la carga pesada de `OnEnter` al cambiar de estado
    // no provoque un salto brusco en el primer frame de FadeIn.
    let dt = raw_dt.min(0.033);

    if *overlay_vis != Visibility::Visible {
        *overlay_vis = Visibility::Visible;
    }

    match transition.phase {
        TransitionPhase::FadeOut => {
            transition.progress += dt / transition.duration;
            if transition.progress >= 1.0 {
                transition.progress = 1.0;
                if let Some(target) = transition.target_state.take() {
                    next_state.set(target);
                }
                transition.phase = TransitionPhase::FadeIn;
            }
        }
        TransitionPhase::FadeIn => {
            transition.progress -= dt / transition.duration;
            if transition.progress <= 0.0 {
                transition.progress = 0.0;
                transition.phase = TransitionPhase::Idle;
                *overlay_vis = Visibility::Hidden;
                return;
            }
        }
        TransitionPhase::Idle => return,
    }

    let raw_alpha = transition.progress.clamp(0.0, 1.0);
    // Curva smoothstep para transición visual sedosa
    let alpha = raw_alpha * raw_alpha * (3.0 - 2.0 * raw_alpha);

    // Efecto de color de fondo
    if transition.effect == TransitionEffect::CombatFlash
        && transition.phase == TransitionPhase::FadeOut
        && raw_alpha < 0.25
    {
        let flash = (1.0 - raw_alpha * 4.0).clamp(0.0, 1.0);
        *bg_col = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, flash));
    } else {
        *bg_col = BackgroundColor(Color::srgba(0.05, 0.06, 0.09, alpha));
    }

    // Textos (solo mutar String cuando cambia el título para no recalcular glifos en cada frame)
    if let Ok((mut text, mut color)) = q_title.single_mut() {
        if text.as_str() != transition.banner_title {
            **text = transition.banner_title.clone();
        }
        *color = TextColor(Color::srgba(1.0, 1.0, 1.0, alpha));
    }
    if let Ok((mut text, mut color)) = q_sub.single_mut() {
        if text.as_str() != transition.banner_subtitle {
            **text = transition.banner_subtitle.clone();
        }
        *color = TextColor(Color::srgba(0.7, 0.75, 0.85, alpha));
    }

    // Rotación de la Pokéball mediante UiTransform
    if let Ok((mut transform, mut spin_vis)) = q_spinner.single_mut() {
        let target_vis = if alpha > 0.1 { Visibility::Visible } else { Visibility::Hidden };
        if *spin_vis != target_vis {
            *spin_vis = target_vis;
        }
        transform.rotation = Rot2::radians(time.elapsed_secs() * 5.0);
        let scale = (0.7 + 0.3 * alpha).clamp(0.0, 1.0);
        transform.scale = Vec2::splat(scale);
    }
}
