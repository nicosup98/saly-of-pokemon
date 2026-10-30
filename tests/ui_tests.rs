#![allow(deprecated)]

use bevy::ecs::system::RunSystemOnce;

use bevy::prelude::*;
use slay_of_pokemon::card::create_starter_deck;
use slay_of_pokemon::combat::CombatPlugin;
use slay_of_pokemon::map::MapPlugin;
use slay_of_pokemon::pokemon_center::{upgrade_card_at_center, PokemonCenterPlugin};
use slay_of_pokemon::run::{AppState, CombatPhase, CurrentRunState, MetaProfile};
use slay_of_pokemon::shop::{setup_shop, ShopInventory, ShopPlugin};
use slay_of_pokemon::types::ElementType;
use slay_of_pokemon::ui::GameUiPlugin;
use slay_of_pokemon::ui::victory_ui::{VictoryContinueButton, VictoryCelebrationSprite, VictoryRewardCardButton};

#[test]
fn test_ui_plugin_initialization_and_state_transitions() {
    let mut app = App::new();

    // Entorno de prueba ECS con todos los plugins de juego y UI
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(bevy::asset::AssetPlugin::default())
        .init_asset::<Image>()
        .init_state::<AppState>()
        .add_sub_state::<CombatPhase>()
        .init_resource::<CurrentRunState>()
        .init_resource::<MetaProfile>()
        .add_plugins(GameUiPlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(MapPlugin)
        .add_plugins(ShopPlugin)
        .add_plugins(PokemonCenterPlugin)
        .add_plugins(slay_of_pokemon::gachapon::GachaponPlugin)
        .add_plugins(slay_of_pokemon::save::SavePlugin);

    // 1. Iniciar en ProfileSelect (pantalla previa donde se escoge el perfil antes del menú)
    app.update();
    assert_eq!(*app.world().resource::<State<AppState>>().get(), AppState::ProfileSelect);

    // Transición a MainMenu
    app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::MainMenu);
    app.update();
    assert_eq!(*app.world().resource::<State<AppState>>().get(), AppState::MainMenu);

    // 2. Transición a MapNavigation
    app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::MapNavigation);
    app.update();
    assert_eq!(*app.world().resource::<State<AppState>>().get(), AppState::MapNavigation);

    // 3. Transición a Shop
    app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::Shop);
    app.update();
    assert_eq!(*app.world().resource::<State<AppState>>().get(), AppState::Shop);

    // Verificar que la tienda generó 6 cartas y las reliquias
    let inv = app.world().get_resource::<ShopInventory>();
    assert!(inv.is_some(), "El inventario de la tienda debe existir al entrar a AppState::Shop");
    let inventory = inv.unwrap();
    assert_eq!(inventory.cards.len(), 6, "La tienda debe ofrecer exactamente 6 cartas");
    assert_eq!(inventory.relics.len(), 3, "La tienda debe ofrecer 3 reliquias");
    assert_eq!(inventory.consumables.len(), 3, "La tienda debe ofrecer 3 consumibles");

    // 4. Transición a PokemonCenter
    app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::PokemonCenter);
    app.update();
    assert_eq!(*app.world().resource::<State<AppState>>().get(), AppState::PokemonCenter);

    // 5. Transición a GachaponShop
    app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::GachaponShop);
    app.update();
    assert_eq!(*app.world().resource::<State<AppState>>().get(), AppState::GachaponShop);

    // 6. Transición a Combat
    app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::Combat);
    app.update();
    assert_eq!(*app.world().resource::<State<AppState>>().get(), AppState::Combat);
}

#[test]
fn test_pokemon_center_upgrade_system() {
    let mut run_state = CurrentRunState::default();
    run_state.player_deck = create_starter_deck(ElementType::Fire, true);

    assert!(!run_state.player_deck[0].upgraded);

    // Mejorar primera carta en el centro
    let upgraded = upgrade_card_at_center(&mut run_state, 0);
    assert!(upgraded);
    assert!(run_state.player_deck[0].upgraded);
    assert!(run_state.player_deck[0].display_name().ends_with('+'));

    // No se puede volver a mejorar la misma carta
    let upgrade_again = upgrade_card_at_center(&mut run_state, 0);
    assert!(!upgrade_again);
}

