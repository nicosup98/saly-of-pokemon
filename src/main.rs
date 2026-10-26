#![allow(deprecated)]

pub mod types;
pub mod stats;
pub mod card;
pub mod relics;
pub mod pokemon;
pub mod run;
pub mod combat;
pub mod map;
pub mod shop;
pub mod pokemon_center;
pub mod gachapon;
pub mod save;
pub mod vfx;
pub mod ui;

use bevy::prelude::*;
use crate::card::{Card, InHand, create_starter_deck};
use crate::combat::events::{PlayCardEvent, EndPlayerTurnEvent};
use crate::combat::{CombatPlugin, PlayerPokemon, EnemyPokemon};
use crate::map::MapPlugin;
use crate::pokemon::check_evolution;
use crate::run::{AppState, CombatPhase, CurrentRunState, MetaProfile};
use crate::shop::ShopPlugin;
use crate::stats::PokemonStats;
use crate::types::ElementType;
use bevy_tweening::TweeningPlugin;
use bevy_hanabi::HanabiPlugin;

fn main() {
    println!("============================================================");
    println!("🔥 SPIREMON: ROGUELIKE DECKBUILDER RPG (RUST + BEVY ENGINE) 🔥");
    println!("============================================================\n");

    let args: Vec<String> = std::env::args().collect();
    let is_demo = args.contains(&"--demo".to_string());
    let has_display = std::env::var("WAYLAND_DISPLAY").is_ok() || std::env::var("DISPLAY").is_ok();

    let mut app = App::new();

    if is_demo || !has_display {
        println!("🚀 Modo Demostración / Simulación CLI activado.");
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<AppState>()
            .add_sub_state::<CombatPhase>()
            .init_resource::<CurrentRunState>()
            .init_resource::<MetaProfile>()
            .add_plugins(CombatPlugin)
            .add_plugins(MapPlugin)
            .add_plugins(ShopPlugin)
            .add_plugins(pokemon_center::PokemonCenterPlugin)
            .add_plugins(gachapon::GachaponPlugin)
            .add_plugins(save::SavePlugin)
        .add_plugins(vfx::HanabiVfxPlugin)
            .add_systems(Startup, print_welcome)
            .add_systems(Update, display_combat_hud.run_if(in_state(AppState::Combat)));

        app.update();
        run_demo_simulation(&mut app);
    } else {
        println!("[INFO] Lanzando interfaz gráfica 2D con Bevy Engine (1280x720)...");
        use bevy::render::{RenderPlugin, settings::{Backends, WgpuSettings}};

        let backend = match std::env::var("WGPU_BACKEND").as_deref() {
            Ok("vulkan") => {
                println!("🔧 Backend gráfico seleccionado: Vulkan");
                Some(Backends::VULKAN)
            }
            Ok("gl") | Ok("gles") => {
                println!("🔧 Backend gráfico seleccionado: OpenGL / GL");
                Some(Backends::GL)
            }
            _ => {
                // En Wayland nativo, Vulkan (Backends::PRIMARY) utiliza VK_KHR_wayland_surface
                // de forma directa y estable.
                println!("🔧 Backend gráfico seleccionado: Primario (Vulkan nativo Wayland)");
                Some(Backends::PRIMARY)
            }
        };

        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "SpireMon: Roguelike Deckbuilder RPG (Rust + Bevy)".into(),
                resolution: bevy::window::WindowResolution::new(1280, 720),
                resizable: true,
                ..default()
            }),
            ..default()
        }).set(RenderPlugin {
            render_creation: WgpuSettings {
                backends: backend,
                ..default()
            }.into(),
            ..default()
        }))
        .init_state::<AppState>()
        .add_sub_state::<CombatPhase>()
        .init_resource::<CurrentRunState>()
        .init_resource::<MetaProfile>()
        .add_plugins(TweeningPlugin)
        .add_plugins(HanabiPlugin)
        .add_plugins(ui::GameUiPlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(MapPlugin)
        .add_plugins(ShopPlugin)
        .add_plugins(pokemon_center::PokemonCenterPlugin)
        .add_plugins(gachapon::GachaponPlugin)
        .add_plugins(save::SavePlugin)
        .add_plugins(vfx::HanabiVfxPlugin)
        .add_systems(Startup, print_welcome)
        .add_systems(Update, display_combat_hud.run_if(in_state(AppState::Combat)));

        app.run();
    }
}

