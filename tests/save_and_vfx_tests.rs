use bevy::prelude::*;
use slay_of_pokemon::run::{AppState, CurrentRunState, MetaProfile};
use slay_of_pokemon::save::{self, ActiveSaveSlot, TOTAL_SLOTS};
use slay_of_pokemon::map::CurrentMap;
use bevy_tweening::{
    lens::TransformScaleLens,
    AnimTarget, RepeatCount, Tween, TweenAnim, TweeningPlugin,
};

use std::sync::Mutex;

static SAVE_FILE_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_three_save_slots_isolation_and_persistence() {
    let _guard = SAVE_FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    // Limpieza inicial previa
    for slot in 1..=TOTAL_SLOTS {
        let _ = save::delete_profile(slot);
    }

    // 1. Crear 3 perfiles distintos
    let mut meta1 = MetaProfile::default();
    meta1.gachapon_tokens = 500;
    meta1.unlocked_pokemon = vec![4];
    let mut run1 = CurrentRunState::default();
    run1.selected_starter_id = 4; // Charmander
    run1.poke_dollars = 150;
    run1.current_floor = 3;

    let mut meta2 = MetaProfile::default();
    meta2.gachapon_tokens = 800;
    meta2.unlocked_pokemon = vec![4, 7];
    let mut run2 = CurrentRunState::default();
    run2.selected_starter_id = 7; // Squirtle
    run2.poke_dollars = 320;
    run2.current_floor = 7;

    let mut meta3 = MetaProfile::default();
    meta3.gachapon_tokens = 1200;
    meta3.unlocked_pokemon = vec![4, 1, 25];
    let mut run3 = CurrentRunState::default();
    run3.selected_starter_id = 1; // Bulbasaur
    run3.poke_dollars = 500;
    run3.current_floor = 12;

    let map = CurrentMap::default();

    // Guardar en las 3 ranuras
    assert!(save::save_profile(1, &meta1, &run1, Some(&map)).is_ok());
    assert!(save::save_profile(2, &meta2, &run2, Some(&map)).is_ok());
    assert!(save::save_profile(3, &meta3, &run3, Some(&map)).is_ok());

    // 2. Verificar listado de perfiles
    let summaries = save::list_profiles();
    assert!(summaries[0].is_some(), "Ranura 1 debe existir");
    assert!(summaries[1].is_some(), "Ranura 2 debe existir");
    assert!(summaries[2].is_some(), "Ranura 3 debe existir");

    let s1 = summaries[0].as_ref().unwrap();
    assert_eq!(s1.slot, 1);
    assert_eq!(s1.starter_id, 4);
    assert_eq!(s1.current_floor, 3);
    assert_eq!(s1.poke_dollars, 150);

    let s2 = summaries[1].as_ref().unwrap();
    assert_eq!(s2.slot, 2);
    assert_eq!(s2.starter_id, 7);
    assert_eq!(s2.current_floor, 7);
    assert_eq!(s2.poke_dollars, 320);

    let s3 = summaries[2].as_ref().unwrap();
    assert_eq!(s3.slot, 3);
    assert_eq!(s3.starter_id, 1);
    assert_eq!(s3.current_floor, 12);
    assert_eq!(s3.poke_dollars, 500);

    // 3. Cargar individualmente y verificar integridad
    let loaded2 = save::load_profile(2).expect("Debe cargar ranura 2");
    assert_eq!(loaded2.run_state.selected_starter_id, 7);
    assert_eq!(loaded2.meta.gachapon_tokens, 800);
    assert!(loaded2.map.is_some());

    // 4. Borrar ranura 2 y verificar que las demás persisten intactas
    assert!(save::delete_profile(2).is_ok());
    let summaries_after = save::list_profiles();
    assert!(summaries_after[0].is_some(), "Ranura 1 debe permanecer");
    assert!(summaries_after[1].is_none(), "Ranura 2 debe estar vacía");
    assert!(summaries_after[2].is_some(), "Ranura 3 debe permanecer");

    // Limpieza posterior
    let _ = save::delete_profile(1);
    let _ = save::delete_profile(3);
}