#[test]
fn test_shop_inventory_discounts_and_vip() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .init_state::<AppState>()
        .init_resource::<CurrentRunState>()
        .init_resource::<MetaProfile>();

    app.world_mut().resource_mut::<CurrentRunState>().poke_dollars = 300;
    app.world_mut().resource_mut::<CurrentRunState>().relics.push("vip_card".to_string());

    // Ejecuta setup_shop
    let _ = app.world_mut().run_system_once(setup_shop);
    app.update();

    let inv = app.world().get_resource::<ShopInventory>().expect("ShopInventory generado");
    assert_eq!(inv.cards.len(), 6);

    // Al menos 1 carta tiene descuento aplicado
    let has_discount = inv.cards.iter().any(|c| c.discount_percent > 0 && c.final_price < c.original_price);
    assert!(has_discount, "Debe haber al menos 1 carta en oferta con descuento en el Poké Mart");
}

// ============================================================
// Elección de nodos en el mapa y animaciones de combate
// ============================================================

use slay_of_pokemon::combat::events::{CombatAnimEvent, EndPlayerTurnEvent};
use slay_of_pokemon::map::CurrentMap;
use slay_of_pokemon::ui::combat_ui::FloatingText;

fn build_full_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(bevy::asset::AssetPlugin::default())
        .init_asset::<Image>()
        .init_state::<AppState>()
        .add_sub_state::<CombatPhase>()
        .init_resource::<CurrentRunState>()
        .init_resource::<MetaProfile>()
        .add_plugins(GameUiPlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(MapPlugin)
        .add_plugins(ShopPlugin)
        .add_plugins(PokemonCenterPlugin)
        .add_plugins(slay_of_pokemon::gachapon::GachaponPlugin)
        .add_plugins(slay_of_pokemon::save::SavePlugin);
    app
}

fn go_to(app: &mut App, state: AppState) {
    app.world_mut().resource_mut::<NextState<AppState>>().set(state);
    app.update();
}

#[test]
fn test_map_waits_for_player_choice() {
    let mut app = build_full_app();
    app.update();
    go_to(&mut app, AppState::MapNavigation);

    // Antes el mapa saltaba solo al nodo 0: ahora debe esperar la elección del jugador
    for _ in 0..5 {
        app.update();
    }
    assert_eq!(*app.world().resource::<State<AppState>>().get(), AppState::MapNavigation);
}

#[test]
fn test_map_reachable_nodes_follow_connections() {
    let mut map = CurrentMap::default();

    // Piso 1: se puede elegir cualquiera de los 3 caminos
    assert_eq!(map.reachable_indices(1), vec![0, 1, 2]);

    // Desde el camino 0 solo se llega a los caminos 0 y 1 del piso 2
    map.current_path_index = 0;
    assert_eq!(map.reachable_indices(2), vec![0, 1]);

    // Desde el camino central se puede ir a cualquiera
    map.current_path_index = 1;
    assert_eq!(map.reachable_indices(2), vec![0, 1, 2]);

    // Desde el camino 2 solo a los caminos 1 y 2
    map.current_path_index = 2;
    assert_eq!(map.reachable_indices(2), vec![1, 2]);

    // Todos los caminos del piso 14 convergen en el Líder de Gimnasio
    for i in 0..3 {
        map.current_path_index = i;
        assert_eq!(map.reachable_indices(15), vec![0]);
    }
}

#[test]
fn test_combat_uses_chosen_node_type() {
    use slay_of_pokemon::combat::EnemyPokemon;
    use slay_of_pokemon::map::generator::NodeType;
    use slay_of_pokemon::stats::PokemonStats;

    let mut app = build_full_app();
    app.update();

    // Fuerza que el nodo elegido en el piso 1 sea un Mini Jefe
    {
        let mut map = app.world_mut().resource_mut::<CurrentMap>();
        map.act_map.floors[0][0].node_type = NodeType::MiniBoss;
        map.current_path_index = 0;
    }
    go_to(&mut app, AppState::Combat);
    app.update();

    let mut q = app.world_mut().query_filtered::<&PokemonStats, With<EnemyPokemon>>();
    let enemy = q.single(app.world()).unwrap();
    assert_eq!(enemy.species_id, 123, "un Mini Jefe debe ser un Scyther salvaje");
}

#[test]
fn test_combat_animations_are_triggered() {
    let mut app = build_full_app();
    app.update();
    go_to(&mut app, AppState::Combat);
    app.update();
    app.update();

    // Terminar turno: el rival actúa y la lógica emite eventos de animación
    app.world_mut().write_message(EndPlayerTurnEvent);
    app.update();
    let emitted = app
        .world()
        .resource::<bevy::ecs::message::Messages<CombatAnimEvent>>()
        .len();
    assert!(emitted > 0, "la acción del rival debe emitir al menos una animación");

    // La UI traduce los eventos en textos flotantes sobre los sprites
    app.update();
    app.update();
    let mut q = app.world_mut().query::<&FloatingText>();
    assert!(q.iter(app.world()).count() > 0, "debe aparecer algún texto flotante");
}