fn print_welcome(mut run_state: ResMut<CurrentRunState>, meta: Res<MetaProfile>) {
    println!("[PERFIL] Perfil de Jugador inicializado:");
    println!("   - Fichas Gachapón: {}", meta.gachapon_tokens);
    println!("   - Pokémon inicial desbloqueado: Charmander (ID: 4)");
    println!("   - Moneda de Run: {} P$\n", run_state.poke_dollars);

    // Inicializa el mazo de Charmander (10 cartas según GDD 5.4)
    run_state.player_deck = create_starter_deck(ElementType::Fire, true);
    println!("[MAZO] Mazo inicial de 10 cartas cargado:");
    for (i, c) in run_state.player_deck.iter().enumerate() {
        println!("   [{}] {} ({:?} | Coste: {})", i + 1, c.display_name(), c.element, c.energy_cost);
    }
    println!();
}

fn display_combat_hud(
    run_state: Res<CurrentRunState>,
    q_player: Query<(&PokemonStats, &crate::pokemon::PokemonSprite), With<PlayerPokemon>>,
    q_enemy: Query<(&PokemonStats, &crate::pokemon::PokemonSprite), With<EnemyPokemon>>,
    q_hand: Query<&Card, With<InHand>>,
) {
    if let (Ok((player, player_sprite)), Ok((enemy, enemy_sprite))) = (q_player.single(), q_enemy.single()) {
        println!("\n------------------------------------------------------------");
        println!(
            "❤️ JUGADOR: {} (Nv. {}) [Sprite: {}] | HP: {}/{} | Bloqueo: {} | Energía: {}/{}",
            player.name, player.level, player_sprite.relative_path(), player.hp, player.max_hp, player.block, run_state.energy, run_state.max_energy
        );
        println!(
            "   Stats: Atk: {:.1} | Def: {:.1} | Sp.Atk: {:.1} | Sp.Def: {:.1} | Spe: {:.1}",
            player.atk, player.def, player.sp_atk, player.sp_def, player.spe
        );
        println!(
            "👾 RIVAL:   {} (Nv. {}) [Sprite: {}] | HP: {}/{} | Bloqueo: {}",
            enemy.name, enemy.level, enemy_sprite.relative_path(), enemy.hp, enemy.max_hp, enemy.block
        );
        println!("🎴 MANO:");
        for (i, card) in q_hand.iter().enumerate() {
            println!(
                "   [{}] {} ({:?}, Coste {}, Mult: {:.2}x)",
                i, card.display_name(), card.element, card.energy_cost, card.multiplier
            );
        }
        println!("------------------------------------------------------------");
    }
}