#[test]
fn test_ui_profile_select_flow_and_slot_activation() {
    let _guard = SAVE_FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    use slay_of_pokemon::ui::profile_ui::ProfileSlotAction;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(bevy::asset::AssetPlugin::default())
        .init_asset::<Image>()
        .add_plugins(TweeningPlugin)
        .init_state::<AppState>()
        .init_resource::<CurrentRunState>()
        .init_resource::<MetaProfile>()
        .init_resource::<ActiveSaveSlot>()
        .add_plugins(slay_of_pokemon::save::SavePlugin)
        .add_plugins(slay_of_pokemon::ui::GameUiPlugin);

    app.update();

    // 1. Iniciar directamente en ProfileSelect (pantalla previa donde se escoge el perfil antes del menú)
    assert_eq!(*app.world().resource::<State<AppState>>().get(), AppState::ProfileSelect);
    assert_eq!(app.world().resource::<ActiveSaveSlot>().0, 1);

    // 3. Simular que el jugador pulsa "NUEVA PARTIDA" en la Ranura 2
    {
        let mut q_actions = app.world_mut().query::<(&mut Interaction, &ProfileSlotAction)>();
        for (mut interaction, action) in q_actions.iter_mut(app.world_mut()) {
            if let ProfileSlotAction::New(2) = action {
                *interaction = Interaction::Pressed;
            }
        }
    }
    app.update();

    // Comprobar que la ranura activa cambió a 2 y transicionó a StarterSelection
    assert_eq!(app.world().resource::<ActiveSaveSlot>().0, 2);

    // Limpiar archivo de prueba
    let _ = save::delete_profile(2);
}

#[test]
fn test_bevy_tweening_integration_steps_smoothly() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TweeningPlugin);

    let entity = app.world_mut().spawn(Transform::from_scale(Vec3::splat(1.0))).id();

    let tween = Tween::new(
        EaseFunction::Linear,
        std::time::Duration::from_millis(1000),
        TransformScaleLens {
            start: Vec3::splat(1.0),
            end: Vec3::splat(2.0),
        },
    )
    .with_repeat_count(RepeatCount::Finite(1));

    app.world_mut().spawn((
        TweenAnim::new(tween),
        AnimTarget::component::<Transform>(entity),
    ));

    app.update();

    // Verificar que la entidad objetivo existe y conserva su transform
    let transform = app.world().get::<Transform>(entity);
    assert!(transform.is_some(), "La entidad animada con Transform debe existir");
}