#[test]
fn test_treasure_and_summary_screens() {
    let mut app = build_full_app();
    app.update();

    let gold_before = app.world().resource::<CurrentRunState>().poke_dollars;
    go_to(&mut app, AppState::Treasure);
    app.update();
    assert_eq!(*app.world().resource::<State<AppState>>().get(), AppState::Treasure);

    // El cofre siempre entrega algo (oro y/o una reliquia)
    let run = app.world().resource::<CurrentRunState>();
    assert!(run.poke_dollars > gold_before || !run.relics.is_empty());

    go_to(&mut app, AppState::RunSummary);
    app.update();
    assert_eq!(*app.world().resource::<State<AppState>>().get(), AppState::RunSummary);
}

// ============================================================
// Persistencia del HP entre combates
// ============================================================

use slay_of_pokemon::combat::{EnemyPokemon, PlayerPokemon};
use slay_of_pokemon::pokemon::create_pokemon_from_species;
use slay_of_pokemon::stats::PokemonStats;
use slay_of_pokemon::ui::center_ui::CenterHealButton;

#[test]
fn test_hp_is_kept_between_combats() {
    let mut app = build_full_app();
    app.update();

    // Un combate empieza con el HP guardado, no con el máximo
    app.world_mut().resource_mut::<CurrentRunState>().player_hp = Some(30);
    go_to(&mut app, AppState::Combat);
    app.update();

    let mut q = app.world_mut().query_filtered::<&mut PokemonStats, With<PlayerPokemon>>();
    assert_eq!(q.single(app.world()).unwrap().hp, 30);

    // Se deja el HP en 50 y se derrota al rival: al terminar el turno se guarda
    q.single_mut(app.world_mut()).unwrap().hp = 50;
    let mut qe = app.world_mut().query_filtered::<&mut PokemonStats, With<EnemyPokemon>>();
    qe.single_mut(app.world_mut()).unwrap().hp = 0;
    app.world_mut().write_message(EndPlayerTurnEvent);
    app.update();
    app.update();

    assert_eq!(app.world().resource::<CurrentRunState>().player_hp, Some(50));
    assert_eq!(*app.world().resource::<State<AppState>>().get(), AppState::Victory);

    // Al continuar desde la pantalla de victoria, avanza al mapa
    go_to(&mut app, AppState::MapNavigation);
    assert_eq!(*app.world().resource::<State<AppState>>().get(), AppState::MapNavigation);

    // El siguiente combate sigue con 50 HP
    go_to(&mut app, AppState::Combat);
    app.update();
    let mut q = app.world_mut().query_filtered::<&PokemonStats, With<PlayerPokemon>>();
    assert_eq!(q.single(app.world()).unwrap().hp, 50);
}

#[test]
fn test_pokemon_center_heals_persistent_hp() {
    let mut app = build_full_app();
    app.update();

    let (max_hp, level) = {
        let run = app.world().resource::<CurrentRunState>();
        let p = create_pokemon_from_species(run.selected_starter_id, run.player_level()).unwrap();
        (p.max_hp, p.level)
    };
    assert_eq!(level, 5);
    app.world_mut().resource_mut::<CurrentRunState>().player_hp = Some(10);

    go_to(&mut app, AppState::PokemonCenter);
    app.update();

    // Pulsa "Descansar"
    let mut q = app.world_mut().query_filtered::<&mut Interaction, With<CenterHealButton>>();
    *q.single_mut(app.world_mut()).unwrap() = Interaction::Pressed;
    app.update();

    let expected = (10 + ((max_hp as f32) * 0.35).round() as i32).min(max_hp);
    assert_eq!(app.world().resource::<CurrentRunState>().player_hp, Some(expected));
}