/// Simula un combate por turnos completo, subida de nivel y evolución canónica
fn run_demo_simulation(app: &mut App) {
    println!("\n🚀 === INICIANDO SIMULACIÓN DE RUN (PISO 1 / ACTO 1) ===");

    // Cambia el estado a Combate
    app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::Combat);
    app.update();

    // Turno 1: Jugador juega carta 0 (Ataque)
    println!("\n👉 [ACCIÓN JUGADOR]: Jugando carta de mano en índice 0...");
    app.world_mut().write_message(PlayCardEvent { card_index: 0 });
    app.update();

    // Jugador juega otra carta
    println!("\n👉 [ACCIÓN JUGADOR]: Jugando carta de mano en índice 1...");
    app.world_mut().write_message(PlayCardEvent { card_index: 0 });
    app.update();

    // Fin de turno
    println!("\n👉 [ACCIÓN JUGADOR]: Fin de turno...");
    app.world_mut().write_message(EndPlayerTurnEvent);
    app.update();

    // Imprime logs de combate
    let run_state = app.world().resource::<CurrentRunState>();
    println!("\n📜 Registro de eventos de combate:");
    for log in &run_state.battle_logs {
        println!("   {}", log);
    }

    // Demostración de evolución canónica (Sección 5.3) y actualización de sprites
    println!("\n🧬 === DEMOSTRACIÓN DE SALTO EVOLUTIVO CANÓNICO (Charmander -> Charmeleon -> Charizard) ===");
    let mut demo_charmander = pokemon::create_pokemon_from_species(4, 5).unwrap();
    let mut demo_sprite = pokemon::PokemonSprite::new(4, true);
    println!(
        "1. Nv 5: {} [Sprite: {}] | HP: {} | Sp. Atk: {:.1} | Ascuas inflige: {:.1} daño",
        demo_charmander.name, demo_sprite.relative_path(), demo_charmander.max_hp, demo_charmander.sp_atk,
        demo_charmander.sp_atk * 1.1 * 1.2 // Stat * 1.1x * 1.2 STAB
    );

    // Sube a nivel 16
    demo_charmander.level = 16;
    if let Some(msg) = check_evolution(&mut demo_charmander) {
        println!("{}", msg);
        demo_sprite.species_id = demo_charmander.species_id;
    }
    println!(
        "2. Nv 16: {} [Sprite: {}] | HP: {} | Sp. Atk: {:.1} | Ascuas+ inflige: {:.1} daño",
        demo_charmander.name, demo_sprite.relative_path(), demo_charmander.max_hp, demo_charmander.sp_atk,
        demo_charmander.sp_atk * 1.4 * 1.2
    );

    // Sube a nivel 36
    demo_charmander.level = 36;
    if let Some(msg) = check_evolution(&mut demo_charmander) {
        println!("{}", msg);
        demo_sprite.species_id = demo_charmander.species_id;
    }
    println!(
        "3. Nv 36: {} [Sprite: {}] | HP: {} | Sp. Atk: {:.1} | Lanzallamas inflige: {:.1} daño",
        demo_charmander.name, demo_sprite.relative_path(), demo_charmander.max_hp, demo_charmander.sp_atk,
        demo_charmander.sp_atk * 2.2 * 1.2
    );

    // Demostración de generación del mapa procedural (Sección 6.3)
    println!("\n🗺️ === GENERACIÓN PROCEDURAL DE MAPA (ACTO 1 - 15 PISOS, 3 CAMINOS) ===");
    let act_map = map::generator::generate_act_map(1);
    for floor in &act_map.floors {
        let node_icons: Vec<String> = floor
            .iter()
            .map(|n| format!("{}[Piso {}:{}]", n.node_type.icon(), n.floor, n.node_type.name()))
            .collect();
        println!("   {}", node_icons.join("  <--->  "));
    }

    // Demostración de Gachapón con fragmentos
    println!("\n🎰 === METAPROGRESIÓN: TIRADA GACHAPÓN CON FRAGMENTOS ===");
    let mut meta = app.world_mut().resource_mut::<MetaProfile>();
    meta.pokemon_fragments.insert(123, 3); // 3 fragmentos de Scyther
    println!("Insertando 3 fragmentos de Scyther (Tasa potenciada al 35%)...");
    let pulled = gachapon::pull_gachapon(&mut meta, Some(123), 3);
    println!("¡Resultado de la tirada: ID de especie obtenido = {:?}! Desbloqueados actuales: {:?}\n", pulled, meta.unlocked_pokemon);

    println!("✅ Todas las mecánicas y sistemas del GDD han sido ejecutados con éxito en Bevy ECS.");
}