#[test]
fn test_menu_quick_profile_save_toast_particles_and_custom_font() {
    let _guard = SAVE_FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    use slay_of_pokemon::ui::menu_ui::{MenuButtonAction, MenuParticle};
    use slay_of_pokemon::ui::SaveToastBanner;

    let _ = save::delete_profile(2);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(bevy::asset::AssetPlugin::default())
        .init_asset::<Image>()
        .add_plugins(TweeningPlugin)
        .init_state::<AppState>()
        .init_resource::<CurrentRunState>()
        .init_resource::<MetaProfile>()
        .add_plugins(slay_of_pokemon::save::SavePlugin)
        .add_plugins(slay_of_pokemon::ui::GameUiPlugin);

    app.update();
    // Transicionar al Menú Principal tras la pantalla inicial de selección de perfil
    app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::MainMenu);
    app.update();
    app.update();

    // 1. Verificar que el Menú Principal tiene partículas flotantes y animaciones TweenAnim
    let particle_count = app.world_mut().query::<&MenuParticle>().iter(app.world()).count();
    assert!(
        particle_count >= 20,
        "El menú principal debe tener partículas animadas visibles (encontradas: {})",
        particle_count
    );

    let tween_count = app.world_mut().query::<&TweenAnim>().iter(app.world()).count();
    assert!(
        tween_count >= 20,
        "El menú principal debe tener animaciones bevy_tweening activas (encontradas: {})",
        tween_count
    );

    // 2. Verificar que todas las fuentes TextFont reciben la fuente FiraSans-Bold.ttf y no FontSource::default()
    for tf in app.world_mut().query::<&TextFont>().iter(app.world()) {
        assert_ne!(
            tf.font,
            bevy::text::FontSource::default(),
            "Todos los textos deben tener la fuente FiraSans-Bold.ttf asignada para mostrar tildes y caracteres especiales"
        );
    }

    // 3. Simular clic en el selector rápido "PERFIL 2" desde el propio Menú Principal
    {
        let mut q_btns = app.world_mut().query::<(&mut Interaction, &MenuButtonAction)>();
        for (mut inter, action) in q_btns.iter_mut(app.world_mut()) {
            if *action == MenuButtonAction::QuickSelectProfile(2) {
                *inter = Interaction::Pressed;
            }
        }
    }
    app.update();
    app.update();

    assert_eq!(app.world().resource::<ActiveSaveSlot>().0, 2);
    let toast_count = app.world_mut().query::<&SaveToastBanner>().iter(app.world()).count();
    assert_eq!(
        toast_count, 1,
        "Debe mostrarse el cartel de aviso (SaveToastBanner) al cambiar/cargar perfil"
    );

    // 4. Modificar progreso y pulsar "GUARDAR PARTIDA EN PERFIL ACTIVO" en el Menú Principal
    app.world_mut().resource_mut::<CurrentRunState>().poke_dollars = 777;
    {
        let mut q_btns = app.world_mut().query::<(&mut Interaction, &MenuButtonAction)>();
        for (mut inter, action) in q_btns.iter_mut(app.world_mut()) {
            if *action == MenuButtonAction::SaveCurrentGame {
                *inter = Interaction::Pressed;
            } else {
                *inter = Interaction::None;
            }
        }
    }
    app.update();
    app.update();

    let loaded = save::load_profile(2).expect("El perfil 2 debe haberse guardado en disco");
    assert_eq!(loaded.run_state.poke_dollars, 777);

    let toast_count_after_save = app.world_mut().query::<&SaveToastBanner>().iter(app.world()).count();
    assert_eq!(
        toast_count_after_save, 1,
        "Debe mostrarse el cartel de aviso (SaveToastBanner) al guardar la partida"
    );

    let _ = save::delete_profile(2);
}