#[test]
fn test_combat_victory_immediately_without_end_turn() {
    let mut app = build_full_app();
    app.update();

    go_to(&mut app, AppState::Combat);
    app.update();

    let floor_before = app.world().resource::<CurrentRunState>().current_floor;

    // Reducimos el HP del rival a 0 directamente (simulando un ataque letal con carta)
    {
        let mut qe = app.world_mut().query_filtered::<&mut PokemonStats, With<EnemyPokemon>>();
        qe.single_mut(app.world_mut()).unwrap().hp = 0;
    }

    // Un update ejecuta check_combat_outcome y cambia NextState
    app.update();
    // El siguiente ciclo aplica la transición de estado de Bevy
    app.update();

    // Debe resolver la victoria de inmediato pasando a la pantalla de victoria e incrementando piso
    assert_eq!(*app.world().resource::<State<AppState>>().get(), AppState::Victory);
    assert_eq!(app.world().resource::<CurrentRunState>().current_floor, floor_before + 1);

    // Al pulsar continuar en la pantalla de victoria, avanza al mapa
    let mut q_btn = app.world_mut().query_filtered::<&mut Interaction, With<VictoryContinueButton>>();
    *q_btn.single_mut(app.world_mut()).unwrap() = Interaction::Pressed;
    app.update();
    if let Some(mut trans) = app.world_mut().get_resource_mut::<slay_of_pokemon::ui::transition_ui::SceneTransition>() {
        trans.progress = 1.0;
    }
    app.update();
    app.update();
    assert_eq!(*app.world().resource::<State<AppState>>().get(), AppState::MapNavigation);
}

#[test]
fn test_play_card_lethal_triggers_immediate_victory() {
    use slay_of_pokemon::combat::events::PlayCardEvent;

    let mut app = build_full_app();
    app.update();

    go_to(&mut app, AppState::Combat);
    app.update();

    let floor_before = app.world().resource::<CurrentRunState>().current_floor;

    // Ponemos al rival con 1 HP y 0 Bloqueo para que cualquier ataque sea letal
    {
        let mut qe = app.world_mut().query_filtered::<&mut PokemonStats, With<EnemyPokemon>>();
        let mut enemy = qe.single_mut(app.world_mut()).unwrap();
        enemy.hp = 1;
        enemy.block = 0;
    }

    // Buscamos una carta ofensiva en la mano para asegurar daño
    use slay_of_pokemon::card::InHand;
    use slay_of_pokemon::types::MoveCategory;
    let mut attack_index = None;
    {
        let mut q_cards = app.world_mut().query::<(&slay_of_pokemon::card::Card, &InHand)>();
        for (i, (card, _)) in q_cards.iter(app.world()).enumerate() {
            if card.category == MoveCategory::Physical || card.category == MoveCategory::Special {
                attack_index = Some(i);
                break;
            }
        }
    }
    let card_idx = attack_index.unwrap_or(0);

    // Simulamos clic real en el botón de la carta (lo que genera CardDiscardFlight) y jugamos la carta letal
    {
        use slay_of_pokemon::ui::combat_ui::CardButton;
        let mut q_btns = app.world_mut().query::<(&mut Interaction, &CardButton)>();
        for (mut inter, btn) in q_btns.iter_mut(app.world_mut()) {
            if btn.index == card_idx {
                *inter = Interaction::Pressed;
            }
        }
    }
    app.world_mut().write_message(PlayCardEvent { card_index: card_idx });

    // Ejecutamos actualizaciones: handle_play_card reduce HP a 0, check_combat_outcome resuelve victoria
    app.update();
    app.update();

    // El combate debe haber terminado de inmediato sin necesidad de emitir EndPlayerTurnEvent pasando a la pantalla de victoria
    assert_eq!(*app.world().resource::<State<AppState>>().get(), AppState::Victory);
    assert_eq!(app.world().resource::<CurrentRunState>().current_floor, floor_before + 1);

    // Verificar que ninguna carta en vuelo al descarte (CardDiscardFlight) quede fija en Victory u otras pantallas
    use slay_of_pokemon::ui::combat_ui::{CardDiscardFlight, DeckShuffleFlight, DeckShuffleBanner};
    let stuck_discard_cards = app.world_mut().query::<&CardDiscardFlight>().iter(app.world()).count();
    let stuck_shuffle_cards = app.world_mut().query::<&DeckShuffleFlight>().iter(app.world()).count();
    let stuck_shuffle_banners = app.world_mut().query::<&DeckShuffleBanner>().iter(app.world()).count();
    assert_eq!(stuck_discard_cards, 0, "La carta que derrota al oponente NO debe quedar fija en la pantalla de Victoria ni en el resto de pantallas");
    assert_eq!(stuck_shuffle_cards, 0, "No deben quedar cartas de barajado colgadas fuera de combate");
    assert_eq!(stuck_shuffle_banners, 0, "No debe quedar el banner de barajado colgado fuera de combate");
}

