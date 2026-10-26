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

use bevy::prelude::*;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
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