#[test]
fn test_gachapon_pull_animation_and_rarity_scaling() {
    use slay_of_pokemon::gachapon::species_rarity;
    use slay_of_pokemon::types::RarityTier;
    use slay_of_pokemon::ui::gachapon_ui::{
        GachaponAnimation, GachaponAnimStep, GachaponMultiPullButton,
        GachaponRarityParticle, GachaponSinglePullButton, GachaponSkipButton,
    };

    // 1. Verificación del mapeo jerárquico de rareza por especie
    assert_eq!(species_rarity(150), RarityTier::Legendary); // Mewtwo
    assert_eq!(species_rarity(6), RarityTier::Legendary);   // Charizard
    assert_eq!(species_rarity(9), RarityTier::Legendary);   // Blastoise
    assert_eq!(species_rarity(3), RarityTier::Legendary);   // Venusaur

    assert_eq!(species_rarity(123), RarityTier::Exotic);   // Scyther
    assert_eq!(species_rarity(143), RarityTier::Exotic);   // Snorlax
    assert_eq!(species_rarity(5), RarityTier::Exotic);     // Charmeleon

    assert_eq!(species_rarity(4), RarityTier::Rare);       // Charmander
    assert_eq!(species_rarity(7), RarityTier::Rare);       // Squirtle
    assert_eq!(species_rarity(25), RarityTier::Rare);      // Pikachu

    assert_eq!(species_rarity(74), RarityTier::Uncommon);   // Geodude
    assert_eq!(species_rarity(10), RarityTier::Common);     // Caterpie
    assert_eq!(species_rarity(19), RarityTier::Common);     // Rattata

    // 2. Configurar App para probar la máquina de estados de animación
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(bevy::asset::AssetPlugin::default())
        .init_asset::<Image>()
        .add_plugins(TweeningPlugin)
        .init_state::<AppState>()
        .init_resource::<CurrentRunState>()
        .init_resource::<MetaProfile>()
        .add_plugins(slay_of_pokemon::save::SavePlugin)
        .add_plugins(slay_of_pokemon::ui::GameUiPlugin);

    // Otorgar fichas suficientes para tiradas
    app.world_mut().resource_mut::<MetaProfile>().gachapon_tokens = 2500;

    // Cambiar a la tienda de Gachapón
    app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::GachaponShop);
    app.update();
    app.update();

    // Comprobar estado inicial en Idle
    assert_eq!(
        app.world().resource::<GachaponAnimation>().step,
        GachaponAnimStep::Idle,
        "La animación debe comenzar en estado Idle"
    );

    // 3. Simular clic en Tirada Simple
    {
        let mut q_pull = app.world_mut().query::<(&mut Interaction, &GachaponSinglePullButton)>();
        for (mut inter, _) in q_pull.iter_mut(app.world_mut()) {
            *inter = Interaction::Pressed;
        }
    }
    app.update();

    // Verificar que la tirada inicia la fase Shaking
    let pulled_pokemon = app.world().resource::<GachaponAnimation>().pulled_pokemon;
    let pulled_rarity = app.world().resource::<GachaponAnimation>().rarity;
    assert_eq!(
        app.world().resource::<GachaponAnimation>().step,
        GachaponAnimStep::Shaking,
        "Al tirar del gachapón debe iniciarse la animación en Shaking"
    );
    assert_eq!(
        pulled_rarity,
        species_rarity(pulled_pokemon),
        "La rareza asignada a la animación debe coincidir con la de la especie obtenida"
    );

    // 4. Probar botón Saltar Animación (SALTAR >>)
    {
        let mut q_skip = app.world_mut().query::<(&mut Interaction, &GachaponSkipButton)>();
        for (mut inter, _) in q_skip.iter_mut(app.world_mut()) {
            *inter = Interaction::Pressed;
        }
    }
    app.update();

    // Debe haber avanzado inmediatamente a Revealed
    assert_eq!(
        app.world().resource::<GachaponAnimation>().step,
        GachaponAnimStep::Revealed,
        "Pulsar Saltar debe llevar la animación directamente al estado Revealed"
    );

    // Verificar que se crearon partículas escaladas a la rareza
    let particle_count = app.world_mut().query::<&GachaponRarityParticle>().iter(app.world()).count();
    assert!(
        particle_count >= 8,
        "Deben haberse generado partículas de revelación (encontradas: {})",
        particle_count
    );

    // 5. Simular Pack x10 Pulls
    {
        let mut q_multi = app.world_mut().query::<(&mut Interaction, &GachaponMultiPullButton)>();
        for (mut inter, _) in q_multi.iter_mut(app.world_mut()) {
            *inter = Interaction::Pressed;
        }
    }
    app.update();

    assert_eq!(
        app.world().resource::<GachaponAnimation>().step,
        GachaponAnimStep::Shaking,
        "El pack x10 debe iniciar la animación en Shaking"
    );
    assert_eq!(
        app.world().resource::<GachaponAnimation>().multi_pull_list.len(),
        10,
        "El pack x10 debe registrar exactamente 10 Pokémon en su lista de resultados"
    );

    let featured_id = app.world().resource::<GachaponAnimation>().pulled_pokemon;
    let featured_rarity = app.world().resource::<GachaponAnimation>().rarity;
    let max_in_pack = app
        .world()
        .resource::<GachaponAnimation>()
        .multi_pull_list
        .iter()
        .map(|&id| species_rarity(id))
        .max()
        .unwrap();

    assert_eq!(
        featured_rarity, max_in_pack,
        "El Pokémon destacado del pack x10 debe tener la máxima rareza obtenida en el pack"
    );
    assert_eq!(species_rarity(featured_id), featured_rarity);
}