#[test]
fn test_scene_transition_lifecycle() {
    use slay_of_pokemon::ui::transition_ui::{SceneTransition, TransitionToEvent, TransitionPhase};

    let mut app = build_full_app();
    app.update();

    // Estado inicial en ProfileSelect antes de pasar al menú principal
    assert_eq!(*app.world().resource::<State<AppState>>().get(), AppState::ProfileSelect);
    assert!(app.world().get_resource::<SceneTransition>().is_some());

    // Emitir mensaje para transicionar a MapNavigation
    app.world_mut().write_message(TransitionToEvent {
        target: AppState::MapNavigation,
    });

    // Procesa el mensaje e inicia la fase de FadeOut
    app.update();
    assert_eq!(app.world().resource::<SceneTransition>().phase, TransitionPhase::FadeOut);
    assert_eq!(app.world().resource::<SceneTransition>().banner_title, "MAPA DE KANTO");

    // Simulamos la finalización del FadeOut
    app.world_mut().resource_mut::<SceneTransition>().progress = 1.0;
    app.update(); // Transiciona a FadeIn y setea NextState
    app.update(); // Bevy aplica el nuevo estado

    // Verificamos que el sistema avanzó exitosamente al nuevo estado
    assert_eq!(*app.world().resource::<State<AppState>>().get(), AppState::MapNavigation);
}

#[test]
fn test_victory_screen_rewards_and_celebration_animation() {
    let mut app = build_full_app();
    app.update();

    go_to(&mut app, AppState::Combat);
    app.update();

    let dollars_before = app.world().resource::<CurrentRunState>().poke_dollars;
    let tokens_before = app.world().resource::<MetaProfile>().gachapon_tokens;

    // Derrotar al rival
    {
        let mut qe = app.world_mut().query_filtered::<&mut PokemonStats, With<EnemyPokemon>>();
        qe.single_mut(app.world_mut()).unwrap().hp = 0;
    }
    app.update();
    app.update();

    // Debe encontrarse en AppState::Victory
    assert_eq!(*app.world().resource::<State<AppState>>().get(), AppState::Victory);

    // Deben existir los sprites animados de celebración (Entrenador y Pokémon)
    let mut q_anim = app.world_mut().query::<&VictoryCelebrationSprite>();
    assert_eq!(q_anim.iter(app.world()).count(), 2, "Deben instanciarse los sprites animados de victoria del Entrenador y del Pokémon");

    // Los datos de victoria deben haberse poblado
    let v_data = app.world().get_resource::<slay_of_pokemon::combat::CombatVictoryData>().expect("CombatVictoryData generado");
    assert!(v_data.exp_gained > 0);
    assert!(v_data.money_gained > 0);
    assert_eq!(v_data.reward_cards.len(), 3, "Debe ofrecer 3 cartas de recompensa");

    // Los Poké-dólares y tokens deben haber aumentado
    assert!(app.world().resource::<CurrentRunState>().poke_dollars > dollars_before);
    assert!(app.world().resource::<MetaProfile>().gachapon_tokens > tokens_before);
}

#[test]
fn test_victory_screen_card_reward_selection() {
    let mut app = build_full_app();
    app.update();

    go_to(&mut app, AppState::Combat);
    app.update();

    let initial_deck_len = app.world().resource::<CurrentRunState>().player_deck.len();

    // Derrotar al rival
    {
        let mut qe = app.world_mut().query_filtered::<&mut PokemonStats, With<EnemyPokemon>>();
        qe.single_mut(app.world_mut()).unwrap().hp = 0;
    }
    app.update();
    app.update();

    assert_eq!(*app.world().resource::<State<AppState>>().get(), AppState::Victory);

    // Seleccionar la primera carta de recompensa
    {
        let mut q_cards = app.world_mut().query::<(&mut Interaction, &VictoryRewardCardButton)>();
        for (mut interaction, btn) in q_cards.iter_mut(app.world_mut()) {
            if btn.index == 0 {
                *interaction = Interaction::Pressed;
                break;
            }
        }
    }
    app.update();

    // El mazo debe tener 1 carta más
    assert_eq!(
        app.world().resource::<CurrentRunState>().player_deck.len(),
        initial_deck_len + 1,
        "La carta elegida debe añadirse al mazo del jugador"
    );

    let v_data = app.world().get_resource::<slay_of_pokemon::combat::CombatVictoryData>().unwrap();
    assert!(v_data.card_chosen, "El estado debe marcar que una carta fue elegida");
}