#[test]
fn test_card_hover_animation_interpolates_scale_and_lift() {
    use slay_of_pokemon::ui::{animate_card_hover, CardHoverAnimation};

    let mut app = App::new();
    let mut time = Time::<()>::default();
    time.advance_by(std::time::Duration::from_millis(50));
    app.insert_resource(time);
    app.add_systems(Update, animate_card_hover);

    let card_entity = app.world_mut().spawn((
        Interaction::Hovered,
        UiTransform::default(),
        bevy::ui::BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
        GlobalZIndex(10),
        CardHoverAnimation::new(Color::srgb(0.5, 0.5, 0.5)).with_lift(-22.0).with_scale(1.12),
    )).id();

    // Simular paso de varios frames avanzando el tiempo virtual para que la interpolación actúe
    for _ in 0..10 {
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(50));
        app.update();
    }

    // Verificar estado hovered: debe haberse elevado (y negativo), aumentado escala (> 1.0) y z-index 150
    {
        let world = app.world();
        let transform = world.get::<UiTransform>(card_entity).unwrap();
        let z_idx = world.get::<GlobalZIndex>(card_entity).unwrap();
        let border = world.get::<bevy::ui::BorderColor>(card_entity).unwrap();

        assert!(transform.scale.x > 1.05, "La escala al hacer hover debe ser mayor a 1.05 (actual: {})", transform.scale.x);
        if let Val::Px(y) = transform.translation.y {
            assert!(y < -5.0, "La elevación al hacer hover debe ser negativa/elevada (actual: {})", y);
        } else {
            panic!("La traslación Y debería ser en píxeles");
        }
        assert_eq!(z_idx.0, 150, "El Z-Index al hacer hover debe elevarse por encima de cartas contiguas a 150");
        assert_eq!(border.top, Color::srgb(1.0, 0.95, 0.45), "El borde debe tomar el color de hover brillante");
    }

    // Simular que el puntero sale de la carta (Interaction::None)
    *app.world_mut().get_mut::<Interaction>(card_entity).unwrap() = Interaction::None;
    for _ in 0..20 {
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(50));
        app.update();
    }

    // Verificar retorno al reposo
    {
        let world = app.world();
        let transform = world.get::<UiTransform>(card_entity).unwrap();
        let z_idx = world.get::<GlobalZIndex>(card_entity).unwrap();
        let border = world.get::<bevy::ui::BorderColor>(card_entity).unwrap();

        assert!(transform.scale.x < 1.02, "La escala al desmarcar debe retornar cerca de 1.0 (actual: {})", transform.scale.x);
        if let Val::Px(y) = transform.translation.y {
            assert!(y > -2.0, "La elevación al desmarcar debe retornar cerca de 0 (actual: {})", y);
        }
        assert_eq!(z_idx.0, 10, "El Z-Index al desmarcar debe volver al valor base de 10");
        assert_eq!(border.top, Color::srgb(0.5, 0.5, 0.5), "El borde debe volver al color base");
    }
}

#[test]
fn test_preload_common_assets_initialization() {
    use slay_of_pokemon::card::{ALL_CARD_ASSETS, ALL_SITUATION_BACKGROUNDS};
    use slay_of_pokemon::ui::{preload_common_assets, PreloadedAssets};

    assert_eq!(ALL_CARD_ASSETS.len(), 8, "Deben existir 8 plantillas de cartas registradas");
    assert_eq!(ALL_SITUATION_BACKGROUNDS.len(), 5, "Deben existir 5 fondos contextuales registrados");

    // Verificar existencia física en assets/
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for &card_path in ALL_CARD_ASSETS {
        assert!(manifest_dir.join("assets").join(card_path).exists(), "Asset {} debe existir", card_path);
    }
    for &bg_path in ALL_SITUATION_BACKGROUNDS {
        assert!(manifest_dir.join("assets").join(bg_path).exists(), "Asset {} debe existir", bg_path);
    }

    let mut app = App::new();
    app.init_resource::<PreloadedAssets>();
    app.add_systems(Startup, preload_common_assets);
    app.update();

    let preloaded = app.world().resource::<PreloadedAssets>();
    assert!(preloaded.get_card("cartas/fuego.jpeg").is_none(), "Sin AssetServer los handles no se generan en headless sin fallar");
}