#[test]
fn test_common_trainer_pokemon_pool_variety() {
    use std::collections::HashSet;
    use slay_of_pokemon::combat::roll_enemy_encounter;
    use slay_of_pokemon::map::generator::NodeType;

    // Ejecuta múltiples tiradas para el Acto 1 y comprueba variedad
    let mut species_set = HashSet::new();
    let mut trainers_set = HashSet::new();

    for _ in 0..50 {
        let (species_id, _, trainer) = roll_enemy_encounter(NodeType::Enemy, 1, 1, 5);
        species_set.insert(species_id);
        trainers_set.insert(trainer.name);
    }

    // Debe haber múltiples Pokémon distintos (no solo Geodude)
    assert!(
        species_set.len() >= 4,
        "La pool de entrenadores comunes del Acto 1 debe contener al menos 4 especies distintas, encontradas: {:?}",
        species_set
    );
    // Debe incluir más de un entrenador distinto
    assert!(
        trainers_set.len() >= 4,
        "Debe haber variedad de entrenadores comunes, encontrados: {:?}",
        trainers_set
    );

    // En el Acto 2, debe incluir especies evolucionadas (Charmeleon 5, Wartortle 8, Ivysaur 2, Raichu 26)
    let mut act2_species = HashSet::new();
    for _ in 0..50 {
        let (species_id, _, _) = roll_enemy_encounter(NodeType::Enemy, 2, 16, 15);
        act2_species.insert(species_id);
    }
    assert!(
        act2_species.contains(&5) || act2_species.contains(&8) || act2_species.contains(&2) || act2_species.contains(&26),
        "El Acto 2 debe incluir Pokémon evolucionados en la pool de entrenadores"
    );
}

#[test]
fn test_mini_boss_pokemon_pool_variety() {
    use std::collections::HashSet;
    use slay_of_pokemon::combat::roll_enemy_encounter;
    use slay_of_pokemon::map::generator::NodeType;

    // En piso 1, se garantiza Scyther (123) para compatibilidad
    let (piso1_species, _, _) = roll_enemy_encounter(NodeType::MiniBoss, 1, 1, 5);
    assert_eq!(piso1_species, 123);

    // En pisos avanzados del Acto 1 (ej. piso 6), debe haber variedad de Mini Jefes
    let mut act1_miniboss_species = HashSet::new();
    for _ in 0..60 {
        let (species_id, _, _) = roll_enemy_encounter(NodeType::MiniBoss, 1, 6, 8);
        act1_miniboss_species.insert(species_id);
    }
    assert!(
        act1_miniboss_species.len() >= 3,
        "La pool de Mini Jefes del Acto 1 debe contener al menos 3 especies distintas, encontradas: {:?}",
        act1_miniboss_species
    );

    // En el Acto 2, debe incluir variedad con formas finales o poderosas
    let mut act2_miniboss_species = HashSet::new();
    for _ in 0..60 {
        let (species_id, _, _) = roll_enemy_encounter(NodeType::MiniBoss, 2, 20, 20);
        act2_miniboss_species.insert(species_id);
    }
    assert!(
        act2_miniboss_species.len() >= 3,
        "La pool de Mini Jefes del Acto 2 debe contener al menos 3 especies distintas, encontradas: {:?}",
        act2_miniboss_species
    );
}

#[test]
fn test_status_effects_hit_chance_and_miss_rate() {
    use slay_of_pokemon::card::{Card, InHand};
    use slay_of_pokemon::types::{ElementType, MoveCategory, RarityTier, StatusType};
    use slay_of_pokemon::combat::events::PlayCardEvent;

    // 1. Verificar getters de probabilidad de estado
    let attack_card = Card::new("ember", "Ascuas", ElementType::Fire, MoveCategory::Special, RarityTier::Common, 1, 1.1);
    assert_eq!(attack_card.get_status_chance(), 0.60);

    let skill_card = Card::new("leech_seed", "Drenadoras", ElementType::Grass, MoveCategory::Skill, RarityTier::Uncommon, 1, 0.0);
    assert_eq!(skill_card.get_status_chance(), 0.75);

    let mut custom_card = Card::new("test", "Test", ElementType::Normal, MoveCategory::Physical, RarityTier::Common, 1, 1.0);
    custom_card.status_chance = Some(0.40);
    assert_eq!(custom_card.get_status_chance(), 0.40);

    // 2. Verificar descripción con porcentaje
    let mut desc_card = Card::new("ember", "Ascuas", ElementType::Fire, MoveCategory::Special, RarityTier::Common, 1, 1.1);
    desc_card.secondary_status = Some((StatusType::Burn, 2));
    assert!(desc_card.description().contains("Quemadura: 2 (60%)"));

    // 3. Verificar en combate que los efectos de estado no aplican siempre al 100%
    let mut app = build_full_app();
    app.update();
    go_to(&mut app, AppState::Combat);
    app.update();

    let mut hits = 0;
    let mut misses = 0;

    for _ in 0..100 {
        // Otorgar energía suficiente
        app.world_mut().resource_mut::<CurrentRunState>().energy = 10;

        // Limpiar estados y asegurar vida alta en el rival
        {
            let mut qe = app.world_mut().query_filtered::<&mut PokemonStats, With<EnemyPokemon>>();
            let mut enemy = qe.single_mut(app.world_mut()).unwrap();
            enemy.hp = 999;
            enemy.max_hp = 999;
            enemy.statuses.clear();
        }

        // Spawnear una carta con Quemadura en la mano
        let mut burn_card = Card::new("ember", "Ascuas", ElementType::Fire, MoveCategory::Special, RarityTier::Common, 1, 1.0);
        burn_card.secondary_status = Some((StatusType::Burn, 2));

        let card_entity = app.world_mut().spawn((burn_card, InHand)).id();

        // Obtener el índice de la carta en la mano
        let mut idx = 0;
        {
            let mut q_cards = app.world_mut().query::<(Entity, &InHand)>();
            for (i, (e, _)) in q_cards.iter(app.world()).enumerate() {
                if e == card_entity {
                    idx = i;
                    break;
                }
            }
        }

        app.world_mut().write_message(PlayCardEvent { card_index: idx });
        app.update();

        // Verificar si aplicó estado o falló
        {
            let mut qe = app.world_mut().query_filtered::<&PokemonStats, With<EnemyPokemon>>();
            let enemy = qe.single(app.world()).unwrap();
            if enemy.statuses.contains_key(&StatusType::Burn) {
                hits += 1;
            } else {
                misses += 1;
            }
        }
    }

    assert!(hits > 0, "Debe haber al menos un acierto de estado en 100 intentos");
    assert!(misses > 0, "No debe aplicar siempre al 100%: debe haber fallos de estado");
}

#[test]
fn test_pokemon_selector_switches_starter_and_deck() {
    use slay_of_pokemon::ui::starter_selection_ui::StarterCardButton;
    use slay_of_pokemon::ui::gachapon_ui::GachaponSelectPokemonBadgeButton;

    let mut app = build_full_app();
    app.update();

    // 1. Simular que el jugador desbloqueó a Squirtle (#7) y Bulbasaur (#1) en el Gachapón
    {
        let mut meta = app.world_mut().resource_mut::<MetaProfile>();
        if !meta.unlocked_pokemon.contains(&7) {
            meta.unlocked_pokemon.push(7);
        }
        if !meta.unlocked_pokemon.contains(&1) {
            meta.unlocked_pokemon.push(1);
        }
    }

    // Inicialmente el seleccionado es Charmander (#4)
    assert_eq!(app.world().resource::<CurrentRunState>().selected_starter_id, 4);

    // 2. Entrar a StarterSelection
    go_to(&mut app, AppState::StarterSelection);
    app.update();

    // Verificar que existen tarjetas para los Pokémon desbloqueados
    {
        let mut q_cards = app.world_mut().query::<&StarterCardButton>();
        let count = q_cards.iter(app.world()).count();
        assert!(count >= 3, "Debe haber tarjetas para Charmander, Squirtle y Bulbasaur");
    }

    // Simular que el jugador hace clic en Squirtle (#7)
    {
        let mut q_cards = app.world_mut().query::<(&mut Interaction, &StarterCardButton)>();
        for (mut interaction, btn) in q_cards.iter_mut(app.world_mut()) {
            if btn.0 == 7 {
                *interaction = Interaction::Pressed;
            }
        }
    }
    app.update();

    // Comprobar que el Pokémon activo cambió a Squirtle (#7) y su mazo ahora tiene Pistola Agua
    let run = app.world().resource::<CurrentRunState>();
    assert_eq!(run.selected_starter_id, 7);
    assert!(run.player_deck.iter().any(|c| c.id == "water_gun"), "El mazo de Squirtle debe tener Pistola Agua");

    // 3. Probar selección directa desde la tienda Gachapón
    go_to(&mut app, AppState::GachaponShop);
    app.update();

    // Simular clic en Bulbasaur (#1) en la colección desbloqueada
    {
        let mut q_badges = app.world_mut().query::<(&mut Interaction, &GachaponSelectPokemonBadgeButton)>();
        for (mut interaction, btn) in q_badges.iter_mut(app.world_mut()) {
            if btn.0 == 1 {
                *interaction = Interaction::Pressed;
            }
        }
    }
    app.update();

    let run_after = app.world().resource::<CurrentRunState>();
    assert_eq!(run_after.selected_starter_id, 1);
    assert!(run_after.player_deck.iter().any(|c| c.id == "vine_whip"), "El mazo de Bulbasaur debe tener Látigo Cepa");
}