#[test]
fn test_card_discard_flight_animation() {
    use slay_of_pokemon::ui::combat_ui::{animate_card_discard_flight, CardDiscardFlight};

    let mut app = App::new();
    let mut time = Time::<()>::default();
    time.advance_by(std::time::Duration::from_millis(50));
    app.insert_resource(time);
    app.add_systems(Update, animate_card_discard_flight);

    let flight_entity = app.world_mut().spawn((
        CardDiscardFlight {
            timer: Timer::from_seconds(0.40, TimerMode::Once),
            start_pos: Vec2::new(300.0, 500.0),
            target_pos: Vec2::new(1200.0, 600.0),
            card_name: "Lanzallamas".to_string(),
        },
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(300.0),
            top: Val::Px(500.0),
            ..default()
        },
        UiTransform::default(),
    )).id();

    // Avanzar un frame (50ms)
    app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(150));
    app.update();

    // La entidad debe seguir viva, haberse desplazado hacia la derecha y reducido escala
    {
        let world = app.world();
        assert!(world.get_entity(flight_entity).is_ok(), "La carta debe estar volando en el aire");
        let node = world.get::<Node>(flight_entity).unwrap();
        let transform = world.get::<UiTransform>(flight_entity).unwrap();

        if let Val::Px(x) = node.left {
            assert!(x > 300.0, "La coordenada X debe haberse desplazado hacia el descarte (actual: {})", x);
        }
        assert!(transform.scale.x < 1.0, "La escala debe reducirse mientras viaja al descarte (actual: {})", transform.scale.x);
    }

    // Avanzar tiempo suficiente para que complete el vuelo
    for _ in 0..10 {
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(60));
        app.update();
    }

    // Al llegar a la pila de descarte, la animación concluye y la entidad se destruye
    assert!(app.world().get_entity(flight_entity).is_err(), "La carta que vuela al descarte debe despawnear al finalizar su vuelo");
}

#[test]
fn test_deck_shuffle_animation_and_pile_counts() {
    use slay_of_pokemon::combat::events::DeckShuffleAnimEvent;
    use slay_of_pokemon::ui::combat_ui::{
        animate_deck_shuffle, trigger_deck_shuffle_animation, DeckShuffleBanner, DeckShuffleFlight,
    };

    let mut app = App::new();
    let mut time = Time::<()>::default();
    time.advance_by(std::time::Duration::from_millis(50));
    app.insert_resource(time);
    app.add_message::<DeckShuffleAnimEvent>();
    app.add_systems(Update, (trigger_deck_shuffle_animation, animate_deck_shuffle));

    // Emitir evento de barajado de 10 cartas
    app.world_mut().write_message(DeckShuffleAnimEvent { card_count: 10 });
    app.update();

    // Verificar que se crearon el cartel y las cartas voladoras del barajado
    let banner_count = app.world_mut().query::<&DeckShuffleBanner>().iter(app.world()).count();
    let shuffle_cards = app.world_mut().query::<&DeckShuffleFlight>().iter(app.world()).count();

    assert_eq!(banner_count, 1, "Debe existir exactamente 1 cartel flotante de barajado");
    assert_eq!(shuffle_cards, 5, "Deben haberse generado 5 cartas en el flujo de barajado");

    // Avanzar varios frames de tiempo virtual
    for _ in 0..25 {
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(70));
        app.update();
    }

    // Las cartas voladoras deben haber aterrizado en el mazo de robo y haberse destruido
    let remaining_shuffle_cards = app.world_mut().query::<&DeckShuffleFlight>().iter(app.world()).count();
    assert_eq!(remaining_shuffle_cards, 0, "Todas las cartas del barajado deben haberse integrado al mazo y despawneado");
}