#[test]
fn test_enemy_intent_vs_active_status_display() {
    use slay_of_pokemon::combat::ai::{EnemyAI, EnemyIntentType};
    use slay_of_pokemon::combat::EnemyPokemon;
    use slay_of_pokemon::pokemon::create_pokemon_from_species;
    use slay_of_pokemon::stats::PokemonStats;
    use slay_of_pokemon::types::{ElementType, StatusType};
    use slay_of_pokemon::ui::combat_ui::{EnemyIntentText, EnemyStatusesText};

    // 1. Verificar que la IA no asigna Burn indiscriminadamente a Pokémon no fuego
    let mut geodude_stats = create_pokemon_from_species(74, 5).unwrap();
    geodude_stats.primary_types = vec![ElementType::Rock, ElementType::Ground];
    let mut geodude_ai = EnemyAI::new(false, false);

    // Forzar turno para debuff/buff
    for _ in 0..20 {
        let intent = geodude_ai.decide_next_intent(&geodude_stats);
        if let EnemyIntentType::Debuff { status, .. } = intent {
            assert_ne!(
                status,
                StatusType::Burn,
                "Geodude (Roca/Tierra) no debe tener la intención de aplicar Quemadura"
            );
        }
    }

    // 2. Probar HUD en combate: Intención anunciada vs Estados activos
    let mut app = build_full_app();
    app.update();
    go_to(&mut app, AppState::Combat);
    app.update();

    // Establecer intención explícita de debuff en el enemigo
    {
        let mut q_ai = app.world_mut().query::<&mut EnemyAI>();
        let mut ai = q_ai.single_mut(app.world_mut()).unwrap();
        ai.current_intent = Some(EnemyIntentType::Debuff {
            status: StatusType::Burn,
            stacks: 2,
        });
    }

    app.update();

    // Comprobar que EnemyIntentText muestra claramente la intención (acción futura hacia el jugador)
    // y no la etiqueta engañosa "[ESTADO] Quemadura"
    {
        let mut q_intent = app.world_mut().query::<(&Text, &EnemyIntentText)>();
        let (intent_text, _) = q_intent.single(app.world()).unwrap();
        assert!(
            intent_text.as_str().contains("[ACCIÓN RIVAL] Te lanzará: Quemadura (+2)"),
            "La intención del enemigo debe indicar claramente la acción hacia el jugador: actual='{}'",
            intent_text.as_str()
        );
        assert!(
            !intent_text.as_str().starts_with("[ESTADO]"),
            "La intención no debe usar el prefijo ambiguo '[ESTADO]'"
        );
    }

    // Comprobar que los estados activos del rival están vacíos si no ha sido afectado
    {
        let mut q_statuses = app.world_mut().query::<(&Text, &EnemyStatusesText)>();
        let (status_text, _) = q_statuses.single(app.world()).unwrap();
        assert_eq!(
            status_text.as_str(), "",
            "El rival sin estados activos no debe mostrar nada en su línea de estados"
        );
    }

    // 3. Aplicar estado activo real al rival y comprobar que ahora sí aparece en [ESTADOS]
    {
        let mut q_enemy = app.world_mut().query_filtered::<&mut PokemonStats, With<EnemyPokemon>>();
        let mut enemy = q_enemy.single_mut(app.world_mut()).unwrap();
        enemy.apply_status(StatusType::Burn, 3);
    }

    app.update();

    {
        let mut q_statuses = app.world_mut().query::<(&Text, &EnemyStatusesText)>();
        let (status_text, _) = q_statuses.single(app.world()).unwrap();
        assert_eq!(
            status_text.as_str(),
            "[ESTADOS RIVAL] Quemadura: 3",
            "Los estados activos reales deben mostrarse en el texto de estados del rival"
        );
    }
}