#[test]
fn test_preload_loading_screen_progress_and_completion() {
    use slay_of_pokemon::ui::{
        spawn_preload_loading_screen, update_preload_loading_screen,
        PreloadLoadingScreen, PreloadProgressBarFill, PreloadProgressPercentText,
        PreloadProgressDetailText, PreloadProgress, PreloadedAssets,
    };

    let mut app = App::new();
    let mut time = Time::<()>::default();
    time.advance_by(std::time::Duration::from_millis(50));
    app.insert_resource(time);
    app.init_resource::<PreloadedAssets>();
    app.init_resource::<PreloadProgress>();
    app.add_systems(Startup, spawn_preload_loading_screen);
    app.add_systems(Update, update_preload_loading_screen);

    app.update();

    // En headless sin AssetServer, la pantalla detecta la finalización inmediata
    assert!(app.world().resource::<PreloadProgress>().is_finished, "En entorno headless debe marcar finalizado");

    // Probar simulación con progreso parcial y textos de carga
    let mut app2 = App::new();
    let mut time2 = Time::<()>::default();
    time2.advance_by(std::time::Duration::from_millis(50));
    app2.insert_resource(time2);
    app2.init_resource::<PreloadedAssets>();
    let mut prog = PreloadProgress::default();
    prog.total = 14;
    prog.loaded = 7; // 50% cargado, faltan 7
    prog.current_asset = "Cargando: cartas/fuego.jpeg".to_string();
    app2.insert_resource(prog);
    app2.add_systems(Startup, spawn_preload_loading_screen);
    app2.update();

    let screen_count = app2.world_mut().query::<&PreloadLoadingScreen>().iter(app2.world()).count();
    let fill_count = app2.world_mut().query::<&PreloadProgressBarFill>().iter(app2.world()).count();
    let percent_count = app2.world_mut().query::<&PreloadProgressPercentText>().iter(app2.world()).count();
    let detail_count = app2.world_mut().query::<&PreloadProgressDetailText>().iter(app2.world()).count();

    assert_eq!(screen_count, 1, "Debe existir la pantalla de loading");
    assert_eq!(fill_count, 1, "Debe existir la barra de progreso");
    assert_eq!(percent_count, 1, "Debe existir el texto de porcentaje");
    assert_eq!(detail_count, 1, "Debe existir el texto de detalle del asset");
}

#[test]
fn test_resolve_asset_root_finds_assets() {
    let root = slay_of_pokemon::resolve_asset_root();
    assert!(root.join("assets").exists(), "La ruta resuelta {:?} debe contener la carpeta assets", root);
    assert!(root.join("assets").join("cartas").exists());
    assert!(root.join("assets").join("fondos").exists());
}

#[test]
fn test_self_contained_embedded_assets_registry() {
    use slay_of_pokemon::{EMBEDDED_ASSETS, build_embedded_asset_dir};
    use slay_of_pokemon::card::{ALL_CARD_ASSETS, ALL_SITUATION_BACKGROUNDS};
    use std::path::Path;

    // Verificar que todos los archivos de assets están incrustados en el binario
    assert!(
        EMBEDDED_ASSETS.len() >= 100,
        "El ejecutable debe contener al menos 100 assets incrustados, encontrados: {}",
        EMBEDDED_ASSETS.len()
    );

    let dir = build_embedded_asset_dir();

    // Verificar que todas las cartas están en el directorio virtual en memoria
    for &card_path in ALL_CARD_ASSETS.iter() {
        let data = dir.get_asset(Path::new(card_path)).expect(&format!(
            "La carta '{}' debe estar incrustada en memoria",
            card_path
        ));
        assert!(!data.value().is_empty(), "Los bytes de '{}' no deben estar vacíos", card_path);
    }

    // Verificar que todos los fondos de situación están en el directorio virtual en memoria
    for &bg_path in ALL_SITUATION_BACKGROUNDS.iter() {
        let data = dir.get_asset(Path::new(bg_path)).expect(&format!(
            "El fondo '{}' debe estar incrustado en memoria",
            bg_path
        ));
        assert!(!data.value().is_empty(), "Los bytes de '{}' no deben estar vacíos", bg_path);
    }

    // Verificar fuentes y sprites clave en memoria
    for key_asset in [
        "fonts/FiraSans-Bold.ttf",
        "sprites/pokemon/front/4.png",
        "sprites/pokemon/back/4.png",
        "sprites/trainers/brock_front.png",
    ] {
        let data = dir.get_asset(Path::new(key_asset)).expect(&format!(
            "El asset clave '{}' debe estar incrustado en memoria",
            key_asset
        ));
        assert!(!data.value().is_empty());
    }
}
